//! Global shortcut handling for ShortCut
//!
//! Supports dynamic registration/unregistration of keyboard shortcuts.
//!
//! ## Module Structure
//! - `parser` - Shortcut string parsing
//! - `registration` - Register/unregister shortcuts

mod parser;
mod registration;

use crate::config::HotkeyConfig;
use crate::errors::AppError;
use tauri::AppHandle;

// Re-export public items
pub use parser::parse_shortcut_string;
pub use registration::{register_shortcuts_from_config, unregister_all_shortcuts};

/// Action name constants (used in shortcut events emitted to frontend)
pub const ACTION_DICTATION: &str = "dictation";
pub const ACTION_DICTATION_START: &str = "dictation_start";
pub const ACTION_DICTATION_STOP: &str = "dictation_stop";
pub const ACTION_GRAMMAR: &str = "grammar";
pub const ACTION_TRANSLATE: &str = "translate";
pub const ACTION_IMPROVE: &str = "improve";
#[allow(dead_code)] // Reserved: wired into default bindings; emitter path currently commented.
pub const ACTION_OPEN_MENU: &str = "open_menu";
pub const ACTION_SCREEN_QUESTION: &str = "screen_question";

/// Default hotkey configurations
/// macOS: Cmd+Shift+key (Option+key produces special characters)
/// Windows/Linux: Alt+key
#[cfg(target_os = "macos")]
pub const DEFAULT_DICTATION_SHORTCUT: &str = "Cmd+Shift+D";
#[cfg(target_os = "macos")]
pub const DEFAULT_GRAMMAR_SHORTCUT: &str = "Cmd+Shift+G";
#[cfg(target_os = "macos")]
pub const DEFAULT_TRANSLATE_SHORTCUT: &str = "Cmd+Shift+T";
#[cfg(target_os = "macos")]
pub const DEFAULT_IMPROVE_SHORTCUT: &str = "Cmd+Shift+I";
#[cfg(target_os = "macos")]
pub const DEFAULT_OPEN_MENU_SHORTCUT: &str = "Cmd+Shift+J";
#[cfg(target_os = "macos")]
pub const DEFAULT_SCREEN_QUESTION_SHORTCUT: &str = "Cmd+Shift+S";

#[cfg(not(target_os = "macos"))]
pub const DEFAULT_DICTATION_SHORTCUT: &str = "Alt+D";
#[cfg(not(target_os = "macos"))]
pub const DEFAULT_GRAMMAR_SHORTCUT: &str = "Alt+G";
#[cfg(not(target_os = "macos"))]
pub const DEFAULT_TRANSLATE_SHORTCUT: &str = "Alt+T";
#[cfg(not(target_os = "macos"))]
pub const DEFAULT_IMPROVE_SHORTCUT: &str = "Alt+I";
#[cfg(not(target_os = "macos"))]
pub const DEFAULT_OPEN_MENU_SHORTCUT: &str = "Alt+J";
#[cfg(not(target_os = "macos"))]
pub const DEFAULT_SCREEN_QUESTION_SHORTCUT: &str = "Alt+S";

/// Get default hotkey configuration
pub fn get_default_hotkeys() -> HotkeyConfig {
    HotkeyConfig {
        dictation: DEFAULT_DICTATION_SHORTCUT.to_string(),
        grammar: DEFAULT_GRAMMAR_SHORTCUT.to_string(),
        translate: DEFAULT_TRANSLATE_SHORTCUT.to_string(),
        improve: DEFAULT_IMPROVE_SHORTCUT.to_string(),
        open_menu: DEFAULT_OPEN_MENU_SHORTCUT.to_string(),
        screen_question: DEFAULT_SCREEN_QUESTION_SHORTCUT.to_string(),
    }
}

/// Shortcut information for UI display
#[derive(serde::Serialize, Clone)]
pub struct ShortcutInfo {
    pub action: String,
    pub shortcut: String,
    pub description: String,
}

// ============================================
// Tauri Commands
// ============================================

/// Update shortcuts at runtime - unregisters old ones and registers new ones.
///
/// Returns a classified [`AppError`] so the frontend can distinguish a parse
/// failure (bad shortcut string), a collision, or a generic OS registration
/// error. This is the first command migrated from `Result<_, String>` to
/// `Result<_, AppError>`; migrating the remaining `Result<_, String>` commands
/// is tracked as follow-up work.
#[tauri::command]
pub async fn update_shortcuts(app: AppHandle, hotkeys: HotkeyConfig) -> Result<(), AppError> {
    log::info!("Updating shortcuts to new configuration");

    // Unregister existing shortcuts
    unregister_all_shortcuts(&app).map_err(AppError::Config)?;

    // Register new shortcuts
    register_shortcuts_from_config(&app, &hotkeys)
        .map_err(|e| AppError::Config(format!("Failed to register new shortcuts: {}", e)))?;

    Ok(())
}

/// Get the default shortcut configuration
#[tauri::command]
pub fn get_default_shortcuts() -> HotkeyConfig {
    get_default_hotkeys()
}

/// Get list of currently registered shortcuts (for settings UI)
#[tauri::command]
pub fn get_registered_shortcuts(
    state: tauri::State<'_, crate::config::ConfigState>,
) -> Vec<ShortcutInfo> {
    // Read from managed state, fall back to defaults
    let config = match state.0.lock() {
        Ok(cfg) => cfg.hotkeys.clone(),
        Err(_) => get_default_hotkeys(),
    };

    vec![
        ShortcutInfo {
            action: ACTION_DICTATION.to_string(),
            shortcut: config.dictation,
            description: "Hold to dictate (release to paste)".to_string(),
        },
        ShortcutInfo {
            action: ACTION_GRAMMAR.to_string(),
            shortcut: config.grammar,
            description: "Fix grammar of selected text".to_string(),
        },
        ShortcutInfo {
            action: ACTION_TRANSLATE.to_string(),
            shortcut: config.translate,
            description: "Translate selected text".to_string(),
        },
        ShortcutInfo {
            action: ACTION_IMPROVE.to_string(),
            shortcut: config.improve,
            description: "Improve selected text with AI".to_string(),
        },
        // Action Wheel (open_menu) is temporarily hidden
        // ShortcutInfo {
        //     action: ACTION_OPEN_MENU.to_string(),
        //     shortcut: config.open_menu,
        //     description: "Open action wheel".to_string(),
        // },
        ShortcutInfo {
            action: ACTION_SCREEN_QUESTION.to_string(),
            shortcut: config.screen_question,
            description: "Ask about your screen".to_string(),
        },
    ]
}
