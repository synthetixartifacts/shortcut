//! Model lifecycle management for local STT
//!
//! Downloads, verifies, and manages the Parakeet TDT 0.6B v3 INT8 model files.
//! Only compiled with the `local-stt` Cargo feature.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

/// HuggingFace base URL for the INT8 quantized Parakeet model
const HF_BASE: &str =
    "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main";

/// A model file entry: (url_path, local_filename, expected_size_bytes, expected_sha256_hex).
///
/// `expected_sha256` may be an empty string while we wire in upstream-published
/// hashes; when empty, we compute the digest and log it at INFO but do not reject
/// the download. Once hashes are pinned, set this to the lowercase hex digest.
pub struct ModelFile {
    pub filename: &'static str,
    pub expected_size: u64,
    pub expected_sha256: &'static str,
}

const MODEL_FILES: &[ModelFile] = &[
    ModelFile {
        filename: "encoder-model.int8.onnx",
        expected_size: 652_000_000,
        expected_sha256: "",
    },
    ModelFile {
        filename: "decoder_joint-model.int8.onnx",
        expected_size: 18_200_000,
        expected_sha256: "",
    },
    ModelFile {
        filename: "nemo128.onnx",
        expected_size: 140_000,
        expected_sha256: "",
    },
    ModelFile {
        filename: "vocab.txt",
        expected_size: 94_000,
        expected_sha256: "",
    },
];

/// Global cancellation flag for downloads
static DOWNLOAD_CANCELLED: AtomicBool = AtomicBool::new(false);

/// Model status reported to the frontend
#[derive(Debug, Clone, Serialize)]
pub struct ModelStatus {
    pub state: String,
    pub progress: Option<f64>,
    pub size_bytes: Option<u64>,
    pub path: Option<String>,
    pub error: Option<String>,
}

/// Get the model directory path (app_data/models/parakeet-tdt-0.6b-v3/)
pub fn get_model_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let dir = base.join("models").join("parakeet-tdt-0.6b-v3");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create model dir: {}", e))?;
    Ok(dir)
}

/// Get current model status
pub fn get_model_status(app: AppHandle) -> Result<ModelStatus, String> {
    let dir = get_model_dir(&app)?;
    let encoder = dir.join("encoder-model.int8.onnx");

    if !encoder.exists() {
        return Ok(ModelStatus {
            state: "not_downloaded".to_string(),
            progress: None,
            size_bytes: None,
            path: None,
            error: None,
        });
    }

    // Check all required files exist and encoder has reasonable size
    let meta = std::fs::metadata(&encoder)
        .map_err(|e| format!("Cannot read model file: {}", e))?;

    if meta.len() < 100_000_000 {
        return Ok(ModelStatus {
            state: "corrupt".to_string(),
            progress: None,
            size_bytes: Some(meta.len()),
            path: Some(dir.to_string_lossy().to_string()),
            error: Some("Model file appears incomplete or corrupted".to_string()),
        });
    }

    // Verify all supporting files exist
    for entry in MODEL_FILES.iter().skip(1) {
        if !dir.join(entry.filename).exists() {
            return Ok(ModelStatus {
                state: "corrupt".to_string(),
                progress: None,
                size_bytes: Some(meta.len()),
                path: Some(dir.to_string_lossy().to_string()),
                error: Some(format!("Missing model file: {}", entry.filename)),
            });
        }
    }

    // Calculate total size
    let total: u64 = MODEL_FILES
        .iter()
        .filter_map(|entry| std::fs::metadata(dir.join(entry.filename)).ok())
        .map(|m| m.len())
        .sum();

    Ok(ModelStatus {
        state: "ready".to_string(),
        progress: None,
        size_bytes: Some(total),
        path: Some(dir.to_string_lossy().to_string()),
        error: None,
    })
}

/// Download the model with progress reporting
///
/// Emits events: model-download-progress, model-download-complete, model-download-error
pub async fn download_model(app: AppHandle) -> Result<(), String> {
    DOWNLOAD_CANCELLED.store(false, Ordering::SeqCst);

    let model_dir = get_model_dir(&app)?;

    // Check disk space (best-effort, not all platforms support this)
    check_disk_space(&model_dir)?;

    // Calculate total expected download size
    let total_bytes: u64 = MODEL_FILES.iter().map(|e| e.expected_size).sum();
    let mut downloaded_total: u64 = 0;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    for entry in MODEL_FILES {
        if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
            cleanup_partial(&model_dir);
            return Err("Download cancelled".to_string());
        }

        let url = format!("{}/{}", HF_BASE, entry.filename);
        log::info!("Downloading model file: {}", url);

        match download_file(&app, &client, &url, &model_dir, entry, &mut downloaded_total, total_bytes).await {
            Ok(()) => {}
            Err(e) => {
                let msg = format!("Failed to download {}: {}", entry.filename, e);
                log::error!("{}", msg);
                let _ = app.emit("model-download-error", serde_json::json!({ "error": &msg }));
                cleanup_partial(&model_dir);
                return Err(msg);
            }
        }
    }

    log::info!("Model download complete: {:?}", model_dir);
    let _ = app.emit("model-download-complete", serde_json::json!({}));
    Ok(())
}

/// Download a single file with progress tracking and SHA-256 integrity check.
///
/// The file is streamed to a `.tmp` path while a rolling `Sha256` hash is
/// updated in lock-step with the writes. On completion we compare the digest
/// against `entry.expected_sha256` — a mismatch triggers tmp cleanup and an
/// error. When the expected hash is empty we log the observed digest but do
/// not fail the download (gap documented in Handoff Notes).
async fn download_file(
    app: &AppHandle,
    client: &reqwest::Client,
    url: &str,
    model_dir: &PathBuf,
    entry: &ModelFile,
    downloaded_total: &mut u64,
    total_bytes: u64,
) -> Result<(), String> {
    use futures_util::StreamExt;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let dest = model_dir.join(entry.filename);
    let tmp = model_dir.join(format!("{}.tmp", entry.filename));

    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut hasher = Sha256::new();
    let mut stream = response.bytes_stream();
    let mut file_bytes: u64 = 0;

    while let Some(chunk) = stream.next().await {
        if DOWNLOAD_CANCELLED.load(Ordering::SeqCst) {
            drop(file);
            let _ = tokio::fs::remove_file(&tmp).await;
            return Err("Download cancelled".to_string());
        }

        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|e| format!("Write error: {}", e))?;

        hasher.update(&chunk);
        file_bytes += chunk.len() as u64;
        *downloaded_total += chunk.len() as u64;

        // Emit progress every ~500KB
        if file_bytes % 524_288 < chunk.len() as u64 {
            let progress = *downloaded_total as f64 / total_bytes as f64;
            let _ = app.emit(
                "model-download-progress",
                serde_json::json!({
                    "progress": progress.min(1.0),
                    "bytes_downloaded": *downloaded_total,
                    "total_bytes": total_bytes,
                    "current_file": entry.filename,
                }),
            );
        }
    }

    // Flush before integrity check so the on-disk file matches what we hashed.
    tokio::io::AsyncWriteExt::flush(&mut file)
        .await
        .map_err(|e| format!("Flush error: {}", e))?;
    drop(file);

    let observed = hex_lower(&hasher.finalize());
    if entry.expected_sha256.is_empty() {
        log::info!(
            "{} downloaded ({} bytes) sha256={} (no expected hash pinned — integrity check skipped)",
            entry.filename,
            file_bytes,
            observed
        );
    } else if !eq_ignore_ascii_case(&observed, entry.expected_sha256) {
        let _ = tokio::fs::remove_file(&tmp).await;
        return Err(format!(
            "SHA-256 mismatch for {}: expected {}, got {}",
            entry.filename, entry.expected_sha256, observed
        ));
    } else {
        log::info!("{} sha256 verified ({})", entry.filename, observed);
    }

    tokio::fs::rename(&tmp, &dest)
        .await
        .map_err(|e| format!("Failed to finalize {}: {}", entry.filename, e))?;

    log::info!("Downloaded {} ({} bytes)", entry.filename, file_bytes);
    Ok(())
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .all(|(x, y)| x.eq_ignore_ascii_case(&y))
}

/// Check available disk space (best-effort)
fn check_disk_space(path: &PathBuf) -> Result<(), String> {
    // fs2 or sysinfo could give us exact numbers, but for now just check
    // that we can create files in the directory
    let test = path.join(".space_check");
    std::fs::write(&test, b"ok")
        .map_err(|e| format!("Cannot write to model directory: {}", e))?;
    let _ = std::fs::remove_file(&test);
    Ok(())
}

/// Remove partially downloaded files on error/cancel
fn cleanup_partial(model_dir: &PathBuf) {
    for entry in MODEL_FILES {
        let tmp = model_dir.join(format!("{}.tmp", entry.filename));
        if tmp.exists() {
            let _ = std::fs::remove_file(&tmp);
        }
    }
}

/// Delete the downloaded model
pub async fn delete_model(app: AppHandle) -> Result<(), String> {
    let model_dir = get_model_dir(&app)?;
    if model_dir.exists() {
        std::fs::remove_dir_all(&model_dir)
            .map_err(|e| format!("Failed to delete model: {}", e))?;
        log::info!("Model deleted from {:?}", model_dir);
    }
    Ok(())
}

/// Cancel an in-progress model download
pub async fn cancel_model_download(_app: AppHandle) -> Result<(), String> {
    DOWNLOAD_CANCELLED.store(true, Ordering::SeqCst);
    log::info!("Model download cancellation requested");
    Ok(())
}
