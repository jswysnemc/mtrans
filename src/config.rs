use crate::error::{MtransError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use colored::Colorize;
use dirs::config_dir;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const CONFIG_FILE: &str = "mtrans/config.toml";
const PROMPT_DIR: &str = "mtrans/prompt.d";
const NONCE_SIZE: usize = 12;

/// 存储在文件中的配置（API Key 已加密）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    pub encrypted_api_key: String,
    pub base_url: String,
    pub model: String,
}

/// 运行时使用的配置（API Key 已解密）
#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_key: String::new(),
            base_url: String::from("https://api.openai.com/v1"),
            model: String::from("gpt-3.5-turbo"),
        }
    }
}

impl Config {
    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .ok_or_else(|| MtransError::Config("无法获取配置目录".to_string()))?;
        Ok(config_dir.join(CONFIG_FILE))
    }

    pub fn get_prompt_dir() -> Result<PathBuf> {
        let config_dir = config_dir()
            .ok_or_else(|| MtransError::Config("无法获取配置目录".to_string()))?;
        let prompt_dir = config_dir.join(PROMPT_DIR);
        fs::create_dir_all(&prompt_dir)
            .map_err(|e| MtransError::Config(format!("无法创建 prompt 目录: {}", e)))?;
        Ok(prompt_dir)
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Err(MtransError::Config(
                "配置文件不存在，请先运行 `mtrans config init` 初始化配置".to_string(),
            ));
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| MtransError::Config(format!("无法读取配置文件: {}", e)))?;

        let config_file: ConfigFile = toml::from_str(&content)
            .map_err(|e| MtransError::Config(format!("配置文件格式错误: {}", e)))?;

        // 解密 API Key
        let api_key = decrypt_api_key(&config_file.encrypted_api_key)?;

        Ok(Config {
            api_key,
            base_url: config_file.base_url,
            model: config_file.model,
        })
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let config_dir = config_path
            .parent()
            .ok_or_else(|| MtransError::Config("无法获取配置目录".to_string()))?;

        fs::create_dir_all(config_dir)
            .map_err(|e| MtransError::Config(format!("无法创建配置目录: {}", e)))?;

        // 加密 API Key
        let encrypted_api_key = encrypt_api_key(&self.api_key)?;

        let config_file = ConfigFile {
            encrypted_api_key,
            base_url: self.base_url.clone(),
            model: self.model.clone(),
        };

        let content = toml::to_string_pretty(&config_file)
            .map_err(|e| MtransError::Config(format!("序列化配置失败: {}", e)))?;

        fs::write(&config_path, content)
            .map_err(|e| MtransError::Config(format!("无法写入配置文件: {}", e)))?;

        Ok(())
    }

    pub fn show_info(&self) -> Result<()> {
        println!("{}", "当前配置:".green().bold());
        println!("  Base URL: {}", self.base_url);
        println!("  Model: {}", self.model);
        let key_preview = if self.api_key.len() > 8 {
            format!("{}...{}", &self.api_key[..4], &self.api_key[self.api_key.len()-4..])
        } else {
            "***".to_string()
        };
        println!("  API Key: {} {}", key_preview, "(已加密存储)".dimmed());
        Ok(())
    }
}

/// 获取硬件指纹
fn get_machine_fingerprint() -> Result<String> {
    // Linux: 读取 /etc/machine-id
    #[cfg(target_os = "linux")]
    {
        if let Ok(id) = fs::read_to_string("/etc/machine-id") {
            return Ok(id.trim().to_string());
        }
        if let Ok(id) = fs::read_to_string("/var/lib/dbus/machine-id") {
            return Ok(id.trim().to_string());
        }
    }

    // macOS: 使用 IOPlatformUUID
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("IOPlatformUUID") {
                    if let Some(uuid) = line.split('"').nth(3) {
                        return Ok(uuid.to_string());
                    }
                }
            }
        }
    }

    // Windows: 使用 MachineGuid
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("reg")
            .args(["query", r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography", "/v", "MachineGuid"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("MachineGuid") {
                    if let Some(guid) = line.split_whitespace().last() {
                        return Ok(guid.to_string());
                    }
                }
            }
        }
    }

    // 回退方案：使用用户名 + 主机名
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(format!("{}@{}", username, hostname))
}

/// 从硬件指纹派生 AES-256 密钥
fn derive_key() -> Result<[u8; 32]> {
    let fingerprint = get_machine_fingerprint()?;
    let salt = "mtrans-encryption-salt-v1"; // 固定盐值
    
    let mut hasher = Sha256::new();
    hasher.update(fingerprint.as_bytes());
    hasher.update(salt.as_bytes());
    
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    Ok(key)
}

/// 加密 API Key
fn encrypt_api_key(api_key: &str) -> Result<String> {
    if api_key.is_empty() {
        return Ok(String::new());
    }

    let key = derive_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| MtransError::Config(format!("创建加密器失败: {}", e)))?;

    // 生成随机 nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密
    let ciphertext = cipher
        .encrypt(nonce, api_key.as_bytes())
        .map_err(|e| MtransError::Config(format!("加密失败: {}", e)))?;

    // 组合 nonce + ciphertext，然后 base64 编码
    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

/// 解密 API Key
fn decrypt_api_key(encrypted: &str) -> Result<String> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    let key = derive_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| MtransError::Config(format!("创建解密器失败: {}", e)))?;

    // Base64 解码
    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| MtransError::Config(format!("Base64 解码失败: {}", e)))?;

    if combined.len() < NONCE_SIZE {
        return Err(MtransError::Config("加密数据格式错误".to_string()));
    }

    // 分离 nonce 和 ciphertext
    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // 解密
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| MtransError::Config(
            "解密失败：可能是配置文件被复制到其他机器，或硬件指纹已更改。请重新运行 `mtrans config init`".to_string()
        ))?;

    String::from_utf8(plaintext)
        .map_err(|e| MtransError::Config(format!("解密后的数据不是有效 UTF-8: {}", e)))
}

pub async fn handle_config_command(cmd: crate::cli::ConfigCommand) -> Result<()> {
    match cmd.action {
        crate::cli::ConfigAction::Init => init_config().await?,
        crate::cli::ConfigAction::Set { key, model, base_url } => set_config(key, model, base_url)?,
        crate::cli::ConfigAction::Info => show_config_info()?,
        crate::cli::ConfigAction::Test => test_connection().await?,
        crate::cli::ConfigAction::Clear => clear_config()?,
    }
    Ok(())
}

async fn init_config() -> Result<()> {
    println!("{}", "欢迎使用 mtrans 配置向导".green().bold());
    println!("{}", "请输入以下信息:".cyan());

    let mut config = Config::load().unwrap_or_default();

    let base_url = prompt("Base URL (默认: https://api.openai.com/v1): ")?;
    if !base_url.is_empty() {
        config.base_url = base_url;
    }

    let api_key = prompt("API Key: ")?;
    if !api_key.is_empty() {
        config.api_key = api_key;
    }

    let model = prompt("Model (默认: gpt-3.5-turbo): ")?;
    if !model.is_empty() {
        config.model = model;
    }

    config.save()?;
    println!("{}", "✓ 配置已保存".green());
    Ok(())
}

fn set_config(key: Option<String>, model: Option<String>, base_url: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

    if let Some(k) = key {
        config.api_key = k;
        println!("{}", "✓ API Key 已更新".green());
    }

    if let Some(m) = model {
        config.model = m;
        println!("{}", "✓ Model 已更新".green());
    }

    if let Some(u) = base_url {
        config.base_url = u;
        println!("{}", "✓ Base URL 已更新".green());
    }

    config.save()?;
    Ok(())
}

fn show_config_info() -> Result<()> {
    let config = Config::load()?;
    config.show_info()?;
    Ok(())
}

async fn test_connection() -> Result<()> {
    let config = Config::load()?;
    println!("{}", "正在测试连接...".cyan());

    let client = reqwest::Client::new();
    let url = format!("{}/models", config.base_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .send()
        .await?;

    if response.status().is_success() {
        println!("{}", "✓ 连接成功!".green());
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        println!("{}: {}", "连接失败".red(), error_text);
        return Err(MtransError::Api(format!("HTTP {}", status)));
    }

    Ok(())
}

fn clear_config() -> Result<()> {
    let config_path = Config::get_config_path()?;
    if config_path.exists() {
        fs::remove_file(&config_path)?;
        println!("{}", "✓ 配置已清除".green());
    } else {
        println!("{}", "配置文件不存在".yellow());
    }
    Ok(())
}

fn prompt(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
