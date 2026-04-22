use serde::{Deserialize, Serialize};

pub fn default_en() -> String { "en".to_string() }
pub fn default_es() -> String { "es".to_string() }
pub fn default_true() -> bool { true }
pub fn default_dark() -> String { "dark".to_string() }

/// App-level settings (theme, language, debug visibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettingsConfig {
    /// UI theme: "light" or "dark"
    #[serde(default = "default_dark")]
    pub theme: String,
    /// UI language: "en", "fr", or "es"
    #[serde(default = "default_en")]
    pub language: String,
    /// Whether debug logs menu is visible
    #[serde(default)]
    pub debug_enabled: bool,
}

impl Default for AppSettingsConfig {
    fn default() -> Self {
        Self {
            theme: default_dark(),
            language: default_en(),
            debug_enabled: false,
        }
    }
}
