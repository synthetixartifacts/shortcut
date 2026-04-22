//! Provider abstraction layer for LLM and STT providers.
//!
//! Each AI provider implements the LlmProvider trait.
//! Providers are instantiated from config via get_llm_provider() and dispatched by task.
//!
//! Phase 2: LlmProvider trait + OpenAI stub + provider factory.
//! Phase 3: Full implementations for all 5 providers.
//! Phase 4: SttProvider trait + Soniox direct.

pub mod anthropic;
pub mod discovery;
pub mod gemini;
pub mod grok;
pub mod http;
pub mod local;
pub mod ollama;
pub mod openai;

use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

/// A single message in a chat conversation (shared by all providers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,    // "system", "user", "assistant"
    pub content: String,
}

/// Optional image attachment (for vision providers)
#[derive(Debug, Clone)]
pub struct ImageAttachment {
    pub base64: String,
    pub mime_type: String,  // "image/jpeg", "image/png"
}

/// A request to an LLM provider
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub image: Option<ImageAttachment>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Tauri AppHandle event sink for streaming — each call receives one text chunk
pub type EventSinkFn = Box<dyn Fn(&str) + Send + Sync>;

/// Capabilities of an LLM provider
#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    #[allow(dead_code)] // Declared by every provider impl; reserved for future UI/streaming gating.
    pub supports_streaming: bool,
    pub supports_vision: bool,
}

/// Trait implemented by each LLM provider (OpenAI, Anthropic, Gemini, Grok, Ollama)
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// Non-streaming completion — returns full response text
    async fn complete(&self, req: &ChatRequest) -> Result<String, AppError>;

    /// Streaming completion — emits chunks via the provided sink function.
    /// Each call to sink receives one chunk of text.
    async fn stream(
        &self,
        req: &ChatRequest,
        chunk_sink: &EventSinkFn,
    ) -> Result<(), AppError>;

    fn capabilities(&self) -> ProviderCapabilities;

    #[allow(dead_code)] // Part of the trait contract; used by providers for self-identification in future logging paths.
    fn provider_id(&self) -> &'static str;
}

// =============================================================================
// Provider factory
// =============================================================================

use crate::config::ConfigState;
use tauri::{AppHandle, Emitter, Manager};

/// Instantiate an LLM provider from config for a given provider_id.
///
/// Returns an error if the provider is unknown or its credentials are not configured.
/// For Local, no API key is required (it runs on the user's machine or LAN).
///
/// Note: all config data is extracted before returning — no MutexGuard escapes.
pub fn get_llm_provider(
    app: &AppHandle,
    provider_id: &str,
) -> Result<Box<dyn LlmProvider>, AppError> {
    let client = app.state::<reqwest::Client>().inner().clone();
    let state = app.state::<ConfigState>();
    let config = state.0.lock().map_err(|_| AppError::Config("Lock poisoned".to_string()))?;
    let creds = config.providers.credentials.clone();
    drop(config); // Release lock before constructing provider

    match provider_id {
        "openai" => {
            if creds.openai_api_key.is_empty() {
                return Err(AppError::ProviderError(
                    "OpenAI API key not configured. Add it in Settings > AI Providers.".to_string(),
                ));
            }
            Ok(Box::new(openai::OpenAiProvider::new(client, creds.openai_api_key, None)))
        }

        "anthropic" => {
            if creds.anthropic_api_key.is_empty() {
                return Err(AppError::ProviderError(
                    "Anthropic API key not configured. Add it in Settings > AI Providers.".to_string(),
                ));
            }
            Ok(Box::new(anthropic::AnthropicProvider::new(client, creds.anthropic_api_key)))
        }

        "gemini" => {
            if creds.gemini_api_key.is_empty() {
                return Err(AppError::ProviderError(
                    "Gemini API key not configured. Add it in Settings > AI Providers.".to_string(),
                ));
            }
            Ok(Box::new(gemini::GeminiProvider::new(client, creds.gemini_api_key)))
        }

        "grok" => {
            if creds.grok_api_key.is_empty() {
                return Err(AppError::ProviderError(
                    "Grok API key not configured. Add it in Settings > AI Providers.".to_string(),
                ));
            }
            Ok(Box::new(grok::GrokProvider::new(client, creds.grok_api_key)))
        }

        "local" => {
            // Local LLM — dispatches to either OllamaProvider or OpenAiProvider
            // (LM Studio / LocalAI / vLLM / llama.cpp server) based on the
            // resolved protocol. See `providers::local::resolve_protocol`.
            Ok(local::build(client, &creds.local))
        }

        _ => Err(AppError::ProviderError(format!("Unknown provider: {}", provider_id))),
    }
}

// =============================================================================
// Vision / Screen Question dispatch
// =============================================================================

/// Single global cancellation flag for the screen-question overlay. The overlay
/// is a single window, so one flag is sufficient. Frontend triggers it via
/// `cancel_screen_question` when the overlay closes; `stream_screen_question`
/// races it against the provider stream.
///
/// Mirrors the pattern used by `transcription::model_manager::DOWNLOAD_CANCELLED`.
fn screen_question_cancel_flag() -> &'static Arc<AtomicBool> {
    static FLAG: OnceLock<Arc<AtomicBool>> = OnceLock::new();
    FLAG.get_or_init(|| Arc::new(AtomicBool::new(false)))
}

/// Mark any in-flight screen-question stream as cancelled. Safe to call even
/// when no stream is running.
pub fn cancel_screen_question_stream() {
    screen_question_cancel_flag().store(true, Ordering::Relaxed);
    log::info!("[screen_question] cancellation requested");
}

/// Stream a screen question response using the configured vision provider.
///
/// Looks up the `screen_question` task assignment from config, validates that
/// the selected provider supports vision, then streams the response via the
/// provider's `stream()` method.
///
/// Emits Tauri events (stable contract):
/// - `"screen-answer-chunk"` → `{ content: String }` for each text chunk
/// - `"screen-answer-complete"` → `()` when streaming is done
/// - `"screen-answer-error"` → `{ error: String }` on failure
///
/// Note: errors are both returned AND emitted so callers can choose how to
/// handle them without the frontend missing the event.
pub async fn stream_screen_question(
    app: &AppHandle,
    image_base64: &str,
    image_mime_type: &str,
    messages: Vec<ChatMessage>,
) -> Result<(), AppError> {
    // Extract provider_id + model + per-model vision flag from config under a short-lived lock.
    // `supports_vision` carries the per-model discovery flag when known; falling
    // back to the provider-level capability preserves backward compat.
    let (provider_id, model, assignment_vision) = {
        let state = app.state::<ConfigState>();
        let config = state
            .0
            .lock()
            .map_err(|_| AppError::Config("Lock poisoned".to_string()))?;
        let assignment = &config.providers.task_assignments.screen_question;
        (
            assignment.provider_id.clone(),
            assignment.model.clone(),
            assignment.supports_vision,
        )
    };

    // Instantiate provider and validate vision capability.
    // Gate precedence (per-assignment, protocol-agnostic — Phase 4 / D5 / R5):
    //   Some(true)  → trust the discovery result OR the user's custom-model
    //                 vision checkbox; allow.
    //   Some(false) → user picked a non-vision model; reject.
    //   None        → unknown; fall back to provider-level capability.
    // For Local: Ollama discovery sets the flag via `/api/show`; OpenAI-compat
    // discovery defaults to false, so the user opts custom ids in via the UI.
    let provider = get_llm_provider(app, &provider_id)?;
    let vision_allowed = match assignment_vision {
        Some(flag) => flag,
        None => provider.capabilities().supports_vision,
    };
    if !vision_allowed {
        let msg = format!(
            "Model '{}' on provider '{}' does not support vision. \
             Screen Question requires a vision-capable model. \
             Pick a vision-capable model in Settings.",
            model, provider_id
        );
        let _ = app.emit("screen-answer-error", serde_json::json!({ "error": msg }));
        return Err(AppError::ProviderError(msg));
    }

    log::info!(
        "[screen_question] provider={} model={} messages={} image_bytes={}",
        provider_id,
        model,
        messages.len(),
        image_base64.len(),
    );

    // Mirror the composed prompt into the debug panel when debug is enabled
    // so users can verify screen_question system prompt edits take effect.
    let debug_on = app
        .state::<ConfigState>()
        .0
        .lock()
        .map(|c| c.app_settings.debug_enabled)
        .unwrap_or(false);
    if debug_on {
        let pretty = serde_json::to_string_pretty(&messages)
            .unwrap_or_else(|_| "<serialize error>".to_string());
        let msg = format!(
            "[screen_question] → {}/{} (image {} bytes)\n{}",
            provider_id, model, image_base64.len(), pretty
        );
        let _ = app.emit("debug-log", serde_json::json!({ "level": "info", "message": msg }));
    }

    let req = ChatRequest {
        model,
        messages,
        image: Some(ImageAttachment {
            base64: image_base64.to_string(),
            mime_type: image_mime_type.to_string(),
        }),
        max_tokens: None,
        temperature: Some(0.7),
    };

    // Reset the cancel flag for this run. Any emit attempt from the chunk sink
    // short-circuits when the flag flips (best-effort — the provider's
    // underlying stream also checks the flag via `read_sse_cancellable`
    // in a follow-up wiring step).
    let cancel = screen_question_cancel_flag().clone();
    cancel.store(false, Ordering::Relaxed);

    // Build the chunk sink — each call emits one screen-answer-chunk event,
    // skipping emission when the flag is set so the frontend stops seeing
    // output immediately after the overlay closes.
    let app_clone = app.clone();
    let cancel_for_sink = cancel.clone();
    let chunk_sink: EventSinkFn = Box::new(move |chunk| {
        if cancel_for_sink.load(Ordering::Relaxed) {
            return;
        }
        let _ = app_clone.emit(
            "screen-answer-chunk",
            serde_json::json!({ "content": chunk }),
        );
    });

    // Race the provider stream against the cancel flag. We don't have a way
    // to abort the underlying TCP read from outside the trait without a
    // signature change, so we use a periodic check + `tokio::select!`. The
    // sink above also drops emits once cancelled so the UI stops updating
    // immediately even before the stream future exits.
    let stream_fut = provider.stream(&req, &chunk_sink);
    let cancel_for_watch = cancel.clone();
    let watcher = async move {
        loop {
            if cancel_for_watch.load(Ordering::Relaxed) {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    };

    let result = tokio::select! {
        biased;
        res = stream_fut => res,
        _ = watcher => {
            log::info!("[screen_question] stream aborted by overlay close");
            let _ = app.emit("screen-answer-complete", ());
            return Ok(());
        }
    };

    if let Err(e) = &result {
        let _ = app.emit(
            "screen-answer-error",
            serde_json::json!({ "error": e.to_string() }),
        );
    }
    result?;

    let _ = app.emit("screen-answer-complete", ());
    log::info!("[screen_question] stream complete");
    Ok(())
}
