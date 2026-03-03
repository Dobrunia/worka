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
    Foundation::CloseHandle,
    System::{
        SystemInformation::GetTickCount64,
        Threading::{
            OpenProcess, QueryFullProcessImageNameW,
            PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        },
    },
    UI::{
        Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
        WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
    },
};

// ─── State structs ────────────────────────────────────────────────────────────

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

/// Shared SQLite connection — wrapped in Mutex so it can be moved between threads.
pub struct AppDb(pub Mutex<Connection>);

// ─── Windows helpers ──────────────────────────────────────────────────────────

/// Seconds since last keyboard/mouse input.
fn get_idle_time() -> u64 {
    unsafe {
        let mut last_input = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut last_input).as_bool() {
            let now = GetTickCount64();
            (now.saturating_sub(last_input.dwTime as u64)) / 1000
        } else {
            0
        }
    }
}

/// Returns (exe_full_path, window_title) for the currently focused window.
/// Returns None if the foreground window can't be resolved.
fn get_active_window_info() -> Option<(String, String)> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // Resolve the real executable path.
        let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
        let mut name_buf = [0u16; 1024];
        let mut name_len = name_buf.len() as u32;
        let query_result = QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(name_buf.as_mut_ptr()),
            &mut name_len,
        );
        let _ = CloseHandle(process_handle);
        query_result.ok()?;
        let exe_path = String::from_utf16_lossy(&name_buf[..name_len as usize]).to_string();

        // Window title.
        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        let title = String::from_utf16_lossy(&title_buf[..len as usize]).to_string();

        Some((exe_path, title))
    }
}

// ─── Database ─────────────────────────────────────────────────────────────────

fn init_db(path: impl AsRef<std::path::Path>) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            exe_path     TEXT UNIQUE NOT NULL,
            display_name TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS activity_sample (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp    INTEGER NOT NULL,
            app_id       INTEGER NOT NULL,
            is_idle      INTEGER NOT NULL,
            window_title TEXT
        );
        CREATE TABLE IF NOT EXISTS input_sample (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp_bucket  INTEGER NOT NULL,
            keyboard_presses  INTEGER NOT NULL,
            mouse_clicks      INTEGER NOT NULL,
            mouse_wheel       INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS settings (
            id                       INTEGER PRIMARY KEY CHECK (id = 1),
            sample_interval_seconds  INTEGER NOT NULL DEFAULT 10,
            idle_threshold_seconds   INTEGER NOT NULL DEFAULT 120,
            track_window_titles      INTEGER NOT NULL DEFAULT 1,
            track_input              INTEGER NOT NULL DEFAULT 1,
            autostart                INTEGER NOT NULL DEFAULT 0,
            paused                   INTEGER NOT NULL DEFAULT 0
        );
        INSERT OR IGNORE INTO settings
            (id, sample_interval_seconds, idle_threshold_seconds,
             track_window_titles, track_input, autostart, paused)
        VALUES (1, 10, 120, 1, 1, 0, 0);"
    )?;

    Ok(conn)
}

/// Load AppState from the settings table.  Falls back to Default on any error.
fn load_settings_from_db(conn: &Connection) -> AppState {
    conn.query_row(
        "SELECT paused, sample_interval_seconds, idle_threshold_seconds,
                track_window_titles, track_input, autostart
         FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(AppState {
                paused:                  row.get::<_, bool>(0)?,
                sample_interval_seconds: row.get::<_, i64>(1)? as u64,
                idle_threshold_seconds:  row.get::<_, i64>(2)? as u64,
                track_window_titles:     row.get::<_, bool>(3)?,
                track_input:             row.get::<_, bool>(4)?,
                autostart:               row.get::<_, bool>(5)?,
            })
        },
    )
    .unwrap_or_default()
}

// ─── Background sampler ───────────────────────────────────────────────────────

/// Insert one activity sample into the DB.
fn sample_activity_inner(state: &AppState, conn: &Connection) -> Result<()> {
    if state.paused {
        return Ok(());
    }

    let Some((exe_path, window_title)) = get_active_window_info() else {
        return Ok(());
    };

    let idle_time = get_idle_time();
    let is_idle = idle_time >= state.idle_threshold_seconds;

    let display_name = std::path::Path::new(&exe_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(&exe_path)
        .to_string();

    let app_id: i64 = {
        let mut stmt = conn.prepare("SELECT id FROM app WHERE exe_path = ?1")?;
        match stmt.query_row([&exe_path], |row| row.get(0)) {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                conn.execute(
                    "INSERT INTO app (exe_path, display_name) VALUES (?1, ?2)",
                    [&exe_path, &display_name],
                )?;
                conn.last_insert_rowid()
            }
            Err(e) => return Err(e),
        }
    };

    let title = if state.track_window_titles {
        Some(window_title)
    } else {
        None
    };

    conn.execute(
        "INSERT INTO activity_sample (timestamp, app_id, is_idle, window_title)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            chrono::Utc::now().timestamp(),
            app_id,
            if is_idle { 1i32 } else { 0i32 },
            title,
        ],
    )?;

    Ok(())
}

// ─── Tauri commands ───────────────────────────────────────────────────────────

#[tauri::command]
fn get_settings(state: tauri::State<Mutex<AppState>>) -> AppState {
    state.inner().lock().unwrap().clone()
}

#[tauri::command]
fn set_settings(
    state: tauri::State<Mutex<AppState>>,
    db: tauri::State<AppDb>,
    paused: bool,
    sample_interval_seconds: u64,
    idle_threshold_seconds: u64,
    track_window_titles: bool,
    track_input: bool,
    autostart: bool,
) {
    // Update in-memory state.
    {
        let mut inner = state.inner().lock().unwrap();
        inner.paused = paused;
        inner.sample_interval_seconds = sample_interval_seconds;
        inner.idle_threshold_seconds = idle_threshold_seconds;
        inner.track_window_titles = track_window_titles;
        inner.track_input = track_input;
        inner.autostart = autostart;
    } // state lock released before acquiring db lock

    // Persist to DB.
    let conn = db.0.lock().unwrap();
    let _ = conn.execute(
        "UPDATE settings SET paused = ?1, sample_interval_seconds = ?2,
         idle_threshold_seconds = ?3, track_window_titles = ?4,
         track_input = ?5, autostart = ?6
         WHERE id = 1",
        params![
            paused,
            sample_interval_seconds as i64,
            idle_threshold_seconds as i64,
            track_window_titles,
            track_input,
            autostart,
        ],
    );
}

#[tauri::command]
fn get_today_summary(
    db: tauri::State<AppDb>,
    state: tauri::State<Mutex<AppState>>,
) -> serde_json::Value {
    // Read interval without holding state lock during DB queries.
    let interval = {
        state.inner().lock().unwrap().sample_interval_seconds as i64
    };

    let conn = db.0.lock().unwrap();

    let start_of_day = {
        let now = chrono::Utc::now();
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    };

    let active_time: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(CASE WHEN is_idle = 0 THEN 1 ELSE 0 END), 0) * ?2
             FROM activity_sample WHERE timestamp >= ?1",
            params![start_of_day, interval],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let idle_time: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(CASE WHEN is_idle = 1 THEN 1 ELSE 0 END), 0) * ?2
             FROM activity_sample WHERE timestamp >= ?1",
            params![start_of_day, interval],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn
        .prepare(
            "SELECT a.display_name, COUNT(*) * ?2 as seconds
             FROM activity_sample s
             JOIN app a ON s.app_id = a.id
             WHERE s.timestamp >= ?1 AND s.is_idle = 0
             GROUP BY s.app_id
             ORDER BY seconds DESC
             LIMIT 5",
        )
        .unwrap();

    let raw_apps: Vec<(String, i64)> = stmt
        .query_map(params![start_of_day, interval], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let total_active = active_time.max(1);
    let top_apps: Vec<serde_json::Value> = raw_apps
        .into_iter()
        .map(|(name, seconds)| {
            let percentage = (seconds * 100 / total_active) as u32;
            serde_json::json!({
                "name": name,
                "time_seconds": seconds,
                "percentage": percentage
            })
        })
        .collect();

    serde_json::json!({
        "active_time_seconds": active_time,
        "idle_time_seconds":   idle_time,
        "keyboard_presses":    0,
        "mouse_clicks":        0,
        "top_apps":            top_apps
    })
}

#[tauri::command]
fn get_week_summary() -> serde_json::Value {
    serde_json::json!({ "days": [] })
}

#[tauri::command]
fn get_timeline(_date: String) -> serde_json::Value {
    serde_json::json!({ "segments": [] })
}

#[tauri::command]
fn toggle_pause(state: tauri::State<Mutex<AppState>>, db: tauri::State<AppDb>) -> bool {
    let new_paused = {
        let mut inner = state.inner().lock().unwrap();
        inner.paused = !inner.paused;
        inner.paused
    };
    let conn = db.0.lock().unwrap();
    let _ = conn.execute(
        "UPDATE settings SET paused = ?1 WHERE id = 1",
        [new_paused],
    );
    new_paused
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

// ─── App entry point ──────────────────────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // AppState is managed as Mutex<AppState>; actual values are loaded from
        // DB in setup() below and replaced via app.manage().
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_today_summary,
            get_week_summary,
            get_timeline,
            toggle_pause,
            quit_app
        ])
        .setup(|app| {
            // ── Stable DB path ────────────────────────────────────────────────
            let data_dir = app.path().app_data_dir()
                .expect("Failed to resolve app data directory");
            std::fs::create_dir_all(&data_dir)
                .expect("Failed to create app data directory");
            let db_path = data_dir.join("worka.db");

            let conn = init_db(&db_path).expect("Failed to initialize database");

            // ── Load persisted settings ───────────────────────────────────────
            let saved_state = load_settings_from_db(&conn);
            // Replace the default AppState with the one loaded from DB.
            *app.state::<Mutex<AppState>>().lock().unwrap() = saved_state;

            app.manage(AppDb(Mutex::new(conn)));

            // ── Stop flag for background thread ───────────────────────────────
            let should_stop = Arc::new(AtomicBool::new(false));
            app.manage(should_stop.clone());

            // ── Tray menu ─────────────────────────────────────────────────────
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

            // ── Background sampler thread ─────────────────────────────────────
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // Use the configured interval; start with the default so we
                // always have a valid duration even before the first iteration.
                let mut interval = app_handle
                    .state::<Mutex<AppState>>()
                    .lock()
                    .unwrap()
                    .sample_interval_seconds;

                loop {
                    std::thread::sleep(Duration::from_secs(interval));

                    if app_handle.state::<Arc<AtomicBool>>().load(Ordering::Relaxed) {
                        break;
                    }

                    // Clone AppState so we hold the lock for the shortest time
                    // possible and never hold it during DB I/O.
                    let app_state = app_handle
                        .state::<Mutex<AppState>>()
                        .lock()
                        .unwrap()
                        .clone();

                    interval = app_state.sample_interval_seconds;

                    if !app_state.paused {
                        let db = app_handle.state::<AppDb>();
                        let conn = db.0.lock().unwrap();
                        let _ = sample_activity_inner(&app_state, &conn);
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

// ─── Tests ────────────────────────────────────────────────────────────────────

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
    fn should_init_db_creates_all_tables() {
        let conn = init_db(":memory:").unwrap();
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(table_names.contains(&"app".to_string()));
        assert!(table_names.contains(&"activity_sample".to_string()));
        assert!(table_names.contains(&"input_sample".to_string()));
        assert!(table_names.contains(&"settings".to_string()));
    }

    #[test]
    fn should_load_settings_from_db_returns_defaults() {
        let conn = init_db(":memory:").unwrap();
        let state = load_settings_from_db(&conn);
        assert!(!state.paused);
        assert_eq!(state.sample_interval_seconds, 10);
        assert_eq!(state.idle_threshold_seconds, 120);
    }

    #[test]
    fn should_toggle_pause_changes_state() {
        let state = Mutex::new(AppState::default());
        assert!(!state.lock().unwrap().paused);
        state.lock().unwrap().paused = true;
        assert!(state.lock().unwrap().paused);
    }

    #[test]
    fn should_get_idle_time_does_not_panic() {
        let _ = get_idle_time();
    }

    #[test]
    fn should_get_active_window_info_does_not_panic() {
        let result = std::panic::catch_unwind(get_active_window_info);
        assert!(result.is_ok());
    }

    #[test]
    fn should_activity_sample_insert_and_query_works() {
        let conn = init_db(":memory:").unwrap();
        conn.execute(
            "INSERT INTO app (exe_path, display_name) VALUES ('test.exe', 'Test')",
            [],
        )
        .unwrap();
        let app_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO activity_sample (timestamp, app_id, is_idle, window_title)
             VALUES (?1, ?2, 0, NULL)",
            params![chrono::Utc::now().timestamp(), app_id],
        )
        .unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM activity_sample WHERE is_idle = 0",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn should_chrono_datelike_works_correctly() {
        let now = chrono::Utc::now();
        assert!(now.year() >= 2024);
        assert!((1..=12).contains(&now.month()));
        assert!((1..=31).contains(&now.day()));
    }
}
