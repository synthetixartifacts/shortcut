// History management for dictation transcriptions

use crate::config::get_app_data_path;
use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use uuid::Uuid;

/// Maximum number of history entries retained on disk.
/// Oldest entries are dropped (LRU by timestamp) when the cap is exceeded so
/// `history.json` cannot grow unbounded.
const MAX_HISTORY_ENTRIES: usize = 10_000;

/// A single history entry representing one dictation transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Transcribed text
    pub text: String,
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    /// Audio duration in milliseconds
    pub duration_ms: u32,
    /// Detected or configured language (e.g., "en", "fr")
    pub language: Option<String>,
    /// Which transcription engine produced this entry
    #[serde(default)]
    pub engine: Option<String>,
}

/// Paginated history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPage {
    pub entries: Vec<HistoryEntry>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

/// Load all history entries from file
fn load_history(app: &AppHandle) -> Result<Vec<HistoryEntry>, AppError> {
    let path = get_app_data_path(app, "history.json")
        .map_err(AppError::Config)?;

    if path.exists() {
        let content = fs::read_to_string(&path)?;
        let entries: Vec<HistoryEntry> = serde_json::from_str(&content)?;
        Ok(entries)
    } else {
        Ok(Vec::new())
    }
}

/// Save history entries to file.
///
/// Uses a tmp→rename commit to make the write atomic: readers never observe a
/// partially written `history.json` even if the process is killed mid-write.
/// Mirrors the pattern in `crate::config::mod::persist_config`.
fn save_history(app: &AppHandle, entries: &[HistoryEntry]) -> Result<(), AppError> {
    let path = get_app_data_path(app, "history.json")
        .map_err(AppError::Config)?;
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

/// Get current timestamp in milliseconds
fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Get history entries with pagination (newest first), optionally filtered by query
#[tauri::command]
pub fn get_history(app: AppHandle, page: usize, page_size: usize, query: Option<String>) -> Result<HistoryPage, AppError> {
    let page = if page == 0 { 1 } else { page };
    let page_size = if page_size == 0 { 20 } else { page_size };

    let entries = load_history(&app)?;

    let mut sorted_entries = entries;
    sorted_entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp));

    // Filter by search query (case-insensitive substring match on text)
    if let Some(ref q) = query {
        let q_lower = q.to_lowercase();
        if !q_lower.is_empty() {
            sorted_entries.retain(|e| e.text.to_lowercase().contains(&q_lower));
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

    Ok(HistoryPage {
        entries: page_entries,
        total,
        page,
        page_size,
        total_pages,
    })
}

/// Add a new history entry
#[tauri::command]
pub fn add_history_entry(
    app: AppHandle,
    text: String,
    duration_ms: u32,
    language: Option<String>,
    engine: Option<String>,
) -> Result<HistoryEntry, AppError> {
    let mut entries = load_history(&app)?;

    let entry = HistoryEntry {
        id: Uuid::new_v4().to_string(),
        text,
        timestamp: current_timestamp_ms(),
        duration_ms,
        language,
        engine,
    };

    entries.push(entry.clone());

    // Enforce LRU cap by dropping the oldest entries (by insertion order).
    // Entries are always appended chronologically; on-disk ordering therefore
    // matches timestamp ordering, so draining from the front removes the oldest.
    if entries.len() > MAX_HISTORY_ENTRIES {
        let excess = entries.len() - MAX_HISTORY_ENTRIES;
        entries.drain(0..excess);
    }

    save_history(&app, &entries)?;

    log::info!("Added history entry: {}", entry.id);
    Ok(entry)
}

/// Delete a single history entry by ID
#[tauri::command]
pub fn delete_history_entry(app: AppHandle, id: String) -> Result<(), AppError> {
    let mut entries = load_history(&app)?;
    let original_len = entries.len();

    entries.retain(|e| e.id != id);

    if entries.len() == original_len {
        return Err(AppError::General(format!("History entry not found: {}", id)));
    }

    save_history(&app, &entries)?;
    log::info!("Deleted history entry: {}", id);
    Ok(())
}

/// Clear all history entries
#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), AppError> {
    save_history(&app, &[])?;
    log::info!("Cleared all history entries");
    Ok(())
}
