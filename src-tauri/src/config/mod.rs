// Configuration management for ShortCut

pub mod commands;
mod types;

pub use types::*;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

// Re-export all commands so lib.rs can reference them as config::*
pub use commands::*;

/// Thread-safe wrapper around AppConfig for Tauri managed state.
/// Eliminates race conditions from per-call disk reads.
pub struct ConfigState(pub Mutex<AppConfig>);

/// Get a file path in the app's data directory.
///
/// Shared by config, history, and any module that needs app data persistence.
pub fn get_app_data_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(dir.join(filename))
}

/// Resolve the app data directory using platform-specific paths.
///
/// This does NOT require an AppHandle, so it can be called before
/// the Tauri app is fully built. Uses the bundle identifier from tauri.conf.json.
fn resolve_app_data_dir() -> Option<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        dirs_next().map(|h| h.join("Library/Application Support"))
    } else {
        // Linux: XDG_DATA_HOME or ~/.local/share
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| dirs_next().map(|h| h.join(".local/share")))
    };
    base.map(|b| b.join("com.g-prompter.shortcut"))
}

/// Helper: get home directory
fn dirs_next() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Load config from disk without requiring AppHandle.
///
/// Used to initialize ConfigState in Builder::manage() before .setup() runs,
/// preventing a race condition where the frontend reads default (empty) state
/// before the real config is loaded.
pub fn load_config_early() -> AppConfig {
    let Some(dir) = resolve_app_data_dir() else {
        log::warn!("Could not resolve app data dir for early config load");
        return AppConfig::default();
    };
    let config_path = dir.join("config.json");
    if !config_path.exists() {
        return AppConfig::default();
    }
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            serde_json::from_str::<AppConfig>(&content).unwrap_or_else(|e| {
                log::warn!("Early config parse failed: {}, using defaults", e);
                AppConfig::default()
            })
        }
        Err(e) => {
            log::warn!("Early config read failed: {}, using defaults", e);
            AppConfig::default()
        }
    }
}

/// Load configuration from disk (used once at startup via AppHandle)
pub fn load_config_from_disk(app: &AppHandle) -> Result<AppConfig, String> {
    let config_path = get_app_data_path(app, "config.json")?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        log::info!("Loaded config from {:?}", config_path);
        Ok(config)
    } else {
        log::info!("Using default config (no config file found)");
        Ok(AppConfig::default())
    }
}

/// Persist configuration to disk
pub fn persist_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let config_path = get_app_data_path(app, "config.json")?;

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    let tmp_path = config_path.with_extension("json.tmp");
    fs::write(&tmp_path, content)
        .map_err(|e| format!("Failed to write config tmp file: {}", e))?;
    fs::rename(&tmp_path, &config_path).map_err(|e| {
        // Best-effort cleanup of the tmp file on rename failure.
        let _ = fs::remove_file(&tmp_path);
        format!("Failed to commit config file: {}", e)
    })?;

    log::info!("Saved config to {:?}", config_path);
    Ok(())
}
