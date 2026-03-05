mod commands;
mod db;
mod state;
mod windows_integration;

use commands::{
    get_all_time_summary, get_settings, get_timeline, get_today_summary, get_week_summary,
    quit_app, set_settings, toggle_pause,
};
use db::{init_db, load_settings_from_db, sample_activity_inner};
use state::{AppDb, AppState, INPUT_COUNTERS, InputCounters};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Duration;
use tauri::{
    Manager, RunEvent,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use windows_integration::spawn_input_hook_thread;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_today_summary,
            get_week_summary,
            get_all_time_summary,
            get_timeline,
            toggle_pause,
            quit_app
        ])
        .setup(|app| {
            // Stable DB path.
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");
            std::fs::create_dir_all(&data_dir).expect("Failed to create app data directory");
            let db_path = data_dir.join("worka.db");

            let conn = init_db(&db_path).expect("Failed to initialize database");

            // Load persisted settings.
            let saved_state = load_settings_from_db(&conn);
            *app.state::<Mutex<AppState>>().lock().unwrap() = saved_state;
            app.manage(AppDb(Mutex::new(conn)));

            // Input counters.
            let input_counters = Arc::new(InputCounters {
                keyboard: AtomicU64::new(0),
                mouse_clicks: AtomicU64::new(0),
                mouse_wheel: AtomicU64::new(0),
            });
            INPUT_COUNTERS.set(input_counters.clone()).ok();
            app.manage(input_counters);

            // Stop flag for worker threads.
            let should_stop = Arc::new(AtomicBool::new(false));
            app.manage(should_stop.clone());

            // Tray menu.
            let handle = app.handle();
            let show = MenuItem::with_id(handle, "show", "Открыть", true, None::<&str>)?;
            let quit = MenuItem::with_id(handle, "quit", "Выйти", true, None::<&str>)?;
            let menu = Menu::with_items(handle, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.state::<Arc<AtomicBool>>().store(true, Ordering::Relaxed);
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Low-level input hook thread.
            spawn_input_hook_thread();

            // Background sampler thread.
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut interval = app_handle
                    .state::<Mutex<AppState>>()
                    .lock()
                    .unwrap()
                    .sample_interval_seconds;

                loop {
                    std::thread::sleep(Duration::from_secs(interval));

                    if app_handle
                        .state::<Arc<AtomicBool>>()
                        .load(Ordering::Relaxed)
                    {
                        break;
                    }

                    let app_state = app_handle.state::<Mutex<AppState>>().lock().unwrap().clone();
                    interval = app_state.sample_interval_seconds;

                    if !app_state.paused {
                        let db = app_handle.state::<AppDb>();
                        let conn = db.0.lock().unwrap();

                        // 1) app activity sample (+ icon enrichment when missing).
                        let _ = sample_activity_inner(&app_state, &conn);

                        // 2) flush input counters.
                        let counters = app_handle.state::<Arc<InputCounters>>();
                        let kbd = counters.keyboard.swap(0, Ordering::Relaxed) as i64;
                        let clicks = counters.mouse_clicks.swap(0, Ordering::Relaxed) as i64;
                        let wheel = counters.mouse_wheel.swap(0, Ordering::Relaxed) as i64;

                        if app_state.track_input && (kbd > 0 || clicks > 0 || wheel > 0) {
                            let ts = chrono::Utc::now().timestamp();
                            let _ = conn.execute(
                                "INSERT INTO input_sample
                                 (timestamp_bucket, keyboard_presses, mouse_clicks, mouse_wheel)
                                 VALUES (?1, ?2, ?3, ?4)",
                                rusqlite::params![ts, kbd, clicks, wheel],
                            );
                        }
                    } else {
                        // Paused: drain counters so they don't burst on resume.
                        let counters = app_handle.state::<Arc<InputCounters>>();
                        counters.keyboard.store(0, Ordering::Relaxed);
                        counters.mouse_clicks.store(0, Ordering::Relaxed);
                        counters.mouse_wheel.store(0, Ordering::Relaxed);
                    }
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let RunEvent::WindowEvent { label, event, .. } = event {
                if label == "main" {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    #[test]
    fn should_chrono_datelike_works_correctly() {
        let now = chrono::Utc::now();
        assert!(now.year() >= 2024);
        assert!((1..=12).contains(&now.month()));
        assert!((1..=31).contains(&now.day()));
    }
}
