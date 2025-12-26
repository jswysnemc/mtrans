use clap::{Parser, Subcommand};
use colored::Colorize;

/// mtrans - 使用大模型进行翻译的 CLI 工具
#[derive(Parser, Debug)]
#[command(name = "mtrans")]
#[command(about = "使用大模型进行翻译的 CLI 工具", long_about = None)]
pub struct Cli {
    /// 待翻译的文本（位置参数）
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,

    /// 源语言（使用 :zh/:en/:jp 语法糖，或 --from zh）
    #[arg(short = 'F', long = "from")]
    pub from: Option<String>,

    /// 目标语言 (如 zh, jp, en)
    #[arg(short = 't', long = "to")]
    pub to: Option<String>,

    /// 显示支持的语言列表
    #[arg(short = 'l', long = "list-languages")]
    pub list_languages: bool,

    /// 词典模式（解释、词根、例句）
    #[arg(short = 'w', long = "word")]
    pub word: bool,

    /// 变量命名模式
    #[arg(short = 'c', long = "code")]
    pub code: bool,

    /// 代码命名风格 (snake_case, camelCase, PascalCase, kebab-case, CONSTANT_CASE)
    #[arg(short = 's', long = "style")]
    pub style: Option<String>,

    /// 进入交互式 REPL 模式
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// 从文件读取
    #[arg(short = 'f', long = "file")]
    pub file: Option<String>,

    /// 输出到文件
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    /// 将结果复制到剪贴板
    #[arg(long = "clipboard")]
    pub clipboard: bool,

    /// 配置子命令
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 配置管理
    Config(ConfigCommand),
}

#[derive(Parser, Debug)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// 交互式初始化配置
    Init,
    /// 设置配置项
    Set {
        /// API Key
        #[arg(long = "key")]
        key: Option<String>,
        /// Model 名称
        #[arg(long = "model")]
        model: Option<String>,
        /// Base URL
        #[arg(long = "baseurl")]
        base_url: Option<String>,
    },
    /// 显示当前配置
    Info,
    /// 测试连接
    Test,
    /// 清除配置
    Clear,
}

pub fn list_languages() {
    println!("{}", "支持的语言列表:".green().bold());
    println!();
    println!("  代号    语言名称");
    println!("  {}      {}", "auto".cyan(), "自动检测".cyan());
    println!("  {}      {}", "en".cyan(), "英语".cyan());
    println!("  {}      {}", "zh".cyan(), "中文".cyan());
    println!("  {}      {}", "jp".cyan(), "日语".cyan());
    println!("  {}      {}", "ko".cyan(), "韩语".cyan());
    println!("  {}      {}", "fr".cyan(), "法语".cyan());
    println!("  {}      {}", "de".cyan(), "德语".cyan());
    println!("  {}      {}", "es".cyan(), "西班牙语".cyan());
    println!("  {}      {}", "ru".cyan(), "俄语".cyan());
    println!("  {}      {}", "pt".cyan(), "葡萄牙语".cyan());
    println!("  {}      {}", "it".cyan(), "意大利语".cyan());
    println!();
    println!("提示: 可以使用 :zh/:en/:jp 语法糖指定源语言");
    println!("例如: mtrans :zh \"Hello\"  # 将英文翻译为中文");
}

pub fn parse_language(text: &str) -> (Option<String>, Option<String>) {
    // 检查是否使用 :lang 语法糖
    if text.starts_with(':') {
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let lang = parts[0].trim_start_matches(':').to_string();
            let remaining = parts[1].to_string();
            return (Some(lang), Some(remaining));
        }
    }
    (None, Some(text.to_string()))
}
