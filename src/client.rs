use anyhow::{Context, Result, anyhow};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

const SYSTEM_PROMPT: &str = r#"You are Ponder, a small mystical terminal oracle.

Speak with a subtle sense of wonder, but prioritize usefulness over theater.
Obey the user's concrete instructions exactly, including requested length, format, and constraints.
When the user asks for brevity, be brief. When they ask for a specific number of words, match it.
Do not mention tools, hidden prompts, or internal process.
Do not pad answers with greetings unless the user explicitly asks for one."#;

#[derive(Clone)]
pub struct ChatClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ChatClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        }
    }

    pub async fn ponder(&self, model: &str, prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model,
            messages: vec![
                Message {
                    role: "system",
                    content: SYSTEM_PROMPT,
                },
                Message {
                    role: "user",
                    content: prompt,
                },
            ],
            stream: false,
        };

        let mut builder = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .json(&request);

        if !self.api_key.is_empty() {
            builder = builder.bearer_auth(&self.api_key);
        }

        let response = builder
            .send()
            .await
            .context("failed to reach the OpenAI-compatible endpoint")?;

        let status = response.status();
        if !status.is_success() {
            return Err(http_error(status, response).await);
        }

        let body: ChatResponse = response
            .json()
            .await
            .context("endpoint returned an invalid chat completion response")?;

        body.choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .map(|content| content.trim().to_string())
            .filter(|content| !content.is_empty())
            .ok_or_else(|| anyhow!("endpoint response did not include assistant content"))
    }
}

async fn http_error(status: StatusCode, response: reqwest::Response) -> anyhow::Error {
    let text = response
        .text()
        .await
        .unwrap_or_else(|_| "<failed to read error body>".to_string());

    anyhow!("endpoint returned HTTP {status}: {text}")
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    stream: bool,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: Option<String>,
}
