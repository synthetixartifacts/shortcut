//! Grok LLM provider (xAI).
//!
//! Grok exposes an OpenAI-compatible API at https://api.x.ai.
//! This wrapper delegates everything to OpenAiProvider with a custom base URL,
//! reusing all request building and SSE parsing logic from openai.rs.

use crate::errors::AppError;
use crate::providers::openai::OpenAiProvider;
use crate::providers::{ChatRequest, EventSinkFn, LlmProvider, ProviderCapabilities};
use reqwest::Client;

const GROK_BASE_URL: &str = "https://api.x.ai";

pub struct GrokProvider(OpenAiProvider);

impl GrokProvider {
    pub fn new(client: Client, api_key: String) -> Self {
        Self(OpenAiProvider::new(client, api_key, Some(GROK_BASE_URL.to_string())))
    }
}

#[async_trait::async_trait]
impl LlmProvider for GrokProvider {
    async fn complete(&self, req: &ChatRequest) -> Result<String, AppError> {
        self.0.complete(req).await
    }

    async fn stream(&self, req: &ChatRequest, chunk_sink: &EventSinkFn) -> Result<(), AppError> {
        self.0.stream(req, chunk_sink).await
    }

    fn capabilities(&self) -> ProviderCapabilities {
        // xAI text models do not accept images at the chat/completions endpoint. PHASE 5 reintroduces vision per-model via TaskAssignment.
        ProviderCapabilities { supports_streaming: true, supports_vision: false }
    }

    fn provider_id(&self) -> &'static str {
        "grok"
    }
}
