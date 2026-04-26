// Text-transform history domain: persists successful Grammar / Translate /
// Improve outputs in a separate file from dictation `history.rs`.
//
// Storage lives at `<app data dir>/text_transform_history.json`. Atomic
// tmp→rename writes mirror `history.rs` and `config::persist_config`.
// Retention is capped at 10,000 entries — oldest dropped first.
//
// This module is intentionally parallel to `history.rs` rather than a shared
// generic abstraction (decision D1 in the feature plan): the entry shapes
// differ and v1 keeps the dictation domain entirely untouched.

use crate::config::get_app_data_path;
use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use uuid::Uuid;

/// Maximum number of text-transform history entries retained on disk.
/// Oldest entries are dropped (LRU by insertion order) when the cap is exceeded
/// so `text_transform_history.json` cannot grow unbounded.
const MAX_HISTORY_ENTRIES: usize = 10_000;

/// Storage filename inside the app data directory.
const STORAGE_FILE: &str = "text_transform_history.json";

/// Accepted action discriminators. The frontend gates the union; the backend
/// validates incoming `add` calls against this list as a belt-and-braces check.
const VALID_ACTIONS: &[&str] = &["grammar", "translate", "improve"];

/// A single text-transform history entry — one successful Grammar / Translate
/// / Improve output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextTransformHistoryEntry {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    /// Action discriminator: "grammar" | "translate" | "improve"
    pub action: String,
    /// The transformed output text (no original / source kept — decision D2)
    pub result: String,
}

/// Paginated text-transform history response (mirrors `history::HistoryPage`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextTransformHistoryPage {
    pub entries: Vec<TextTransformHistoryEntry>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

/// Load all text-transform history entries from disk.
/// Missing file returns an empty vec (first-launch is not an error).
fn load(app: &AppHandle) -> Result<Vec<TextTransformHistoryEntry>, AppError> {
    let path = get_app_data_path(app, STORAGE_FILE).map_err(AppError::Config)?;

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let entries: Vec<TextTransformHistoryEntry> = serde_json::from_str(&content)?;
        Ok(entries)
    } else {
        Ok(Vec::new())
    }
}

/// Save text-transform history entries to disk.
///
/// Uses a tmp→rename commit to make the write atomic: readers never observe a
/// partially written `text_transform_history.json` even if the process is
/// killed mid-write. Mirrors the pattern in `history::save_history` /
/// `config::mod::persist_config`.
fn save(app: &AppHandle, entries: &[TextTransformHistoryEntry]) -> Result<(), AppError> {
    let path = get_app_data_path(app, STORAGE_FILE).map_err(AppError::Config)?;
    let content = serde_json::to_string_pretty(entries)?;

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, content)?;
    if let Err(e) = fs::rename(&tmp_path, &path) {
        // Best-effort cleanup of the tmp file on rename failure.
        let _ = fs::remove_file(&tmp_path);
        return Err(e.into());
    }
    Ok(())
}

/// Current Unix timestamp in milliseconds.
fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Validate the `action` discriminator against the accepted set.
fn validate_action(action: &str) -> Result<(), AppError> {
    if VALID_ACTIONS.contains(&action) {
        Ok(())
    } else {
        Err(AppError::General(format!(
            "Unknown action: {}. Valid: grammar, translate, improve",
            action
        )))
    }
}

/// Get text-transform history entries with pagination (newest first), optionally
/// filtered by case-insensitive substring search on `result` and / or by
/// exact-match `action` discriminator.
#[tauri::command]
pub fn get_text_transform_history(
    app: AppHandle,
    page: usize,
    page_size: usize,
    query: Option<String>,
    action: Option<String>,
) -> Result<TextTransformHistoryPage, AppError> {
    let page = if page == 0 { 1 } else { page };
    let page_size = if page_size == 0 { 20 } else { page_size };

    let entries = load(&app)?;

    let mut sorted_entries = entries;
    sorted_entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp));

    // Filter by action discriminator (exact match).
    if let Some(ref a) = action {
        if !a.is_empty() && a != "all" {
            sorted_entries.retain(|e| e.action == *a);
        }
    }

    // Filter by search query (case-insensitive substring match on `result`).
    if let Some(ref q) = query {
        let q_lower = q.to_lowercase();
        if !q_lower.is_empty() {
            sorted_entries.retain(|e| e.result.to_lowercase().contains(&q_lower));
        }
    }

    let total = sorted_entries.len();
    let total_pages = if total == 0 { 1 } else { total.div_ceil(page_size) };

    let start = (page - 1) * page_size;
    let end = std::cmp::min(start + page_size, total);

    let page_entries = if start < total {
        sorted_entries[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(TextTransformHistoryPage {
        entries: page_entries,
        total,
        page,
        page_size,
        total_pages,
    })
}

/// Add a new text-transform history entry.
///
/// `action` must be one of "grammar" | "translate" | "improve".
/// `result` is rejected if empty after trimming — the frontend already filters
/// but the backend never persists empties either.
#[tauri::command]
pub fn add_text_transform_history_entry(
    app: AppHandle,
    action: String,
    result: String,
) -> Result<TextTransformHistoryEntry, AppError> {
    validate_action(&action)?;

    if result.trim().is_empty() {
        return Err(AppError::General("Empty result not allowed".into()));
    }

    let mut entries = load(&app)?;

    let entry = TextTransformHistoryEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: current_timestamp_ms(),
        action,
        result,
    };

    entries.push(entry.clone());

    // Enforce LRU cap by dropping the oldest entries (by insertion order).
    // Entries are always appended chronologically; on-disk ordering matches
    // timestamp ordering, so draining from the front removes the oldest.
    if entries.len() > MAX_HISTORY_ENTRIES {
        let excess = entries.len() - MAX_HISTORY_ENTRIES;
        entries.drain(0..excess);
    }

    save(&app, &entries)?;

    log::info!("Added text-transform history entry: {}", entry.id);
    Ok(entry)
}

/// Delete a single text-transform history entry by ID.
#[tauri::command]
pub fn delete_text_transform_history_entry(app: AppHandle, id: String) -> Result<(), AppError> {
    let mut entries = load(&app)?;
    let original_len = entries.len();

    entries.retain(|e| e.id != id);

    if entries.len() == original_len {
        return Err(AppError::General(format!(
            "Text-transform history entry not found: {}",
            id
        )));
    }

    save(&app, &entries)?;
    log::info!("Deleted text-transform history entry: {}", id);
    Ok(())
}

/// Clear all text-transform history entries.
#[tauri::command]
pub fn clear_text_transform_history(app: AppHandle) -> Result<(), AppError> {
    save(&app, &[])?;
    log::info!("Cleared all text-transform history entries");
    Ok(())
}
