//! Anthropic Claude LLM provider.
//!
//! Uses the Anthropic Messages API (v2023-06-01).
//! Supports non-streaming (complete) and streaming via SSE.
//! Vision: image passed as a base64 `image` source block in the content array.

use crate::errors::AppError;
use crate::providers::http::{ensure_ok, read_sse, ControlFlow};
use crate::providers::{ChatRequest, EventSinkFn, ImageAttachment, LlmProvider, ProviderCapabilities};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_MAX_TOKENS: u32 = 4096;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(client: Client, api_key: String) -> Self {
        Self { client, api_key }
    }
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: String,
}

// SSE streaming types
#[derive(Deserialize)]
struct AnthropicStreamDelta {
    #[serde(rename = "type")]
    delta_type: String,
    #[serde(default)]
    text: String,
}

#[derive(Deserialize)]
struct AnthropicStreamData {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<AnthropicStreamDelta>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Separate system messages from user/assistant messages.
/// Anthropic uses a dedicated `system` field rather than a system role message.
fn extract_system(req: &ChatRequest) -> (Option<String>, Vec<serde_json::Value>) {
    let mut system: Option<String> = None;
    let mut messages: Vec<serde_json::Value> = Vec::with_capacity(req.messages.len());

    for (i, msg) in req.messages.iter().enumerate() {
        if msg.role == "system" {
            // Concatenate multiple system messages if present
            let current = system.get_or_insert_with(String::new);
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(&msg.content);
            continue;
        }

        let is_last_user = msg.role == "user" && i == req.messages.len() - 1;
        if is_last_user {
            if let Some(img) = &req.image {
                let content = build_vision_content(&msg.content, img);
                messages.push(serde_json::json!({ "role": "user", "content": content }));
                continue;
            }
        }
        messages.push(serde_json::json!({ "role": msg.role, "content": msg.content }));
    }
    (system, messages)
}

fn build_vision_content(text: &str, img: &ImageAttachment) -> serde_json::Value {
    serde_json::json!([
        {
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": img.mime_type,
                "data": img.base64
            }
        },
        { "type": "text", "text": text }
    ])
}

// ---------------------------------------------------------------------------
// LlmProvider impl
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, req: &ChatRequest) -> Result<String, AppError> {
        let (system, messages) = extract_system(req);
        let max_tokens = req.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        let body = AnthropicRequest {
            model: &req.model,
            max_tokens,
            messages,
            system,
            temperature: req.temperature,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", BASE_URL))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ProviderError(format!("Anthropic request failed: {}", e)))?;

        let response = ensure_ok(response, "Anthropic").await?;

        let parsed: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AppError::ProviderError(format!("Anthropic parse error: {}", e)))?;

        parsed
            .content
            .into_iter()
            .find(|b| b.block_type == "text")
            .map(|b| b.text)
            .ok_or_else(|| AppError::ProviderError("Anthropic returned no text content".to_string()))
    }

    async fn stream(&self, req: &ChatRequest, chunk_sink: &EventSinkFn) -> Result<(), AppError> {
        let (system, messages) = extract_system(req);
        let max_tokens = req.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        let body = AnthropicRequest {
            model: &req.model,
            max_tokens,
            messages,
            system,
            temperature: req.temperature,
            stream: true,
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", BASE_URL))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ProviderError(format!("Anthropic stream request failed: {}", e)))?;

        let response = ensure_ok(response, "Anthropic").await?;

        // Anthropic SSE: each event is "event: <type>\ndata: <json>\n\n"
        read_sse(response, |event_block| {
            let mut data_line: Option<&str> = None;
            for line in event_block.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    data_line = Some(data);
                }
            }

            if let Some(data) = data_line {
                if let Ok(parsed) = serde_json::from_str::<AnthropicStreamData>(data) {
                    if parsed.event_type == "content_block_delta" {
                        if let Some(delta) = parsed.delta {
                            if delta.delta_type == "text_delta" && !delta.text.is_empty() {
                                chunk_sink(&delta.text);
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
        "anthropic"
    }
}
