use serde::{Deserialize, Serialize};

fn default_local_base_url() -> String { "http://localhost:11434/api/chat".to_string() }
fn default_local_protocol() -> String { "auto".to_string() }
fn default_soniox_url() -> String { "https://api.soniox.com".to_string() }

/// Local LLM provider credentials.
///
/// Singleton in v1 (MASTER_PLAN D7); shape designed so a future
/// `local_profiles: [...]` can slot in additively without breaking config.
///
/// `protocol`:
/// - `"auto"` — run protocol auto-detect on save (Phase 3).
/// - `"ollama"` — use the Ollama-native API.
/// - `"openai_compatible"` — use an OpenAI-compatible server (LM Studio, etc.).
///
/// `detected_protocol` caches the last successful auto-detect so dispatch
/// doesn't probe on every call.
///
/// `api_key` is optional and only meaningful for openai-compatible servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalCredentials {
    #[serde(default = "default_local_base_url")]
    pub base_url: String,
    #[serde(default = "default_local_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub detected_protocol: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
}

impl Default for LocalCredentials {
    fn default() -> Self {
        Self {
            base_url: default_local_base_url(),
            protocol: default_local_protocol(),
            detected_protocol: None,
            api_key: None,
        }
    }
}

/// Per-provider API credentials
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderCredentials {
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub anthropic_api_key: String,
    #[serde(default)]
    pub gemini_api_key: String,
    #[serde(default)]
    pub grok_api_key: String,
    #[serde(default)]
    pub soniox_api_key: String,
    /// Local LLM credentials (base URL + protocol + optional API key).
    #[serde(default)]
    pub local: LocalCredentials,
    /// Legacy field retained for read-time migration only.
    /// `#[serde(skip_serializing_if = "String::is_empty")]` keeps fresh saves
    /// clean; the migration in `config/mod.rs` copies the value into
    /// `local.base_url` on load and then clears it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub ollama_base_url: String,
    /// Legacy hidden field kept for config compatibility; ignored by routing.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub openai_base_url: String,
    /// Legacy hidden field kept for config compatibility; ignored by routing.
    #[serde(default = "default_soniox_url")]
    pub soniox_base_url: String,
}

/// Task-to-provider assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    /// Provider ID: "openai", "anthropic", "gemini", "grok", "local", "soniox", "local-windows"
    pub provider_id: String,
    pub model: String,
    /// Per-model vision capability as reported by the provider's discovery endpoint.
    /// `None` means "unknown" — callers should fall back to the provider-level
    /// [`ProviderCapabilities::supports_vision`] flag. `Some(true)` unlocks vision
    /// for providers like Grok/Local where the flag depends on which model the
    /// user picked (per-model, not per-provider).
    #[serde(default)]
    pub supports_vision: Option<bool>,
}

fn default_grammar_assignment() -> TaskAssignment {
    TaskAssignment { provider_id: "openai".to_string(), model: "gpt-4o-mini".to_string(), supports_vision: None }
}
fn default_translate_assignment() -> TaskAssignment {
    TaskAssignment { provider_id: "openai".to_string(), model: "gpt-4o-mini".to_string(), supports_vision: None }
}
fn default_improve_assignment() -> TaskAssignment {
    TaskAssignment { provider_id: "openai".to_string(), model: "gpt-4o".to_string(), supports_vision: None }
}
fn default_screen_assignment() -> TaskAssignment {
    TaskAssignment { provider_id: "openai".to_string(), model: "gpt-4o".to_string(), supports_vision: Some(true) }
}

/// Task assignments: which provider+model handles each AI task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignments {
    #[serde(default = "default_grammar_assignment")]
    pub grammar: TaskAssignment,
    #[serde(default = "default_translate_assignment")]
    pub translate: TaskAssignment,
    #[serde(default = "default_improve_assignment")]
    pub improve: TaskAssignment,
    #[serde(default = "default_screen_assignment")]
    pub screen_question: TaskAssignment,
}

impl Default for TaskAssignments {
    fn default() -> Self {
        Self {
            grammar: default_grammar_assignment(),
            translate: default_translate_assignment(),
            improve: default_improve_assignment(),
            screen_question: default_screen_assignment(),
        }
    }
}

/// Top-level providers configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub credentials: ProviderCredentials,
    #[serde(default)]
    pub task_assignments: TaskAssignments,
}
