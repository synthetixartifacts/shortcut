//! Indicator Window Management
//!
//! Controls the floating activity indicator window that provides visual feedback
//! when actions (dictation, grammar fix, translation) are in progress.
//!
//! Handles multi-monitor setups and monitor topology changes (plugging/unplugging)
//! by using the main window (always visible) for cursor detection, avoiding stale
//! DPI context issues that occur when using hidden windows.

mod lifecycle;
mod positioning;
mod topology;

use tauri::{AppHandle, Manager};

// Re-export for external callers (lib.rs runtime event hooks).
pub use lifecycle::handle_display_change;
pub use positioning::{get_cursor_monitor, is_position_valid};

use lifecycle::{ensure_indicator_window, recreate_indicator_window};
use positioning::position_indicator;
use topology::has_topology_changed;

/// Show the indicator window at bottom-center of screen
#[tauri::command]
pub async fn show_indicator(app: AppHandle) -> Result<(), String> {
    // Use main window for cursor detection - it's always visible and has valid DPI context
    let main_window = app.get_webview_window("main");

    // Layer 1a: Detect monitor topology changes (catches hub swaps, monitor
    // plug/unplug even when ScaleFactorChanged doesn't fire)
    let indicator_window = app.get_webview_window("indicator");
    let ref_window = main_window.as_ref().or(indicator_window.as_ref());
    let topology_changed = ref_window.is_some_and(has_topology_changed);

    let indicator = if topology_changed {
        log::info!("Monitor topology changed, force-recreating indicator window");
        recreate_indicator_window(&app).await?
    } else {
        // Layer 1b: Validate window health, recreate if stale
        ensure_indicator_window(&app).await?
    };

    // Position at bottom center (above taskbar) on the monitor where cursor is
    position_indicator(&indicator, main_window.as_ref())?;

    // Show window without stealing focus from the target application.
    crate::window_style::show_without_focus_steal(&indicator)?;

    log::info!("Indicator window shown");
    Ok(())
}

/// Hide the indicator window
#[tauri::command]
pub async fn hide_indicator(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("indicator") else {
        log::debug!("Indicator window not found during hide (already destroyed?)");
        return Ok(());
    };

    // Use platform-matched hide to stay in sync with show_without_focus_steal.
    crate::window_style::hide_overlay_window(&window)?;

    log::info!("Indicator window hidden");
    Ok(())
}

/// Force-reset the indicator window (manual user action from dashboard).
#[tauri::command]
pub async fn reset_indicator(app: AppHandle) -> Result<(), String> {
    log::info!("Manual indicator reset requested");
    recreate_indicator_window(&app).await?;
    Ok(())
}
