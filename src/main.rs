mod cli;
mod client;
mod config;
mod error;
mod prompt;
mod translator;

use clap::Parser;
use cli::{Cli, Commands};
use error::Result;

/// 预处理命令行参数，处理语法糖
/// 支持的格式：
/// - `:zh hello` → `--to zh "hello"` (翻译为中文)
/// - `:zh,en hello` → `--from zh --to en "hello"` (从中文翻译为英文)
/// - `i:zh hello` → `--from zh "hello"` (源语言为中文)
/// - `o:en hello` → `--to en "hello"` (目标语言为英文)
/// - `i:zh o:en hello` → `--from zh --to en "hello"` (完整指定)
fn preprocess_args() -> Vec<String> {
    let args: Vec<String> = std::env::args().collect();
    let mut result = vec![args[0].clone()]; // 保留程序名

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        // 处理 i:lang 格式 (输入/源语言)
        if arg.starts_with("i:") && arg.len() > 2 {
            let lang = &arg[2..];
            if is_valid_lang(lang) {
                result.push("--from".to_string());
                result.push(lang.to_string());
                i += 1;
                continue;
            }
        }

        // 处理 o:lang 格式 (输出/目标语言)
        if arg.starts_with("o:") && arg.len() > 2 {
            let lang = &arg[2..];
            if is_valid_lang(lang) {
                result.push("--to".to_string());
                result.push(lang.to_string());
                i += 1;
                continue;
            }
        }

        // 处理 :from,to 格式 (如 :zh,en)
        if arg.starts_with(':') && arg.contains(',') {
            let content = &arg[1..]; // 去掉冒号
            let parts: Vec<&str> = content.split(',').collect();
            if parts.len() == 2 && is_valid_lang(parts[0]) && is_valid_lang(parts[1]) {
                result.push("--from".to_string());
                result.push(parts[0].to_string());
                result.push("--to".to_string());
                result.push(parts[1].to_string());

                // 收集剩余的文本参数
                i += 1;
                let text = collect_text_args(&args, &mut i);
                if !text.is_empty() {
                    result.push(text);
                }
                continue;
            }
        }

        // 处理 :lang 格式 (仅目标语言，如 :zh)
        if arg.starts_with(':') && arg.len() > 1 && !arg.contains(',') {
            let lang = &arg[1..];
            if is_valid_lang(lang) {
                result.push("--to".to_string());
                result.push(lang.to_string());

                // 收集剩余的文本参数
                i += 1;
                let text = collect_text_args(&args, &mut i);
                if !text.is_empty() {
                    result.push(text);
                }
                continue;
            }
        }

        result.push(arg.clone());
        i += 1;
    }

    result
}

/// 检查是否是有效的语言代码 (2-4 个字母)
fn is_valid_lang(lang: &str) -> bool {
    lang.chars().all(|c| c.is_ascii_alphabetic()) && lang.len() >= 2 && lang.len() <= 4
}

/// 收集文本参数，直到遇到选项或子命令
fn collect_text_args(args: &[String], i: &mut usize) -> String {
    let mut text_parts = Vec::new();
    while *i < args.len() {
        let next = &args[*i];
        // 如果遇到选项、子命令或语法糖，停止收集
        if next.starts_with('-') || next == "config" || next.starts_with(':') 
            || next.starts_with("i:") || next.starts_with("o:") {
            break;
        }
        text_parts.push(next.clone());
        *i += 1;
    }
    text_parts.join(" ")
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = preprocess_args();
    let cli = Cli::parse_from(args);

    match cli.command {
        Some(Commands::Config(config_cmd)) => {
            config::handle_config_command(config_cmd).await?;
        }
        None => {
            // Translation mode
            translator::handle_translation(cli).await?;
        }
    }

    Ok(())
}
