//! Screen Capture & Window Management for Screen Question
//!
//! Captures screenshots (xcap + image crate) and manages the overlay window.
//! Screenshot taken BEFORE overlay appears. Resized to max 2048px, JPEG quality 85.

use base64::{engine::general_purpose, Engine as _};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewWindow};

use crate::window_style::{self, OverlayConfig};

/// Maximum dimension (longest side) for resized screenshots
const MAX_IMAGE_DIMENSION: u32 = 2048;

/// JPEG encoding quality (0-100)
const JPEG_QUALITY: u8 = 85;

/// Screen question window dimensions (logical pixels)
const WINDOW_WIDTH: f64 = 520.0;
const WINDOW_HEIGHT: f64 = 480.0;

/// Capture the screen of the monitor where the cursor is located.
/// Returns raw RGBA image. Uses cursor position to find the xcap monitor directly.
fn capture_current_monitor(app: &tauri::AppHandle) -> Result<image::RgbaImage, String> {
    // Get cursor position from main window
    let main_window = app
        .get_webview_window("main")
        .ok_or("Main window not found")?;
    let cursor_pos = main_window
        .cursor_position()
        .map_err(|e| format!("Failed to get cursor position: {}", e))?;

    // Use xcap's Monitor::from_point to find the monitor at cursor position
    let target_monitor = xcap::Monitor::from_point(cursor_pos.x as i32, cursor_pos.y as i32)
        .map_err(|e| format!("Failed to find monitor at cursor: {}", e))?;

    // Capture the screen
    let image = target_monitor
        .capture_image()
        .map_err(|e| format!("Screen capture failed: {}", e))?;

    log::info!(
        "Captured screen: {}x{} from monitor at ({}, {})",
        image.width(),
        image.height(),
        cursor_pos.x as i32,
        cursor_pos.y as i32,
    );

    // macOS: when Screen Recording permission (TCC) is denied, xcap still
    // returns a valid-looking image — but one that is completely black or
    // white except for the app's own window. Detect that heuristically and
    // surface an actionable error keyed on the `screen_question.permission_denied_macos`
    // i18n key so the frontend can render a "Open System Settings" banner.
    #[cfg(target_os = "macos")]
    if looks_like_tcc_denied(&image) {
        log::warn!("Screen capture appears blank — likely Screen Recording permission denied");
        return Err(
            "screen_question.permission_denied_macos: \
             Screen Recording permission is required. \
             Open System Settings → Privacy & Security → Screen Recording \
             and enable ShortCut."
                .to_string(),
        );
    }

    Ok(image)
}

/// macOS-only heuristic: a TCC-denied capture returns an image whose pixels
/// are essentially a single flat color (nearly all black or all white).
/// We sample a stride of pixels and count how many match the "flat" tone.
///
/// Kept as a heuristic rather than a `CGPreflightScreenCaptureAccess` FFI
/// call — adding `core-graphics` as a dep for this single probe is not worth
/// the build-graph cost. If the heuristic proves noisy in practice we can
/// revisit and add an objc2-based preflight.
#[cfg(target_os = "macos")]
fn looks_like_tcc_denied(img: &image::RgbaImage) -> bool {
    let total = (img.width() as usize) * (img.height() as usize);
    if total < 1_000 {
        // Too small to judge — don't false-positive.
        return false;
    }
    // Sample every ~100th pixel for speed.
    let stride = (total / 1_000).max(1);
    let mut flat = 0usize;
    let mut sampled = 0usize;
    for (i, pixel) in img.pixels().enumerate() {
        if i % stride != 0 {
            continue;
        }
        sampled += 1;
        let [r, g, b, _] = pixel.0;
        let is_black = r < 5 && g < 5 && b < 5;
        let is_white = r > 250 && g > 250 && b > 250;
        if is_black || is_white {
            flat += 1;
        }
    }
    sampled > 0 && (flat as f64 / sampled as f64) > 0.98
}

/// Resize to max dimension, convert RGBA->RGB, encode as JPEG, return as base64.
fn resize_and_encode_jpeg(img: image::RgbaImage) -> Result<(String, String), String> {
    let (width, height) = (img.width(), img.height());

    // Resize if either dimension exceeds the max (maintain aspect ratio)
    let resized = if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        let scale = MAX_IMAGE_DIMENSION as f64 / width.max(height) as f64;
        let new_width = (width as f64 * scale) as u32;
        let new_height = (height as f64 * scale) as u32;

        log::info!(
            "Resizing screenshot: {}x{} -> {}x{}",
            width,
            height,
            new_width,
            new_height
        );

        image::imageops::resize(&img, new_width, new_height, FilterType::Triangle)
    } else {
        img
    };

    // Convert RGBA to RGB (JPEG doesn't support alpha channel)
    let rgb_image = image::DynamicImage::ImageRgba8(resized).to_rgb8();

    // Encode to JPEG
    let mut jpeg_bytes = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, JPEG_QUALITY);
    encoder
        .encode(
            rgb_image.as_raw(),
            rgb_image.width(),
            rgb_image.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("JPEG encoding failed: {}", e))?;

    log::info!(
        "Encoded screenshot: {} bytes JPEG ({:.1} KB)",
        jpeg_bytes.len(),
        jpeg_bytes.len() as f64 / 1024.0
    );

    // Base64 encode
    let base64_string = general_purpose::STANDARD.encode(&jpeg_bytes);

    Ok((base64_string, "image/jpeg".to_string()))
}

// ---------------------------------------------------------------------------
// Tauri commands for Screen Question
// ---------------------------------------------------------------------------

/// Capture screen, show overlay immediately, process image in background.
///
/// Sequence:
/// 1. Capture raw screenshot (fast, ~100ms)
/// 2. Show overlay window immediately with loading state
/// 3. Spawn background task for resize + JPEG encode + base64
/// 4. Emit "screen-captured" when processing is done
#[tauri::command]
pub async fn screen_question(app: AppHandle) -> Result<(), String> {
    // Toggle: if window is already visible, hide it (and cancel any in-flight stream).
    if let Some(window) = app.get_webview_window("screen-question") {
        if window.is_visible().unwrap_or(false) {
            crate::providers::cancel_screen_question_stream();
            window.hide().map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    // 1. Fast capture (~100ms) — just grab raw pixels, no processing yet
    let raw_image = capture_current_monitor(&app)?;

    // 2. Show overlay window immediately (user sees loading state)
    show_screen_question_window(&app).await?;

    // 3. Process in background (resize + encode can take 1-2s)
    let app_clone = app.clone();
    tokio::spawn(async move {
        match resize_and_encode_jpeg(raw_image) {
            Ok((base64, mime)) => {
                log::info!("Screen question processed: {} bytes base64", base64.len());
                let _ = app_clone.emit("screen-captured", serde_json::json!({
                    "image_base64": base64,
                    "image_mime_type": mime,
                }));
            }
            Err(e) => {
                log::error!("Screenshot processing failed: {}", e);
                let _ = app_clone.emit("screen-answer-error", serde_json::json!({
                    "error": format!("Screenshot processing failed: {}", e)
                }));
            }
        }
    });

    Ok(())
}

/// Send image + conversation to the configured vision provider.
/// Streams back via "screen-answer-chunk" / "screen-answer-complete" / "screen-answer-error" events.
#[tauri::command]
pub async fn send_screen_question(
    app: AppHandle,
    image_base64: String,
    image_mime_type: String,
    messages: Vec<crate::providers::ChatMessage>,
) -> Result<(), String> {
    // Pull the screen-question system prompt from config inside a short-lived
    // lock (never hold MutexGuard across .await).
    let system_prompt = {
        let state = app.state::<crate::config::ConfigState>();
        let config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;
        config.screen_question.system_prompt.clone()
    };

    // Prepend system message only when non-empty (backward compat).
    let mut full_messages = Vec::with_capacity(messages.len() + 1);
    if !system_prompt.is_empty() {
        full_messages.push(crate::providers::ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        });
    }
    full_messages.extend(messages);

    crate::providers::stream_screen_question(
        &app,
        &image_base64,
        &image_mime_type,
        full_messages,
    )
    .await
    .map_err(|e| e.to_string())
}

/// Hide the screen question overlay window and cancel any in-flight stream.
#[tauri::command]
pub async fn hide_screen_question(app: AppHandle) -> Result<(), String> {
    // Cancel before hiding so the provider loop stops emitting events that
    // would otherwise land in a hidden window.
    crate::providers::cancel_screen_question_stream();
    if let Some(window) = app.get_webview_window("screen-question") {
        window.hide().map_err(|e| e.to_string())?;
        log::info!("Screen question window hidden");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Window management
// ---------------------------------------------------------------------------

/// Show the screen question overlay window centered on the current monitor.
///
/// Unlike indicator/action-menu, this window NEEDS focus (user types in it).
/// Uses standard Tauri .show() + .set_focus() instead of show_without_focus_steal().
async fn show_screen_question_window(app: &AppHandle) -> Result<(), String> {
    let window = ensure_screen_question_window(app).await?;
    let main_window = app.get_webview_window("main");
    position_center_of_monitor(&window, main_window.as_ref())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    log::info!("Screen question window shown with focus");
    Ok(())
}

/// Position window at center of the monitor where cursor is.
fn position_center_of_monitor(
    window: &WebviewWindow,
    reference_window: Option<&WebviewWindow>,
) -> Result<(), String> {
    let monitor = crate::indicator::get_cursor_monitor(window, reference_window)?;
    let scale_factor = monitor.scale_factor();

    let monitor_size = monitor.size().to_logical::<f64>(scale_factor);
    let monitor_position = monitor.position().to_logical::<f64>(scale_factor);

    let _ = window.set_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

    let x = monitor_position.x + (monitor_size.width - WINDOW_WIDTH) / 2.0;
    let y = monitor_position.y + (monitor_size.height - WINDOW_HEIGHT) / 2.0;

    if !crate::indicator::is_position_valid(x, y, reference_window.unwrap_or(window)) {
        log::warn!("Screen question position off-screen, using center");
        let _ = window.center().map_err(|e| e.to_string());
        return Ok(());
    }

    window
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    log::debug!(
        "Screen question positioned at ({}, {}) on {:?}",
        x,
        y,
        monitor.name()
    );
    Ok(())
}

/// Ensure screen question window exists and is healthy, recreating if needed.
async fn ensure_screen_question_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("screen-question") {
        if window_style::is_window_healthy(&window, "Screen question") {
            return Ok(window);
        }
        log::warn!("Screen question window unhealthy, recreating");
    } else {
        log::info!("Screen question window not found, creating");
    }
    recreate_screen_question_window(app).await
}

/// Destroy and recreate the screen question window from scratch.
async fn recreate_screen_question_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(existing) = app.get_webview_window("screen-question") {
        log::info!("Destroying stale screen question window");
        let _ = existing.destroy();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let window = window_style::build_overlay_window(
        app,
        "screen-question",
        "/screen-question",
        OverlayConfig {
            title: "ShortCut Screen Question",
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            focused: true, // KEY: We WANT focus for text input
        },
    )
    .map_err(|e| format!("Failed to recreate screen question window: {}", e))?;

    // No apply_non_focusable/apply_mouse_no_activate — we WANT focus for typing
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    log::info!("Screen question window recreated");
    Ok(window)
}

/// Handle display topology change for screen question window.
pub fn handle_display_change_screen_question(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        log::info!("Display change detected, validating screen question window");
        match ensure_screen_question_window(&app).await {
            Ok(_) => log::info!("Screen question window validated"),
            Err(e) => log::error!("Failed to validate screen question window: {}", e),
        }
    });
}
