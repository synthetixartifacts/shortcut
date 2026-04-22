//! Monitor topology tracking for the indicator.
//!
//! Detects display changes that don't trigger `ScaleFactorChanged`
//! (e.g., swapping hubs with same DPI, adding/removing monitors with
//! matching scale factors) so the indicator can be force-recreated when the
//! physical display arrangement shifts.

use std::sync::Mutex;
use tauri::WebviewWindow;

/// Per-monitor tuple: (name, x, y, width, height).
pub type MonitorTuple = (String, i32, i32, u32, u32);

/// Cached monitor topology. Module-private; mutated only via `has_topology_changed`
/// and `reset_topology_cache`.
pub(super) static LAST_TOPOLOGY: Mutex<Option<Vec<MonitorTuple>>> = Mutex::new(None);

/// Capture current monitor topology: name, position, and size for each monitor.
pub fn current_topology(window: &WebviewWindow) -> Option<Vec<MonitorTuple>> {
    let monitors = window.available_monitors().ok()?;
    let mut list: Vec<_> = monitors
        .iter()
        .map(|m| {
            let name = m.name().cloned().unwrap_or_default();
            let pos = m.position();
            let size = m.size();
            (name, pos.x, pos.y, size.width, size.height)
        })
        .collect();
    list.sort_by(|a, b| a.0.cmp(&b.0));
    Some(list)
}

/// Check if monitor topology changed since last check. Updates the cache.
pub fn has_topology_changed(window: &WebviewWindow) -> bool {
    let current = current_topology(window);
    // Recover from poisoned mutex — topology cache is best-effort.
    let mut last = LAST_TOPOLOGY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if *last == current {
        return false;
    }

    log::info!(
        "Monitor topology changed: {:?} -> {:?}",
        last.as_ref().map(|t| t.len()),
        current.as_ref().map(|t| t.len())
    );
    *last = current;
    true
}

/// Clear the cached topology so the next check establishes a fresh baseline.
pub fn reset_topology_cache() {
    if let Ok(mut topo) = LAST_TOPOLOGY.lock() {
        *topo = None;
    }
}
