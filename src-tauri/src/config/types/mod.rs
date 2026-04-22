// Configuration type definitions for ShortCut
//
// Split across submodules to keep each file small:
//   user       — UserConfig
//   hotkeys    — HotkeyConfig
//   dictation  — DictationConfig, AudioSettings, TranslationTerm, TranscriptionConfig
//   app        — AppSettingsConfig + app-wide default helpers
//   prompts    — ImproveConfig, GrammarConfig, TranslateConfig, ScreenQuestionConfig + default prompts
//   providers  — ProviderCredentials, TaskAssignment, TaskAssignments, ProvidersConfig

mod app;
mod dictation;
mod hotkeys;
mod prompts;
mod providers;
mod user;

pub use app::AppSettingsConfig;
pub use dictation::{DictationConfig, TranscriptionConfig};
pub use hotkeys::HotkeyConfig;
pub use prompts::{GrammarConfig, ImproveConfig, ScreenQuestionConfig, TranslateConfig};
pub use providers::{LocalCredentials, ProvidersConfig};
pub use user::UserConfig;

use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub hotkeys: HotkeyConfig,
    #[serde(default)]
    pub user: UserConfig,
    #[serde(default)]
    pub dictation: DictationConfig,
    #[serde(default)]
    pub app_settings: AppSettingsConfig,
    #[serde(default)]
    pub improve: ImproveConfig,
    #[serde(default)]
    pub transcription: TranscriptionConfig,
    #[serde(default)]
    pub providers: ProvidersConfig,
    #[serde(default)]
    pub grammar: GrammarConfig,
    #[serde(default)]
    pub translate: TranslateConfig,
    #[serde(default)]
    pub screen_question: ScreenQuestionConfig,
    /// Schema version for Local-protocol detection migrations. Bumped once per
    /// migration step in `config::migrate_providers_config`. A fresh config
    /// serializes this at `0` which triggers the first migration pass.
    #[serde(default)]
    pub local_detection_schema_version: u32,
}
