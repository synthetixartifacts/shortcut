//! Shortcut string parsing
//!
//! Parses shortcut strings like "Ctrl+Shift+D" into Tauri Shortcut objects.

use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

/// Parse a shortcut string into modifiers and key code
/// Supported modifiers: Control, Ctrl, Alt, Shift, Super, Meta, Command, Cmd
/// Example: "Super+Shift+Space" -> (SUPER | SHIFT, Space)
pub fn parse_shortcut_string(shortcut: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = shortcut.split('+').map(|s| s.trim()).collect();

    if parts.is_empty() {
        return Err("Empty shortcut string".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        let part_lower = part.to_lowercase();

        match part_lower.as_str() {
            // Control variants
            "control" | "ctrl" => modifiers |= Modifiers::CONTROL,
            // Alt variants
            "alt" | "option" => modifiers |= Modifiers::ALT,
            // Shift
            "shift" => modifiers |= Modifiers::SHIFT,
            // Super/Meta/Command (Windows key on Windows, Cmd on macOS)
            "super" | "meta" | "command" | "cmd" | "win" => modifiers |= Modifiers::SUPER,
            // Key codes
            _ => {
                if key_code.is_some() {
                    return Err(format!("Multiple key codes found: {}", part));
                }
                key_code = Some(parse_key_code(part)?);
            }
        }
    }

    let key = key_code.ok_or("No key code found in shortcut string")?;

    if modifiers.is_empty() {
        return Err("At least one modifier is required".to_string());
    }

    Ok(Shortcut::new(Some(modifiers), key))
}

/// Parse a key name to Code enum
pub fn parse_key_code(key: &str) -> Result<Code, String> {
    let key_upper = key.to_uppercase();

    match key_upper.as_str() {
        // Letters
        "A" => Ok(Code::KeyA),
        "B" => Ok(Code::KeyB),
        "C" => Ok(Code::KeyC),
        "D" => Ok(Code::KeyD),
        "E" => Ok(Code::KeyE),
        "F" => Ok(Code::KeyF),
        "G" => Ok(Code::KeyG),
        "H" => Ok(Code::KeyH),
        "I" => Ok(Code::KeyI),
        "J" => Ok(Code::KeyJ),
        "K" => Ok(Code::KeyK),
        "L" => Ok(Code::KeyL),
        "M" => Ok(Code::KeyM),
        "N" => Ok(Code::KeyN),
        "O" => Ok(Code::KeyO),
        "P" => Ok(Code::KeyP),
        "Q" => Ok(Code::KeyQ),
        "R" => Ok(Code::KeyR),
        "S" => Ok(Code::KeyS),
        "T" => Ok(Code::KeyT),
        "U" => Ok(Code::KeyU),
        "V" => Ok(Code::KeyV),
        "W" => Ok(Code::KeyW),
        "X" => Ok(Code::KeyX),
        "Y" => Ok(Code::KeyY),
        "Z" => Ok(Code::KeyZ),

        // Numbers
        "0" | "DIGIT0" => Ok(Code::Digit0),
        "1" | "DIGIT1" => Ok(Code::Digit1),
        "2" | "DIGIT2" => Ok(Code::Digit2),
        "3" | "DIGIT3" => Ok(Code::Digit3),
        "4" | "DIGIT4" => Ok(Code::Digit4),
        "5" | "DIGIT5" => Ok(Code::Digit5),
        "6" | "DIGIT6" => Ok(Code::Digit6),
        "7" | "DIGIT7" => Ok(Code::Digit7),
        "8" | "DIGIT8" => Ok(Code::Digit8),
        "9" | "DIGIT9" => Ok(Code::Digit9),

        // Function keys
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),

        // Special keys
        "SPACE" | "SPACEBAR" => Ok(Code::Space),
        "ENTER" | "RETURN" => Ok(Code::Enter),
        "TAB" => Ok(Code::Tab),
        "BACKSPACE" => Ok(Code::Backspace),
        "DELETE" | "DEL" => Ok(Code::Delete),
        "ESCAPE" | "ESC" => Ok(Code::Escape),
        "INSERT" | "INS" => Ok(Code::Insert),
        "HOME" => Ok(Code::Home),
        "END" => Ok(Code::End),
        "PAGEUP" | "PGUP" => Ok(Code::PageUp),
        "PAGEDOWN" | "PGDN" => Ok(Code::PageDown),

        // Arrow keys
        "UP" | "ARROWUP" => Ok(Code::ArrowUp),
        "DOWN" | "ARROWDOWN" => Ok(Code::ArrowDown),
        "LEFT" | "ARROWLEFT" => Ok(Code::ArrowLeft),
        "RIGHT" | "ARROWRIGHT" => Ok(Code::ArrowRight),

        // Punctuation
        "MINUS" | "-" => Ok(Code::Minus),
        "EQUAL" | "=" => Ok(Code::Equal),
        "BRACKETLEFT" | "[" => Ok(Code::BracketLeft),
        "BRACKETRIGHT" | "]" => Ok(Code::BracketRight),
        "BACKSLASH" | "\\" => Ok(Code::Backslash),
        "SEMICOLON" | ";" => Ok(Code::Semicolon),
        "QUOTE" | "'" => Ok(Code::Quote),
        "BACKQUOTE" | "`" => Ok(Code::Backquote),
        "COMMA" | "," => Ok(Code::Comma),
        "PERIOD" | "." => Ok(Code::Period),
        "SLASH" | "/" => Ok(Code::Slash),

        _ => Err(format!("Unknown key code: {}", key)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_rejected() {
        // Parser currently treats "" as a single empty part and rejects it at
        // the "no key code" / "no modifier" check. Either is acceptable — just
        // make sure it doesn't produce a Shortcut.
        assert!(parse_shortcut_string("").is_err());
    }

    #[test]
    fn single_modifier_without_key_is_rejected() {
        assert!(parse_shortcut_string("Ctrl").is_err());
        assert!(parse_shortcut_string("Shift").is_err());
    }

    #[test]
    fn key_without_modifier_is_rejected() {
        assert!(parse_shortcut_string("A").is_err());
        assert!(parse_shortcut_string("Space").is_err());
    }

    #[test]
    fn multiple_keys_is_rejected() {
        assert!(parse_shortcut_string("Ctrl+A+B").is_err());
    }

    #[test]
    fn invalid_key_name_is_rejected() {
        assert!(parse_shortcut_string("Ctrl+NotAKey").is_err());
    }

    #[test]
    fn each_modifier_alias_parses() {
        // Control
        assert!(parse_shortcut_string("Ctrl+A").is_ok());
        assert!(parse_shortcut_string("Control+A").is_ok());
        // Alt
        assert!(parse_shortcut_string("Alt+A").is_ok());
        assert!(parse_shortcut_string("Option+A").is_ok());
        // Shift
        assert!(parse_shortcut_string("Shift+A").is_ok());
        // Super/Meta/Command/Cmd/Win
        assert!(parse_shortcut_string("Super+A").is_ok());
        assert!(parse_shortcut_string("Meta+A").is_ok());
        assert!(parse_shortcut_string("Command+A").is_ok());
        assert!(parse_shortcut_string("Cmd+A").is_ok());
        assert!(parse_shortcut_string("Win+A").is_ok());
    }

    #[test]
    fn combined_modifiers_parse() {
        assert!(parse_shortcut_string("Ctrl+Shift+D").is_ok());
        assert!(parse_shortcut_string("Cmd+Shift+Alt+T").is_ok());
    }

    #[test]
    fn case_insensitive() {
        assert!(parse_shortcut_string("ctrl+shift+d").is_ok());
        assert!(parse_shortcut_string("CTRL+SHIFT+D").is_ok());
    }

    #[test]
    fn digit_and_function_keys() {
        assert!(parse_shortcut_string("Ctrl+1").is_ok());
        assert!(parse_shortcut_string("Ctrl+F12").is_ok());
    }

    #[test]
    fn punctuation_aliases() {
        assert!(parse_shortcut_string("Ctrl+-").is_ok());
        assert!(parse_shortcut_string("Ctrl+=").is_ok());
        assert!(parse_shortcut_string("Ctrl+/").is_ok());
        assert!(parse_shortcut_string("Ctrl+.").is_ok());
    }

    #[test]
    fn parse_key_code_knows_enter_aliases() {
        assert!(parse_key_code("Enter").is_ok());
        assert!(parse_key_code("Return").is_ok());
    }
}
