//! Local discovery dispatcher.
//!
//! Resolves `local.protocol` to either ollama or openai_compatible discovery.
//! When `protocol == "auto"`, races both probes in parallel; ollama wins
//! ties (MASTER_PLAN R1 — `/api/tags` + `/api/show` exposes strictly more info
//! than `/v1/models`). The winner is cached into `local.detected_protocol`
//! via `ConfigState` so subsequent dispatch doesn't re-probe.
//!
//! Stale-cache guard (MASTER_PLAN R3): the frontend clears
//! `detected_protocol` when the user edits the URL or protocol, so a URL
//! change deterministically re-runs detection the next time Settings kicks
//! off discovery. If both probes fail, we leave the cache as `None` and
//! surface a typed `AppError::Provider` so the next probe runs fresh rather
//! than silently sticking on a wrong guess.

use super::ollama::fetch_ollama_models;
use super::openai_compat::fetch_openai_compat_models;
use super::ProviderModelInfo;
use crate::config::ConfigState;
use crate::errors::{AppError, ProviderErrorKind};
use crate::providers::local::{normalize_local_base_url, resolve_protocol};
use reqwest::Client;
use tauri::{AppHandle, Manager};

pub(super) async fn fetch_local_models(
    app: &AppHandle,
    client: &Client,
) -> Result<Vec<ProviderModelInfo>, AppError> {
    // Snapshot creds under a short-lived lock.
    let mut local = {
        let state = app.state::<ConfigState>();
        let config = state
            .0
            .lock()
            .map_err(|_| AppError::Config("Lock poisoned".into()))?;
        config.providers.credentials.local.clone()
    };

    // When the user picked "auto" and we haven't detected yet (or the cache was
    // cleared by a URL/protocol edit), run the probe race and persist the
    // winner. Explicit (non-"auto") choices skip detection entirely.
    let protocol: &'static str = if local.protocol == "auto" && local.detected_protocol.is_none() {
        let detected = auto_detect(client, &local.base_url).await?;
        local.detected_protocol = Some(detected.to_string());
        // Persist the cache. Failure here is non-fatal — we still return models.
        if let Err(e) = persist_detected(app, detected) {
            log::warn!("[local] failed to persist detected_protocol: {}", e);
        }
        detected
    } else {
        resolve_protocol(&local)
    };

    log::debug!(
        "[local] discovery protocol={} base_url={} api_key={}",
        protocol,
        local.base_url,
        local.api_key.as_deref().map(|k| !k.is_empty()).unwrap_or(false)
    );

    match protocol {
        "openai_compatible" => {
            fetch_openai_compat_models(client, &local.base_url, local.api_key.as_deref()).await
        }
        _ => fetch_ollama_models(client, &local.base_url).await,
    }
}

/// Race ollama + openai-compat probes in parallel. Ollama wins any tie
/// (MASTER_PLAN R1). Each probe must return 2xx AND a JSON body with the
/// expected shape (`{"models": [...]}` for ollama, `{"data": [...]}` for
/// openai-compat) — a plain 2xx isn't enough because some servers (e.g.
/// LM Studio) respond 2xx with an HTML catchall on unknown paths. If *both*
/// fail we return a typed `AppError::Provider` so the caller surfaces an
/// actionable error and the `detected_protocol` cache stays `None` — the
/// next probe re-runs instead of sticking on a bad guess.
async fn auto_detect(client: &Client, base_url: &str) -> Result<&'static str, AppError> {
    let base = normalize_local_base_url(base_url);
    let ollama_url = format!("{base}/api/tags");
    let openai_url = format!("{base}/v1/models");

    let ollama_probe = probe_and_read(client, &ollama_url);
    let openai_probe = probe_and_read(client, &openai_url);
    let (o, p) = tokio::join!(ollama_probe, openai_probe);

    let ollama_ok = match o {
        Some((status, body)) => parse_ollama_probe(status, &body),
        None => false,
    };
    let openai_ok = match p {
        Some((status, body)) => parse_openai_probe(status, &body),
        None => false,
    };

    match (ollama_ok, openai_ok) {
        (true, _) => {
            // Ollama wins ties (R1).
            log::info!("[local] auto-detect: ollama");
            Ok("ollama")
        }
        (false, true) => {
            log::info!("[local] auto-detect: openai_compatible");
            Ok("openai_compatible")
        }
        (false, false) => {
            log::info!(
                "[local] auto-detect: both probes failed ({} and {})",
                ollama_url,
                openai_url
            );
            Err(AppError::provider(
                ProviderErrorKind::Network,
                format!(
                    "Couldn't auto-detect Local protocol. Tried {} and {} — neither responded. Check the URL or set protocol manually.",
                    ollama_url, openai_url
                ),
            ))
        }
    }
}

/// Send a GET and collect (status, body_text). Returns `None` on network /
/// body-read errors so the probe counts as failed.
async fn probe_and_read(client: &Client, url: &str) -> Option<(reqwest::StatusCode, String)> {
    let resp = client.get(url).send().await.ok()?;
    let status = resp.status();
    let body = resp.text().await.ok()?;
    Some((status, body))
}

/// Shape check for the Ollama probe: 2xx + JSON with a top-level `models`
/// array. Catches LM Studio's catchall where `/api/tags` returns 2xx but the
/// body is HTML or a JSON object without `models`.
pub(super) fn parse_ollama_probe(status: reqwest::StatusCode, body: &str) -> bool {
    if !status.is_success() {
        return false;
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };
    value
        .get("models")
        .and_then(|v| v.as_array())
        .is_some()
}

/// Shape check for the OpenAI-compat probe: 2xx + JSON with a top-level
/// `data` array. Ollama's `/v1/models` also returns this shape, so the tie
/// is broken by ollama winning (R1) at the caller.
pub(super) fn parse_openai_probe(status: reqwest::StatusCode, body: &str) -> bool {
    if !status.is_success() {
        return false;
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };
    value
        .get("data")
        .and_then(|v| v.as_array())
        .is_some()
}

fn persist_detected(app: &AppHandle, detected: &str) -> Result<(), String> {
    let state = app.state::<ConfigState>();
    let mut config = state
        .0
        .lock()
        .map_err(|_| "Config lock poisoned".to_string())?;
    config.providers.credentials.local.detected_protocol = Some(detected.to_string());
    crate::config::persist_config(app, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn ollama_probe_accepts_models_array() {
        assert!(parse_ollama_probe(StatusCode::OK, r#"{"models": []}"#));
        assert!(parse_ollama_probe(
            StatusCode::OK,
            r#"{"models": [{"name": "gemma3"}]}"#
        ));
    }

    #[test]
    fn ollama_probe_rejects_catchall_json_without_models() {
        // LM Studio's catchall may respond 2xx with unrelated JSON.
        assert!(!parse_ollama_probe(StatusCode::OK, r#"{"foo": "bar"}"#));
        // `models` exists but isn't an array — still rejected.
        assert!(!parse_ollama_probe(
            StatusCode::OK,
            r#"{"models": "not-an-array"}"#
        ));
    }

    #[test]
    fn ollama_probe_rejects_non_2xx() {
        assert!(!parse_ollama_probe(
            StatusCode::NOT_FOUND,
            r#"{"models": []}"#
        ));
        assert!(!parse_ollama_probe(
            StatusCode::INTERNAL_SERVER_ERROR,
            r#"{"models": []}"#
        ));
    }

    #[test]
    fn openai_probe_accepts_data_array() {
        assert!(parse_openai_probe(StatusCode::OK, r#"{"data": []}"#));
        assert!(parse_openai_probe(
            StatusCode::OK,
            r#"{"data": [{"id": "llama-3.2-3b-instruct"}], "object": "list"}"#
        ));
    }

    #[test]
    fn openai_probe_rejects_html_body() {
        // Catchall HTML — not valid JSON, must fail.
        assert!(!parse_openai_probe(
            StatusCode::OK,
            "<!DOCTYPE html><html><body>404</body></html>"
        ));
        // JSON without `data` field.
        assert!(!parse_openai_probe(StatusCode::OK, r#"{"models": []}"#));
    }
}
