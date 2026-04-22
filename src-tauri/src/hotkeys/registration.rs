//! Shortcut registration
//!
//! Functions to register and unregister global shortcuts with the OS.

use crate::config::HotkeyConfig;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use super::{
    parser::parse_shortcut_string,
    ACTION_DICTATION, ACTION_DICTATION_START, ACTION_DICTATION_STOP,
    ACTION_GRAMMAR, ACTION_TRANSLATE, ACTION_IMPROVE,
    ACTION_SCREEN_QUESTION,
};

/// Determine the action for a given shortcut based on the config
fn get_action_for_shortcut(shortcut: &Shortcut, config: &HotkeyConfig) -> Option<&'static str> {
    // Parse all configured shortcuts and compare
    if let Ok(dictation) = parse_shortcut_string(&config.dictation) {
        if shortcut.key == dictation.key && shortcut.mods == dictation.mods {
            return Some(ACTION_DICTATION);
        }
    }
    if let Ok(grammar) = parse_shortcut_string(&config.grammar) {
        if shortcut.key == grammar.key && shortcut.mods == grammar.mods {
            return Some(ACTION_GRAMMAR);
        }
    }
    if let Ok(translate) = parse_shortcut_string(&config.translate) {
        if shortcut.key == translate.key && shortcut.mods == translate.mods {
            return Some(ACTION_TRANSLATE);
        }
    }
    if let Ok(improve) = parse_shortcut_string(&config.improve) {
        if shortcut.key == improve.key && shortcut.mods == improve.mods {
            return Some(ACTION_IMPROVE);
        }
    }
    // Action Wheel (open_menu) is temporarily hidden — kept for future use
    // if let Ok(open_menu) = parse_shortcut_string(&config.open_menu) {
    //     if shortcut.key == open_menu.key && shortcut.mods == open_menu.mods {
    //         return Some(ACTION_OPEN_MENU);
    //     }
    // }
    if let Ok(screen_question) = parse_shortcut_string(&config.screen_question) {
        if shortcut.key == screen_question.key && shortcut.mods == screen_question.mods {
            return Some(ACTION_SCREEN_QUESTION);
        }
    }
    None
}

/// Register shortcuts from a HotkeyConfig
pub fn register_shortcuts_from_config(
    app: &AppHandle,
    config: &HotkeyConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Registering global shortcuts from config");
    log::info!("  Dictation: {}", config.dictation);
    log::info!("  Grammar: {}", config.grammar);
    log::info!("  Translate: {}", config.translate);
    log::info!("  Improve: {}", config.improve);
    // Action Wheel (open_menu) is temporarily hidden
    // log::info!("  Open menu: {}", config.open_menu);
    log::info!("  Screen question: {}", config.screen_question);

    // Unregister any existing shortcuts first to avoid "already registered" errors
    // This handles cases where a previous instance didn't close cleanly
    if let Err(e) = app.global_shortcut().unregister_all() {
        log::warn!(
            "Failed to unregister existing shortcuts (may be none): {}",
            e
        );
    }

    // Parse shortcuts
    let dictation_shortcut = parse_shortcut_string(&config.dictation)
        .map_err(|e| format!("Invalid dictation shortcut: {}", e))?;
    let grammar_shortcut = parse_shortcut_string(&config.grammar)
        .map_err(|e| format!("Invalid grammar shortcut: {}", e))?;
    let translate_shortcut = parse_shortcut_string(&config.translate)
        .map_err(|e| format!("Invalid translate shortcut: {}", e))?;
    let improve_shortcut = parse_shortcut_string(&config.improve)
        .map_err(|e| format!("Invalid improve shortcut: {}", e))?;
    // Action Wheel (open_menu) is temporarily hidden
    // let open_menu_shortcut = parse_shortcut_string(&config.open_menu)
    //     .map_err(|e| format!("Invalid open_menu shortcut: {}", e))?;
    let screen_question_shortcut = parse_shortcut_string(&config.screen_question)
        .map_err(|e| format!("Invalid screen_question shortcut: {}", e))?;

    // Collect shortcuts to register - we'll try each individually
    let shortcuts_to_register = [
        ("dictation", dictation_shortcut, &config.dictation),
        ("grammar", grammar_shortcut, &config.grammar),
        ("translate", translate_shortcut, &config.translate),
        ("improve", improve_shortcut, &config.improve),
        // Action Wheel (open_menu) is temporarily hidden
        // ("open_menu", open_menu_shortcut, &config.open_menu),
        ("screen_question", screen_question_shortcut, &config.screen_question),
    ];

    // Collision detection: two actions bound to the exact same (modifiers, key)
    // pattern would shadow each other non-deterministically at runtime. Reject
    // before we touch the OS, so the frontend shows a clear error.
    //
    // We key by the `Shortcut`'s `Debug` representation — it's stable for
    // identical (mods, key) pairs and avoids importing bitflags/trait-bounds
    // juggling for the `Modifiers` type from tauri_plugin_global_shortcut.
    {
        use std::collections::HashMap;
        let mut seen: HashMap<String, &str> = HashMap::new();
        for (name, shortcut, shortcut_str) in &shortcuts_to_register {
            let key = format!("{:?}", shortcut);
            if let Some(existing) = seen.insert(key, name) {
                return Err(format!(
                    "Hotkey collision: '{}' conflicts with '{}' — \
                     two actions cannot share the same key combination",
                    existing, shortcut_str
                )
                .into());
            }
        }
    }

    // Try to unregister each shortcut first (in case it's held by a crashed process)
    for (name, shortcut, _shortcut_str) in &shortcuts_to_register {
        if let Err(e) = app.global_shortcut().unregister(*shortcut) {
            log::debug!(
                "Could not unregister {} shortcut (may not be registered): {}",
                name,
                e
            );
        }
    }

    // Clone config for the closure
    let config_clone = config.clone();

    // Try to register shortcuts individually using on_shortcut (singular)
    // This allows us to handle failures for each shortcut independently
    let mut registered_count = 0;
    for (name, shortcut, shortcut_str) in shortcuts_to_register {
        let config_for_handler = config_clone.clone();

        let result = app.global_shortcut().on_shortcut(
            shortcut,
            move |app, _shortcut, event| {
                // Determine which action this shortcut corresponds to
                let base_action = match get_action_for_shortcut(_shortcut, &config_for_handler) {
                    Some(action) => action,
                    None => return,
                };

                // For dictation, we need hold-to-talk: send press/release events
                // For other actions, only trigger on press
                let action = if base_action == ACTION_DICTATION {
                    match event.state {
                        ShortcutState::Pressed => ACTION_DICTATION_START,
                        ShortcutState::Released => ACTION_DICTATION_STOP,
                    }
                } else {
                    // Grammar and translate only on press
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    base_action
                };

                log::info!("Shortcut triggered: {} (state: {:?})", action, event.state);

                // Emit event to frontend
                if let Err(e) = app.emit("shortcut-triggered", action) {
                    log::error!("Failed to emit shortcut event: {}", e);
                }
            },
        );

        match result {
            Ok(_) => {
                log::info!("  Registered {} shortcut: {}", name, shortcut_str);
                registered_count += 1;
            }
            Err(e) => {
                log::warn!(
                    "  Failed to register {} shortcut ({}): {} - shortcut may be in use by another app",
                    name,
                    shortcut_str,
                    e
                );
            }
        }
    }

    if registered_count == 0 {
        log::error!("No shortcuts could be registered! All may be in use by other applications.");
        log::error!(
            "Try closing other applications or rebooting if shortcuts were held by a crashed process."
        );
        return Err(
            "No shortcuts could be registered. Try rebooting or closing conflicting apps.".into(),
        );
    }

    log::info!("Global shortcuts registered: {}/5", registered_count);
    Ok(())
}

/// Unregister all global shortcuts
pub fn unregister_all_shortcuts(app: &AppHandle) -> Result<(), String> {
    log::info!("Unregistering all global shortcuts");
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {}", e))?;
    log::info!("All shortcuts unregistered");
    Ok(())
}
