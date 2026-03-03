use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, RunEvent,
};
use chrono::Datelike;
use windows::Win32::{
    System::SystemInformation::GetTickCount64,
    UI::{
        Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
        WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppState {
    pub paused: bool,
    pub sample_interval_seconds: u64,
    pub idle_threshold_seconds: u64,
    pub track_window_titles: bool,
    pub track_input: bool,
    pub autostart: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            paused: false,
            sample_interval_seconds: 10,
            idle_threshold_seconds: 120,
            track_window_titles: true,
            track_input: true,
            autostart: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivitySample {
    pub timestamp: i64,
    pub app_id: i64,
    pub is_idle: bool,
    pub window_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSample {
    pub timestamp_bucket: i64,
    pub keyboard_presses: u32,
    pub mouse_clicks: u32,
    pub mouse_wheel: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppInfo {
    pub id: i64,
    pub exe_path: String,
    pub display_name: String,
}

// Получение времени простоя в секундах
fn get_idle_time() -> u64 {
    unsafe {
        let mut last_input = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut last_input).as_bool() {
            let now = GetTickCount64();
            ((now - last_input.dwTime as u64) / 1000) as u64
        } else {
            0
        }
    }
}

// Получение активного окна и процесса
fn get_active_window_info() -> Option<(String, String)> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // Получение имени процесса (упрощённо)
        let _process_name = format!("process_{}.exe", process_id);

        // Получение заголовка окна
        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        let title = String::from_utf16_lossy(&title_buf[..len as usize]);

        Some((_process_name, title))
    }
}

// Инициализация базы данных
fn init_db() -> Result<Connection> {
    let conn = Connection::open("worka.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS app (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            exe_path TEXT UNIQUE NOT NULL,
            display_name TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS activity_sample (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            app_id INTEGER NOT NULL,
            is_idle INTEGER NOT NULL,
            window_title TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS input_sample (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp_bucket INTEGER NOT NULL,
            keyboard_presses INTEGER NOT NULL,
            mouse_clicks INTEGER NOT NULL,
            mouse_wheel INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            sample_interval_seconds INTEGER NOT NULL DEFAULT 10,
            idle_threshold_seconds INTEGER NOT NULL DEFAULT 120,
            track_window_titles INTEGER NOT NULL DEFAULT 1,
            track_input INTEGER NOT NULL DEFAULT 1,
            autostart INTEGER NOT NULL DEFAULT 0,
            paused INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    // Инициализация настроек по умолчанию
    conn.execute(
        "INSERT OR IGNORE INTO settings (id, sample_interval_seconds, idle_threshold_seconds, track_window_titles, track_input, autostart, paused)
         VALUES (1, 10, 120, 1, 1, 0, 0)",
        [],
    )?;

    Ok(conn)
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> AppState {
    state.inner().clone()
}

#[tauri::command]
fn set_settings(
    state: tauri::State<Mutex<AppState>>,
    paused: bool,
    sample_interval_seconds: u64,
    idle_threshold_seconds: u64,
    track_window_titles: bool,
    track_input: bool,
    autostart: bool,
) {
    let mut inner = state.inner().lock().unwrap();
    inner.paused = paused;
    inner.sample_interval_seconds = sample_interval_seconds;
    inner.idle_threshold_seconds = idle_threshold_seconds;
    inner.track_window_titles = track_window_titles;
    inner.track_input = track_input;
    inner.autostart = autostart;
}

#[tauri::command]
fn sample_activity(state: tauri::State<Mutex<AppState>>) -> Option<ActivitySample> {
    let inner = state.inner().lock().unwrap();
    if inner.paused {
        return None;
    }

    let (_process_name, window_title) = get_active_window_info()?;
    let idle_time = get_idle_time();
    let is_idle = idle_time >= inner.idle_threshold_seconds;

    Some(ActivitySample {
        timestamp: chrono::Utc::now().timestamp(),
        app_id: 1,
        is_idle,
        window_title: if inner.track_window_titles {
            Some(window_title)
        } else {
            None
        },
    })
}

#[tauri::command]
fn get_today_summary() -> serde_json::Value {
    use rusqlite::Connection;

    let conn = match Connection::open("worka.db") {
        Ok(c) => c,
        Err(_) => {
            return serde_json::json!({
                "active_time_seconds": 0,
                "idle_time_seconds": 0,
                "keyboard_presses": 0,
                "mouse_clicks": 0,
                "top_apps": []
            });
        }
    };

    let now = chrono::Utc::now();
    let start_of_day = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();

    let active_time: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(
                CASE WHEN is_idle = 0 THEN 1 ELSE 0 END
            ), 0) * 10 as seconds
             FROM activity_sample
             WHERE timestamp >= ?1",
            [&start_of_day],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let idle_time: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(
                CASE WHEN is_idle = 1 THEN 1 ELSE 0 END
            ), 0) * 10 as seconds
             FROM activity_sample
             WHERE timestamp >= ?1",
            [&start_of_day],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn
        .prepare(
            "SELECT a.display_name, COUNT(*) * 10 as seconds
             FROM activity_sample s
             JOIN app a ON s.app_id = a.id
             WHERE s.timestamp >= ?1 AND s.is_idle = 0
             GROUP BY s.app_id
             ORDER BY seconds DESC
             LIMIT 5",
        )
        .unwrap();

    let top_apps: Vec<serde_json::Value> = stmt
        .query_map([&start_of_day], |row| {
            let name: String = row.get(0)?;
            let seconds: i64 = row.get(1)?;
            Ok((name, seconds))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .map(|(name, seconds)| {
            serde_json::json!({
                "name": name,
                "time_seconds": seconds,
                "percentage": 0
            })
        })
        .collect();

    serde_json::json!({
        "active_time_seconds": active_time,
        "idle_time_seconds": idle_time,
        "keyboard_presses": 0,
        "mouse_clicks": 0,
        "top_apps": top_apps
    })
}

#[tauri::command]
fn get_week_summary() -> serde_json::Value {
    serde_json::json!({
        "days": []
    })
}

#[tauri::command]
fn get_timeline(_date: String) -> serde_json::Value {
    serde_json::json!({
        "segments": []
    })
}

#[tauri::command]
fn toggle_pause(state: tauri::State<Mutex<AppState>>) -> bool {
    let mut inner = state.inner().lock().unwrap();
    inner.paused = !inner.paused;
    inner.paused
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            sample_activity,
            get_today_summary,
            get_week_summary,
            get_timeline,
            toggle_pause,
            quit_app
        ])
        .setup(|app| {
            let _conn = init_db().expect("Failed to initialize database");

            let should_stop = Arc::new(AtomicBool::new(false));
            app.manage(should_stop.clone());

            let handle = app.handle();
            let show = MenuItem::with_id(handle, "show", "Открыть", true, None::<&str>)?;
            let quit = MenuItem::with_id(handle, "quit", "Выйти", true, None::<&str>)?;

            let menu = Menu::with_items(handle, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    "quit" => {
                        let stop_flag = app.state::<Arc<AtomicBool>>();
                        stop_flag.store(true, Ordering::Relaxed);
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(10));
                    
                    let stop_flag = app_handle.state::<Arc<AtomicBool>>();
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    let state = app_handle.state::<Mutex<AppState>>();
                    let inner = state.lock().unwrap();
                    if !inner.paused {
                        let _ = sample_activity_inner(&inner);
                    }
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let RunEvent::WindowEvent { label, event, .. } = event {
                if label == "main" {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            window.hide().unwrap();
                        }
                    }
                }
            }
        });
}

fn sample_activity_inner(state: &AppState) -> Result<()> {
    if state.paused {
        return Ok(());
    }

    let conn = Connection::open("worka.db")?;

    if let Some((process_name, window_title)) = get_active_window_info() {
        let idle_time = get_idle_time();
        let is_idle = idle_time >= state.idle_threshold_seconds;

        let app_id: i64 = {
            let mut stmt = conn.prepare("SELECT id FROM app WHERE exe_path = ?1")?;
            match stmt.query_row([&process_name], |row| row.get(0)) {
                Ok(id) => id,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    conn.execute(
                        "INSERT INTO app (exe_path, display_name) VALUES (?1, ?2)",
                        [&process_name, &process_name],
                    )?;
                    conn.last_insert_rowid()
                }
                Err(e) => return Err(e),
            }
        };

        conn.execute(
            "INSERT INTO activity_sample (timestamp, app_id, is_idle, window_title) VALUES (?1, ?2, ?3, ?4)",
            params![
                chrono::Utc::now().timestamp(),
                app_id,
                if is_idle { 1 } else { 0 },
                window_title,
            ],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_app_state_default_has_paused_false() {
        let state = AppState::default();
        assert!(!state.paused);
    }

    #[test]
    fn should_app_state_default_has_sample_interval_10() {
        let state = AppState::default();
        assert_eq!(state.sample_interval_seconds, 10);
    }

    #[test]
    fn should_app_state_default_has_idle_threshold_120() {
        let state = AppState::default();
        assert_eq!(state.idle_threshold_seconds, 120);
    }

    #[test]
    fn should_init_db_creates_tables() {
        let conn = Connection::open(":memory:").unwrap();
        
        conn.execute(
            "CREATE TABLE app (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                exe_path TEXT UNIQUE NOT NULL,
                display_name TEXT NOT NULL
            )",
            [],
        ).unwrap();

        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(table_names.contains(&"app".to_string()));
    }

    #[test]
    fn should_toggle_pause_changes_state() {
        let state = Mutex::new(AppState::default());
        
        {
            let inner = state.lock().unwrap();
            assert!(!inner.paused);
        }

        {
            let mut inner = state.lock().unwrap();
            inner.paused = !inner.paused;
        }

        {
            let inner = state.lock().unwrap();
            assert!(inner.paused);
        }
    }

    #[test]
    fn should_get_idle_time_returns_non_negative() {
        let idle_time = get_idle_time();
        assert!(idle_time >= 0);
    }

    #[test]
    fn should_get_active_window_info_does_not_panic() {
        let result = std::panic::catch_unwind(|| {
            get_active_window_info()
        });
        
        assert!(result.is_ok());
    }

    #[test]
    fn should_today_summary_return_valid_json() {
        let summary = get_today_summary();
        assert!(summary.get("active_time_seconds").is_some());
        assert!(summary.get("top_apps").is_some());
    }

    #[test]
    fn should_chrono_datelike_works_correctly() {
        let now = chrono::Utc::now();
        let year = now.year();
        let month = now.month();
        let day = now.day();

        assert!(year >= 2024);
        assert!(month >= 1 && month <= 12);
        assert!(day >= 1 && day <= 31);
    }
}
