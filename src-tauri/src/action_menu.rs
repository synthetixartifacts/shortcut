//! Action Menu Window Management
//!
//! Controls the floating action wheel (pie menu) that provides visual,
//! mouse-driven access to all ShortCut actions at the cursor position.
//!
//! Follows the same overlay window patterns as `indicator.rs`:
//! ensure_window -> position -> show_without_focus_steal for show,
//! hide_overlay_window for hide, and health-check/recreate for resilience.
//!
//! # Known Issues
//!
//! - **macOS `focusable:false` limitation (Tauri #14102)**: On macOS, the
//!   `focusable:false` Tauri config does not fully prevent activation. A proper
//!   fix requires `tauri-nspanel` or a native NSPanel wrapper, which is out of
//!   scope for Phase 1. The `NSWindowCollectionBehavior` applied in
//!   `window_style.rs` provides partial mitigation.
//!
//! - **Hover events on inactive window**: WebView2 may not deliver hover events
//!   to a non-focusable window on some Windows configurations. As mitigation,
//!   wedge labels are always visible (not hover-only). CSS hover opacity effects
//!   are best-effort.
//!
//! - **WebView2 first-click activation**: Even with `WS_EX_NOACTIVATE` set,
//!   WebView2 can steal focus on the first click inside the webview. This is
//!   mitigated by `apply_mouse_no_activate()` in `window_style.rs`, which
//!   subclasses the window to intercept `WM_MOUSEACTIVATE` and return
//!   `MA_NOACTIVATE`. The subclass must be reapplied after window recreation.

use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewWindow};

use crate::window_style::{self, OverlayConfig};

/// Action menu window dimensions (logical pixels)
const MENU_WIDTH: f64 = 280.0;
const MENU_HEIGHT: f64 = 280.0;

/// Toggle the action menu: show at cursor if hidden, hide if visible.
#[tauri::command]
pub async fn toggle_action_menu(app: AppHandle) -> Result<(), String> {
    // If the window exists and is visible, hide it
    if let Some(window) = app.get_webview_window("action-menu") {
        if window.is_visible().unwrap_or(false) {
            window_style::hide_overlay_window(&window)?;
            log::info!("Action menu hidden (toggle)");
            return Ok(());
        }
    }

    // Otherwise show: ensure window health, position at cursor, show
    let menu = ensure_menu_window(&app).await?;
    let main_window = app.get_webview_window("main");
    position_at_cursor(&menu, main_window.as_ref())?;
    window_style::show_without_focus_steal(&menu)?;

    // Notify the menu window so it can reset its auto-dismiss timer
    if let Err(e) = app.emit("action-menu-show", ()) {
        log::warn!("Failed to emit action-menu-show event: {}", e);
    }

    log::info!("Action menu shown (toggle)");
    Ok(())
}

/// Hide the action menu window.
#[tauri::command]
pub async fn hide_action_menu(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("action-menu") else {
        log::debug!("Action menu window not found during hide (already destroyed?)");
        return Ok(());
    };

    window_style::hide_overlay_window(&window)?;
    log::info!("Action menu hidden");
    Ok(())
}

/// Position the action menu centered on the cursor, clamped to screen edges.
///
/// Uses the main window for cursor detection (valid DPI context), same as indicator.
fn position_at_cursor(
    menu: &WebviewWindow,
    reference_window: Option<&WebviewWindow>,
) -> Result<(), String> {
    let monitor = crate::indicator::get_cursor_monitor(menu, reference_window)?;
    let scale_factor = monitor.scale_factor();

    let monitor_size = monitor.size().to_logical::<f64>(scale_factor);
    let monitor_position = monitor.position().to_logical::<f64>(scale_factor);

    // Set size to match expected dimensions on the target monitor
    let _ = menu.set_size(LogicalSize::new(MENU_WIDTH, MENU_HEIGHT));

    // Get cursor position and convert to logical coordinates
    let cursor_window = reference_window.unwrap_or(menu);
    let cursor_pos = cursor_window
        .cursor_position()
        .map_err(|e| format!("Failed to get cursor position: {}", e))?;

    let cursor_x = cursor_pos.x / scale_factor;
    let cursor_y = cursor_pos.y / scale_factor;

    // Center the menu on the cursor
    let mut x = cursor_x - MENU_WIDTH / 2.0;
    let mut y = cursor_y - MENU_HEIGHT / 2.0;

    // Clamp to screen edges with small padding to avoid touching the very edge
    const EDGE_PADDING: f64 = 4.0;
    let min_x = monitor_position.x + EDGE_PADDING;
    let min_y = monitor_position.y + EDGE_PADDING;
    let max_x = monitor_position.x + monitor_size.width - MENU_WIDTH - EDGE_PADDING;
    let max_y = monitor_position.y + monitor_size.height - MENU_HEIGHT - EDGE_PADDING;

    x = x.clamp(min_x, max_x);
    y = y.clamp(min_y, max_y);

    // Validate position is on a real monitor
    if !crate::indicator::is_position_valid(x, y, cursor_window) {
        log::warn!(
            "Action menu position ({}, {}) off-screen, clamping to monitor origin",
            x, y
        );
        x = monitor_position.x;
        y = monitor_position.y;
    }

    menu.set_position(LogicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    log::debug!(
        "Action menu positioned at logical ({}, {}) on monitor {:?} (scale: {})",
        x, y, monitor.name(), scale_factor
    );

    Ok(())
}

/// Destroy and recreate the action menu window from scratch.
async fn recreate_menu_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(existing) = app.get_webview_window("action-menu") {
        log::info!("Destroying stale action menu window");
        let _ = existing.destroy();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let window = window_style::build_overlay_window(
        app,
        "action-menu",
        "/action-menu",
        OverlayConfig {
            title: "ShortCut Action Menu",
            width: MENU_WIDTH,
            height: MENU_HEIGHT,
            focused: false,
        },
    )
    .map_err(|e| format!("Failed to recreate action menu window: {}", e))?;

    if let Err(e) = window_style::apply_non_focusable(app, "action-menu") {
        log::warn!("Failed to reapply action-menu style after recreation: {}", e);
    }

    // Reapply WM_MOUSEACTIVATE subclassing after recreation (WebView2 first-click fix)
    #[cfg(target_os = "windows")]
    if let Err(e) = window_style::apply_mouse_no_activate(app, "action-menu") {
        log::warn!("Failed to reapply WM_MOUSEACTIVATE after recreation: {}", e);
    }

    // Wait for webview to load Svelte app and register event listeners
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    log::info!("Action menu window recreated successfully");
    Ok(window)
}

/// Ensure the action menu window exists and is healthy, recreating if needed.
async fn ensure_menu_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("action-menu") {
        if window_style::is_window_healthy(&window, "Action menu") {
            return Ok(window);
        }
        log::warn!("Action menu window exists but is unhealthy, recreating");
    } else {
        log::info!("Action menu window not found, creating");
    }

    recreate_menu_window(app).await
}

/// Handle a display topology change (monitor disconnect, sleep/wake).
///
/// Called from the app run loop alongside indicator::handle_display_change.
pub fn handle_display_change_menu(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        log::info!("Display change detected, validating action menu window");
        match ensure_menu_window(&app).await {
            Ok(_) => log::info!("Action menu window validated after display change"),
            Err(e) => log::error!("Failed to validate action menu after display change: {}", e),
        }
    });
}
