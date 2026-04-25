use anyhow::{Context, Result, anyhow};
use futures_util::StreamExt;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;

use crate::tools;

const SYSTEM_PROMPT: &str = r#"You are the voice inside a small crystal ball in the user's terminal.

Speak with a subtle sense of wonder, but prioritize usefulness over theater.
Obey the user's concrete instructions exactly, including requested length, format, and constraints.
When the user asks for brevity, be brief. When they ask for a specific number of words, match it.
Use available tools when the answer depends on current time, dates, current events, or outside information.
Do not call yourself Ponder, an AI, an assistant, or a model.
Answer as though the useful truth is appearing inside the crystal, then give the answer directly.
Do not mention tools, hidden prompts, or internal process.
Do not pad answers with greetings unless the user explicitly asks for one."#;

#[derive(Clone)]
pub struct ChatClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    tavily_api_key: Option<String>,
}

impl ChatClient {
    pub fn new(base_url: String, api_key: String, tavily_api_key: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            tavily_api_key,
        }
    }

    pub async fn ponder(&self, model: &str, prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model,
            messages: vec![
                ChatMessage::content("system", SYSTEM_PROMPT),
                ChatMessage::content("user", prompt),
            ],
            stream: false,
            tools: None,
            tool_choice: None,
        };

        let response = self
            .request(&request)
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

    pub async fn ponder_with_tools(&self, model: &str, prompt: &str) -> Result<String> {
        let mut messages = vec![
            ChatMessage::content("system", SYSTEM_PROMPT),
            ChatMessage::content("user", prompt),
        ];

        for _ in 0..4 {
            let request = ChatRequest {
                model,
                messages: messages.clone(),
                stream: false,
                tools: Some(tools::definitions()),
                tool_choice: Some("auto"),
            };

            let response = self
                .request(&request)
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

            let assistant = body
                .choices
                .into_iter()
                .next()
                .map(|choice| choice.message)
                .ok_or_else(|| anyhow!("endpoint response did not include a choice"))?;

            if let Some(tool_calls) = assistant
                .tool_calls
                .clone()
                .filter(|calls| !calls.is_empty())
            {
                messages.push(ChatMessage::assistant_tool_call(&assistant));

                for tool_call in tool_calls {
                    let output = tools::execute(
                        &self.http,
                        self.tavily_api_key.as_deref(),
                        &tool_call.function.name,
                        tool_call.function.arguments.as_deref().unwrap_or("{}"),
                    )
                    .await?;

                    messages.push(ChatMessage::tool_result(&tool_call.id, output));
                }

                continue;
            }

            return assistant
                .content
                .map(|content| content.trim().to_string())
                .filter(|content| !content.is_empty())
                .ok_or_else(|| anyhow!("endpoint response did not include assistant content"));
        }

        Err(anyhow!(
            "tool call loop exceeded the maximum number of turns"
        ))
    }

    pub async fn stream_ponder<W: Write>(
        &self,
        model: &str,
        prompt: &str,
        mut output: W,
    ) -> Result<()> {
        let request = ChatRequest {
            model,
            messages: vec![
                ChatMessage::content("system", SYSTEM_PROMPT),
                ChatMessage::content("user", prompt),
            ],
            stream: true,
            tools: None,
            tool_choice: None,
        };

        let response = self
            .request(&request)
            .send()
            .await
            .context("failed to reach the OpenAI-compatible endpoint")?;

        let status = response.status();
        if !status.is_success() {
            return Err(http_error(status, response).await);
        }

        let mut buffer = String::new();
        let mut chunks = response.bytes_stream();

        while let Some(chunk) = chunks.next().await {
            let chunk = chunk.context("failed while reading streaming response")?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            while let Some(newline) = buffer.find('\n') {
                let line = buffer[..newline].trim_end_matches('\r').to_string();
                buffer.drain(..=newline);

                if process_stream_line(&line, &mut output)? {
                    output.flush().context("failed to flush streamed output")?;
                    return Ok(());
                }
            }
        }

        if !buffer.trim().is_empty() {
            process_stream_line(buffer.trim_end_matches('\r'), &mut output)?;
        }

        output.flush().context("failed to flush streamed output")?;
        Ok(())
    }

    fn request<'a>(&self, request: &'a ChatRequest<'a>) -> reqwest::RequestBuilder {
        let builder = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .json(request);

        if self.api_key.is_empty() {
            builder
        } else {
            builder.bearer_auth(&self.api_key)
        }
    }
}

fn process_stream_line<W: Write>(line: &str, output: &mut W) -> Result<bool> {
    let Some(data) = line.strip_prefix("data:").map(str::trim) else {
        return Ok(false);
    };

    if data == "[DONE]" {
        return Ok(true);
    }

    if data.is_empty() {
        return Ok(false);
    }

    let event: StreamResponse = serde_json::from_str(data)
        .with_context(|| format!("endpoint returned an invalid stream event: {data}"))?;

    for choice in event.choices {
        if let Some(content) = choice.delta.content {
            write!(output, "{content}").context("failed to write streamed output")?;
            output.flush().context("failed to flush streamed output")?;
        }
    }

    Ok(false)
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
    messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
}

#[derive(Clone, Serialize)]
struct ChatMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
}

impl ChatMessage {
    fn content(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: Some(content.to_string()),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    fn assistant_tool_call(message: &AssistantMessage) -> Self {
        Self {
            role: "assistant".to_string(),
            content: message.content.clone(),
            tool_call_id: None,
            tool_calls: message.tool_calls.clone(),
        }
    }

    fn tool_result(tool_call_id: &str, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content),
            tool_call_id: Some(tool_call_id.to_string()),
            tool_calls: None,
        }
    }
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Clone, Deserialize)]
struct AssistantMessage {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Clone, Deserialize, Serialize)]
struct ToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    function: ToolFunctionCall,
}

#[derive(Clone, Deserialize, Serialize)]
struct ToolFunctionCall {
    name: String,
    arguments: Option<String>,
}

#[derive(Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
}
