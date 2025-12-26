use crate::error::{MtransError, Result};
use std::fs;
use std::path::PathBuf;

pub struct PromptManager {
    prompt_dir: PathBuf,
}

impl PromptManager {
    pub fn new() -> Result<Self> {
        let prompt_dir = crate::config::Config::get_prompt_dir()?;
        Ok(PromptManager { prompt_dir })
    }

    pub fn get_word_prompt(&self) -> Result<String> {
        self.load_prompt("word.prompt", DEFAULT_WORD_PROMPT)
    }

    pub fn get_code_prompt(&self) -> Result<String> {
        self.load_prompt("code.prompt", DEFAULT_CODE_PROMPT)
    }

    pub fn get_common_prompt(&self) -> Result<String> {
        self.load_prompt("common.prompt", DEFAULT_COMMON_PROMPT)
    }

    fn load_prompt(&self, filename: &str, default: &str) -> Result<String> {
        let path = self.prompt_dir.join(filename);
        if path.exists() {
            fs::read_to_string(&path)
                .map_err(|e| MtransError::Prompt(format!("无法读取 prompt 文件: {}", e)))
        } else {
            // 创建默认 prompt 文件
            fs::write(&path, default)
                .map_err(|e| MtransError::Prompt(format!("无法创建 prompt 文件: {}", e)))?;
            Ok(default.to_string())
        }
    }

    pub fn create_default_prompts(&self) -> Result<()> {
        self.load_prompt("word.prompt", DEFAULT_WORD_PROMPT)?;
        self.load_prompt("code.prompt", DEFAULT_CODE_PROMPT)?;
        self.load_prompt("common.prompt", DEFAULT_COMMON_PROMPT)?;
        Ok(())
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new().expect("无法创建 PromptManager")
    }
}

const DEFAULT_WORD_PROMPT: &str = r#"你是专业词典助手。请解释单词 "{{text}}"。

## 输出格式示例

输入: ephemeral

**ephemeral** /ɪˈfemərəl/

📖 **释义**
- [adj.] 短暂的，转瞬即逝的
- [adj.] 朝生暮死的（生物学）

🔤 **词源**
希腊语 ephemeros（仅持续一天）= epi-（在...上）+ hemera（天）

📝 **例句**
1. Fame in the internet age is often ephemeral.
   → 互联网时代的名声往往转瞬即逝。
2. The ephemeral beauty of cherry blossoms attracts many tourists.
   → 樱花短暂的美丽吸引了许多游客。

🔗 **相关词汇**
- 同义词: fleeting, transient, momentary
- 反义词: permanent, enduring, eternal

---
请按以上格式回答，保持简洁。若某项无内容可省略。"#;

const DEFAULT_CODE_PROMPT: &str = r#"你是代码命名助手。将文本转换为程序变量名。

## 风格说明
- snake_case: 全小写下划线 (user_login_count)
- camelCase: 小驼峰 (userLoginCount)
- PascalCase: 大驼峰 (UserLoginCount)
- kebab-case: 短横线 (user-login-count)
- CONSTANT_CASE: 全大写下划线 (USER_LOGIN_COUNT)

## 示例

输入: "获取用户信息"
风格: all

snake_case: get_user_info
camelCase: getUserInfo
PascalCase: GetUserInfo
kebab-case: get-user-info
CONSTANT_CASE: GET_USER_INFO

输入: "is February February"
风格: camelCase

isFebruaryLeapYear

---
输入: "{{text}}"
风格: {{style}}

只输出变量名，不加解释。若风格为 all，输出所有风格。"#;

const DEFAULT_COMMON_PROMPT: &str = r#"你是专业翻译。将文本翻译为{{target_lang}}。

## 规则
1. 保持原文格式、换行、缩进不变
2. 命令行选项/参数/路径/代码保持原样（如 -c, --help, /dev/st0）
3. 技术术语使用标准译法
4. 只输出译文

## 示例

输入 (en→zh):
```
Usage: grep [OPTION]... PATTERN [FILE]...
  -i, --ignore-case     ignore case distinctions
  -v, --invert-match    select non-matching lines
  -n, --line-number     print line number with output
```

输出:
```
用法: grep [选项]... 模式 [文件]...
  -i, --ignore-case     忽略大小写区别
  -v, --invert-match    选择不匹配的行
  -n, --line-number     输出时打印行号
```

---
源语言: {{source_lang}}

{{text}}"#;

pub fn build_word_prompt(text: &str) -> Result<String> {
    let manager = PromptManager::new()?;
    let template = manager.get_word_prompt()?;
    Ok(template.replace("{{text}}", text))
}

pub fn build_code_prompt(text: &str, style: &str) -> Result<String> {
    let manager = PromptManager::new()?;
    let template = manager.get_code_prompt()?;
    let result = template
        .replace("{{text}}", text)
        .replace("{{style}}", style);
    Ok(result)
}

pub fn build_common_prompt(text: &str, source_lang: &str, target_lang: &str) -> Result<String> {
    let manager = PromptManager::new()?;
    let template = manager.get_common_prompt()?;
    let result = template
        .replace("{{text}}", text)
        .replace("{{source_lang}}", source_lang)
        .replace("{{target_lang}}", target_lang);
    Ok(result)
}
