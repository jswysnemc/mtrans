use crate::config::Config;
use crate::error::{MtransError, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    index: Option<u32>,
    message: Option<ChatMessage>,
    delta: Option<ChatMessage>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

pub struct LLMClient {
    config: Config,
    client: reqwest::Client,
}

impl LLMClient {
    pub fn new(config: Config) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;
        Ok(LLMClient { config, client })
    }

    pub async fn chat(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: Some("user".to_string()),
                content: Some(prompt.to_string()),
            }],
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(MtransError::Api(format!("HTTP {}: {}", status, error_text)));
        }

        let chat_response: ChatResponse = response.json().await?;

        chat_response
            .choices
            .first()
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content.as_ref())
            .map(|s| s.clone())
            .ok_or_else(|| MtransError::Api("响应中没有内容".to_string()))
    }

    pub async fn chat_stream<F>(&self, prompt: &str, mut callback: F) -> Result<String>
    where
        F: FnMut(&str) -> (),
    {
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![ChatMessage {
                role: Some("user".to_string()),
                content: Some(prompt.to_string()),
            }],
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(MtransError::Api(format!("HTTP {}: {}", status, error_text)));
        }

        let mut full_content = String::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);

            // 处理 SSE 格式
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        continue;
                    }

                    if let Ok(chat_response) = serde_json::from_str::<ChatResponse>(data) {
                        if let Some(choice) = chat_response.choices.first() {
                            if let Some(delta) = &choice.delta {
                                if let Some(content) = &delta.content {
                                    callback(content);
                                    full_content.push_str(content);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(full_content)
    }
}
