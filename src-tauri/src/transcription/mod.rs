//! Transcription engine dispatch
//!
//! Routes transcription requests to the active engine (Soniox or Local).
//! The frontend calls a single command; the backend picks the provider.
//!
//! Engine IDs:
//!   "soniox"        — direct Soniox API (default)
//!   "local-windows" — on-device Parakeet (Windows, requires local-stt feature)
//!   "local-macos"   — on-device (macOS, not yet implemented)

pub mod soniox_provider;
pub mod soniox_api;
pub mod utils;

#[cfg(feature = "local-stt")]
pub mod local_provider;
#[cfg(feature = "local-stt")]
pub mod model_manager;

use crate::config::ConfigState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

/// Result of a transcription operation (engine-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionData {
    pub text: String,
    pub duration_ms: i64,
    #[serde(default)]
    pub language: Option<String>,
    /// Which engine produced this result
    #[serde(default)]
    pub engine: Option<String>,
}

/// Transcribe audio via the active engine
///
/// This is the single entry point from the frontend.
/// Single entry point from the frontend.
#[tauri::command]
pub async fn transcribe_audio(
    app: AppHandle,
    audio_base64: Option<String>,
    audio_path: Option<String>,
    mime_type: String,
    language_hints: Vec<String>,
    context_terms: Vec<String>,
    context_text: Option<String>,
) -> Result<TranscriptionData, String> {
    let active_engine = {
        let state = app.state::<ConfigState>();
        let config = state.0.lock().map_err(|e| format!("Config lock: {}", e))?;
        let engine = config.transcription.active_engine.clone();
        // Validate engine identifier — unknown values fall back to "soniox"
        match engine.as_str() {
            "soniox" | "local-windows" | "local-macos" => engine,
            unknown => {
                log::warn!("Unknown engine '{}', falling back to soniox", unknown);
                "soniox".to_string()
            }
        }
    };

    log::info!("Transcription dispatch: engine={}", active_engine);
    let _ = app.emit("transcribe-log", format!("Dispatch: engine={}", active_engine));

    match active_engine.as_str() {
        "local-windows" => {
            #[cfg(not(target_os = "windows"))]
            {
                Err("Local Windows engine is only available on Windows".to_string())
            }
            #[cfg(target_os = "windows")]
            {
                #[cfg(feature = "local-stt")]
                {
                    let _ = app.emit("transcribe-log", "Routing to local-windows provider");
                    local_provider::transcribe(
                        &app, audio_base64, audio_path, mime_type, language_hints,
                    )
                    .await
                }
                #[cfg(not(feature = "local-stt"))]
                {
                    Err("Local transcription is not available in this build. \
                         Rebuild with --features local-stt to enable it."
                        .to_string())
                }
            }
        }
        "local-macos" => {
            #[cfg(not(target_os = "macos"))]
            {
                Err("Local macOS engine is only available on macOS".to_string())
            }
            #[cfg(target_os = "macos")]
            {
                Err("Local macOS transcription is not yet available.".to_string())
            }
        }
        "soniox" => {
            soniox_provider::transcribe(
                &app,
                audio_base64,
                audio_path,
                mime_type,
                language_hints,
                context_terms,
                context_text,
            )
            .await
        }
        // Fallback for any unrecognised engine ID
        _ => {
            log::warn!("Unknown engine '{}', falling back to soniox", active_engine);
            soniox_provider::transcribe(
                &app,
                audio_base64,
                audio_path,
                mime_type,
                language_hints,
                context_terms,
                context_text,
            )
            .await
        }
    }
}

// --- Model management command stubs ---
// These are always compiled so they can be registered in generate_handler!
// When local-stt feature is disabled, they return informative errors.

/// Get the local model status
#[tauri::command]
pub fn get_model_status(app: AppHandle) -> Result<serde_json::Value, String> {
    #[cfg(feature = "local-stt")]
    {
        model_manager::get_model_status(app).map(|s| serde_json::to_value(s).unwrap())
    }
    #[cfg(not(feature = "local-stt"))]
    {
        let _ = app;
        Ok(serde_json::json!({
            "state": "unavailable",
            "progress": null,
            "size_bytes": null,
            "path": null,
            "error": "Local STT not available in this build"
        }))
    }
}

/// Download the local model
#[tauri::command]
pub async fn download_model(app: AppHandle) -> Result<(), String> {
    #[cfg(feature = "local-stt")]
    {
        model_manager::download_model(app).await
    }
    #[cfg(not(feature = "local-stt"))]
    {
        let _ = app;
        Err("Local STT not available in this build".to_string())
    }
}

/// Delete the local model
#[tauri::command]
pub async fn delete_model(app: AppHandle) -> Result<(), String> {
    #[cfg(feature = "local-stt")]
    {
        model_manager::delete_model(app).await
    }
    #[cfg(not(feature = "local-stt"))]
    {
        let _ = app;
        Err("Local STT not available in this build".to_string())
    }
}

/// Cancel an in-progress model download
#[tauri::command]
pub async fn cancel_model_download(app: AppHandle) -> Result<(), String> {
    #[cfg(feature = "local-stt")]
    {
        model_manager::cancel_model_download(app).await
    }
    #[cfg(not(feature = "local-stt"))]
    {
        let _ = app;
        Err("Local STT not available in this build".to_string())
    }
}
