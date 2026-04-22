//! Soniox API request/response shapes — deserialized from the JSON payloads
//! returned by the `/v1/files`, `/v1/transcriptions` and
//! `/v1/transcriptions/{id}/transcript` endpoints.
//!
//! Extracted from `soniox_api.rs` during the PHASE 3B file-size split so the
//! HTTP surface (`mod.rs`) stays focused on operations and retry logic.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(super) struct SonioxFileResponse {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct SonioxTranscriptionResponse {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct SonioxPollResponse {
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SonioxToken {
    #[serde(default)]
    pub end_ms: i64,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SonioxTranscriptResponse {
    pub text: String,
    #[serde(default)]
    pub tokens: Vec<SonioxToken>,
}

#[derive(Debug, Serialize)]
pub(super) struct CreateTranscriptionBody<'a> {
    pub model: &'a str,
    pub file_id: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub language_hints: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub context_terms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_text: Option<String>,
}
