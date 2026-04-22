use super::app::{default_en, default_es, default_true};
use serde::{Deserialize, Serialize};

/// Audio processing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub noise_suppression: bool,
    pub echo_cancellation: bool,
    pub auto_gain_control: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            noise_suppression: true,
            echo_cancellation: true,
            auto_gain_control: true,
        }
    }
}

/// Translation term mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationTerm {
    pub source: String,
    pub target: String,
}

fn default_language_hints() -> Vec<String> {
    Vec::new()
}
fn default_translation_mode() -> String { "off".to_string() }
fn default_engine() -> String { "soniox".to_string() }

/// Dictation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationConfig {
    pub selected_microphone_id: Option<String>,
    #[serde(default)]
    pub audio_settings: AudioSettings,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(default)]
    pub background_text: String,
    #[serde(default)]
    pub custom_terms: Vec<String>,
    #[serde(default = "default_language_hints")]
    pub language_hints: Vec<String>,
    #[serde(default)]
    pub enable_language_identification: bool,
    #[serde(default = "default_translation_mode")]
    pub translation_mode: String,
    #[serde(default = "default_en")]
    pub translation_target_language: String,
    #[serde(default = "default_en")]
    pub translation_language_a: String,
    #[serde(default = "default_es")]
    pub translation_language_b: String,
    #[serde(default)]
    pub translation_terms: Vec<TranslationTerm>,
    #[serde(default = "default_true")]
    pub enable_endpoint_detection: bool,
    #[serde(default)]
    pub enable_speaker_diarization: bool,
}

impl Default for DictationConfig {
    fn default() -> Self {
        Self {
            selected_microphone_id: None,
            audio_settings: AudioSettings::default(),
            topic: String::new(),
            names: Vec::new(),
            background_text: String::new(),
            custom_terms: Vec::new(),
            language_hints: default_language_hints(),
            enable_language_identification: false,
            translation_mode: default_translation_mode(),
            translation_target_language: default_en(),
            translation_language_a: default_en(),
            translation_language_b: default_es(),
            translation_terms: Vec::new(),
            enable_endpoint_detection: true,
            enable_speaker_diarization: false,
        }
    }
}

/// Transcription engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    /// Active engine: "soniox", "local-windows", "local-macos"
    #[serde(default = "default_engine")]
    pub active_engine: String,
    #[serde(default)]
    pub first_run_completed: bool,
    #[serde(default)]
    pub slowness_dismissed: bool,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            active_engine: default_engine(),
            first_run_completed: false,
            slowness_dismissed: false,
        }
    }
}
