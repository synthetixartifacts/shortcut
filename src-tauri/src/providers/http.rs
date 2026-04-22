//! Shared HTTP client factory + streaming helpers for all providers.
//!
//! Forces HTTP/1.1 to avoid multipart upload issues with HTTP/2 (reqwest#252).
//! reqwest 0.13 + rustls enables ALPN, which silently negotiates HTTP/2.
//! Larger multipart uploads (>65 KB) fail over h2 due to flow control framing
//! issues. HTTP/1.1 with Content-Length works reliably.
//!
//! The client is created once at app startup and managed as Tauri state.
//!
//! In addition to the client factory, this module exposes three shared helpers
//! used by every provider's `complete()` / `stream()` implementation:
//!
//! - [`ensure_ok`] — converts a non-2xx [`reqwest::Response`] into a classified
//!   [`AppError::Provider`] (Auth / RateLimit / Server / InvalidRequest / Other)
//!   so the frontend can branch on the kind of failure.
//! - [`read_sse`] — buffers raw bytes across TCP chunks and yields complete
//!   SSE events (delimited by `\n\n`). Decodes UTF-8 only once a full event is
//!   in the buffer, which fixes the "multi-byte char split across chunks"
//!   corruption bug that previously appeared with `String::from_utf8_lossy`.
//! - [`read_ndjson`] — same buffering contract but framed by `\n`, with typed
//!   deserialization. Parse errors on individual lines are logged at debug and
//!   skipped — consistent with how Ollama's streaming protocol is structured.
//!
//! Both streaming helpers accept an optional cancellation token
//! ([`Arc<AtomicBool>`]) so callers (e.g. `stream_screen_question`) can abort
//! in-flight streams when a UI window closes. Using the AtomicBool pattern
//! keeps the trait object-safe and avoids pulling in `tokio_util`.

use crate::errors::{AppError, ProviderErrorKind};
use futures_util::StreamExt;
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Create the shared HTTP client (called once during app setup).
///
/// HTTP/1.1 forced: larger multipart uploads fail over h2 due to flow control
/// framing issues. HTTP/1.1 with Content-Length works reliably.
///
/// In debug builds, accepts invalid TLS certs for local dev proxy testing.
pub fn create_http_client() -> Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "X-Client",
        reqwest::header::HeaderValue::from_static("ShortCut/1.0"),
    );

    Client::builder()
        .http1_only()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .pool_idle_timeout(std::time::Duration::from_secs(10))
        .pool_max_idle_per_host(1)
        .tcp_keepalive(std::time::Duration::from_secs(15))
        .tls_danger_accept_invalid_certs(cfg!(debug_assertions))
        .build()
        .unwrap_or_else(|e| {
            log::warn!("Custom HTTP client builder failed: {}, using default", e);
            Client::new()
        })
}

// ---------------------------------------------------------------------------
// Shared provider helpers
// ---------------------------------------------------------------------------

/// Signal from an SSE/NDJSON callback whether to continue reading or stop.
pub enum ControlFlow {
    Continue,
    Break,
}

/// A shared cancellation token used across provider streams. Consumers flip it
/// from `false` → `true` to signal the stream should abort at the next event
/// boundary.
pub type CancelFlag = Arc<AtomicBool>;

/// Convenience: construct a fresh cancellation flag in the "not cancelled" state.
#[allow(dead_code)]
pub fn new_cancel_flag() -> CancelFlag {
    Arc::new(AtomicBool::new(false))
}

/// Check an HTTP response for success; on non-2xx, log the body at debug and
/// return an [`AppError::Provider`] carrying the classified failure kind.
///
/// Status → kind mapping:
/// - 401 / 403 → [`ProviderErrorKind::Auth`]
/// - 429       → [`ProviderErrorKind::RateLimit`] (parses `Retry-After`)
/// - 400       → [`ProviderErrorKind::InvalidRequest`]
/// - 5xx       → [`ProviderErrorKind::Server`]
/// - other     → [`ProviderErrorKind::Other`]
pub async fn ensure_ok(response: Response, provider: &'static str) -> Result<Response, AppError> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    // Capture the request URL + HTTP method *before* consuming the response
    // body, so the error message can pinpoint exactly which endpoint rejected
    // us. Diagnostics show both the full URL and up to ~200 chars of body —
    // enough to surface provider-level JSON error messages without swamping
    // logs.
    let url = response.url().to_string();
    let retry_after = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok());
    let body = response.text().await.unwrap_or_default();
    log::debug!("{provider} error body ({status}) at {url}: {body}");

    let kind = match status.as_u16() {
        401 | 403 => ProviderErrorKind::Auth,
        429 => ProviderErrorKind::RateLimit { retry_after_secs: retry_after },
        400 => ProviderErrorKind::InvalidRequest,
        500..=599 => ProviderErrorKind::Server { status: status.as_u16() },
        _ => ProviderErrorKind::Other,
    };
    let body_preview = truncate_preview(&body, 200);
    let message = format!(
        "{provider} {url} failed: HTTP {} — body: {}",
        status.as_u16(),
        if body_preview.is_empty() {
            "(empty)".to_string()
        } else {
            body_preview
        }
    );
    Err(AppError::provider(kind, message))
}

/// Truncate a string to at most `max_chars` user-visible characters, appending
/// `…` on overflow. Used by provider error formatters so we never dump a full
/// 50 KB JSON error body into the UI.
pub(crate) fn truncate_preview(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars).collect();
    out.push('…');
    out
}

/// Locate the first occurrence of `needle` inside `haystack` — small helper so
/// callers don't need to pull in `memchr`.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Read a Server-Sent Events (SSE) response, buffering raw bytes and decoding
/// UTF-8 only once a full event is available.
///
/// Events are delimited by `\n\n`. The callback receives each decoded event
/// block (minus the trailing delimiter) as a `&str` and returns either
/// [`ControlFlow::Continue`] to keep reading or [`ControlFlow::Break`] to stop
/// (e.g. on `data: [DONE]`).
pub async fn read_sse<F>(response: Response, on_event: F) -> Result<(), AppError>
where
    F: FnMut(&str) -> Result<ControlFlow, AppError>,
{
    read_sse_cancellable(response, None, on_event).await
}

/// Same as [`read_sse`], but honors a cancellation flag checked at every event
/// boundary so long-running streams can abort promptly when UI closes.
pub async fn read_sse_cancellable<F>(
    response: Response,
    cancel: Option<CancelFlag>,
    mut on_event: F,
) -> Result<(), AppError>
where
    F: FnMut(&str) -> Result<ControlFlow, AppError>,
{
    let mut stream = response.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();

    while let Some(chunk_result) = stream.next().await {
        if is_cancelled(&cancel) {
            log::info!("SSE stream cancelled via cancel flag");
            return Ok(());
        }
        let chunk = chunk_result.map_err(|e| {
            AppError::provider(ProviderErrorKind::Network, format!("SSE stream read error: {e}"))
        })?;
        buf.extend_from_slice(&chunk);

        while let Some(pos) = find_subslice(&buf, b"\n\n") {
            if is_cancelled(&cancel) {
                log::info!("SSE stream cancelled via cancel flag (mid-event)");
                return Ok(());
            }
            let event_bytes: Vec<u8> = buf.drain(..pos + 2).collect();
            let event = std::str::from_utf8(&event_bytes[..pos]).map_err(|e| {
                AppError::provider(ProviderErrorKind::Parse, format!("Invalid UTF-8 in SSE: {e}"))
            })?;
            if matches!(on_event(event)?, ControlFlow::Break) {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// Read a newline-delimited JSON response, buffering raw bytes across chunks.
pub async fn read_ndjson<F, T>(response: Response, on_line: F) -> Result<(), AppError>
where
    F: FnMut(T) -> Result<ControlFlow, AppError>,
    T: DeserializeOwned,
{
    read_ndjson_cancellable(response, None, on_line).await
}

/// Cancellation-aware variant of [`read_ndjson`].
pub async fn read_ndjson_cancellable<F, T>(
    response: Response,
    cancel: Option<CancelFlag>,
    mut on_line: F,
) -> Result<(), AppError>
where
    F: FnMut(T) -> Result<ControlFlow, AppError>,
    T: DeserializeOwned,
{
    let mut stream = response.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();

    while let Some(chunk_result) = stream.next().await {
        if is_cancelled(&cancel) {
            log::info!("NDJSON stream cancelled via cancel flag");
            return Ok(());
        }
        let chunk = chunk_result.map_err(|e| {
            AppError::provider(ProviderErrorKind::Network, format!("NDJSON stream read error: {e}"))
        })?;
        buf.extend_from_slice(&chunk);

        while let Some(pos) = find_subslice(&buf, b"\n") {
            if is_cancelled(&cancel) {
                log::info!("NDJSON stream cancelled via cancel flag (mid-line)");
                return Ok(());
            }
            let line_bytes: Vec<u8> = buf.drain(..pos + 1).collect();
            let line_slice = &line_bytes[..pos];
            if line_slice.iter().all(u8::is_ascii_whitespace) {
                continue;
            }

            let line = std::str::from_utf8(line_slice).map_err(|e| {
                AppError::provider(
                    ProviderErrorKind::Parse,
                    format!("Invalid UTF-8 in NDJSON line: {e}"),
                )
            })?;

            match serde_json::from_str::<T>(line) {
                Ok(value) => {
                    if matches!(on_line(value)?, ControlFlow::Break) {
                        return Ok(());
                    }
                }
                Err(e) => {
                    log::debug!("NDJSON parse error: {e} on line: {line}");
                }
            }
        }
    }
    Ok(())
}

fn is_cancelled(cancel: &Option<CancelFlag>) -> bool {
    cancel
        .as_ref()
        .map(|flag| flag.load(Ordering::Relaxed))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Unit tests (fixture-based; no network required)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- find_subslice ---

    #[test]
    fn find_subslice_finds_match() {
        assert_eq!(find_subslice(b"hello\n\nworld", b"\n\n"), Some(5));
    }

    #[test]
    fn find_subslice_returns_none_when_absent() {
        assert_eq!(find_subslice(b"hello world", b"\n\n"), None);
    }

    #[test]
    fn find_subslice_handles_empty_needle() {
        assert_eq!(find_subslice(b"anything", b""), None);
    }

    // --- SSE event framing (via the buffer invariants of find_subslice) ---
    //
    // The shape-level unit tests for read_sse require mocking a Response, which
    // is painful with reqwest. The tests below exercise the byte-level framing
    // logic directly — which is exactly where the UTF-8 chunking bug lived.

    /// Reconstruct the exact buffer-draining logic used inside `read_sse` so we
    /// can assert that split UTF-8 codepoints survive the boundary.
    fn drain_events(chunks: &[&[u8]]) -> Vec<String> {
        let mut buf: Vec<u8> = Vec::new();
        let mut events = Vec::new();
        for chunk in chunks {
            buf.extend_from_slice(chunk);
            while let Some(pos) = find_subslice(&buf, b"\n\n") {
                let event_bytes: Vec<u8> = buf.drain(..pos + 2).collect();
                let event = std::str::from_utf8(&event_bytes[..pos])
                    .expect("split UTF-8 handled by buffer");
                events.push(event.to_string());
            }
        }
        events
    }

    #[test]
    fn sse_emoji_split_across_chunks_is_decoded_correctly() {
        // "data: hi 🙂" with the emoji bytes split across two chunks.
        let emoji = "🙂".as_bytes(); // 4 bytes
        let mut prefix = b"data: hi ".to_vec();
        prefix.push(emoji[0]);
        prefix.push(emoji[1]);
        let mut suffix = vec![emoji[2], emoji[3]];
        suffix.extend_from_slice(b"\n\n");

        let events = drain_events(&[&prefix, &suffix]);
        assert_eq!(events, vec!["data: hi 🙂".to_string()]);
    }

    #[test]
    fn sse_multiple_events_per_chunk() {
        let chunk: &[u8] = b"data: one\n\ndata: two\n\n";
        let events = drain_events(&[chunk]);
        assert_eq!(events, vec!["data: one", "data: two"]);
    }

    #[test]
    fn sse_event_with_no_data_line() {
        let chunk: &[u8] = b": heartbeat\n\n";
        let events = drain_events(&[chunk]);
        assert_eq!(events, vec![": heartbeat"]);
    }

    #[test]
    fn sse_partial_event_held_until_delimiter() {
        // First chunk ends mid-event; second chunk completes it.
        let events = drain_events(&[b"data: par", b"tial\n\n"]);
        assert_eq!(events, vec!["data: partial"]);
    }

    #[test]
    fn sse_done_sentinel_extracted() {
        let chunk: &[u8] = b"data: [DONE]\n\n";
        let events = drain_events(&[chunk]);
        assert_eq!(events, vec!["data: [DONE]"]);
    }

    // --- cancellation flag ---

    #[test]
    fn cancel_flag_honored_when_set() {
        let flag = new_cancel_flag();
        flag.store(true, Ordering::Relaxed);
        assert!(is_cancelled(&Some(flag)));
    }

    #[test]
    fn cancel_flag_defaults_to_false() {
        let flag = new_cancel_flag();
        assert!(!is_cancelled(&Some(flag)));
    }

    #[test]
    fn cancel_flag_absent_is_not_cancelled() {
        assert!(!is_cancelled(&None));
    }

    // --- NDJSON line extraction (buffer invariants) ---

    fn drain_lines(chunks: &[&[u8]]) -> Vec<String> {
        let mut buf: Vec<u8> = Vec::new();
        let mut lines = Vec::new();
        for chunk in chunks {
            buf.extend_from_slice(chunk);
            while let Some(pos) = find_subslice(&buf, b"\n") {
                let line_bytes: Vec<u8> = buf.drain(..pos + 1).collect();
                let slice = &line_bytes[..pos];
                if slice.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }
                lines.push(std::str::from_utf8(slice).unwrap().to_string());
            }
        }
        lines
    }

    #[test]
    fn ndjson_line_extraction_basic() {
        let lines = drain_lines(&[b"{\"a\":1}\n{\"b\":2}\n"]);
        assert_eq!(lines, vec!["{\"a\":1}", "{\"b\":2}"]);
    }

    #[test]
    fn ndjson_skips_blank_lines() {
        let lines = drain_lines(&[b"{\"a\":1}\n\n{\"b\":2}\n"]);
        assert_eq!(lines, vec!["{\"a\":1}", "{\"b\":2}"]);
    }

    #[test]
    fn ndjson_holds_partial_line_until_newline() {
        let lines = drain_lines(&[b"{\"a\":", b"1}\n"]);
        assert_eq!(lines, vec!["{\"a\":1}"]);
    }
}
