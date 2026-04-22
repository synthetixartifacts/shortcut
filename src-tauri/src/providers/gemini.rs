//! Google Gemini LLM provider.
//!
//! Uses the Gemini generateContent / streamGenerateContent REST APIs.
//! Authentication: API key passed via `x-goog-api-key` header.
//!
//! System messages: passed via the `systemInstruction` field (v1beta+).
//! Vision: image passed as an `inline_data` part in the user content.

use crate::errors::AppError;
use crate::providers::http::{ensure_ok, read_sse, ControlFlow};
use crate::providers::{ChatRequest, EventSinkFn, ImageAttachment, LlmProvider, ProviderCapabilities};
use reqwest::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://generativelanguage.googleapis.com";

pub struct GeminiProvider {
    client: Client,
    api_key: String,
}

impl GeminiProvider {
    pub fn new(client: Client, api_key: String) -> Self {
        Self { client, api_key }
    }
}

// ---------------------------------------------------------------------------
// Response types (shared between streaming and non-streaming)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Deserialize)]
struct GeminiContent {
    #[serde(default)]
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiPart {
    #[serde(default)]
    text: String,
}

// ---------------------------------------------------------------------------
// Request body builders (using serde_json::Value for flexibility)
// ---------------------------------------------------------------------------

/// Build the Gemini request body.
/// - `system_text`: extracted from system-role messages
/// - `contents`: user/model turns
fn build_request_body(
    req: &ChatRequest,
) -> serde_json::Value {
    // Separate system messages from conversation turns
    let mut system_text = String::new();
    let mut contents: Vec<serde_json::Value> = Vec::new();

    for (i, msg) in req.messages.iter().enumerate() {
        if msg.role == "system" {
            if !system_text.is_empty() {
                system_text.push('\n');
            }
            system_text.push_str(&msg.content);
            continue;
        }

        // Map OpenAI roles to Gemini roles
        let role = if msg.role == "assistant" { "model" } else { "user" };

        let is_last_user = msg.role == "user" && i == req.messages.len() - 1;
        let parts = if is_last_user {
            if let Some(img) = &req.image {
                build_vision_parts(&msg.content, img)
            } else {
                serde_json::json!([{ "text": msg.content }])
            }
        } else {
            serde_json::json!([{ "text": msg.content }])
        };

        contents.push(serde_json::json!({ "role": role, "parts": parts }));
    }

    let mut body = serde_json::json!({ "contents": contents });

    // systemInstruction field (Gemini 1.5+)
    if !system_text.is_empty() {
        body["systemInstruction"] = serde_json::json!({
            "parts": [{ "text": system_text }]
        });
    }

    let mut gen_config = serde_json::Map::new();
    if let Some(t) = req.temperature {
        gen_config.insert("temperature".to_string(), serde_json::json!(t));
    }
    if let Some(m) = req.max_tokens {
        gen_config.insert("maxOutputTokens".to_string(), serde_json::json!(m));
    }
    if !gen_config.is_empty() {
        body["generationConfig"] = serde_json::Value::Object(gen_config);
    }

    body
}

fn build_vision_parts(text: &str, img: &ImageAttachment) -> serde_json::Value {
    serde_json::json!([
        {
            "inline_data": {
                "mime_type": img.mime_type,
                "data": img.base64
            }
        },
        { "text": text }
    ])
}

/// Extract the text from the first candidate/part in a Gemini response value.
fn extract_text_from_response(val: &serde_json::Value) -> Option<String> {
    let text = val
        .get("candidates")?
        .get(0)?
        .get("content")?
        .get("parts")?
        .get(0)?
        .get("text")?
        .as_str()?
        .to_string();
    Some(text)
}

// ---------------------------------------------------------------------------
// LlmProvider impl
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl LlmProvider for GeminiProvider {
    async fn complete(&self, req: &ChatRequest) -> Result<String, AppError> {
        let url = format!("{}/v1beta/models/{}:generateContent", BASE_URL, req.model);
        let body = build_request_body(req);

        let response = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ProviderError(format!("Gemini request failed: {}", e)))?;

        let response = ensure_ok(response, "Gemini").await?;

        let parsed: GeminiResponse = response
            .json()
            .await
            .map_err(|e| AppError::ProviderError(format!("Gemini parse error: {}", e)))?;

        parsed
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| AppError::ProviderError("Gemini returned no content".to_string()))
    }

    async fn stream(&self, req: &ChatRequest, chunk_sink: &EventSinkFn) -> Result<(), AppError> {
        let url = format!("{}/v1beta/models/{}:streamGenerateContent?alt=sse", BASE_URL, req.model);
        let body = build_request_body(req);

        let response = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ProviderError(format!("Gemini stream request failed: {}", e)))?;

        let response = ensure_ok(response, "Gemini").await?;

        // Gemini SSE: standard "data: <json>\n\n" format
        read_sse(response, |event| {
            for line in event.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(text) = extract_text_from_response(&val) {
                            if !text.is_empty() {
                                chunk_sink(&text);
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
        "gemini"
    }
}
