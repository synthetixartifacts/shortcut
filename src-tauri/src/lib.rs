// ShortCut
// Voice-to-text desktop app with AI text transformation

mod action_menu;
mod build_config;
mod clipboard;
mod config;
mod errors;
mod history;
mod hotkeys;
mod indicator;
mod providers;
mod screen_capture;
mod text_transform;
mod text_transform_history;
mod transcription;
mod window_style;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent, WindowEvent,
};

/// Initialize and run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Development only: Load .env file for local testing
    // Production uses config.json managed through the Settings UI
    #[cfg(debug_assertions)]
    {
        if let Ok(path) = dotenvy::dotenv() {
            eprintln!("Development: Loaded .env from: {:?}", path);
        }
    }

    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting ShortCut");

    // Suppress Alt key menu activation in foreground apps (Word, Excel, etc.)
    // Default shortcuts now use Ctrl+Shift, but users with custom Alt shortcuts
    // still need this hook. It injects a dummy key-up on Alt release to cancel
    // Windows menu activation.
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = prevent_alt_win_menu::start(Default::default()) {
            log::error!("Alt menu suppression failed: {} — users with Alt shortcuts may experience menu activation in target apps", e);
        }
    }

    tauri::Builder::default()
        // === Managed state: registered on Builder so it's available BEFORE windows load ===
        // Load config from disk EARLY so the frontend gets real values immediately.
        // Without this, ConfigState would hold defaults (empty API key) until .setup()
        // loads from disk — causing a race where production builds redirect to /auth.
        .manage(providers::http::create_http_client())
        .manage(config::ConfigState(std::sync::Mutex::new(config::load_config_early())))
        // IMPORTANT: Single instance MUST be the first plugin registered
        // This prevents duplicate instances and handles the case where a previous
        // instance didn't close properly
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("Another instance tried to launch, focusing existing window");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        // Initialize other plugins
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_fs::init())
        // Setup application
        .setup(|app| {
            log::info!("Setting up application");

            // Load config from disk and update the managed state (which was initialized
            // with defaults on the Builder above)
            let loaded_config = config::load_config_from_disk(app.handle())
                .unwrap_or_else(|e| {
                    log::warn!("Failed to load config, using defaults: {}", e);
                    config::AppConfig::default()
                });
            let hotkey_config = loaded_config.hotkeys.clone();
            {
                let state = app.state::<config::ConfigState>();
                let mut config = state.0.lock().expect("ConfigState lock poisoned during setup");
                *config = loaded_config;
            }
            log::info!("ConfigState loaded from disk");

            // macOS: Hide dock icon, show only in menu bar
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Create tray menu
            let show_hide_item = MenuItem::with_id(app, "show_hide", "Show/Hide", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_hide_item, &quit_item])?;

            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        log::info!("Quit requested from tray menu");
                        app.exit(0);
                    }
                    "show_hide" => {
                        log::info!("Show/Hide requested from tray menu");
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Register global shortcuts from config (or defaults)
            if let Err(e) = hotkeys::register_shortcuts_from_config(app.handle(), &hotkey_config) {
                log::error!("Failed to register shortcuts: {} — app will run without global shortcuts", e);
            }

            // Make overlay windows non-focusable using platform-specific APIs
            if let Err(e) = window_style::apply_indicator_non_focusable(app.handle()) {
                log::warn!("Failed to apply indicator window style: {}", e);
            }
            if let Err(e) = window_style::apply_non_focusable(app.handle(), "action-menu") {
                log::warn!("Failed to apply action-menu window style: {}", e);
            }

            // Subclass overlay windows to intercept WM_MOUSEACTIVATE (WebView2 first-click fix)
            #[cfg(target_os = "windows")]
            {
                if let Err(e) = window_style::apply_mouse_no_activate(app.handle(), "indicator") {
                    log::warn!("Failed to apply WM_MOUSEACTIVATE handler to indicator: {}", e);
                }
                if let Err(e) = window_style::apply_mouse_no_activate(app.handle(), "action-menu") {
                    log::warn!("Failed to apply WM_MOUSEACTIVATE handler to action-menu: {}", e);
                }
            }

            log::info!("Application setup complete");
            Ok(())
        })
        // Register Tauri commands
        .invoke_handler(tauri::generate_handler![
            action_menu::toggle_action_menu,
            action_menu::hide_action_menu,
            hotkeys::get_registered_shortcuts,
            hotkeys::update_shortcuts,
            hotkeys::get_default_shortcuts,
            clipboard::paste_text,
            clipboard::get_selection_with_format,
            clipboard::paste_formatted,
            clipboard::frontend_log,
            text_transform::transform_text,
            transcription::transcribe_audio,
            transcription::get_model_status,
            transcription::download_model,
            transcription::delete_model,
            transcription::cancel_model_download,
            config::get_config,
            config::save_config,
            config::update_user_config,
            config::update_dictation_config,
            config::update_hotkeys_config,
            config::update_app_settings_config,
            config::update_improve_config,
            config::get_default_improve_config,
            config::get_active_engine,
            config::set_active_engine,
            config::update_transcription_config,
            config::update_providers_config,
            config::get_providers_config,
            config::get_provider_status,
            providers::discovery::get_provider_models,
            config::get_default_grammar_config,
            config::get_default_translate_config,
            config::update_grammar_config,
            config::update_translate_config,
            config::get_default_screen_question_config,
            config::update_screen_question_config,
            indicator::show_indicator,
            indicator::hide_indicator,
            indicator::reset_indicator,
            screen_capture::screen_question,
            screen_capture::send_screen_question,
            screen_capture::hide_screen_question,
            history::get_history,
            history::add_history_entry,
            history::delete_history_entry,
            history::clear_history,
            text_transform_history::get_text_transform_history,
            text_transform_history::add_text_transform_history_entry,
            text_transform_history::delete_text_transform_history_entry,
            text_transform_history::clear_text_transform_history,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            match event {
                // Handle window events on the main window
                RunEvent::WindowEvent { label, event, .. } if label == "main" => {
                    match event {
                        WindowEvent::CloseRequested { api, .. } => {
                            log::info!("Main window close requested, hiding to tray");
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                            api.prevent_close();
                        }
                        // Layer 2: Display topology change detected
                        WindowEvent::ScaleFactorChanged { .. } => {
                            log::info!("Scale factor changed on main window, resetting overlay windows");
                            indicator::handle_display_change(app);
                            action_menu::handle_display_change_menu(app);
                            screen_capture::handle_display_change_screen_question(app);
                        }
                        _ => {}
                    }
                }
                // Layer 2: System resumed from sleep/hibernate
                RunEvent::Resumed => {
                    log::info!("App resumed from sleep, resetting overlay windows");
                    indicator::handle_display_change(app);
                    action_menu::handle_display_change_menu(app);
                    screen_capture::handle_display_change_screen_question(app);
                }
                // macOS: prevent exit when last window is hidden.
                // code == None → user closed the last window (keep app alive in tray)
                // code == Some(_) → explicit quit via app.exit() from tray menu (allow exit)
                RunEvent::ExitRequested { code, api, .. } if code.is_none() => {
                    log::info!("Exit requested by window close, preventing (tray app)");
                    api.prevent_exit();
                }
                RunEvent::Exit => {
                    log::info!("Application exiting, cleaning up...");
                }
                _ => {}
            }
        });
}
