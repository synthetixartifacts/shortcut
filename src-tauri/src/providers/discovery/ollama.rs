//! Ollama model catalog fetcher.
//!
//! Lists local tags via `/api/tags` then queries `/api/show` for each one to
//! detect vision support. The per-tag `show` calls are fanned out via
//! `join_all` — unbounded parallelism is acceptable because Ollama is local.

use super::parse_json_response;
use super::ProviderModelInfo;
use crate::errors::AppError;
use crate::providers::ollama::normalize_local_chat_url;
use futures_util::future::join_all;
use reqwest::{Client, Url};
use serde::Deserialize;

#[derive(Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaTagModel>,
}

#[derive(Deserialize)]
struct OllamaTagModel {
    name: String,
}

#[derive(Deserialize)]
struct OllamaShowResponse {
    #[serde(default)]
    capabilities: Vec<String>,
}

pub(super) async fn fetch_ollama_models(
    client: &Client,
    chat_url: &str,
) -> Result<Vec<ProviderModelInfo>, AppError> {
    let tags_url = derive_ollama_api_url(chat_url, "/api/tags")?;
    let show_url = derive_ollama_api_url(chat_url, "/api/show")?;
    let response = client.get(tags_url).send().await?;
    let parsed: OllamaTagsResponse = parse_json_response(response, "Local models").await?;

    let fetches = parsed.models.into_iter().map(|model| {
        let client = client.clone();
        let show_url = show_url.clone();
        let model_name = model.name;
        async move {
            let supports_vision = match client
                .post(show_url)
                .json(&serde_json::json!({ "model": model_name.clone() }))
                .send()
                .await
            {
                Ok(response) => match parse_json_response::<OllamaShowResponse>(
                    response,
                    "Local model details",
                )
                .await
                {
                    Ok(details) => details
                        .capabilities
                        .iter()
                        .any(|capability| capability == "vision"),
                    Err(_) => false,
                },
                Err(_) => false,
            };

            ProviderModelInfo {
                id: model_name.clone(),
                label: model_name,
                supports_vision,
            }
        }
    });

    let mut models = join_all(fetches).await;
    models.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(models)
}

fn derive_ollama_api_url(chat_url: &str, path: &str) -> Result<Url, AppError> {
    let normalized = normalize_local_chat_url(chat_url);
    let mut parsed = Url::parse(&normalized)
        .map_err(|e| AppError::ProviderError(format!("Invalid local URL: {}", e)))?;
    parsed.set_path(path);
    parsed.set_query(None);
    parsed.set_fragment(None);
    Ok(parsed)
}
