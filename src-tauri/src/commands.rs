use crate::db::{build_all_time_summary, build_timeline, build_today_summary, build_week_summary};
use crate::state::{AppDb, AppState};
use std::sync::Mutex;

#[tauri::command]
pub fn get_settings(state: tauri::State<Mutex<AppState>>) -> AppState {
    state.inner().lock().unwrap().clone()
}

#[tauri::command]
pub fn set_settings(
    state: tauri::State<Mutex<AppState>>,
    db: tauri::State<AppDb>,
    paused: bool,
    sample_interval_seconds: u64,
    idle_threshold_seconds: u64,
    track_window_titles: bool,
    track_input: bool,
    autostart: bool,
) {
    {
        let mut inner = state.inner().lock().unwrap();
        inner.paused = paused;
        inner.sample_interval_seconds = sample_interval_seconds;
        inner.idle_threshold_seconds = idle_threshold_seconds;
        inner.track_window_titles = track_window_titles;
        inner.track_input = track_input;
        inner.autostart = autostart;
    }

    let conn = db.0.lock().unwrap();
    let _ = conn.execute(
        "UPDATE settings SET paused = ?1, sample_interval_seconds = ?2,
         idle_threshold_seconds = ?3, track_window_titles = ?4,
         track_input = ?5, autostart = ?6
         WHERE id = 1",
        rusqlite::params![
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
pub fn get_today_summary(
    db: tauri::State<AppDb>,
    state: tauri::State<Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let (interval, is_paused) = {
        let s = state.inner().lock().unwrap();
        (s.sample_interval_seconds as i64, s.paused)
    };

    let conn = db.0.lock().unwrap();
    build_today_summary(&conn, interval, is_paused)
}

#[tauri::command]
pub fn get_week_summary(
    db: tauri::State<AppDb>,
    state: tauri::State<Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let interval = {
        let s = state.inner().lock().unwrap();
        s.sample_interval_seconds as i64
    };

    let conn = db.0.lock().unwrap();
    build_week_summary(&conn, interval)
}

#[tauri::command]
pub fn get_all_time_summary(
    db: tauri::State<AppDb>,
    state: tauri::State<Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let interval = {
        let s = state.inner().lock().unwrap();
        s.sample_interval_seconds as i64
    };

    let conn = db.0.lock().unwrap();
    build_all_time_summary(&conn, interval)
}

#[tauri::command]
pub fn get_timeline(
    db: tauri::State<AppDb>,
    state: tauri::State<Mutex<AppState>>,
    date: String,
) -> Result<serde_json::Value, String> {
    let interval = {
        let s = state.inner().lock().unwrap();
        s.sample_interval_seconds as i64
    };

    let conn = db.0.lock().unwrap();
    build_timeline(&conn, interval, &date)
}

#[tauri::command]
pub fn toggle_pause(state: tauri::State<Mutex<AppState>>, db: tauri::State<AppDb>) -> bool {
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
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
