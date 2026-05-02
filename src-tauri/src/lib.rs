#![cfg_attr(not(debug_assertions), deny(warnings))]

mod commands;
mod google;
mod launcher;
mod recurring;
mod shortcuts;
mod storage;
mod timer;
mod window_manager;

use storage::Storage;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tauri_plugin_global_shortcut::Builder as GsBuilder;
use timer::TimerEngine;
use window_manager::WindowManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(GsBuilder::new().with_handler(shortcuts::handler).build())
        .invoke_handler(tauri::generate_handler![
            commands::create_note,
            commands::update_note_content,
            commands::update_note_color,
            commands::update_note_text_color,
            commands::update_note_tags,
            commands::update_note_recurring_rule,
            commands::update_note_opacity,
            commands::update_note_position,
            commands::update_note_size,
            commands::toggle_always_on_top,
            commands::delete_note,
            commands::archive_note,
            commands::unarchive_note,
            commands::list_archived_notes,
            commands::list_notes,
            commands::reorder_notes,
            commands::focus_note,
            commands::hide_all_notes,
            commands::show_all_notes,
            commands::focus_only_note,
            commands::start_timer,
            commands::pause_timer,
            commands::resume_timer,
            commands::cancel_timer,
            commands::get_timer_state,
            commands::get_setting,
            commands::set_setting,
            commands::list_settings,
            commands::open_url,
            commands::pomodoro_stats,
            commands::list_shortcut_bindings,
            commands::set_shortcut,
            commands::reset_shortcut,
            commands::export_backup,
            commands::import_backup,
            commands::google_is_configured,
            commands::google_connect,
            commands::google_disconnect,
            commands::google_account,
            commands::google_sync,
            commands::google_list_events,
            commands::google_set_event_timer,
            commands::google_open_event,
        ])
        .setup(|app| {
            // On macOS, hide Postik from the dock and Cmd-Tab. The user accesses
            // the controller via the menu-bar tray icon — there's nothing to gain
            // from a dock entry, and removing it also keeps the app out of
            // screen-sharing previews of the dock area.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Resolve the app data dir, then open SQLite under it.
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("could not resolve app_data_dir");
            let db_path = data_dir.join("postik.db");
            let storage =
                Storage::open(&db_path).expect("failed to open SQLite at app_data_dir/postik.db");

            // Window manager and timer engine share the storage handle.
            let wm = WindowManager::new(storage.clone());
            let engine = TimerEngine::new(app.handle().clone(), storage.clone());

            app.manage(storage);
            app.manage(wm.clone());
            app.manage(engine);

            // Build the controller window (hidden) and restore persisted notes.
            wm.create_controller(app.handle())?;
            wm.restore_persisted(app.handle())?;

            // Register global shortcuts.
            shortcuts::register_all(app.handle())?;

            // Build the tray icon (using the bundled app icon).
            let show_item =
                MenuItem::with_id(app, "show_controller", "Show Postik", true, None::<&str>)?;
            let new_item = MenuItem::with_id(app, "new_note", "New note", true, None::<&str>)?;
            let hide_all_item =
                MenuItem::with_id(app, "hide_all", "Hide all notes", true, None::<&str>)?;
            let show_all_item =
                MenuItem::with_id(app, "show_all", "Show all notes", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &show_item,
                    &new_item,
                    &hide_all_item,
                    &show_all_item,
                    &quit_item,
                ],
            )?;

            let _tray = TrayIconBuilder::with_id("postik-tray")
                .icon(app.default_window_icon().cloned().expect("default icon"))
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    let wm: tauri::State<WindowManager> = app.state();
                    match event.id.as_ref() {
                        "show_controller" => {
                            let _ = wm.show_controller(app);
                        }
                        "new_note" => {
                            let _ = wm.create_new_note(app, None);
                        }
                        "hide_all" => {
                            let _ = wm.hide_all_notes(app);
                        }
                        "show_all" => {
                            let _ = wm.show_all_notes(app);
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == MouseButton::Left {
                            let app = tray.app_handle();
                            let wm: tauri::State<WindowManager> = app.state();
                            let _ = wm.toggle_controller(app);
                        }
                    }
                })
                .build(app)?;

            // Background auto-sync for Google Calendar. Runs every 15 min;
            // each iteration checks the `google_calendar_auto_sync`
            // setting and syncs the default range when enabled. The setting
            // is read fresh each tick so toggling it from the UI takes
            // effect on the next iteration without a restart.
            spawn_google_auto_sync(app.handle().clone());

            // Recurring per-note schedules. Wakes once a minute and fires
            // matching notes (focus + system notification).
            recurring::spawn(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Spawns a tokio task that, every 15 minutes, checks if the user has
/// enabled `google_calendar_auto_sync` and runs a sync if so. Errors are
/// logged and swallowed — auto-sync should never crash the app.
fn spawn_google_auto_sync(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let interval = std::time::Duration::from_secs(15 * 60);
        loop {
            tokio::time::sleep(interval).await;
            let storage: tauri::State<Storage> = app.state();
            let engine: tauri::State<TimerEngine> = app.state();
            let enabled = storage
                .get_setting("google_calendar_auto_sync")
                .ok()
                .flatten()
                .map(|v| v == "true")
                .unwrap_or(false);
            if !enabled {
                continue;
            }
            match google::sync(&storage, &engine, google::SyncRange::Today).await {
                Ok(events) => {
                    let _ = tauri::Emitter::emit(&app, "google:events-synced", &events);
                }
                Err(e) => log::warn!("auto-sync failed: {}", e),
            }
        }
    });
}
