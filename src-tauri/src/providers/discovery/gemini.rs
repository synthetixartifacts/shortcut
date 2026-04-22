//! Gemini model catalog fetcher.

use super::filters::has_snapshot_suffix;
use super::parse_json_response;
use super::ProviderModelInfo;
use crate::errors::AppError;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
struct GeminiModelsResponse {
    #[serde(default)]
    models: Vec<GeminiModel>,
}

#[derive(Deserialize)]
struct GeminiModel {
    name: String,
    #[serde(default, rename = "baseModelId")]
    base_model_id: String,
    #[serde(default, rename = "displayName")]
    display_name: String,
    #[serde(default, rename = "supportedGenerationMethods")]
    supported_generation_methods: Vec<String>,
}

pub(super) async fn fetch_gemini_models(
    client: &Client,
    api_key: &str,
) -> Result<Vec<ProviderModelInfo>, AppError> {
    let response = client
        .get("https://generativelanguage.googleapis.com/v1beta/models")
        .header("x-goog-api-key", api_key)
        .send()
        .await?;
    let parsed: GeminiModelsResponse = parse_json_response(response, "Gemini models").await?;

    let mut seen = HashSet::new();
    Ok(parsed
        .models
        .into_iter()
        .filter_map(|model| {
            if !model
                .supported_generation_methods
                .iter()
                .any(|method| method == "generateContent")
            {
                return None;
            }

            let model_id = preferred_gemini_model_id(&model);
            let lower = model_id.to_ascii_lowercase();
            if ["embed", "embedding", "aqa", "tts", "image"]
                .iter()
                .any(|term| lower.contains(term))
            {
                return None;
            }
            if !seen.insert(model_id.clone()) {
                return None;
            }

            let label = if model.display_name.is_empty() {
                model_id.clone()
            } else {
                format!("{} ({})", model.display_name, model_id)
            };
            Some(ProviderModelInfo {
                id: model_id,
                label,
                supports_vision: true,
            })
        })
        .collect())
}

/// Gemini: strip a dated-snapshot suffix when the base alias is declared, so the
/// picker surfaces the stable alias rather than a specific snapshot.
fn preferred_gemini_model_id(model: &GeminiModel) -> String {
    let full_name = model.name.trim_start_matches("models/");
    if !model.base_model_id.is_empty()
        && has_snapshot_suffix(full_name.strip_prefix(&model.base_model_id).unwrap_or_default())
    {
        return model.base_model_id.clone();
    }
    full_name.to_string()
}
