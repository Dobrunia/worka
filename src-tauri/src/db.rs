use crate::state::AppState;
use crate::windows_integration::{
    extract_app_icon_data_url, get_active_window_info, get_idle_time,
};
use chrono::Datelike;
use rusqlite::{Connection, Result, params};

pub fn init_db(path: impl AsRef<std::path::Path>) -> Result<Connection> {
    let conn = Connection::open(path)?;

    // WAL mode: readers never block writers, writers never block readers.
    // synchronous=NORMAL is crash-safe with WAL and faster than FULL.
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            exe_path      TEXT UNIQUE NOT NULL,
            display_name  TEXT NOT NULL,
            icon_data_url TEXT
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
        VALUES (1, 10, 120, 1, 1, 0, 0);",
    )?;

    ensure_app_icon_column(&conn)?;
    Ok(conn)
}

fn ensure_app_icon_column(conn: &Connection) -> Result<()> {
    let migration_result = conn.execute("ALTER TABLE app ADD COLUMN icon_data_url TEXT", []);
    if let Err(error) = migration_result {
        if !is_duplicate_column_error(&error, "icon_data_url") {
            return Err(error);
        }
    }
    Ok(())
}

fn is_duplicate_column_error(error: &rusqlite::Error, column_name: &str) -> bool {
    match error {
        rusqlite::Error::SqliteFailure(_, Some(message)) => {
            message.contains("duplicate column name") && message.contains(column_name)
        }
        _ => false,
    }
}

pub fn load_settings_from_db(conn: &Connection) -> AppState {
    conn.query_row(
        "SELECT paused, sample_interval_seconds, idle_threshold_seconds,
                track_window_titles, track_input, autostart
         FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(AppState {
                paused: row.get::<_, bool>(0)?,
                sample_interval_seconds: row.get::<_, i64>(1)? as u64,
                idle_threshold_seconds: row.get::<_, i64>(2)? as u64,
                track_window_titles: row.get::<_, bool>(3)?,
                track_input: row.get::<_, bool>(4)?,
                autostart: row.get::<_, bool>(5)?,
            })
        },
    )
    .unwrap_or_default()
}

pub fn sample_activity_inner(state: &AppState, conn: &Connection) -> Result<()> {
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
        let mut stmt =
            conn.prepare_cached("SELECT id, icon_data_url FROM app WHERE exe_path = ?1")?;
        match stmt.query_row([&exe_path], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
        }) {
            Ok((id, existing_icon)) => {
                if existing_icon.is_none() {
                    if let Some(icon_data_url) = extract_app_icon_data_url(&exe_path) {
                        conn.execute(
                            "UPDATE app SET icon_data_url = ?1 WHERE id = ?2",
                            params![icon_data_url, id],
                        )?;
                    }
                }
                id
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let icon_data_url = extract_app_icon_data_url(&exe_path);
                conn.execute(
                    "INSERT INTO app (exe_path, display_name, icon_data_url)
                     VALUES (?1, ?2, ?3)",
                    params![&exe_path, &display_name, icon_data_url],
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

pub fn build_today_summary(
    conn: &Connection,
    interval: i64,
    is_paused: bool,
) -> Result<serde_json::Value, String> {
    let start_of_day = {
        let now = chrono::Utc::now();
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    };

    let (active_time, idle_time): (i64, i64) = conn
        .query_row(
            "SELECT
               COALESCE(SUM(CASE WHEN is_idle = 0 THEN 1 ELSE 0 END), 0) * ?2,
               COALESCE(SUM(CASE WHEN is_idle = 1 THEN 1 ELSE 0 END), 0) * ?2
             FROM activity_sample WHERE timestamp >= ?1",
            params![start_of_day, interval],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare_cached(
            "SELECT a.display_name, a.icon_data_url, COUNT(*) * ?2 as seconds
             FROM activity_sample s
             JOIN app a ON s.app_id = a.id
             WHERE s.timestamp >= ?1 AND s.is_idle = 0
             GROUP BY s.app_id
             ORDER BY seconds DESC
             LIMIT 15",
        )
        .map_err(|e| e.to_string())?;

    let raw_apps: Vec<(String, Option<String>, i64)> = stmt
        .query_map(params![start_of_day, interval], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let (keyboard_total, mouse_total): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(keyboard_presses), 0), COALESCE(SUM(mouse_clicks), 0)
             FROM input_sample WHERE timestamp_bucket >= ?1",
            params![start_of_day],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0, 0));

    let total_active = active_time.max(1);
    let top_apps: Vec<serde_json::Value> = raw_apps
        .into_iter()
        .map(|(name, icon_data_url, seconds)| {
            let percentage = (seconds * 100 / total_active) as u32;
            serde_json::json!({
                "name": name,
                "icon_data_url": icon_data_url,
                "time_seconds": seconds,
                "percentage": percentage
            })
        })
        .collect();

    Ok(serde_json::json!({
        "active_time_seconds": active_time,
        "idle_time_seconds":   idle_time,
        "keyboard_presses":    keyboard_total,
        "mouse_clicks":        mouse_total,
        "top_apps":            top_apps,
        "is_paused":           is_paused
    }))
}

pub fn build_week_summary(conn: &Connection, interval: i64) -> Result<serde_json::Value, String> {
    let now = chrono::Utc::now();
    let days: Vec<serde_json::Value> = (0..7)
        .rev()
        .map(|days_ago| {
            let date = now.date_naive() - chrono::Duration::days(days_ago);
            let start = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
            let end = date.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp();

            let (active, idle): (i64, i64) = conn
                .query_row(
                    "SELECT
                       COALESCE(SUM(CASE WHEN is_idle = 0 THEN 1 ELSE 0 END), 0) * ?1,
                       COALESCE(SUM(CASE WHEN is_idle = 1 THEN 1 ELSE 0 END), 0) * ?1
                     FROM activity_sample WHERE timestamp >= ?2 AND timestamp <= ?3",
                    params![interval, start, end],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap_or((0, 0));

            let (keyboard, mouse): (i64, i64) = conn
                .query_row(
                    "SELECT COALESCE(SUM(keyboard_presses), 0), COALESCE(SUM(mouse_clicks), 0)
                     FROM input_sample WHERE timestamp_bucket >= ?1 AND timestamp_bucket <= ?2",
                    params![start, end],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap_or((0, 0));

            serde_json::json!({
                "date": date.format("%Y-%m-%d").to_string(),
                "day_name": date.format("%a").to_string(),
                "active_time_seconds": active,
                "idle_time_seconds": idle,
                "keyboard_presses": keyboard,
                "mouse_clicks": mouse,
            })
        })
        .collect();

    Ok(serde_json::json!({ "days": days }))
}

pub fn build_all_time_summary(conn: &Connection, interval: i64) -> Result<serde_json::Value, String> {
    let (active_time, idle_time): (i64, i64) = conn
        .query_row(
            "SELECT
               COALESCE(SUM(CASE WHEN is_idle = 0 THEN 1 ELSE 0 END), 0) * ?1,
               COALESCE(SUM(CASE WHEN is_idle = 1 THEN 1 ELSE 0 END), 0) * ?1
             FROM activity_sample",
            params![interval],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let (keyboard_total, mouse_total): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(keyboard_presses), 0), COALESCE(SUM(mouse_clicks), 0)
             FROM input_sample",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0, 0));

    let mut stmt = conn
        .prepare_cached(
            "SELECT a.display_name, a.icon_data_url, COUNT(*) * ?1 as seconds
             FROM activity_sample s
             JOIN app a ON s.app_id = a.id
             WHERE s.is_idle = 0
             GROUP BY s.app_id
             ORDER BY seconds DESC
             LIMIT 15",
        )
        .map_err(|e| e.to_string())?;

    let raw_apps: Vec<(String, Option<String>, i64)> = stmt
        .query_map(params![interval], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let total_active = active_time.max(1);
    let top_apps: Vec<serde_json::Value> = raw_apps
        .into_iter()
        .map(|(name, icon_data_url, seconds)| {
            let percentage = (seconds * 100 / total_active) as u32;
            serde_json::json!({
                "name": name,
                "icon_data_url": icon_data_url,
                "time_seconds": seconds,
                "percentage": percentage,
            })
        })
        .collect();

    let days_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT date(timestamp, 'unixepoch')) FROM activity_sample",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(serde_json::json!({
        "active_time_seconds": active_time,
        "idle_time_seconds": idle_time,
        "keyboard_presses": keyboard_total,
        "mouse_clicks": mouse_total,
        "top_apps": top_apps,
        "days_tracked": days_count,
    }))
}

pub fn build_timeline(
    conn: &Connection,
    interval: i64,
    date: &str,
) -> Result<serde_json::Value, String> {
    let selected_date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date '{date}'. Expected format: YYYY-MM-DD"))?;

    let start_ts = selected_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    let end_ts = selected_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc()
        .timestamp();

    let mut hourly_activity = vec![(0i64, 0i64); 24];
    let mut hourly_input = vec![(0i64, 0i64); 24];
    let mut hourly_apps: Vec<Vec<(String, Option<String>, i64)>> = vec![Vec::new(); 24];

    {
        let mut stmt = conn
            .prepare_cached(
                "SELECT
                   CAST((timestamp - ?1) / 3600 AS INTEGER) as hour_idx,
                   COALESCE(SUM(CASE WHEN is_idle = 0 THEN 1 ELSE 0 END), 0) * ?3,
                   COALESCE(SUM(CASE WHEN is_idle = 1 THEN 1 ELSE 0 END), 0) * ?3
                 FROM activity_sample
                 WHERE timestamp >= ?1 AND timestamp <= ?2
                 GROUP BY CAST((timestamp - ?1) / 3600 AS INTEGER)",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![start_ts, end_ts, interval], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        for row in rows.flatten() {
            let (hour_idx, active, idle) = row;
            if (0..24).contains(&hour_idx) {
                hourly_activity[hour_idx as usize] = (active, idle);
            }
        }
    }

    {
        let mut stmt = conn
            .prepare_cached(
                "SELECT
                   CAST((timestamp_bucket - ?1) / 3600 AS INTEGER) as hour_idx,
                   COALESCE(SUM(keyboard_presses), 0),
                   COALESCE(SUM(mouse_clicks), 0)
                 FROM input_sample
                 WHERE timestamp_bucket >= ?1 AND timestamp_bucket <= ?2
                 GROUP BY CAST((timestamp_bucket - ?1) / 3600 AS INTEGER)",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![start_ts, end_ts], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        for row in rows.flatten() {
            let (hour_idx, keyboard, mouse) = row;
            if (0..24).contains(&hour_idx) {
                hourly_input[hour_idx as usize] = (keyboard, mouse);
            }
        }
    }

    {
        let mut stmt = conn
            .prepare_cached(
                "SELECT
                   CAST((s.timestamp - ?1) / 3600 AS INTEGER) as hour_idx,
                   a.display_name,
                   a.icon_data_url,
                   COUNT(*) * ?3 as seconds
                 FROM activity_sample s
                 JOIN app a ON s.app_id = a.id
                 WHERE s.timestamp >= ?1 AND s.timestamp <= ?2 AND s.is_idle = 0
                 GROUP BY CAST((s.timestamp - ?1) / 3600 AS INTEGER), s.app_id
                 ORDER BY hour_idx ASC, seconds DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![start_ts, end_ts, interval], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        for row in rows.flatten() {
            let (hour_idx, name, icon_data_url, seconds) = row;
            if (0..24).contains(&hour_idx) {
                hourly_apps[hour_idx as usize].push((name, icon_data_url, seconds));
            }
        }
    }

    let mut has_data = false;
    let hours = (0..24)
        .map(|hour_idx| {
            let (active, idle) = hourly_activity[hour_idx];
            let (keyboard, mouse) = hourly_input[hour_idx];
            let total_active = active.max(1);
            let apps_for_hour = &hourly_apps[hour_idx];
            let top_app = apps_for_hour.first().map(|(name, icon_data_url, seconds)| {
                serde_json::json!({
                    "name": name,
                    "icon_data_url": icon_data_url,
                    "time_seconds": seconds,
                })
            });

            if active > 0 || idle > 0 || keyboard > 0 || mouse > 0 {
                has_data = true;
            }

            let apps = apps_for_hour
                .iter()
                .map(|(name, icon_data_url, seconds)| {
                    serde_json::json!({
                        "name": name,
                        "icon_data_url": icon_data_url,
                        "time_seconds": seconds,
                        "percentage": ((*seconds * 100) / total_active),
                    })
                })
                .collect::<Vec<_>>();

            serde_json::json!({
                "hour": format!("{hour_idx:02}:00"),
                "hour_index": hour_idx,
                "active_time_seconds": active,
                "idle_time_seconds": idle,
                "keyboard_presses": keyboard,
                "mouse_clicks": mouse,
                "top_app": top_app,
                "apps": apps,
            })
        })
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "date": selected_date.format("%Y-%m-%d").to_string(),
        "hours": hours,
        "has_data": has_data,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn should_init_db_creates_app_icon_column() {
        let conn = init_db(":memory:").unwrap();
        let mut stmt = conn.prepare("PRAGMA table_info(app)").unwrap();
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(columns.contains(&"icon_data_url".to_string()));
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
    fn should_input_sample_insert_and_query_works() {
        let conn = init_db(":memory:").unwrap();
        let ts = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO input_sample
             (timestamp_bucket, keyboard_presses, mouse_clicks, mouse_wheel)
             VALUES (?1, 42, 17, 5)",
            params![ts],
        )
        .unwrap();
        let (kbd, clicks): (i64, i64) = conn
            .query_row(
                "SELECT COALESCE(SUM(keyboard_presses), 0), COALESCE(SUM(mouse_clicks), 0)
                 FROM input_sample WHERE timestamp_bucket >= ?1",
                params![ts - 1],
                |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
        assert_eq!(kbd, 42);
        assert_eq!(clicks, 17);
    }

    #[test]
    fn should_build_timeline_returns_24_hours() {
        let conn = init_db(":memory:").unwrap();

        conn.execute(
            "INSERT INTO app (exe_path, display_name, icon_data_url)
             VALUES ('demo.exe', 'Demo', NULL)",
            [],
        )
        .unwrap();
        let app_id = conn.last_insert_rowid();

        let date = chrono::Utc::now().date_naive();
        let day_start = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        conn.execute(
            "INSERT INTO activity_sample (timestamp, app_id, is_idle, window_title)
             VALUES (?1, ?2, 0, NULL)",
            params![day_start + 90, app_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO input_sample
             (timestamp_bucket, keyboard_presses, mouse_clicks, mouse_wheel)
             VALUES (?1, 7, 3, 0)",
            params![day_start + 120],
        )
        .unwrap();

        let timeline = build_timeline(&conn, 10, &date.format("%Y-%m-%d").to_string())
            .unwrap();
        let hours = timeline
            .get("hours")
            .and_then(|v| v.as_array())
            .unwrap();

        assert_eq!(hours.len(), 24);
        assert_eq!(timeline.get("has_data").and_then(|v| v.as_bool()), Some(true));
    }
}
