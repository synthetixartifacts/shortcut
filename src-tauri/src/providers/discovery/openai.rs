//! OpenAI model catalog fetcher.

use super::filters::{has_snapshot_alias, is_openai_shortcut_model};
use super::parse_json_response;
use super::ProviderModelInfo;
use crate::errors::AppError;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Deserialize)]
struct OpenAiModel {
    id: String,
    #[serde(default)]
    created: i64,
}

/// GET `https://api.openai.com/v1/models`, filter to the text family, and sort
/// newest-first. Dated-snapshot ids are dropped when their base alias is also
/// available so the picker only shows the stable alias.
pub(super) async fn fetch_openai_models(
    client: &Client,
    api_key: &str,
) -> Result<Vec<ProviderModelInfo>, AppError> {
    let response = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(api_key)
        .send()
        .await?;
    let parsed: OpenAiModelsResponse = parse_json_response(response, "OpenAI models").await?;
    let ids = parsed
        .data
        .into_iter()
        .filter_map(|model| {
            if !is_openai_shortcut_model(&model.id) {
                return None;
            }
            Some((model.id, model.created))
        })
        .collect::<Vec<_>>();

    let available = ids.iter().map(|(id, _)| id.clone()).collect::<HashSet<_>>();
    let mut models = ids
        .into_iter()
        .filter(|(id, _)| !has_snapshot_alias(id, &available))
        .map(|(id, created)| {
            (
                ProviderModelInfo {
                    id: id.clone(),
                    label: id,
                    supports_vision: true,
                },
                created,
            )
        })
        .collect::<Vec<_>>();
    models.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.id.cmp(&b.0.id)));
    Ok(models.into_iter().map(|(model, _)| model).collect())
}
