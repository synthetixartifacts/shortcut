//! ONNX Runtime DLL discovery + `ORT_DYLIB_PATH` initialization.
//!
//! With `load-dynamic`, the `ort` crate opens `onnxruntime.dll` at runtime via
//! `libloading`. We have to tell it where the DLL lives by setting the
//! `ORT_DYLIB_PATH` environment variable *before* the first ORT call.
//!
//! The setter is wrapped in a `OnceLock` so the (unsafe, on Rust >= 1.80)
//! `env::set_var` call happens exactly once and strictly before any ORT worker
//! thread spins up — this is the single-threaded invariant that keeps the
//! `unsafe` block sound.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tauri::Manager;

use super::emit_log;

/// Find and set `ORT_DYLIB_PATH` for the ONNX Runtime DLL.
///
/// Returns an error if the DLL cannot be found in any of the expected
/// locations. Idempotent — subsequent calls re-use the first result.
pub(super) fn ensure_ort_initialized(
    app: &tauri::AppHandle,
    model_dir: &Path,
) -> Result<(), String> {
    static ORT_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

    ORT_RESULT
        .get_or_init(|| {
            // Search locations for onnxruntime.dll
            let candidates: Vec<PathBuf> = [
                // 1. Tauri resource directory (bundled with installer)
                app.path()
                    .resource_dir()
                    .ok()
                    .map(|d| d.join("onnxruntime.dll")),
                // 2. Model directory
                Some(model_dir.join("onnxruntime.dll")),
                // 3. Next to executable
                std::env::current_exe()
                    .ok()
                    .and_then(|e| e.parent().map(|d| d.join("onnxruntime.dll"))),
            ]
            .into_iter()
            .flatten()
            .collect();

            emit_log(
                app,
                &format!(
                    "Searching for onnxruntime.dll in {} locations",
                    candidates.len()
                ),
            );
            for dll in &candidates {
                let exists = dll.exists();
                emit_log(
                    app,
                    &format!(
                        "  {:?} → {}",
                        dll,
                        if exists { "FOUND" } else { "not found" }
                    ),
                );
                if exists {
                    // Strip Windows extended-length path prefix (\\?\) which Tauri's
                    // resource_dir() adds — libloading/LoadLibraryW can choke on it.
                    let clean_path = strip_extended_prefix(dll);
                    // SAFETY: On Rust >= 1.80, `std::env::set_var` is `unsafe` because
                    // libc's `setenv` is not thread-safe — a concurrent reader could
                    // observe a torn environ pointer.
                    //
                    // This call is sound because:
                    //   * It runs inside `ORT_RESULT.get_or_init`, which the std lib
                    //     guarantees to invoke at most once across all threads.
                    //   * The init closure runs *before* any ORT worker thread is
                    //     spawned — `ParakeetModel::load` blocks on this setter via
                    //     `ensure_ort_initialized`.
                    //   * No other code in ShortCut mutates `ORT_DYLIB_PATH` or any
                    //     other env var at runtime.
                    unsafe {
                        std::env::set_var("ORT_DYLIB_PATH", &clean_path);
                    }
                    emit_log(app, &format!("ORT_DYLIB_PATH set to: {:?}", clean_path));
                    return Ok(());
                }
            }

            let searched = candidates
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let msg = format!(
                "onnxruntime.dll not found. Searched: [{}]. \
                 Please reinstall the app or place onnxruntime.dll next to the executable.",
                searched
            );
            log::error!("{}", msg);
            Err(msg)
        })
        .clone()
}

/// Strip the `\\?\` extended-length path prefix that Tauri adds on Windows.
/// Some libraries (libloading) may not handle it correctly.
fn strip_extended_prefix(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}
