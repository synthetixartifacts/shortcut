//! Provider model-discovery command.
//!
//! Single Tauri command (`get_provider_models`) that dispatches to the
//! per-provider fetcher and returns a unified [`ProviderModelInfo`] list for
//! the frontend model picker.

mod anthropic;
mod filters;
mod gemini;
mod local;
mod ollama;
mod openai;
mod openai_compat;
mod xai;

use crate::config::ConfigState;
use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize)]
pub struct ProviderModelInfo {
    pub id: String,
    pub label: String,
    pub supports_vision: bool,
}

#[tauri::command]
pub async fn get_provider_models(
    app: AppHandle,
    provider_id: String,
) -> Result<Vec<ProviderModelInfo>, String> {
    let client = app.state::<reqwest::Client>().inner().clone();
    let creds = {
        let state = app.state::<ConfigState>();
        let config = state
            .0
            .lock()
            .map_err(|e| format!("Config lock poisoned: {}", e))?;
        config.providers.credentials.clone()
    };

    let models = match provider_id.as_str() {
        "openai" if creds.openai_api_key.is_empty() => {
            return Err("OpenAI API key not configured.".to_string());
        }
        "openai" => openai::fetch_openai_models(&client, &creds.openai_api_key).await,
        "anthropic" if creds.anthropic_api_key.is_empty() => {
            return Err("Anthropic API key not configured.".to_string());
        }
        "anthropic" => {
            anthropic::fetch_anthropic_models(&client, &creds.anthropic_api_key).await
        }
        "gemini" if creds.gemini_api_key.is_empty() => {
            return Err("Gemini API key not configured.".to_string());
        }
        "gemini" => gemini::fetch_gemini_models(&client, &creds.gemini_api_key).await,
        "grok" if creds.grok_api_key.is_empty() => {
            return Err("xAI API key not configured.".to_string());
        }
        "grok" => xai::fetch_xai_models(&client, &creds.grok_api_key).await,
        // Local: protocol-aware dispatcher races ollama + openai-compat probes
        // when `protocol == "auto"` and caches the winner into
        // `local.detected_protocol` (see discovery/local.rs).
        "local" => local::fetch_local_models(&app, &client).await,
        _ => return Err(format!("Unknown provider: {}", provider_id)),
    };

    models.map_err(|e| e.to_string())
}

/// Parse a JSON response from a provider's discovery endpoint, mapping non-2xx
/// responses and parse errors into [`AppError::ProviderError`].
///
/// Shared by every `discovery/*` submodule. Kept here instead of in
/// `providers/http.rs` because (a) its shape — return `T: DeserializeOwned` —
/// is discovery-specific and (b) moving it to `http.rs` would widen that
/// module's surface without another caller. If a second consumer appears, the
/// helper migrates upstream.
async fn parse_json_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
    context: &str,
) -> Result<T, AppError> {
    let url = response.url().to_string();
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        log::debug!("{} error body ({}) at {}: {}", context, status, url, text);
        let preview = super::http::truncate_preview(&text, 200);
        return Err(AppError::ProviderError(format!(
            "{} GET {} failed: HTTP {} — body: {}",
            context,
            url,
            status.as_u16(),
            if preview.is_empty() { "(empty)".to_string() } else { preview }
        )));
    }

    // Read to string so parse errors can include a prefix of the body —
    // essential for Local OpenAI-compat endpoints where a 2xx catchall page
    // (HTML) lands here instead of JSON.
    let text = response.text().await.map_err(|e| {
        AppError::ProviderError(format!("{} body read error at {}: {}", context, url, e))
    })?;
    serde_json::from_str::<T>(&text).map_err(|e| {
        let preview = super::http::truncate_preview(&text, 200);
        AppError::ProviderError(format!(
            "{} parse error at {}: {} — body prefix: {}",
            context,
            url,
            e,
            if preview.is_empty() { "(empty)".to_string() } else { preview }
        ))
    })
}

// Make `Deserialize` trait object accessible without forcing each submodule to
// re-import serde. Keeps per-submodule `use serde::Deserialize;` lines local.
#[allow(dead_code)]
fn _assert_deserialize<T: Deserialize<'static>>() {}
