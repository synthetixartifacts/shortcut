//! Platform-specific window style + overlay window factory.
//!
//! Applies non-focusable window styles to keep overlay windows (indicator,
//! action menu) from stealing focus from the target application, and provides
//! the shared `build_overlay_window` helper consumed by PHASE 3B splits.
//! - Windows: sets `WS_EX_NOACTIVATE` + `ShowWindow(SW_SHOWNOACTIVATE)`.
//! - macOS: sets collection behavior to transient + ignores-cycle.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Check if a webview window handle is still usable.
///
/// A window handle can become stale after display driver resets (monitor
/// disconnect, sleep/wake). We test by attempting low-cost operations —
/// `is_visible()` + `scale_factor()` — both of which fail on a stale handle.
///
/// This is the canonical body previously duplicated across `indicator.rs`,
/// `action_menu.rs`, and `screen_capture.rs`. The `context` argument is used
/// purely for the warning log so operators can tell which overlay surfaced
/// the failure.
pub fn is_window_healthy(window: &WebviewWindow, context: &str) -> bool {
    match window.is_visible() {
        Ok(_) => window.scale_factor().is_ok(),
        Err(e) => {
            log::warn!("{context} window health check failed: {e}");
            false
        }
    }
}

/// Declarative configuration for a ShortCut overlay window.
///
/// Encapsulates the fields that vary between the three overlays
/// (indicator, action-menu, screen-question). Defaults intentionally do NOT
/// set `focused` — each overlay has a deliberate choice:
/// - indicator / action-menu: `focused(false)` (non-focusable HUD)
/// - screen-question: `focused(true)` (receives typed input)
pub struct OverlayConfig<'a> {
    pub title: &'a str,
    pub width: f64,
    pub height: f64,
    pub focused: bool,
}

/// Build a transparent, borderless, always-on-top overlay window.
///
/// Consolidates the 30-line `WebviewWindowBuilder` recipe duplicated across
/// `action_menu::recreate_menu_window`, `indicator::recreate_indicator_window`,
/// and `screen_capture::recreate_screen_question_window`. After building, the
/// caller applies any platform-specific styling (`apply_non_focusable`,
/// `apply_mouse_no_activate`) as appropriate.
pub fn build_overlay_window(
    app: &AppHandle,
    label: &str,
    url_path: &str,
    config: OverlayConfig,
) -> Result<WebviewWindow, tauri::Error> {
    WebviewWindowBuilder::new(app, label, WebviewUrl::App(url_path.into()))
        .title(config.title)
        .inner_size(config.width, config.height)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(config.focused)
        .build()
}

/// Apply non-focusable style to any overlay window by label.
///
/// Must be called after the window is created (during app setup or recreation).
/// Fails gracefully if the window doesn't exist yet.
pub fn apply_non_focusable(app: &AppHandle, label: &str) -> Result<(), String> {
    let window = app
        .get_webview_window(label)
        .ok_or(format!("{} window not found", label))?;

    #[cfg(target_os = "windows")]
    apply_windows(&window)?;

    #[cfg(target_os = "macos")]
    apply_macos(&window)?;

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = &window;
        log::debug!("Non-focusable window style not implemented for this platform");
    }

    Ok(())
}

/// Apply non-focusable style to the indicator window.
///
/// Convenience wrapper around `apply_non_focusable` for backward compatibility.
pub fn apply_indicator_non_focusable(app: &AppHandle) -> Result<(), String> {
    apply_non_focusable(app, "indicator")
}

/// Show an overlay without stealing focus (Windows: SW_SHOWNOACTIVATE;
/// other platforms: Tauri `.show()`). `.show()` on Windows calls
/// `ShowWindow(SW_SHOW)` which activates even with `WS_EX_NOACTIVATE` set.
pub fn show_without_focus_steal(window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        show_no_activate_windows(window)
    }

    #[cfg(not(target_os = "windows"))]
    {
        window.show().map_err(|e| e.to_string())
    }
}

/// Hide an overlay window (Windows: SW_HIDE via Win32; elsewhere Tauri
/// `.hide()`). Required counterpart to `show_without_focus_steal` — Tauri's
/// internal visibility tracking is out of sync after a raw FFI show.
pub fn hide_overlay_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        hide_windows(window)
    }

    #[cfg(not(target_os = "windows"))]
    {
        window.hide().map_err(|e| e.to_string())
    }
}

/// Extract the Win32 HWND from a Tauri WebviewWindow.
#[cfg(target_os = "windows")]
fn get_hwnd(window: &tauri::WebviewWindow) -> Result<isize, String> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = window
        .window_handle()
        .map_err(|e| format!("Failed to get window handle: {}", e))?;

    match handle.as_raw() {
        RawWindowHandle::Win32(h) => Ok(h.hwnd.get() as isize),
        _ => Err("Expected Win32 window handle".to_string()),
    }
}

/// Windows: set `WS_EX_NOACTIVATE` so the overlay never becomes the
/// foreground window. Standard approach for HUD-style windows.
#[cfg(target_os = "windows")]
fn apply_windows(window: &tauri::WebviewWindow) -> Result<(), String> {
    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_NOACTIVATE: isize = 0x08000000;

    extern "system" {
        fn GetWindowLongPtrW(hwnd: isize, nindex: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: isize, nindex: i32, dwnewlong: isize) -> isize;
    }

    let hwnd = get_hwnd(window)?;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE);
    }
    log::info!("Applied WS_EX_NOACTIVATE to overlay window");
    Ok(())
}

/// Windows: show via `SW_SHOWNOACTIVATE` to bypass `SW_SHOW`'s activating
/// side effect (which can steal focus even with `WS_EX_NOACTIVATE`).
#[cfg(target_os = "windows")]
fn show_no_activate_windows(window: &tauri::WebviewWindow) -> Result<(), String> {
    const SW_SHOWNOACTIVATE: i32 = 4;

    extern "system" {
        fn ShowWindow(hwnd: isize, ncmdshow: i32) -> i32;
    }

    let hwnd = get_hwnd(window)?;
    unsafe {
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
    log::debug!("Indicator shown via SW_SHOWNOACTIVATE");
    Ok(())
}

/// Windows: hide via `SW_HIDE`. Required counterpart to the raw-FFI show —
/// Tauri's internal visibility tracking would otherwise skip the call.
#[cfg(target_os = "windows")]
fn hide_windows(window: &tauri::WebviewWindow) -> Result<(), String> {
    const SW_HIDE: i32 = 0;

    extern "system" {
        fn ShowWindow(hwnd: isize, ncmdshow: i32) -> i32;
    }

    let hwnd = get_hwnd(window)?;
    unsafe {
        ShowWindow(hwnd, SW_HIDE);
    }
    log::debug!("Indicator hidden via SW_HIDE");
    Ok(())
}

/// Apply WM_MOUSEACTIVATE subclassing to prevent WebView2 first-click activation.
///
/// WebView2 can steal focus on the first click inside the webview even when
/// `WS_EX_NOACTIVATE` is set. Subclassing the window to intercept
/// `WM_MOUSEACTIVATE` and return `MA_NOACTIVATE` prevents this.
#[cfg(target_os = "windows")]
pub fn apply_mouse_no_activate(app: &AppHandle, label: &str) -> Result<(), String> {
    let window = app
        .get_webview_window(label)
        .ok_or(format!("{} window not found", label))?;
    let hwnd = get_hwnd(&window)?;
    subclass_for_no_activate(hwnd)?;
    log::info!("Applied WM_MOUSEACTIVATE -> MA_NOACTIVATE to {}", label);
    Ok(())
}

/// Windows: Subclass window to intercept WM_MOUSEACTIVATE and return MA_NOACTIVATE.
#[cfg(target_os = "windows")]
fn subclass_for_no_activate(hwnd: isize) -> Result<(), String> {
    const WM_MOUSEACTIVATE: u32 = 0x0021;
    const MA_NOACTIVATE: isize = 3;

    extern "system" {
        fn SetWindowSubclass(
            hwnd: isize,
            pfnsubclass: unsafe extern "system" fn(isize, u32, usize, isize, usize, usize) -> isize,
            uidsubclass: usize,
            dwrefdata: usize,
        ) -> i32;
        fn DefSubclassProc(hwnd: isize, umsg: u32, wparam: usize, lparam: isize) -> isize;
    }

    unsafe extern "system" fn subclass_proc(
        hwnd: isize,
        msg: u32,
        wparam: usize,
        lparam: isize,
        _uid_subclass: usize,
        _dw_ref_data: usize,
    ) -> isize {
        if msg == WM_MOUSEACTIVATE {
            return MA_NOACTIVATE;
        }
        unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
    }

    unsafe {
        let result = SetWindowSubclass(hwnd, subclass_proc, 1, 0);
        if result == 0 {
            return Err("SetWindowSubclass failed".to_string());
        }
    }
    Ok(())
}

/// macOS: Set non-activating collection behavior.
///
/// Sets NSWindowCollectionBehaviorTransient (1<<3) and
/// NSWindowCollectionBehaviorIgnoresCycle (1<<6) to prevent the window
/// from appearing in Cmd+Tab and reduce focus-stealing on show.
#[cfg(target_os = "macos")]
fn apply_macos(window: &tauri::WebviewWindow) -> Result<(), String> {
    use objc2_app_kit::{NSView, NSWindowCollectionBehavior};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = window
        .window_handle()
        .map_err(|e| format!("Failed to get window handle: {}", e))?;

    match handle.as_raw() {
        RawWindowHandle::AppKit(h) => {
            // Safety: ns_view is a valid, non-null NSView pointer from raw-window-handle
            let ns_view: &NSView = unsafe { &*(h.ns_view.as_ptr().cast::<NSView>()) };
            let ns_window = ns_view
                .window()
                .ok_or("NSView has no associated NSWindow")?;

            // Transient: window doesn't persist across spaces
            // IgnoresCycle: excluded from Cmd+Tab window cycling
            let behavior = NSWindowCollectionBehavior::Transient
                | NSWindowCollectionBehavior::IgnoresCycle;
            ns_window.setCollectionBehavior(behavior);

            log::info!("Applied non-activating collection behavior to indicator (macOS)");
            Ok(())
        }
        _ => Err("Expected AppKit window handle".to_string()),
    }
}
