//! Unified error type for the ShortCut application.
//!
//! `AppError` is Serializable and returned from every Tauri command so the
//! frontend gets a consistent shape. For provider-surface failures we carry a
//! structured [`ProviderErrorKind`] so the UI can branch on Auth vs RateLimit
//! vs Server — enabling actionable banners ("Retry", "Reconfigure", etc.).

use serde::Serialize;

/// Classification of provider HTTP failures — lets the frontend branch on the
/// *kind* of failure instead of string-matching on error messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderErrorKind {
    /// 401/403: invalid or missing API key. User should reconfigure.
    Auth,
    /// 429: rate-limited. `retry_after_secs` is parsed from the `Retry-After`
    /// header when present.
    RateLimit { retry_after_secs: Option<u64> },
    /// 400: the request was rejected as malformed by the provider.
    InvalidRequest,
    /// 5xx: provider-side server error. Usually transient — retry is sensible.
    Server { status: u16 },
    /// Transport-layer failure (DNS, TCP, TLS, timeouts).
    Network,
    /// Provider returned bytes we could not decode (JSON/SSE/UTF-8).
    Parse,
    /// Any other failure mode not covered above.
    Other,
}

/// Application-wide error type.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    General(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Classified provider failure with structured payload for the frontend.
    /// The `Display` impl keeps it readable in logs; the `Serialize` impl
    /// surfaces `kind` so the UI can show the right affordance.
    #[error("Provider error ({kind:?}): {message}")]
    Provider {
        kind: ProviderErrorKind,
        message: String,
    },
}

impl AppError {
    /// Shorthand constructor for classified provider errors.
    pub fn provider(kind: ProviderErrorKind, message: impl Into<String>) -> Self {
        AppError::Provider {
            kind,
            message: message.into(),
        }
    }
}

// Tauri requires command return errors to be Serialize. `AppError::Provider`
// serializes as `{ "kind": ..., "message": ... }` so the frontend can read the
// structured payload; every other variant serializes as its `Display` string,
// preserving the shape existing frontend code expects.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        match self {
            AppError::Provider { kind, message } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("error_type", "provider")?;
                map.serialize_entry("kind", kind)?;
                map.serialize_entry("message", message)?;
                map.end()
            }
            other => serializer.serialize_str(&other.to_string()),
        }
    }
}
