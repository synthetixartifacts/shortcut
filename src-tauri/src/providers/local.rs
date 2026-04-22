//! Local LLM dispatcher.
//!
//! Resolves the user's protocol choice (manual or auto-detected) and builds
//! either an Ollama-native or OpenAI-compatible provider. Lives outside the
//! factory in providers/mod.rs to keep that file under the 300-line cap.
//!
//! Precedence for `resolve_protocol`:
//! 1. Manual (non-"auto") choice always wins.
//! 2. `detected_protocol` cache (populated by discovery auto-detect race).
//! 3. Default to "ollama" — most common Local server; a failed chat-path call
//!    surfaces the real error in Debug and the next Settings visit re-runs
//!    detection to correct the cache.

use crate::config::LocalCredentials;
use crate::providers::{ollama::OllamaProvider, openai::OpenAiProvider, LlmProvider};
use reqwest::Client;

/// Resolve `local.protocol`, falling back to `detected_protocol` when "auto".
pub fn resolve_protocol(local: &LocalCredentials) -> &'static str {
    match local.protocol.as_str() {
        "openai_compatible" => "openai_compatible",
        "ollama" => "ollama",
        _ => match local.detected_protocol.as_deref() {
            Some("openai_compatible") => "openai_compatible",
            _ => "ollama",
        },
    }
}

/// Build the right adapter for the resolved protocol.
///
/// Caller has already confirmed `local.base_url` is non-empty. The base URL is
/// normalized via [`normalize_local_base_url`] so suffixes like `/v1`,
/// `/api/chat`, or `/v1/chat/completions` don't double up when the downstream
/// provider appends its own path.
pub fn build(client: Client, local: &LocalCredentials) -> Box<dyn LlmProvider> {
    let base_url = normalize_local_base_url(&local.base_url);
    match resolve_protocol(local) {
        "openai_compatible" => Box::new(OpenAiProvider::new(
            client,
            local.api_key.clone().unwrap_or_default(),
            Some(base_url),
        )),
        _ => Box::new(OllamaProvider::new(client, base_url)),
    }
}

/// Known path suffixes to strip from a user-supplied Local base URL so the
/// downstream provider (Ollama or OpenAI-compat) can append its own path
/// cleanly. Ordered longest-first so the first match wins — critical when one
/// suffix is a prefix of another (e.g. `/api/v1/chat/completions` vs `/api`).
const STRIPPABLE_SUFFIXES: &[&str] = &[
    "/api/v1/chat/completions",
    "/api/v1/chat",
    "/v1/chat/completions",
    "/v1/embeddings",
    "/v1/completions",
    "/v1/responses",
    "/api/generate",
    "/v1/models",
    "/api/tags",
    "/api/chat",
    "/api/show",
    "/api/v1",
    "/api",
    "/v1",
];

/// Normalize a user-supplied Local base URL to a bare origin (or origin+path)
/// with any known API suffix removed.
///
/// Behavior:
/// 1. Trim whitespace and trailing slashes.
/// 2. Strip the longest matching suffix from [`STRIPPABLE_SUFFIXES`]
///    (case-insensitive comparison; the actual bytes from the input are
///    removed so the origin preserves its original case).
/// 3. Trim trailing slashes again after strip.
///
/// Used by every Local code path that builds a request URL (provider build +
/// discovery fetchers) so the user can paste virtually any example URL — for
/// example `http://localhost:1234/v1`, `.../api/chat`, or `.../api/v1/chat` —
/// and the app still lands on the right endpoint.
pub fn normalize_local_base_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }

    let lower = trimmed.to_ascii_lowercase();
    for suffix in STRIPPABLE_SUFFIXES {
        if lower.ends_with(suffix) {
            let cut_at = trimmed.len() - suffix.len();
            return trimmed[..cut_at].trim_end_matches('/').to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creds(protocol: &str, detected: Option<&str>) -> LocalCredentials {
        LocalCredentials {
            base_url: "http://localhost:11434".into(),
            protocol: protocol.into(),
            detected_protocol: detected.map(String::from),
            api_key: None,
        }
    }

    #[test]
    fn manual_ollama_overrides_detected() {
        assert_eq!(
            resolve_protocol(&creds("ollama", Some("openai_compatible"))),
            "ollama"
        );
    }

    #[test]
    fn manual_openai_overrides_detected() {
        assert_eq!(
            resolve_protocol(&creds("openai_compatible", Some("ollama"))),
            "openai_compatible"
        );
    }

    #[test]
    fn auto_uses_detected_openai() {
        assert_eq!(
            resolve_protocol(&creds("auto", Some("openai_compatible"))),
            "openai_compatible"
        );
    }

    #[test]
    fn auto_uses_detected_ollama() {
        assert_eq!(
            resolve_protocol(&creds("auto", Some("ollama"))),
            "ollama"
        );
    }

    #[test]
    fn auto_defaults_to_ollama_when_no_detection() {
        assert_eq!(resolve_protocol(&creds("auto", None)), "ollama");
    }

    #[test]
    fn unknown_protocol_falls_back_to_detection() {
        assert_eq!(
            resolve_protocol(&creds("gibberish", Some("openai_compatible"))),
            "openai_compatible"
        );
    }

    #[test]
    fn normalize_bare_origin_passthrough() {
        assert_eq!(
            normalize_local_base_url("http://localhost:11434"),
            "http://localhost:11434"
        );
    }

    #[test]
    fn normalize_strips_v1() {
        assert_eq!(
            normalize_local_base_url("http://localhost:1234/v1"),
            "http://localhost:1234"
        );
    }

    #[test]
    fn normalize_strips_v1_chat_completions() {
        assert_eq!(
            normalize_local_base_url("http://localhost:1234/v1/chat/completions"),
            "http://localhost:1234"
        );
    }

    #[test]
    fn normalize_strips_api_chat() {
        assert_eq!(
            normalize_local_base_url("http://localhost:11434/api/chat"),
            "http://localhost:11434"
        );
    }

    #[test]
    fn normalize_strips_api_v1_chat() {
        // Longest-first ordering — `/api/v1/chat` wins over `/v1/chat` or `/api`.
        assert_eq!(
            normalize_local_base_url("http://localhost:1234/api/v1/chat"),
            "http://localhost:1234"
        );
    }

    #[test]
    fn normalize_trims_trailing_slash() {
        assert_eq!(
            normalize_local_base_url("http://localhost:11434/"),
            "http://localhost:11434"
        );
        assert_eq!(
            normalize_local_base_url("http://localhost:1234/v1/"),
            "http://localhost:1234"
        );
    }

    #[test]
    fn normalize_case_insensitive_match() {
        // Uppercase suffix still gets stripped.
        assert_eq!(
            normalize_local_base_url("http://Host:1234/V1"),
            "http://Host:1234"
        );
        assert_eq!(
            normalize_local_base_url("http://Host:1234/API/CHAT"),
            "http://Host:1234"
        );
    }

    #[test]
    fn normalize_no_match_passthrough() {
        // Unknown paths are preserved (user explicitly specified something).
        assert_eq!(
            normalize_local_base_url("http://localhost:1234/custom/path"),
            "http://localhost:1234/custom/path"
        );
    }

    #[test]
    fn normalize_empty_stays_empty() {
        assert_eq!(normalize_local_base_url(""), "");
        assert_eq!(normalize_local_base_url("   "), "");
    }
}
