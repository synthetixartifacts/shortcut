//! Ollama LLM provider (local inference).
//!
//! Ollama runs locally and requires no API key.
//! API: POST the configured chat endpoint with `stream: false` or `stream: true`.
//! Non-streaming: single JSON response `{"message":{"content":"..."},...}`.
//! Streaming: newline-delimited JSON, each line is `{"message":{"content":"..."},"done":false}`.
//!
//! Vision (LLaVA-style models): images are provided as `"images": [base64_no_prefix]`
//! on the message object — NOT in the content array like OpenAI/Anthropic.

use crate::errors::AppError;
use crate::providers::http::{ensure_ok, read_ndjson, truncate_preview, ControlFlow};
use crate::providers::{ChatRequest, EventSinkFn, LlmProvider, ProviderCapabilities};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_LOCAL_CHAT_URL: &str = "http://localhost:11434/api/chat";

pub struct OllamaProvider {
    client: Client,
    chat_url: String,
}

impl OllamaProvider {
    pub fn new(client: Client, chat_url: String) -> Self {
        Self {
            client,
            chat_url: normalize_local_chat_url(&chat_url),
        }
    }
}

pub fn normalize_local_chat_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return DEFAULT_LOCAL_CHAT_URL.to_string();
    }

    let output = normalize_local_chat_url_inner(trimmed);
    // Log only when normalization actually rewrote the input so the default
    // zero-config path stays silent. Helps surface silent surprises ("I typed
    // `localhost:11434` and it became `http://localhost:11434/api/chat`").
    if output != trimmed {
        log::info!("Ollama chat URL normalized: {} -> {}", trimmed, output);
    }
    output
}

fn normalize_local_chat_url_inner(trimmed: &str) -> String {
    let mut parsed = match reqwest::Url::parse(trimmed) {
        Ok(parsed) => parsed,
        Err(_) => return trimmed.trim_end_matches('/').to_string(),
    };

    let path = parsed.path().trim_end_matches('/').to_string();
    match path.as_str() {
        "" | "/" => {
            parsed.set_path("/api/chat");
            parsed.set_query(None);
            parsed.set_fragment(None);
            parsed.to_string()
        }
        "/api" => {
            parsed.set_path("/api/chat");
            parsed.to_string()
        }
        _ => {
            parsed.set_path(&path);
            parsed.to_string()
        }
    }
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessageContent,
}

#[derive(Deserialize)]
struct OllamaMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct OllamaStreamChunk {
    #[serde(default)]
    message: Option<OllamaMessageContent>,
    #[serde(default)]
    done: bool,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_options(req: &ChatRequest) -> Option<OllamaOptions> {
    if req.temperature.is_some() || req.max_tokens.is_some() {
        Some(OllamaOptions {
            temperature: req.temperature,
            num_predict: req.max_tokens,
        })
    } else {
        None
    }
}

fn format_transport_err(ctx: &str, url: &str, e: reqwest::Error) -> AppError {
    AppError::ProviderError(format!("Ollama {ctx} {url} failed: {e}"))
}

fn build_messages(req: &ChatRequest) -> Vec<OllamaMessage> {
    let mut out = Vec::with_capacity(req.messages.len());

    for (i, msg) in req.messages.iter().enumerate() {
        let is_last_user = msg.role == "user" && i == req.messages.len() - 1;

        let images = if is_last_user {
            req.image.as_ref().map(|img| vec![img.base64.clone()])
        } else {
            None
        };

        out.push(OllamaMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
            images,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// LlmProvider impl
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(&self, req: &ChatRequest) -> Result<String, AppError> {
        let body = OllamaRequest {
            model: &req.model,
            messages: build_messages(req),
            stream: false,
            options: build_options(req),
        };

        let response = self
            .client
            .post(&self.chat_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format_transport_err("POST", &self.chat_url, e))?;

        let response = ensure_ok(response, "Ollama").await?;

        // Read to String first so a parse error can include a prefix of the
        // body — helps tell an OpenAI-compat response shape apart from truly
        // malformed bytes when debugging Local dispatch.
        let text = response.text().await.map_err(|e| {
            AppError::ProviderError(format!("Ollama body read error at {}: {}", self.chat_url, e))
        })?;
        let parsed: OllamaResponse = serde_json::from_str(&text).map_err(|e| {
            let preview = truncate_preview(&text, 200);
            AppError::ProviderError(format!(
                "Ollama parse error at {}: {} — body prefix: {}",
                self.chat_url,
                e,
                if preview.is_empty() { "(empty)".to_string() } else { preview }
            ))
        })?;

        Ok(parsed.message.content)
    }

    async fn stream(&self, req: &ChatRequest, chunk_sink: &EventSinkFn) -> Result<(), AppError> {
        let body = OllamaRequest {
            model: &req.model,
            messages: build_messages(req),
            stream: true,
            options: build_options(req),
        };

        let response = self
            .client
            .post(&self.chat_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format_transport_err("stream POST", &self.chat_url, e))?;

        let response = ensure_ok(response, "Ollama").await?;

        // Ollama streaming: newline-delimited JSON. read_ndjson buffers bytes
        // across TCP chunks, only deserializes complete lines, and fixes the
        // UTF-8 split-codepoint corruption that previously affected this loop.
        read_ndjson::<_, OllamaStreamChunk>(response, |chunk| {
            if let Some(msg) = chunk.message {
                if !msg.content.is_empty() {
                    chunk_sink(&msg.content);
                }
            }
            if chunk.done {
                Ok(ControlFlow::Break)
            } else {
                Ok(ControlFlow::Continue)
            }
        })
        .await
    }

    fn capabilities(&self) -> ProviderCapabilities {
        // Vision support is per-model in Ollama (llava, llama3.2-vision, etc.).
        // The per-assignment TaskAssignment.supports_vision is the source of
        // truth — see providers/mod.rs::stream_screen_question.
        ProviderCapabilities { supports_streaming: true, supports_vision: false }
    }

    fn provider_id(&self) -> &'static str {
        "ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_empty_returns_default() {
        assert_eq!(normalize_local_chat_url(""), DEFAULT_LOCAL_CHAT_URL);
        assert_eq!(normalize_local_chat_url("   "), DEFAULT_LOCAL_CHAT_URL);
    }

    #[test]
    fn normalize_bare_host_adds_api_chat() {
        assert_eq!(
            normalize_local_chat_url("http://localhost:11434"),
            "http://localhost:11434/api/chat"
        );
        assert_eq!(
            normalize_local_chat_url("http://localhost:11434/"),
            "http://localhost:11434/api/chat"
        );
    }

    #[test]
    fn normalize_api_root_adds_chat() {
        assert_eq!(
            normalize_local_chat_url("http://localhost:11434/api"),
            "http://localhost:11434/api/chat"
        );
        assert_eq!(
            normalize_local_chat_url("http://localhost:11434/api/"),
            "http://localhost:11434/api/chat"
        );
    }

    #[test]
    fn normalize_preserves_full_chat_path() {
        assert_eq!(
            normalize_local_chat_url("http://localhost:11434/api/chat"),
            "http://localhost:11434/api/chat"
        );
    }

    #[test]
    fn normalize_passes_through_non_default_paths() {
        let out = normalize_local_chat_url("http://localhost:11434/v1/chat/completions");
        assert!(out.ends_with("/v1/chat/completions"));
    }

    #[test]
    fn normalize_invalid_url_trimmed() {
        assert_eq!(normalize_local_chat_url("not a url"), "not a url");
        assert_eq!(normalize_local_chat_url("garbage/"), "garbage");
    }

    #[test]
    fn normalize_different_host_and_port() {
        assert_eq!(
            normalize_local_chat_url("http://10.0.0.5:8080"),
            "http://10.0.0.5:8080/api/chat"
        );
    }
}
