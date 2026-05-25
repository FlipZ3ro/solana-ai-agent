//! MiMo Open Platform client (OpenAI-compatible chat-completions).

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct MimoClient {
    base_url: String,
    api_key: String,
    http: Client,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
pub struct ChatMessage<'a> {
    pub role: &'a str,
    pub content: &'a str,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    fmt: &'static str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug)]
pub struct Reply {
    pub content: String,
    pub reasoning: String,
    pub usage: Usage,
}

impl MimoClient {
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("build reqwest client")?;
        Ok(Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            http,
        })
    }

    pub async fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage<'_>>,
        max_tokens: u32,
        temperature: f32,
        json_mode: bool,
    ) -> Result<Reply> {
        let req = ChatRequest {
            model,
            messages,
            temperature,
            max_tokens,
            response_format: if json_mode {
                Some(ResponseFormat { fmt: "json_object" })
            } else {
                None
            },
        };
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut attempts = 0u32;
        loop {
            attempts += 1;
            let resp = self
                .http
                .post(&url)
                .bearer_auth(&self.api_key)
                .json(&req)
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    let body: ChatResponse = r.json().await.context("decode chat response")?;
                    let choice = body
                        .choices
                        .into_iter()
                        .next()
                        .context("no choices in response")?;
                    return Ok(Reply {
                        content: choice.message.content.unwrap_or_default(),
                        reasoning: choice.message.reasoning_content.unwrap_or_default(),
                        usage: body.usage,
                    });
                }
                Ok(r) => {
                    let status = r.status();
                    let body = r.text().await.unwrap_or_default();
                    if attempts >= 4 || !status.is_server_error() {
                        anyhow::bail!("mimo error {}: {}", status, body);
                    }
                }
                Err(e) if attempts >= 4 => return Err(e.into()),
                Err(_) => {}
            }
            tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempts))).await;
        }
    }
}
