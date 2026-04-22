//! Indicator window positioning.
//!
//! Computes and applies the bottom-center placement of the indicator on the
//! monitor containing the cursor. Uses the main window (always visible) for
//! cursor detection to avoid stale DPI context issues that plague hidden
//! windows after monitor swaps.

use tauri::{LogicalPosition, LogicalSize, WebviewWindow};

/// Taskbar height padding (approximate, above taskbar).
const TASKBAR_PADDING: f64 = 40.0;

/// Expected logical size of the indicator window (must match tauri.conf.json).
pub const INDICATOR_WIDTH: f64 = 500.0;
pub const INDICATOR_HEIGHT: f64 = 120.0;

/// Find the monitor containing the cursor by checking physical bounds of all monitors.
///
/// Uses `reference_window` (the main window, if available) for cursor detection because
/// it's always visible and has a valid DPI context. Hidden windows like the indicator
/// can have stale DPI context from a disconnected monitor, causing incorrect coordinates.
///
/// Falls back to primary monitor, then first available monitor.
pub fn get_cursor_monitor(
    indicator: &WebviewWindow,
    reference_window: Option<&WebviewWindow>,
) -> Result<tauri::Monitor, String> {
    let cursor_window = reference_window.unwrap_or(indicator);

    if let Ok(cursor_pos) = cursor_window.cursor_position() {
        let cx = cursor_pos.x;
        let cy = cursor_pos.y;

        log::debug!(
            "Cursor physical position: ({}, {}) from {:?}",
            cx,
            cy,
            if reference_window.is_some() { "main window" } else { "indicator window" }
        );

        if let Ok(monitors) = cursor_window.available_monitors() {
            for monitor in &monitors {
                let pos = monitor.position();
                let size = monitor.size();

                let mx = pos.x as f64;
                let my = pos.y as f64;
                let mw = size.width as f64;
                let mh = size.height as f64;

                if cx >= mx && cx < mx + mw && cy >= my && cy < my + mh {
                    log::debug!(
                        "Cursor is on monitor {:?} (physical bounds: {}x{} at ({},{}))",
                        monitor.name(),
                        mw,
                        mh,
                        mx,
                        my
                    );
                    return Ok(monitor.clone());
                }
            }

            log::warn!(
                "Cursor at ({}, {}) not inside any monitor bounds, trying monitor_from_point",
                cx,
                cy
            );
        }

        // Secondary attempt: try monitor_from_point with each available monitor's scale factor
        if let Ok(monitors) = cursor_window.available_monitors() {
            for monitor in &monitors {
                let sf = monitor.scale_factor();
                let lx = cx / sf;
                let ly = cy / sf;
                if let Ok(Some(found)) = cursor_window.monitor_from_point(lx, ly) {
                    log::debug!(
                        "monitor_from_point found {:?} using scale_factor {} from {:?}",
                        found.name(),
                        sf,
                        monitor.name()
                    );
                    return Ok(found);
                }
            }
        }

        log::warn!("All cursor-based detection failed, falling back to primary monitor");
    } else {
        log::warn!("Could not get cursor position, falling back to primary monitor");
    }

    if let Ok(Some(monitor)) = cursor_window.primary_monitor() {
        log::debug!("Using primary monitor: {:?}", monitor.name());
        return Ok(monitor);
    }

    if let Ok(monitors) = cursor_window.available_monitors() {
        if let Some(monitor) = monitors.into_iter().next() {
            log::debug!("Using first available monitor: {:?}", monitor.name());
            return Ok(monitor);
        }
    }

    Err("No monitor found".to_string())
}

/// Check if a logical position falls within any available monitor's bounds.
pub fn is_position_valid(x: f64, y: f64, window: &WebviewWindow) -> bool {
    if let Ok(monitors) = window.available_monitors() {
        for monitor in &monitors {
            let sf = monitor.scale_factor();
            let pos = monitor.position().to_logical::<f64>(sf);
            let size = monitor.size().to_logical::<f64>(sf);

            if x >= pos.x && x < pos.x + size.width && y >= pos.y && y < pos.y + size.height {
                return true;
            }
        }
    }
    false
}

/// Position indicator at bottom-center of the monitor where cursor is located.
///
/// Uses known logical window dimensions instead of querying `window.outer_size()`,
/// which can return stale physical sizes when the window's DPI context doesn't
/// match the target monitor (e.g., after unplugging an external display).
pub fn position_indicator(
    indicator: &WebviewWindow,
    reference_window: Option<&WebviewWindow>,
) -> Result<(), String> {
    let monitor = get_cursor_monitor(indicator, reference_window)?;
    let scale_factor = monitor.scale_factor();

    let monitor_size = monitor.size().to_logical::<f64>(scale_factor);
    let monitor_position = monitor.position().to_logical::<f64>(scale_factor);

    let _ = indicator.set_size(LogicalSize::new(INDICATOR_WIDTH, INDICATOR_HEIGHT));

    let x = monitor_position.x + (monitor_size.width - INDICATOR_WIDTH) / 2.0;
    let y = monitor_position.y + monitor_size.height - INDICATOR_HEIGHT - TASKBAR_PADDING;

    if !is_position_valid(x, y, reference_window.unwrap_or(indicator)) {
        log::warn!(
            "Calculated position ({}, {}) appears off-screen, repositioning to primary monitor",
            x,
            y
        );
        return position_on_primary(indicator, reference_window);
    }

    indicator
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    log::debug!(
        "Indicator positioned at logical ({}, {}) on monitor {:?} (scale: {}, logical: {}x{})",
        x,
        y,
        monitor.name(),
        scale_factor,
        monitor_size.width,
        monitor_size.height
    );

    Ok(())
}

/// Fallback: position indicator on primary monitor when normal detection fails.
fn position_on_primary(
    indicator: &WebviewWindow,
    reference_window: Option<&WebviewWindow>,
) -> Result<(), String> {
    let window = reference_window.unwrap_or(indicator);

    let monitor = window
        .primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("No primary monitor found")?;

    let scale_factor = monitor.scale_factor();
    let monitor_size = monitor.size().to_logical::<f64>(scale_factor);
    let monitor_position = monitor.position().to_logical::<f64>(scale_factor);

    let _ = indicator.set_size(LogicalSize::new(INDICATOR_WIDTH, INDICATOR_HEIGHT));

    let x = monitor_position.x + (monitor_size.width - INDICATOR_WIDTH) / 2.0;
    let y = monitor_position.y + monitor_size.height - INDICATOR_HEIGHT - TASKBAR_PADDING;

    indicator
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    log::info!("Indicator positioned on primary monitor at ({}, {})", x, y);

    Ok(())
}
