use crate::cli::{Cli, list_languages, parse_language};
use crate::client::LLMClient;
use crate::config::Config;
use crate::error::{MtransError, Result};
use crate::prompt::{build_code_prompt, build_common_prompt, build_word_prompt};
use arboard::Clipboard;
use colored::Colorize;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use std::io::{self, Read, Write};

pub async fn handle_translation(cli: Cli) -> Result<()> {
    // 显示语言列表
    if cli.list_languages {
        list_languages();
        return Ok(());
    }

    // 加载配置
    let config = Config::load()?;

    // 创建客户端
    let client = LLMClient::new(config)?;

    // 交互式模式
    if cli.interactive {
        return interactive_mode(client).await;
    }

    // 获取输入文本
    let text = get_input_text(&cli)?;

    // 获取源语言（优先使用 --from 选项，否则解析语法糖）
    let source_lang = cli.from.or_else(|| {
        let (_, lang) = parse_language(&text);
        lang
    });

    // 确定目标语言
    let target_lang = cli.to.unwrap_or_else(|| {
        // 如果指定了源语言，默认目标语言为中文
        if source_lang.is_some() {
            "zh".to_string()
        } else {
            "en".to_string()
        }
    });

    // 构建提示词
    let prompt = if cli.word {
        // 词典模式
        build_word_prompt(&text)?
    } else if cli.code {
        // 变量命名模式
        let style = cli.style.as_deref().unwrap_or("all");
        build_code_prompt(&text, style)?
    } else {
        // 普通翻译模式
        let source = source_lang.as_deref().unwrap_or("auto");
        build_common_prompt(&text, source, &target_lang)?
    };

    // 判断是否需要流式输出（输出到文件时不使用流式）
    if cli.output.is_some() {
        // 非流式输出（写入文件）
        let result = client.chat(&prompt).await?;
        output_result(&result, &cli.output, cli.clipboard)?;
    } else {
        // 流式输出到终端
        let result = client.chat_stream(&prompt, |chunk| {
            print!("{}", chunk);
            io::stdout().flush().ok();
        }).await?;
        println!();

        // 复制到剪贴板
        if cli.clipboard {
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(result.trim())?;
            eprintln!("{}", "✓ 已复制到剪贴板".green());
        }
    }

    Ok(())
}

fn get_input_text(cli: &Cli) -> Result<String> {
    if let Some(text) = &cli.text {
        return Ok(text.clone());
    }

    if let Some(file_path) = &cli.file {
        return read_file(file_path);
    }

    // 从 stdin 读取
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input.trim().to_string())
}

fn read_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| MtransError::Io(e))?;
    Ok(content)
}

fn output_result(result: &str, output_path: &Option<String>, to_clipboard: bool) -> Result<()> {
    let trimmed = result.trim();

    // 输出到文件
    if let Some(path) = output_path {
        std::fs::write(path, trimmed)?;
        println!("{}", format!("结果已写入: {}", path).green());
    } else {
        // 输出到标准输出
        println!("{}", trimmed);
    }

    // 复制到剪贴板
    if to_clipboard {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(trimmed)?;
        println!("{}", "✓ 已复制到剪贴板".green());
    }

    Ok(())
}

async fn interactive_mode(client: LLMClient) -> Result<()> {
    println!("{}", "进入交互模式 (输入 'exit' 或 'quit' 退出)".green().bold());
    println!();

    let mut rl = Reedline::create();
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("mtrans".to_string()),
        DefaultPromptSegment::Basic(">".to_string()),
    );

    loop {
        let sig = rl.read_line(&prompt);

        match sig {
            Ok(Signal::Success(line)) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                if line == "exit" || line == "quit" || line == ":q" {
                    println!("{}", "再见!".green());
                    break;
                }

                if line == ":help" || line == "help" {
                    print_interactive_help();
                    continue;
                }

                // 解析输入
                let (source_lang, text_opt) = parse_language(line);
                let text = text_opt.unwrap_or_else(|| line.to_string());
                let source = source_lang.as_deref().unwrap_or("auto");
                let target_lang = "zh"; // 默认翻译为中文

                // 构建提示词
                let prompt = build_common_prompt(&text, source, target_lang)?;

                print!("{}", "翻译: ".cyan());
                io::stdout().flush()?;

                // 流式输出
                let _result = client.chat_stream(&prompt, |chunk| {
                    print!("{}", chunk);
                    io::stdout().flush().ok();
                }).await?;

                println!();
                println!();
            }
            Ok(Signal::CtrlC) => {
                println!("\n{}", "输入 Ctrl+C 退出".yellow());
                break;
            }
            Ok(Signal::CtrlD) => {
                println!();
                break;
            }
            Err(e) => {
                eprintln!("{}: {}", "错误".red(), e);
                break;
            }
        }
    }

    Ok(())
}

fn print_interactive_help() {
    println!("{}", "交互模式帮助:".green().bold());
    println!("  直接输入文本进行翻译");
    println!("  :zh 文本  - 指定源语言为中文，翻译为英文");
    println!("  :en 文本  - 指定源语言为英文，翻译为中文");
    println!("  :jp 文本  - 指定源语言为日语，翻译为中文");
    println!("  exit/quit/:q  - 退出交互模式");
    println!();
}
