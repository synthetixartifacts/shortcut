//! Soniox direct transcription provider — public entry point.
//!
//! Implements the 5-step Soniox API flow by delegating HTTP operations to
//! `soniox_api.rs` to keep each file within the 300-line limit:
//!   1. POST /v1/files             — upload audio file
//!   2. POST /v1/transcriptions    — create transcription job
//!   3. Poll GET /v1/transcriptions/{id} — wait for "completed"
//!   4. GET /v1/transcriptions/{id}/transcript — fetch text
//!   5. DELETE /v1/files/{file_id} — fire-and-forget cleanup
//!
//! Retry and event patterns emit Tauri events per the stable event contract.

use crate::config::ConfigState;
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use tauri::{AppHandle, Emitter, Manager};

const SONIOX_BASE_URL: &str = "https://api.soniox.com";

/// Transcribe audio via the Soniox API directly (5-step flow).
pub async fn transcribe(
    app: &AppHandle,
    audio_base64: Option<String>,
    audio_path: Option<String>,
    mime_type: String,
    language_hints: Vec<String>,
    context_terms: Vec<String>,
    context_text: Option<String>,
) -> Result<super::TranscriptionData, String> {
    use super::soniox_api as api;

    // 1. Read credentials and build helpers
    let (client, api_key, base_url) = get_soniox_config(app)?;

    // 2. Decode / read audio bytes
    let audio_bytes = decode_audio(app, audio_base64, audio_path, &mime_type).await?;

    // 3. Upload file (with retry on transient errors)
    let file_id = api::upload_file_with_retry(
        app,
        &client,
        &api_key,
        &base_url,
        &audio_bytes,
        &mime_type,
    )
    .await?;

    // 4. Create transcription job — clean up file on error
    let transcription_id = match api::create_transcription(
        &client,
        &api_key,
        &base_url,
        &file_id,
        &language_hints,
        &context_terms,
        context_text.as_deref(),
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            spawn_delete_file(app.clone(), client.clone(), api_key.clone(), base_url.clone(), file_id);
            return Err(e);
        }
    };

    // 5. Poll for completion — clean up file on error
    let result = match api::poll_transcription(app, &client, &api_key, &base_url, &transcription_id).await {
        Ok(r) => r,
        Err(e) => {
            spawn_delete_file(app.clone(), client.clone(), api_key.clone(), base_url.clone(), file_id);
            return Err(e);
        }
    };

    // 6. Fire-and-forget file cleanup
    spawn_delete_file(app.clone(), client, api_key, base_url, file_id);

    Ok(result)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Read Soniox credentials and base URL from managed config state.
fn get_soniox_config(app: &AppHandle) -> Result<(Client, String, String), String> {
    let client = app.state::<Client>().inner().clone();
    let state = app.state::<ConfigState>();
    let config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    let api_key = config.providers.credentials.soniox_api_key.clone();
    if api_key.is_empty() {
        return Err("Soniox API key is not configured. Go to Settings to add it.".to_string());
    }
    Ok((client, api_key, SONIOX_BASE_URL.to_string()))
}

/// Decode audio bytes from base64 inline data or a temp file path.
///
/// Files >60 KB are sent via `audio_path` to avoid IPC size limits
/// (handled by the frontend `transcribeAudio()` wrapper).
async fn decode_audio(
    app: &AppHandle,
    audio_base64: Option<String>,
    audio_path: Option<String>,
    mime_type: &str,
) -> Result<Vec<u8>, String> {
    if let Some(path) = audio_path {
        log::info!("Soniox: reading audio from file: {}", path);
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| format!("Failed to read audio file: {}", e))?;
        let _ = app.emit(
            "transcribe-diagnostic",
            serde_json::json!({ "stage": "file_read", "fileBytes": bytes.len() }),
        );
        Ok(bytes)
    } else if let Some(b64) = audio_base64 {
        let base64_len = b64.len();
        log::info!("Soniox: decoding audio ({} bytes base64, type: {})", base64_len, mime_type);
        let _ = app.emit(
            "transcribe-diagnostic",
            serde_json::json!({ "stage": "received", "base64Bytes": base64_len }),
        );
        let decoded = general_purpose::STANDARD
            .decode(&b64)
            .map_err(|e| format!("Failed to decode audio: {}", e))?;
        let decoded_len = decoded.len();
        let _ = app.emit(
            "transcribe-diagnostic",
            serde_json::json!({
                "stage": "decoded",
                "decodedBytes": decoded_len,
                "expectedBytes": base64_len * 3 / 4
            }),
        );
        Ok(decoded)
    } else {
        Err("No audio provided".to_string())
    }
}

/// DELETE /v1/files/{file_id} — fire-and-forget cleanup after transcription.
///
/// Spawned in a background task so errors don't affect the caller.
fn spawn_delete_file(
    app: AppHandle,
    client: Client,
    api_key: String,
    base_url: String,
    file_id: String,
) {
    tokio::spawn(async move {
        let url = format!("{}/v1/files/{}", base_url, file_id);
        match client
            .delete(&url)
            .bearer_auth(&api_key)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => {
                log::info!("Soniox: file {} deleted", file_id);
            }
            Ok(r) => {
                log::warn!("Soniox: file {} delete returned {}", file_id, r.status());
            }
            Err(e) => {
                log::warn!("Soniox: file {} delete failed: {}", file_id, e);
            }
        }
        let _ = app;
    });
}
