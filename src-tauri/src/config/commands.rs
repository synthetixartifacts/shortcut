// Tauri command handlers for configuration

use super::{persist_config, AppConfig, AppSettingsConfig, ConfigState, DictationConfig, GrammarConfig, HotkeyConfig, ImproveConfig, ProvidersConfig, ScreenQuestionConfig, TranscriptionConfig, TranslateConfig, UserConfig};
use tauri::AppHandle;

/// Get configuration from managed state (no disk I/O)
#[tauri::command]
pub fn get_config(state: tauri::State<'_, ConfigState>) -> Result<AppConfig, String> {
    let config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    Ok(config.clone())
}

/// Save full configuration: update state + persist to disk
#[tauri::command]
pub fn save_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    config: AppConfig,
) -> Result<(), String> {
    let mut current = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    *current = config.clone();
    persist_config(&app, &current)
}

/// Get the default improve configuration (for frontend reset functionality)
#[tauri::command]
pub fn get_default_improve_config() -> ImproveConfig {
    ImproveConfig::default()
}

/// Update user profile configuration
#[tauri::command]
pub fn update_user_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    user: UserConfig,
) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.user = user;
    persist_config(&app, &config)
}

/// Update dictation configuration atomically
#[tauri::command]
pub fn update_dictation_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    dictation: DictationConfig,
) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.dictation = dictation;
    persist_config(&app, &config)
}

/// Update hotkey configuration atomically.
///
/// Each shortcut string is validated against the hotkey parser before we touch
/// state or disk — invalid input like "InvalidKey" or "Ctrl+" is rejected with
/// a clear message so the frontend can surface it without corrupting config.
#[tauri::command]
pub fn update_hotkeys_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    hotkeys: HotkeyConfig,
) -> Result<(), String> {
    // Validate every shortcut string up front.
    let entries = [
        ("dictation", &hotkeys.dictation),
        ("grammar", &hotkeys.grammar),
        ("translate", &hotkeys.translate),
        ("improve", &hotkeys.improve),
        ("open_menu", &hotkeys.open_menu),
        ("screen_question", &hotkeys.screen_question),
    ];
    for (name, value) in entries {
        crate::hotkeys::parse_shortcut_string(value)
            .map_err(|e| format!("Invalid '{}' shortcut '{}': {}", name, value, e))?;
    }

    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.hotkeys = hotkeys;
    persist_config(&app, &config)
}

/// Update app settings configuration atomically
#[tauri::command]
pub fn update_app_settings_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    app_settings: AppSettingsConfig,
) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.app_settings = app_settings;
    persist_config(&app, &config)
}

/// Update improve configuration atomically
#[tauri::command]
pub fn update_improve_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    improve: ImproveConfig,
) -> Result<(), String> {
    if !improve.prompt.contains("{text}") {
        return Err("Improve prompt template must contain {text} placeholder".to_string());
    }
    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.improve = improve;
    persist_config(&app, &config)
}

/// Get the active transcription engine
#[tauri::command]
pub fn get_active_engine(state: tauri::State<'_, ConfigState>) -> Result<String, String> {
    let config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    Ok(config.transcription.active_engine.clone())
}

/// Set the active transcription engine
#[tauri::command]
pub fn set_active_engine(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    engine: String,
) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.transcription.active_engine = engine;
    persist_config(&app, &config)
}

/// Update transcription configuration atomically
#[tauri::command]
pub fn update_transcription_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    transcription: TranscriptionConfig,
) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.transcription = transcription;
    persist_config(&app, &config)
}

// =============================================================================
// Phase 2: Provider configuration commands
// =============================================================================

/// Update providers configuration (credentials + task assignments).
///
/// Validates that every task-assignment `provider_id` is a known backend
/// provider. Unknown ids would leave the app unable to dispatch that task, so
/// we reject the save with an actionable message.
#[tauri::command]
pub fn update_providers_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    providers: ProvidersConfig,
) -> Result<(), String> {
    const KNOWN_PROVIDERS: &[&str] = &["openai", "anthropic", "gemini", "grok", "local"];
    const KNOWN_LOCAL_PROTOCOLS: &[&str] = &["auto", "ollama", "openai_compatible"];
    let tasks = [
        ("grammar", &providers.task_assignments.grammar.provider_id),
        ("translate", &providers.task_assignments.translate.provider_id),
        ("improve", &providers.task_assignments.improve.provider_id),
        ("screen_question", &providers.task_assignments.screen_question.provider_id),
    ];
    for (task, provider) in tasks {
        if !KNOWN_PROVIDERS.contains(&provider.as_str()) {
            return Err(format!(
                "Unknown provider '{}' for task '{}'. Expected one of: {}",
                provider,
                task,
                KNOWN_PROVIDERS.join(", ")
            ));
        }
    }

    // Sanity-validate `local.protocol` — catches typos and stale writes before
    // they corrupt the saved config. `resolve_protocol` falls back gracefully
    // on unknown strings, but the save path is the right place to reject them.
    let local_protocol = providers.credentials.local.protocol.as_str();
    if !KNOWN_LOCAL_PROTOCOLS.contains(&local_protocol) {
        return Err(format!(
            "Unknown local protocol '{}'. Expected one of: {}",
            local_protocol,
            KNOWN_LOCAL_PROTOCOLS.join(", ")
        ));
    }

    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.providers = providers;
    persist_config(&app, &config)
}

/// Get providers configuration
#[tauri::command]
pub fn get_providers_config(state: tauri::State<'_, ConfigState>) -> Result<ProvidersConfig, String> {
    let config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    Ok(config.providers.clone())
}

/// Get default grammar configuration (for frontend reset functionality)
#[tauri::command]
pub fn get_default_grammar_config() -> GrammarConfig {
    GrammarConfig::default()
}

/// Get default translate configuration (for frontend reset functionality)
#[tauri::command]
pub fn get_default_translate_config() -> TranslateConfig {
    TranslateConfig::default()
}

/// Update grammar configuration (prompt template)
#[tauri::command]
pub fn update_grammar_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    grammar: GrammarConfig,
) -> Result<(), String> {
    if !grammar.prompt.contains("{text}") {
        return Err("Grammar prompt template must contain {text} placeholder".to_string());
    }
    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.grammar = grammar;
    persist_config(&app, &config)
}

/// Update translate configuration (prompt template)
#[tauri::command]
pub fn update_translate_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    translate: TranslateConfig,
) -> Result<(), String> {
    if !translate.prompt.contains("{text}") {
        return Err("Translate prompt template must contain {text} placeholder".to_string());
    }
    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.translate = translate;
    persist_config(&app, &config)
}

/// Get default screen-question configuration (for frontend reset functionality)
#[tauri::command]
pub fn get_default_screen_question_config() -> ScreenQuestionConfig {
    ScreenQuestionConfig::default()
}

/// Update screen-question configuration (system prompt only — no placeholder validation)
#[tauri::command]
pub fn update_screen_question_config(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    screen_question: ScreenQuestionConfig,
) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    config.screen_question = screen_question;
    persist_config(&app, &config)
}

// =============================================================================
// Provider status report
// =============================================================================

/// Per-provider configuration status for the debug page and dashboard.
#[derive(serde::Serialize)]
pub struct ProviderStatusReport {
    pub openai_configured: bool,
    pub anthropic_configured: bool,
    pub gemini_configured: bool,
    pub grok_configured: bool,
    pub soniox_configured: bool,
    pub local_url: String,
    pub active_engine: String,
    pub grammar_provider: String,
    pub grammar_model: String,
    pub translate_provider: String,
    pub translate_model: String,
    pub improve_provider: String,
    pub improve_model: String,
    pub screen_question_provider: String,
    pub screen_question_model: String,
}

/// Return per-provider configuration status.
/// Reads local config only — no network calls.
#[tauri::command]
pub fn get_provider_status(state: tauri::State<'_, ConfigState>) -> Result<ProviderStatusReport, String> {
    let config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
    let creds = &config.providers.credentials;
    let tasks = &config.providers.task_assignments;
    Ok(ProviderStatusReport {
        openai_configured: !creds.openai_api_key.is_empty(),
        anthropic_configured: !creds.anthropic_api_key.is_empty(),
        gemini_configured: !creds.gemini_api_key.is_empty(),
        grok_configured: !creds.grok_api_key.is_empty(),
        soniox_configured: !creds.soniox_api_key.is_empty(),
        local_url: crate::providers::ollama::normalize_local_chat_url(&creds.local.base_url),
        active_engine: config.transcription.active_engine.clone(),
        grammar_provider: tasks.grammar.provider_id.clone(),
        grammar_model: tasks.grammar.model.clone(),
        translate_provider: tasks.translate.provider_id.clone(),
        translate_model: tasks.translate.model.clone(),
        improve_provider: tasks.improve.provider_id.clone(),
        improve_model: tasks.improve.model.clone(),
        screen_question_provider: tasks.screen_question.provider_id.clone(),
        screen_question_model: tasks.screen_question.model.clone(),
    })
}
