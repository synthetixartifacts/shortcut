//! Indicator window lifecycle: creation, recreation, health checks, display
//! change handling.
//!
//! Uses the shared overlay helpers in `window_style` to avoid duplicating the
//! builder recipe across all three overlay windows.

use tauri::{AppHandle, Manager, WebviewWindow};

use super::positioning::{INDICATOR_HEIGHT, INDICATOR_WIDTH};
use super::topology::reset_topology_cache;
use crate::window_style::{self, OverlayConfig};

/// Destroy and recreate the indicator window from scratch.
///
/// Reproduces the window configuration from tauri.conf.json programmatically.
/// After creation, re-applies the non-focusable window style.
pub async fn recreate_indicator_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(existing) = app.get_webview_window("indicator") {
        log::info!("Destroying stale indicator window");
        let _ = existing.destroy();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let window = window_style::build_overlay_window(
        app,
        "indicator",
        "/indicator",
        OverlayConfig {
            title: "ShortCut Indicator",
            width: INDICATOR_WIDTH,
            height: INDICATOR_HEIGHT,
            focused: false,
        },
    )
    .map_err(|e| format!("Failed to recreate indicator window: {}", e))?;

    if let Err(e) = window_style::apply_indicator_non_focusable(app) {
        log::warn!("Failed to reapply indicator style after recreation: {}", e);
    }

    // Wait for webview to load Svelte app and register event listeners
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    log::info!("Indicator window recreated successfully");
    Ok(window)
}

/// Ensure the indicator window exists and is healthy, recreating if needed.
pub async fn ensure_indicator_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("indicator") {
        if window_style::is_window_healthy(&window, "Indicator") {
            return Ok(window);
        }
        log::warn!("Indicator window exists but is unhealthy, recreating");
    } else {
        log::warn!("Indicator window not found, recreating");
    }

    recreate_indicator_window(app).await
}

/// Handle a display topology change (monitor disconnect, sleep/wake).
///
/// Called from the app run loop when system events indicate the display
/// configuration may have changed. Force-recreates the indicator window
/// because health checks alone can miss zombie windows (valid handle but
/// broken native rendering surface).
pub fn handle_display_change(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        log::info!("Display change detected, force-recreating indicator window");

        reset_topology_cache();

        match recreate_indicator_window(&app).await {
            Ok(_) => log::info!("Indicator window recreated after display change"),
            Err(e) => log::error!("Failed to recreate indicator after display change: {}", e),
        }
    });
}
