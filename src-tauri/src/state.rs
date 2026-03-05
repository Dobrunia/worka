use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::AtomicU64,
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

/// Shared SQLite connection — wrapped in Mutex so it can be moved between threads.
pub struct AppDb(pub Mutex<Connection>);

/// Accumulated input event counts between sampler flushes.
/// AtomicU64 allows lock-free increments from the hook thread.
pub struct InputCounters {
    pub keyboard: AtomicU64,
    pub mouse_clicks: AtomicU64,
    pub mouse_wheel: AtomicU64,
}

/// Global static so that `extern "system"` hook callbacks can access the counters
/// without any locks or allocations in the hot path.
pub static INPUT_COUNTERS: OnceLock<Arc<InputCounters>> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

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
    fn should_input_counters_atomic_ops_work() {
        let c = InputCounters {
            keyboard: AtomicU64::new(0),
            mouse_clicks: AtomicU64::new(0),
            mouse_wheel: AtomicU64::new(0),
        };
        c.keyboard.fetch_add(3, Ordering::Relaxed);
        c.mouse_clicks.fetch_add(7, Ordering::Relaxed);
        assert_eq!(c.keyboard.swap(0, Ordering::Relaxed), 3);
        assert_eq!(c.mouse_clicks.swap(0, Ordering::Relaxed), 7);
        assert_eq!(c.keyboard.load(Ordering::Relaxed), 0);
    }
}
