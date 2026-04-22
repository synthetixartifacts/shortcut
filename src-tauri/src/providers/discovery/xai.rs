//! xAI (Grok) model catalog fetcher.

use super::filters::preferred_xai_name;
use super::parse_json_response;
use super::ProviderModelInfo;
use crate::errors::AppError;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
struct XaiModelsResponse {
    #[serde(default)]
    models: Vec<XaiModel>,
}

#[derive(Deserialize)]
struct XaiModel {
    id: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    input_modalities: Vec<String>,
    #[serde(default)]
    output_modalities: Vec<String>,
}

pub(super) async fn fetch_xai_models(
    client: &Client,
    api_key: &str,
) -> Result<Vec<ProviderModelInfo>, AppError> {
    let response = client
        .get("https://api.x.ai/v1/language-models")
        .bearer_auth(api_key)
        .send()
        .await?;
    let parsed: XaiModelsResponse = parse_json_response(response, "xAI models").await?;

    let mut seen = HashSet::new();
    Ok(parsed
        .models
        .into_iter()
        .filter_map(|model| {
            if !model.output_modalities.iter().any(|item| item == "text") {
                return None;
            }
            if !model.input_modalities.iter().any(|item| item == "text") {
                return None;
            }

            let value = preferred_xai_name(&model.id, &model.aliases);
            if !seen.insert(value.clone()) {
                return None;
            }

            let label = if value == model.id {
                value.clone()
            } else {
                format!("{} ({})", value, model.id)
            };
            Some(ProviderModelInfo {
                id: value,
                label,
                supports_vision: model.input_modalities.iter().any(|item| item == "image"),
            })
        })
        .collect())
}
