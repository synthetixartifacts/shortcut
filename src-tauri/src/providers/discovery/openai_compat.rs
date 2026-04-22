//! OpenAI-compatible model catalog fetcher.
//!
//! `GET {base}/v1/models`. Used by the Local discovery dispatcher when the
//! resolved protocol is "openai_compatible" (LM Studio, LocalAI, vLLM,
//! llama.cpp server). Vision capability is not reliably exposed by every
//! compat server; we default to false and let the user override via the
//! per-assignment vision checkbox added in Phase 4.
//!
//! The `base_url` passed here is the user-supplied Local URL, which may or
//! may not include a `/v1` or other API suffix. We run it through
//! [`normalize_local_base_url`] before appending `/v1/models` so virtually
//! any pasted example URL produces a correct request.

use super::parse_json_response;
use super::ProviderModelInfo;
use crate::errors::AppError;
use crate::providers::local::normalize_local_base_url;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct OpenAiCompatModelsResponse {
    data: Vec<OpenAiCompatModel>,
}

#[derive(Deserialize)]
struct OpenAiCompatModel {
    id: String,
}

pub(super) async fn fetch_openai_compat_models(
    client: &Client,
    base_url: &str,
    api_key: Option<&str>,
) -> Result<Vec<ProviderModelInfo>, AppError> {
    let url = format!("{}/v1/models", normalize_local_base_url(base_url));
    let mut req = client.get(&url);
    if let Some(key) = api_key.filter(|k| !k.is_empty()) {
        req = req.bearer_auth(key);
    }
    let response = req
        .send()
        .await
        .map_err(|e| AppError::ProviderError(format!("Local (OpenAI-compat) request failed: {}", e)))?;
    let parsed: OpenAiCompatModelsResponse =
        parse_json_response(response, "Local (OpenAI-compat) models").await?;

    let mut models: Vec<ProviderModelInfo> = parsed
        .data
        .into_iter()
        .map(|m| ProviderModelInfo {
            id: m.id.clone(),
            label: m.id,
            supports_vision: false,
        })
        .collect();
    models.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(models)
}

// Normalization is centralized in `providers::local::normalize_local_base_url`
// so the same suffix-stripping rules apply to every Local code path (provider
// build + discovery fetchers).
