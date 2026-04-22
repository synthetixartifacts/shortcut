//! Soniox HTTP operations — low-level API calls used by soniox_provider.rs.
//!
//! Response/request types live in `types.rs`; this module hosts the three HTTP
//! operations (upload, create, poll) plus transient-error classification.

mod types;

use crate::transcription::utils::{build_multipart_body, is_transient_error, TRANSCRIBE_MAX_ATTEMPTS};
use reqwest::Client;
use tauri::{AppHandle, Emitter};
use types::{
    CreateTranscriptionBody, SonioxFileResponse, SonioxPollResponse,
    SonioxTranscriptResponse, SonioxTranscriptionResponse,
};

/// Maximum number of 500 ms polls before giving up. 240 × 500 ms = 120 s.
const MAX_POLL_ATTEMPTS: u32 = 240;
const POLL_INTERVAL_MS: u64 = 500;

/// Default Soniox async transcription model. Required field per Soniox API.
const SONIOX_ASYNC_MODEL: &str = "stt-async-v4";

/// Classify reqwest errors that should be retried (connection resets, timeouts, HTTP/2 GOAWAY, EOFs).
fn is_transient_reqwest_error(e: &reqwest::Error) -> bool {
    use std::error::Error as _;
    if e.is_timeout() || e.is_connect() {
        return true;
    }
    // Walk the error source chain for hyper/h2 stream errors and IO errors.
    let mut src: Option<&dyn std::error::Error> = e.source();
    while let Some(err) = src {
        let msg = err.to_string().to_lowercase();
        if msg.contains("connection reset")
            || msg.contains("connection closed")
            || msg.contains("broken pipe")
            || msg.contains("unexpected eof")
            || msg.contains("stream closed")
            || msg.contains("goaway")
        {
            return true;
        }
        src = err.source();
    }
    false
}

/// Upload audio to Soniox /v1/files with retry on transient errors.
///
/// Retry loop: up to TRANSCRIBE_MAX_ATTEMPTS attempts with increasing back-off.
pub async fn upload_file_with_retry(
    app: &AppHandle,
    client: &Client,
    api_key: &str,
    base_url: &str,
    audio_bytes: &[u8],
    mime_type: &str,
) -> Result<String, String> {
    let upload_url = format!("{}/v1/files", base_url);
    let empty: Vec<String> = vec![];
    let mut last_error = String::new();

    for attempt in 1..=TRANSCRIBE_MAX_ATTEMPTS {
        if attempt > 1 {
            let delay_ms = 500 * (attempt - 1) as u64;
            log::warn!("Soniox upload: waiting {}ms before retry attempt {}...", delay_ms, attempt);
            let _ = app.emit(
                "transcribe-retry",
                serde_json::json!({
                    "attempt": attempt,
                    "maxAttempts": TRANSCRIBE_MAX_ATTEMPTS,
                    "delayMs": delay_ms,
                    "error": &last_error
                }),
            );
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        }

        let (content_type, body) =
            build_multipart_body(audio_bytes, mime_type, &empty, &empty, None)?;

        let body_len = body.len();
        let _ = app.emit(
            "transcribe-diagnostic",
            serde_json::json!({
                "stage": "request",
                "audioBytes": audio_bytes.len(),
                "contentLength": body_len
            }),
        );

        let result = client
            .post(&upload_url)
            .bearer_auth(api_key)
            .header("Content-Type", &content_type)
            .body(body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await;

        match result {
            Ok(response) => {
                let status = response.status();
                let body_text = response.text().await.unwrap_or_default();

                if status.is_success() {
                    let parsed: SonioxFileResponse = serde_json::from_str(&body_text)
                        .map_err(|e| format!("Failed to parse file upload response: {} — body: {}", e, body_text))?;
                    if attempt > 1 {
                        log::info!("Soniox upload succeeded on attempt {}", attempt);
                    }
                    log::info!("Soniox file uploaded: id={}", parsed.id);
                    return Ok(parsed.id);
                } else if is_transient_error(status.as_u16(), &body_text) && attempt < TRANSCRIBE_MAX_ATTEMPTS {
                    last_error = format!("Soniox upload error ({}): {}", status, body_text);
                    continue;
                } else {
                    return Err(format!("Soniox upload error ({}) after {} attempts: {}", status, attempt, body_text));
                }
            }
            Err(e) => {
                if is_transient_reqwest_error(&e) && attempt < TRANSCRIBE_MAX_ATTEMPTS {
                    last_error = format!("Soniox upload network error: {}", e);
                    log::warn!("Soniox upload transient network error on attempt {}: {}", attempt, e);
                    continue;
                }
                return Err(format!("Soniox upload request failed: {}", e));
            }
        }
    }

    Err(format!("{} (after {} attempts)", last_error, TRANSCRIBE_MAX_ATTEMPTS))
}

/// POST /v1/transcriptions — create a transcription job for the uploaded file.
pub async fn create_transcription(
    client: &Client,
    api_key: &str,
    base_url: &str,
    file_id: &str,
    language_hints: &[String],
    context_terms: &[String],
    context_text: Option<&str>,
) -> Result<String, String> {
    let url = format!("{}/v1/transcriptions", base_url);
    let body = CreateTranscriptionBody {
        model: SONIOX_ASYNC_MODEL,
        file_id,
        language_hints: language_hints.to_vec(),
        context_terms: context_terms.to_vec(),
        background_text: context_text.filter(|t| !t.is_empty()).map(|t| t.to_string()),
    };

    log::info!("Soniox: creating transcription for file_id={}, lang_hints={:?}", file_id, language_hints);

    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Soniox create transcription request failed: {}", e))?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Soniox create transcription error ({}): {}", status, body_text));
    }

    let parsed: SonioxTranscriptionResponse = serde_json::from_str(&body_text)
        .map_err(|e| format!("Failed to parse create transcription response: {} — body: {}", e, body_text))?;

    log::info!("Soniox transcription created: id={}", parsed.id);
    Ok(parsed.id)
}

/// Poll GET /v1/transcriptions/{id} every 500 ms until "completed" or "error".
///
/// Emits `transcribe-retry` events on each poll for frontend visibility.
/// Times out after MAX_POLL_ATTEMPTS × POLL_INTERVAL_MS (= 120 s).
pub async fn poll_transcription(
    app: &AppHandle,
    client: &Client,
    api_key: &str,
    base_url: &str,
    transcription_id: &str,
) -> Result<super::TranscriptionData, String> {
    let poll_url = format!("{}/v1/transcriptions/{}", base_url, transcription_id);
    let transcript_url = format!("{}/v1/transcriptions/{}/transcript", base_url, transcription_id);

    for attempt in 1..=MAX_POLL_ATTEMPTS {
        let _ = app.emit(
            "transcribe-diagnostic",
            serde_json::json!({ "stage": "polling", "attempt": attempt, "maxAttempts": MAX_POLL_ATTEMPTS }),
        );

        tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;

        let response = client
            .get(&poll_url)
            .bearer_auth(api_key)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Soniox poll request failed: {}", e))?;

        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(format!("Soniox poll error ({}): {}", status, body_text));
        }

        let poll: SonioxPollResponse = serde_json::from_str(&body_text)
            .map_err(|e| format!("Failed to parse poll response: {} — body: {}", e, body_text))?;

        match poll.status.as_str() {
            "completed" => {
                log::info!("Soniox transcription completed after {} polls", attempt);
                return fetch_transcript(client, api_key, &transcript_url).await;
            }
            "error" => {
                return Err(format!("Soniox transcription failed: {}", poll.error.unwrap_or_else(|| "Unknown error".to_string())));
            }
            "queued" | "processing" => {
                let _ = app.emit(
                    "transcribe-retry",
                    serde_json::json!({ "attempt": attempt, "maxAttempts": MAX_POLL_ATTEMPTS, "delayMs": POLL_INTERVAL_MS }),
                );
            }
            unknown => log::warn!("Soniox poll: unexpected status '{}', continuing...", unknown),
        }
    }

    Err(format!("Soniox transcription timed out after {} polls ({}s)", MAX_POLL_ATTEMPTS, MAX_POLL_ATTEMPTS as u64 * POLL_INTERVAL_MS / 1000))
}

/// GET /v1/transcriptions/{id}/transcript — retrieve the final transcription text.
async fn fetch_transcript(
    client: &Client,
    api_key: &str,
    url: &str,
) -> Result<super::TranscriptionData, String> {
    let response = client
        .get(url)
        .bearer_auth(api_key)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Soniox fetch transcript request failed: {}", e))?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("Soniox fetch transcript error ({}): {}", status, body_text));
    }

    let parsed: SonioxTranscriptResponse = serde_json::from_str(&body_text)
        .map_err(|e| format!("Failed to parse transcript response: {} — body: {}", e, body_text))?;

    let duration_ms = parsed.tokens.last().map(|t| t.end_ms).unwrap_or(0);
    let language = parsed.tokens.iter().find_map(|t| t.language.clone());

    log::info!("Soniox transcript: {} chars, {} ms", parsed.text.len(), duration_ms);

    Ok(super::TranscriptionData {
        text: parsed.text,
        duration_ms,
        language,
        engine: Some("soniox".to_string()),
    })
}
