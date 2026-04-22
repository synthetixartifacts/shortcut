//! Clipboard operations and input simulation for ShortCut
//!
//! Note: On macOS, keyboard simulation (enigo) MUST run on the main thread.
//! The TSM (Text Services Manager) APIs crash if called from a worker thread.

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde::Serialize;
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
#[cfg(target_os = "macos")]
use tauri::Manager as _;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Delay in milliseconds for clipboard operations
const CLIPBOARD_DELAY_MS: u64 = 100;
/// Delay after releasing modifier keys
const KEY_RELEASE_DELAY_MS: u64 = 50;
/// Maximum retries for clipboard read operations
const MAX_CLIPBOARD_RETRIES: u32 = 3;
/// Delay between clipboard read retries
const RETRY_DELAY_MS: u64 = 100;
/// Delay after paste simulation before restoring clipboard.
/// Must be long enough for heavy apps (e.g. Word) to finish reading the clipboard.
const PASTE_SETTLE_DELAY_MS: u64 = 500;

/// Log a message from the frontend (for debugging)
#[tauri::command]
pub fn frontend_log(message: String) {
    log::info!("[Frontend] {}", message);
}

/// Paste text at the current cursor position.
///
/// Saves clipboard, writes text, simulates Ctrl+V, then restores clipboard.
#[tauri::command]
pub async fn paste_text(app: AppHandle, text: String) -> Result<(), String> {
    log::info!("Pasting text ({} chars)", text.len());

    let saved_clipboard = app.clipboard().read_text().ok();

    let result: Result<(), String> = async {
        app.clipboard()
            .write_text(&text)
            .map_err(|e| format!("Failed to write to clipboard: {}", e))?;

        tokio::time::sleep(Duration::from_millis(CLIPBOARD_DELAY_MS)).await;
        simulate_key_action(&app, 'v')?;
        tokio::time::sleep(Duration::from_millis(PASTE_SETTLE_DELAY_MS)).await;

        Ok(())
    }
    .await;

    if let Some(saved) = saved_clipboard {
        let _ = app.clipboard().write_text(&saved);
    }

    result?;
    log::info!("Text pasted successfully");
    Ok(())
}

/// Simulate a modifier+key action (Cmd+key on macOS, Ctrl+key elsewhere).
/// On macOS, runs on main thread due to TSM API requirements.
fn simulate_key_action(app: &AppHandle, key: char) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        app.run_on_main_thread(move || {
            let _ = tx.send(simulate_key_inner(Key::Meta, key));
        })
        .map_err(|e| format!("Failed to run on main thread: {}", e))?;
        rx.recv()
            .map_err(|e| format!("Failed to receive result: {}", e))?
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        simulate_key_inner(Key::Control, key)
    }
}

/// Inner key simulation: release all modifiers, then press modifier+key.
fn simulate_key_inner(modifier: Key, key: char) -> Result<(), String> {
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Failed to create Enigo: {}", e))?;

    release_modifiers(&mut enigo);
    thread::sleep(Duration::from_millis(KEY_RELEASE_DELAY_MS));

    enigo
        .key(modifier, Direction::Press)
        .map_err(|e| format!("Failed to press modifier: {}", e))?;
    enigo
        .key(Key::Unicode(key), Direction::Click)
        .map_err(|e| format!("Failed to press key: {}", e))?;
    enigo
        .key(modifier, Direction::Release)
        .map_err(|e| format!("Failed to release modifier: {}", e))?;
    Ok(())
}

/// Release ALL modifier keys unconditionally.
///
/// Critical for clipboard operations triggered by global shortcuts.
/// When user holds a shortcut combo, those keys may still be held
/// when we simulate copy/paste, causing incorrect key combinations.
///
/// Must be unconditional: Tauri's RegisterHotKey consumes key events at the OS level,
/// so target apps (especially Electron apps like Teams) may retain stale modifier state
/// even after the user physically releases the keys. Sending explicit key-up events
/// for ALL modifiers ensures a clean slate before simulating Ctrl+C / Ctrl+V.
///
/// The prevent_alt_win_menu hook handles Alt menu activation separately and only
/// triggers on lone Alt taps (Alt-down → Alt-up with no other keys), so the
/// unconditional Alt release here does not cause spurious menu activation.
fn release_modifiers(enigo: &mut Enigo) {
    let _ = enigo.key(Key::Shift, Direction::Release);
    let _ = enigo.key(Key::Control, Direction::Release);
    let _ = enigo.key(Key::Alt, Direction::Release);
    let _ = enigo.key(Key::Meta, Direction::Release);
}

// ============================================
// Format-aware clipboard operations
// ============================================

/// Result of reading a selection with format detection
#[derive(Serialize)]
pub struct SelectionResult {
    pub text: String,
    pub format: String,
}

/// Try to read HTML from the clipboard via arboard (bypasses Tauri plugin).
fn read_clipboard_html() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    clipboard.get().html().ok().filter(|h: &String| !h.is_empty())
}

/// Convert HTML to Markdown using html2md.
fn html_to_markdown(html: &str) -> String {
    html2md::parse_html(html)
}

/// Convert Markdown to HTML using pulldown-cmark.
fn markdown_to_html(md: &str) -> String {
    let parser = pulldown_cmark::Parser::new(md);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output
}

/// Get selected text with format detection.
///
/// Saves clipboard, simulates Ctrl+C, then tries HTML first via arboard.
/// If HTML is found, converts to Markdown and returns format="markdown".
/// Otherwise falls back to plain text with format="plain".
#[tauri::command]
pub async fn get_selection_with_format(app: AppHandle) -> Result<SelectionResult, String> {
    log::info!("Getting selected text with format detection");

    let saved_clipboard = app.clipboard().read_text().ok();

    let result: Result<SelectionResult, String> = async {
        app.clipboard().clear().ok();
        tokio::time::sleep(Duration::from_millis(CLIPBOARD_DELAY_MS)).await;

        simulate_key_action(&app, 'c')?;

        for attempt in 1..=MAX_CLIPBOARD_RETRIES {
            tokio::time::sleep(Duration::from_millis(
                CLIPBOARD_DELAY_MS + (attempt as u64 * RETRY_DELAY_MS),
            ))
            .await;

            // Try HTML first via arboard
            if let Some(html) = read_clipboard_html() {
                let markdown = html_to_markdown(&html);
                if !markdown.trim().is_empty() {
                    log::info!(
                        "HTML clipboard detected on attempt {}, converted to Markdown ({} chars)",
                        attempt,
                        markdown.len()
                    );
                    return Ok(SelectionResult {
                        text: markdown,
                        format: "markdown".to_string(),
                    });
                }
            }

            // Fall back to plain text
            match app.clipboard().read_text() {
                Ok(text) if !text.is_empty() => {
                    log::debug!("Plain text clipboard read on attempt {}", attempt);
                    return Ok(SelectionResult {
                        text,
                        format: "plain".to_string(),
                    });
                }
                Ok(_) => {
                    log::debug!("Clipboard empty on attempt {}, retrying...", attempt);
                }
                Err(e) => {
                    log::debug!("Clipboard read failed on attempt {}: {}", attempt, e);
                }
            }
        }

        log::debug!("No text selected after {} attempts", MAX_CLIPBOARD_RETRIES);
        Ok(SelectionResult {
            text: String::new(),
            format: "plain".to_string(),
        })
    }
    .await;

    if let Some(saved) = saved_clipboard {
        let _ = app.clipboard().write_text(&saved);
    }

    let selection = result?;

    if selection.text.is_empty() {
        log::info!("No text selected, skipping");
    } else {
        log::info!(
            "Selected text retrieved ({} chars, format={})",
            selection.text.len(),
            selection.format
        );
    }

    Ok(selection)
}

/// Paste text with optional Markdown→HTML conversion.
///
/// If format is "markdown", converts to HTML and writes both HTML and plain text
/// to the clipboard via arboard. Otherwise writes plain text (current behavior).
#[tauri::command]
pub async fn paste_formatted(app: AppHandle, text: String, format: String) -> Result<(), String> {
    log::info!("Pasting formatted text ({} chars, format={})", text.len(), format);

    let saved_clipboard = app.clipboard().read_text().ok();

    let result: Result<(), String> = async {
        if format == "markdown" {
            let html = markdown_to_html(&text);
            log::debug!("Converted Markdown to HTML ({} chars)", html.len());

            // Write HTML + plain text alt via arboard
            let mut clipboard = arboard::Clipboard::new()
                .map_err(|e| format!("Failed to open clipboard: {}", e))?;
            clipboard
                .set_html(&html, Some(&text))
                .map_err(|e| format!("Failed to write HTML to clipboard: {}", e))?;
        } else {
            app.clipboard()
                .write_text(&text)
                .map_err(|e| format!("Failed to write to clipboard: {}", e))?;
        }

        tokio::time::sleep(Duration::from_millis(CLIPBOARD_DELAY_MS)).await;
        simulate_key_action(&app, 'v')?;
        tokio::time::sleep(Duration::from_millis(PASTE_SETTLE_DELAY_MS)).await;

        Ok(())
    }
    .await;

    if let Some(saved) = saved_clipboard {
        let _ = app.clipboard().write_text(&saved);
    }

    result?;
    log::info!("Formatted text pasted successfully");
    Ok(())
}
