use serde::{Deserialize, Serialize};

fn default_improve_shortcut() -> String {
    crate::hotkeys::DEFAULT_IMPROVE_SHORTCUT.to_string()
}

fn default_open_menu_shortcut() -> String {
    crate::hotkeys::DEFAULT_OPEN_MENU_SHORTCUT.to_string()
}

fn default_screen_question_shortcut() -> String {
    crate::hotkeys::DEFAULT_SCREEN_QUESTION_SHORTCUT.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub dictation: String,
    pub grammar: String,
    pub translate: String,
    #[serde(default = "default_improve_shortcut")]
    pub improve: String,
    #[serde(default = "default_open_menu_shortcut")]
    pub open_menu: String,
    #[serde(default = "default_screen_question_shortcut")]
    pub screen_question: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        use crate::hotkeys::{
            DEFAULT_DICTATION_SHORTCUT, DEFAULT_GRAMMAR_SHORTCUT, DEFAULT_IMPROVE_SHORTCUT,
            DEFAULT_OPEN_MENU_SHORTCUT, DEFAULT_SCREEN_QUESTION_SHORTCUT,
            DEFAULT_TRANSLATE_SHORTCUT,
        };
        Self {
            dictation: DEFAULT_DICTATION_SHORTCUT.to_string(),
            grammar: DEFAULT_GRAMMAR_SHORTCUT.to_string(),
            translate: DEFAULT_TRANSLATE_SHORTCUT.to_string(),
            improve: DEFAULT_IMPROVE_SHORTCUT.to_string(),
            open_menu: DEFAULT_OPEN_MENU_SHORTCUT.to_string(),
            screen_question: DEFAULT_SCREEN_QUESTION_SHORTCUT.to_string(),
        }
    }
}
