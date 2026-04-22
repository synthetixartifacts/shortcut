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
            let mut cfg = serde_json::from_str::<AppConfig>(&content).unwrap_or_else(|e| {
                log::warn!("Early config parse failed: {}, using defaults", e);
                AppConfig::default()
            });
            migrate_providers_config(&mut cfg);
            cfg
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

        let mut config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        migrate_providers_config(&mut config);
        log::info!("Loaded config from {:?}", config_path);
        Ok(config)
    } else {
        log::info!("Using default config (no config file found)");
        Ok(AppConfig::default())
    }
}

/// Current schema version for Local-protocol detection. Bumped whenever a new
/// migration pass needs to run exactly once per existing config file. See
/// [`migrate_providers_config`] for how the marker gates each pass.
const LOCAL_DETECTION_SCHEMA_VERSION: u32 = 1;

/// Read-time, idempotent migration for the Local LLM rename (Phase 2) and
/// detection-cache cleanup (Phase 3 follow-up).
///
/// Applies four passes in order:
/// 1. If `local.base_url` is still the default AND the legacy `ollama_base_url`
///    is non-empty, copy the legacy URL into `local.base_url` and set
///    `local.protocol = "ollama"` so the user's existing dispatch path is
///    preserved. Clear the legacy field so the next save drops it.
/// 2. Rewrite any `task_assignments[*].provider_id == "ollama"` to `"local"`.
/// 3. Backfill `local.protocol = "auto"` if somehow empty after deserialize.
/// 4. If `local_detection_schema_version < 1`, clear `local.detected_protocol`
///    exactly once. This discards pre-shape-check detection results (which
///    could have stuck on "ollama" for an LM Studio endpoint because the old
///    probe accepted any 2xx); the next discovery re-runs the race using the
///    shape-aware parser. The marker is then bumped to `1`; subsequent loads
///    see the marker and skip the clear, preserving any legitimate detection.
///
/// Idempotency: each step's precondition fails after the first run, so running
/// the function twice on the same config produces an identical result. The
/// unit tests in `migration_tests` below enforce this contract.
fn migrate_providers_config(cfg: &mut AppConfig) {
    // Step 1: copy legacy ollama_base_url -> local.base_url when local is at default.
    let default_local_url = LocalCredentials::default().base_url;
    let legacy_url = std::mem::take(&mut cfg.providers.credentials.ollama_base_url);
    if !legacy_url.is_empty() && cfg.providers.credentials.local.base_url == default_local_url {
        cfg.providers.credentials.local.base_url = legacy_url;
        cfg.providers.credentials.local.protocol = "ollama".to_string();
    }
    // If step 1's precondition did not fire, legacy_url has already been taken
    // into a local binding and dropped — the stored field is empty either way,
    // which is exactly the post-migration invariant.

    // Step 2: rewrite task-assignment provider ids.
    let tasks = &mut cfg.providers.task_assignments;
    for assignment in [
        &mut tasks.grammar,
        &mut tasks.translate,
        &mut tasks.improve,
        &mut tasks.screen_question,
    ] {
        if assignment.provider_id == "ollama" {
            assignment.provider_id = "local".to_string();
        }
    }

    // Step 3: backfill empty protocol.
    if cfg.providers.credentials.local.protocol.is_empty() {
        cfg.providers.credentials.local.protocol = "auto".to_string();
    }

    // Step 4: one-shot clear of `detected_protocol` for configs that predate
    // the shape-aware auto-detect (Phase 3 follow-up). Without this pass a
    // user who detected "ollama" against an LM Studio endpoint via the older
    // probe would stay stuck on the wrong adapter.
    if cfg.local_detection_schema_version < 1 {
        cfg.providers.credentials.local.detected_protocol = None;
        cfg.local_detection_schema_version = LOCAL_DETECTION_SCHEMA_VERSION;
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

#[cfg(test)]
mod migration_tests {
    use super::*;

    #[test]
    fn legacy_ollama_base_url_migrates_to_local() {
        let mut cfg = AppConfig::default();
        cfg.providers.credentials.ollama_base_url = "http://10.0.0.5:11434/api/chat".to_string();
        migrate_providers_config(&mut cfg);
        assert_eq!(
            cfg.providers.credentials.local.base_url,
            "http://10.0.0.5:11434/api/chat"
        );
        assert_eq!(cfg.providers.credentials.local.protocol, "ollama");
        assert!(cfg.providers.credentials.ollama_base_url.is_empty());
    }

    #[test]
    fn legacy_provider_id_ollama_migrates_to_local() {
        let mut cfg = AppConfig::default();
        cfg.providers.task_assignments.grammar.provider_id = "ollama".to_string();
        cfg.providers.task_assignments.translate.provider_id = "ollama".to_string();
        migrate_providers_config(&mut cfg);
        assert_eq!(cfg.providers.task_assignments.grammar.provider_id, "local");
        assert_eq!(cfg.providers.task_assignments.translate.provider_id, "local");
    }

    #[test]
    fn migration_is_idempotent() {
        let mut cfg = AppConfig::default();
        cfg.providers.credentials.ollama_base_url = "http://10.0.0.5:11434/api/chat".to_string();
        cfg.providers.task_assignments.grammar.provider_id = "ollama".to_string();
        migrate_providers_config(&mut cfg);
        let after_first = serde_json::to_string(&cfg).unwrap();
        migrate_providers_config(&mut cfg);
        let after_second = serde_json::to_string(&cfg).unwrap();
        assert_eq!(after_first, after_second);
    }

    #[test]
    fn fresh_config_bumps_schema_version_and_clears_detected() {
        let mut cfg = AppConfig::default();
        // Fresh default has `detected_protocol = None` already; verify the
        // marker flips to 1 and nothing else meaningfully changes.
        assert_eq!(cfg.local_detection_schema_version, 0);
        migrate_providers_config(&mut cfg);
        assert_eq!(cfg.local_detection_schema_version, 1);
        assert!(cfg.providers.credentials.local.detected_protocol.is_none());
    }

    #[test]
    fn empty_protocol_backfills_to_auto() {
        let mut cfg = AppConfig::default();
        cfg.providers.credentials.local.protocol = String::new();
        migrate_providers_config(&mut cfg);
        assert_eq!(cfg.providers.credentials.local.protocol, "auto");
    }

    #[test]
    fn stale_detected_protocol_cleared_once() {
        // Simulates a pre-shape-check config that detected "ollama" against
        // what is actually an LM Studio endpoint.
        let mut cfg = AppConfig::default();
        cfg.providers.credentials.local.detected_protocol = Some("ollama".to_string());
        assert_eq!(cfg.local_detection_schema_version, 0);
        migrate_providers_config(&mut cfg);
        assert!(
            cfg.providers.credentials.local.detected_protocol.is_none(),
            "stale detected_protocol must be cleared on first migration"
        );
        assert_eq!(cfg.local_detection_schema_version, 1);
    }

    #[test]
    fn already_migrated_preserves_detected_protocol() {
        // A config that already bumped past the migration keeps its
        // detected_protocol intact — the one-shot clear must not re-fire.
        let mut cfg = AppConfig::default();
        cfg.providers.credentials.local.detected_protocol = Some("openai_compatible".to_string());
        cfg.local_detection_schema_version = 1;
        migrate_providers_config(&mut cfg);
        assert_eq!(
            cfg.providers.credentials.local.detected_protocol.as_deref(),
            Some("openai_compatible")
        );
        assert_eq!(cfg.local_detection_schema_version, 1);
    }

    #[test]
    fn detection_migration_is_idempotent_on_already_migrated() {
        // Running twice on an already-migrated config produces no further change.
        let mut cfg = AppConfig::default();
        cfg.providers.credentials.local.detected_protocol = Some("openai_compatible".to_string());
        cfg.local_detection_schema_version = 1;
        migrate_providers_config(&mut cfg);
        let after_first = serde_json::to_string(&cfg).unwrap();
        migrate_providers_config(&mut cfg);
        let after_second = serde_json::to_string(&cfg).unwrap();
        assert_eq!(after_first, after_second);
    }
}
