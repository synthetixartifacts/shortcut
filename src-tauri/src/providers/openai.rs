//! OpenAI LLM provider.
//!
//! Supports non-streaming (complete) and streaming via SSE.
//! Vision is handled by building a multi-part content array when req.image is set.
//!
//! This struct is also reused by grok.rs (same OpenAI-compatible API, different base URL).

use crate::errors::AppError;
use crate::providers::http::{ensure_ok, read_sse, truncate_preview, ControlFlow};
use crate::providers::{ChatRequest, EventSinkFn, ImageAttachment, LlmProvider, ProviderCapabilities};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(client: Client, api_key: String, base_url: Option<String>) -> Self {
        Self {
            client,
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct OpenAiRequestBody<'a> {
    model: &'a str,
    messages: Vec<serde_json::Value>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    content: String,
}

// SSE streaming delta
#[derive(Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiDelta,
}

#[derive(Deserialize, Default)]
struct OpenAiDelta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiStreamChunk {
    #[serde(default)]
    choices: Vec<OpenAiStreamChoice>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build the `messages` array for the OpenAI API.
///
/// If `req.image` is set, the last user message is augmented with a vision
/// content array: image_url part first, then the text part.
fn build_messages(req: &ChatRequest) -> Vec<serde_json::Value> {
    let mut out: Vec<serde_json::Value> = Vec::with_capacity(req.messages.len());

    for (i, msg) in req.messages.iter().enumerate() {
        let is_last_user = msg.role == "user" && i == req.messages.len() - 1;

        if is_last_user {
            if let Some(img) = &req.image {
                // Vision path: content array with image + text
                let content = build_vision_content(&msg.content, img);
                out.push(serde_json::json!({ "role": msg.role, "content": content }));
                continue;
            }
        }
        out.push(serde_json::json!({ "role": msg.role, "content": msg.content }));
    }
    out
}

fn build_vision_content(text: &str, img: &ImageAttachment) -> serde_json::Value {
    serde_json::json!([
        {
            "type": "image_url",
            "image_url": {
                "url": format!("data:{};base64,{}", img.mime_type, img.base64)
            }
        },
        { "type": "text", "text": text }
    ])
}

// ---------------------------------------------------------------------------
// LlmProvider impl
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, req: &ChatRequest) -> Result<String, AppError> {
        let messages = build_messages(req);
        let body = OpenAiRequestBody {
            model: &req.model,
            messages,
            stream: false,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
        };

        let endpoint = format!("{}/v1/chat/completions", self.base_url);
        let response = self
            .client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                AppError::ProviderError(format!("OpenAI POST {} failed: {}", endpoint, e))
            })?;

        let response = ensure_ok(response, "OpenAI").await?;

        // Read body first so parse errors carry a prefix — crucial for
        // OpenAI-compatible Local endpoints (LM Studio et al.) where a parse
        // mismatch usually means the server returned a different envelope
        // shape than the OpenAI canon.
        let text = response.text().await.map_err(|e| {
            AppError::ProviderError(format!("OpenAI body read error at {}: {}", endpoint, e))
        })?;
        let parsed: OpenAiResponse = serde_json::from_str(&text).map_err(|e| {
            let preview = truncate_preview(&text, 200);
            AppError::ProviderError(format!(
                "OpenAI parse error at {}: {} — body prefix: {}",
                endpoint,
                e,
                if preview.is_empty() { "(empty)".to_string() } else { preview }
            ))
        })?;

        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| {
                AppError::ProviderError(format!(
                    "OpenAI returned empty choices at {}",
                    endpoint
                ))
            })
    }

    async fn stream(&self, req: &ChatRequest, chunk_sink: &EventSinkFn) -> Result<(), AppError> {
        let messages = build_messages(req);
        let body = OpenAiRequestBody {
            model: &req.model,
            messages,
            stream: true,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
        };

        let endpoint = format!("{}/v1/chat/completions", self.base_url);
        let response = self
            .client
            .post(&endpoint)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                AppError::ProviderError(format!("OpenAI stream POST {} failed: {}", endpoint, e))
            })?;

        let response = ensure_ok(response, "OpenAI").await?;

        read_sse(response, |event| {
            for line in event.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        return Ok(ControlFlow::Break);
                    }
                    if let Ok(chunk) = serde_json::from_str::<OpenAiStreamChunk>(data) {
                        for choice in chunk.choices {
                            if let Some(text) = choice.delta.content {
                                if !text.is_empty() {
                                    chunk_sink(&text);
                                }
                            }
                        }
                    }
                }
            }
            Ok(ControlFlow::Continue)
        })
        .await
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities { supports_streaming: true, supports_vision: true }
    }

    fn provider_id(&self) -> &'static str {
        "openai"
    }
}
