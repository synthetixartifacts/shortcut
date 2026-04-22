//! Anthropic model catalog fetcher.

use super::parse_json_response;
use super::ProviderModelInfo;
use crate::errors::AppError;
use reqwest::Client;
use serde::Deserialize;

const ANTHROPIC_API_VERSION: &str = "2023-06-01";

#[derive(Deserialize)]
struct AnthropicModelsResponse {
    data: Vec<AnthropicModel>,
}

#[derive(Deserialize)]
struct AnthropicModel {
    id: String,
    display_name: String,
}

pub(super) async fn fetch_anthropic_models(
    client: &Client,
    api_key: &str,
) -> Result<Vec<ProviderModelInfo>, AppError> {
    let response = client
        .get("https://api.anthropic.com/v1/models?limit=1000")
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_API_VERSION)
        .send()
        .await?;
    let parsed: AnthropicModelsResponse = parse_json_response(response, "Anthropic models").await?;
    Ok(parsed
        .data
        .into_iter()
        .map(|model| ProviderModelInfo {
            label: format!("{} ({})", model.display_name, model.id),
            id: model.id,
            supports_vision: true,
        })
        .collect())
}
