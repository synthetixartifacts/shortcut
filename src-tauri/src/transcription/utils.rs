//! Shared utilities for transcription providers.
//!
//! Shared utility functions for all transcription providers.

/// Maximum number of retry attempts for transient upload failures
pub(crate) const TRANSCRIBE_MAX_ATTEMPTS: u32 = 3;

/// Check if a transcription error is transient and worth retrying.
///
/// Transient errors include:
/// - 400 with "partially uploaded" = stale connection (UPLOAD_ERR_PARTIAL)
/// - 400 with "No file was uploaded" = stale connection (UPLOAD_ERR_NO_FILE)
/// - 500/502/503/504 = server-side transient errors
pub(crate) fn is_transient_error(status: u16, body: &str) -> bool {
    if status >= 500 {
        return true;
    }
    if status == 400 {
        let lower = body.to_lowercase();
        return lower.contains("partially uploaded") || lower.contains("no file was uploaded");
    }
    false
}

/// Build multipart body as raw bytes.
///
/// Bypasses reqwest's `Form` streaming which has a known bug causing truncated
/// bodies for larger payloads (reqwest#252). Instead, we assemble the complete
/// multipart body in memory and send it as a single contiguous `Vec<u8>`.
/// This matches how curl builds multipart requests.
///
/// Returns `(content_type_header, body_bytes)`.
pub(crate) fn build_multipart_body(
    audio_bytes: &[u8],
    mime_type: &str,
    language_hints: &[String],
    context_terms: &[String],
    context_text: Option<&str>,
) -> Result<(String, Vec<u8>), String> {
    let ext = match mime_type {
        "audio/webm" | "audio/webm;codecs=opus" => "webm",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/mp3" | "audio/mpeg" => "mp3",
        "audio/ogg" | "audio/ogg;codecs=opus" => "ogg",
        "audio/flac" => "flac",
        "audio/m4a" | "audio/x-m4a" | "audio/mp4" => "m4a",
        _ => "webm",
    };

    // Generate a unique boundary that won't appear in binary audio data
    let boundary = format!(
        "----SCBoundary{:016x}{:016x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64,
        std::process::id() as u64 ^ audio_bytes.len() as u64
    );

    // Pre-allocate: audio + generous overhead for headers/boundaries
    let mut body = Vec::with_capacity(audio_bytes.len() + 2048);

    // Audio file part
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"audio.{}\"\r\n",
            ext
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", mime_type).as_bytes());
    body.extend_from_slice(audio_bytes);
    body.extend_from_slice(b"\r\n");

    // Optional text fields
    for (field, values) in [("language_hints", language_hints), ("context_terms", context_terms)] {
        if !values.is_empty() {
            let json = serde_json::to_string(values)
                .map_err(|e| format!("Failed to serialize {}: {}", field, e))?;
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", field).as_bytes(),
            );
            body.extend_from_slice(json.as_bytes());
            body.extend_from_slice(b"\r\n");
        }
    }

    if let Some(text) = context_text.filter(|t| !t.is_empty()) {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"context_text\"\r\n\r\n",
        );
        body.extend_from_slice(text.as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    // Closing boundary
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let content_type = format!("multipart/form-data; boundary={}", boundary);
    Ok((content_type, body))
}
