//! Blocking transcription worker invoked via `spawn_blocking`.
//!
//! Loads the Parakeet ONNX model, decodes the supplied audio, and runs inference.
//! Kept isolated so the async entry-point (`mod.rs::transcribe`) stays focused
//! on orchestration and timeouts.

use std::path::{Path, PathBuf};
use tauri::AppHandle;

use super::audio::decode_audio;
use super::emit_log;

/// Run transcription synchronously (blocking — callers wrap with
/// `tokio::task::spawn_blocking`).
pub(super) fn run_transcription(
    app: &AppHandle,
    model_dir: &Path,
    audio_bytes: &[u8],
    mime_type: &str,
    language_hints: &[String],
) -> Result<String, String> {
    use transcribe_rs::onnx::parakeet::ParakeetModel;
    use transcribe_rs::onnx::Quantization;
    use transcribe_rs::{SpeechModel, TranscribeOptions};

    // 1. Decode to 16 kHz mono f32 PCM
    emit_log(
        app,
        &format!("Decoding audio ({} bytes, {})", audio_bytes.len(), mime_type),
    );
    let decode_start = std::time::Instant::now();
    let samples = decode_audio(app, audio_bytes, mime_type)?;
    emit_log(
        app,
        &format!(
            "Decoded {} samples ({:.1}s of audio) in {}ms",
            samples.len(),
            samples.len() as f32 / 16000.0,
            decode_start.elapsed().as_millis()
        ),
    );

    // 2. Load model (per-transcription for now; caching can be added later).
    //    Wrapped in catch_unwind because ort panics on DLL version mismatch
    //    rather than returning an error — without this, the panic is silently
    //    swallowed by spawn_blocking and surfaces only as a 120 s timeout.
    emit_log(app, &format!("Loading Parakeet model from {:?}", model_dir));
    let load_start = std::time::Instant::now();
    let model_path = PathBuf::from(model_dir);
    let load_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ParakeetModel::load(&model_path, &Quantization::Int8)
    }));
    let mut model = match load_result {
        Ok(Ok(m)) => m,
        Ok(Err(e)) => return Err(format!("Failed to load Parakeet model: {}", e)),
        Err(panic) => {
            let msg = panic
                .downcast_ref::<String>()
                .map(|s| s.as_str())
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("unknown panic");
            return Err(format!(
                "Parakeet model load panicked (likely ORT DLL version mismatch): {}",
                msg
            ));
        }
    };
    emit_log(
        app,
        &format!("Model loaded in {}ms", load_start.elapsed().as_millis()),
    );

    // 3. Transcribe — use language hint only when a single language is configured.
    let lang = if language_hints.len() == 1 {
        language_hints.first().cloned()
    } else {
        None
    };
    emit_log(
        app,
        &format!(
            "Transcribing with language hint: {:?} (configured: {:?})",
            lang, language_hints
        ),
    );
    let transcribe_start = std::time::Instant::now();
    let options = TranscribeOptions {
        language: lang,
        ..Default::default()
    };

    let result = model
        .transcribe(&samples, &options)
        .map_err(|e| format!("Parakeet transcription error: {}", e))?;
    emit_log(
        app,
        &format!(
            "Transcription complete: {} chars in {}ms",
            result.text.len(),
            transcribe_start.elapsed().as_millis()
        ),
    );

    Ok(result.text)
}
