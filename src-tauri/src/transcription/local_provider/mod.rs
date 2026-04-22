//! Local transcription provider using transcribe-rs + Parakeet TDT.
//!
//! Only compiled with the `local-stt` Cargo feature. Orchestrates model-file
//! verification, ORT init, audio byte resolution, and the blocking transcription
//! worker. The submodules split the concerns:
//!
//! - [`audio`]      — WebM/WAV decode + resample to 16 kHz mono PCM
//! - [`ort_init`]   — ONNX Runtime DLL discovery and `ORT_DYLIB_PATH` setup
//! - [`transcribe`] — blocking Parakeet load + inference

mod audio;
mod ort_init;
mod transcribe;

use super::TranscriptionData;
use tauri::{AppHandle, Emitter};

use audio::resolve_audio_bytes;
use ort_init::ensure_ort_initialized;
use transcribe::run_transcription;

/// Emit a diagnostic log event visible in the frontend debug viewer.
pub(crate) fn emit_log(app: &AppHandle, msg: &str) {
    log::info!("[LocalSTT] {}", msg);
    let _ = app.emit("transcribe-log", format!("[LocalSTT] {}", msg));
}

/// Transcribe audio locally using Parakeet via transcribe-rs.
pub async fn transcribe(
    app: &AppHandle,
    audio_base64: Option<String>,
    audio_path: Option<String>,
    mime_type: String,
    language_hints: Vec<String>,
) -> Result<TranscriptionData, String> {
    let model_dir = super::model_manager::get_model_dir(app)?;
    emit_log(app, &format!("Model dir: {:?}", model_dir));

    // Verify all required model files exist
    let required_files = [
        "encoder-model.int8.onnx",
        "decoder_joint-model.int8.onnx",
        "nemo128.onnx",
        "vocab.txt",
    ];
    for filename in &required_files {
        let path = model_dir.join(filename);
        match std::fs::metadata(&path) {
            Ok(meta) => {
                let size = meta.len();
                emit_log(
                    app,
                    &format!("  {} — {:.1} KB", filename, size as f64 / 1024.0),
                );
            }
            Err(_) => {
                let msg = format!(
                    "Missing model file: {}. Please delete and re-download from Settings > Dictation.",
                    filename
                );
                emit_log(app, &msg);
                return Err(msg);
            }
        }
    }

    // Basic corruption check: verify the encoder is not truncated
    const MIN_MODEL_SIZE_BYTES: u64 = 100_000_000; // ~100 MB minimum for INT8 encoder
    let encoder_path = model_dir.join("encoder-model.int8.onnx");
    let encoder_size = std::fs::metadata(&encoder_path).unwrap().len();
    if encoder_size < MIN_MODEL_SIZE_BYTES {
        return Err(
            "Local model file appears corrupted (too small). \
             Please delete and re-download from Settings > Dictation."
                .to_string(),
        );
    }

    // Ensure ONNX Runtime DLL is discoverable before any ort usage
    emit_log(app, "Initializing ONNX Runtime...");
    ensure_ort_initialized(app, &model_dir)?;
    emit_log(app, "ONNX Runtime initialized");

    emit_log(app, "Reading audio file...");
    let audio_bytes = resolve_audio_bytes(audio_base64, audio_path).await?;
    emit_log(app, &format!("Audio: {} bytes", audio_bytes.len()));
    let start = std::time::Instant::now();

    // CPU-bound work — run on blocking thread pool with timeout
    emit_log(app, "Starting transcription (blocking task)...");
    let hints = language_hints;
    let app_handle = app.clone();
    let task = tokio::task::spawn_blocking(move || {
        run_transcription(&app_handle, &model_dir, &audio_bytes, &mime_type, &hints)
    });

    // 120s timeout — first load of 652 MB model can be slow
    let result = tokio::time::timeout(std::time::Duration::from_secs(120), task)
        .await
        .map_err(|_| {
            "Local transcription timed out (120s). Your machine may be too slow for local STT.".to_string()
        })?
        .map_err(|e| format!("Transcription task panicked: {}", e))?
        .map_err(|e| format!("Local transcription failed: {}", e))?;

    let elapsed_ms = start.elapsed().as_millis() as i64;
    emit_log(app, &format!("Done: {} chars in {}ms", result.len(), elapsed_ms));

    Ok(TranscriptionData {
        text: result,
        duration_ms: elapsed_ms,
        language: None,
        engine: Some("local-windows".to_string()),
    })
}
