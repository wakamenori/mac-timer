use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::pomodoro::{PomodoroConfig, PomodoroTimer};
use crate::timer::BasicTimer;
use crate::tray::update_tray_title;

#[derive(Debug)]
pub enum ActiveTimer {
    Basic(BasicTimer),
    Pomodoro(PomodoroTimer),
}

#[derive(Debug)]
pub struct AppState {
    pub active: ActiveTimer,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active: ActiveTimer::Pomodoro(PomodoroTimer::new(PomodoroConfig::default())),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct TimerSnapshot {
    pub mode: String,
    pub display: String,
    pub remaining_secs: u32,
    pub total_secs: u32,
    pub is_running: bool,
    pub is_finished: bool,
    pub phase: Option<String>,
    pub session_display: Option<String>,
    pub tray_title: String,
}

impl TimerSnapshot {
    pub fn from_state(active: &ActiveTimer) -> Self {
        match active {
            ActiveTimer::Basic(t) => Self::from_basic(t),
            ActiveTimer::Pomodoro(t) => Self::from_pomodoro(t),
        }
    }

    pub fn from_basic(t: &BasicTimer) -> Self {
        Self {
            mode: "basic".to_string(),
            display: t.display(),
            remaining_secs: t.remaining_secs(),
            total_secs: t.duration_secs(),
            is_running: t.status() == crate::timer::TimerStatus::Running,
            is_finished: t.is_finished(),
            phase: None,
            session_display: None,
            tray_title: format!("⏱ {}", t.display()),
        }
    }

    pub fn from_pomodoro(t: &PomodoroTimer) -> Self {
        Self {
            mode: "pomodoro".to_string(),
            display: t.display(),
            remaining_secs: t.remaining_secs(),
            total_secs: t.phase_duration_secs(),
            is_running: t.status() == crate::pomodoro::PomodoroStatus::Running,
            is_finished: false,
            phase: Some(format!("{:?}", t.phase())),
            session_display: Some(t.session_display()),
            tray_title: t.tray_title(),
        }
    }
}

// --- Pure state-transition functions (Tauri-independent) ---

pub(crate) fn do_start(state: &mut AppState) -> TimerSnapshot {
    match &mut state.active {
        ActiveTimer::Basic(t) => t.start(),
        ActiveTimer::Pomodoro(t) => t.start(),
    }
    TimerSnapshot::from_state(&state.active)
}

pub(crate) fn do_pause(state: &mut AppState) -> TimerSnapshot {
    match &mut state.active {
        ActiveTimer::Basic(t) => t.pause(),
        ActiveTimer::Pomodoro(t) => t.pause(),
    }
    TimerSnapshot::from_state(&state.active)
}

pub(crate) fn do_reset(state: &mut AppState) -> TimerSnapshot {
    match &mut state.active {
        ActiveTimer::Basic(t) => t.reset(),
        ActiveTimer::Pomodoro(t) => t.reset(),
    }
    TimerSnapshot::from_state(&state.active)
}

pub(crate) fn do_set_duration(state: &mut AppState, secs: u32) -> Option<TimerSnapshot> {
    if let ActiveTimer::Basic(t) = &mut state.active {
        t.set_duration(secs);
        Some(TimerSnapshot::from_basic(t))
    } else {
        None
    }
}

pub(crate) fn do_switch_to_basic(state: &mut AppState) -> TimerSnapshot {
    state.active = ActiveTimer::Basic(BasicTimer::new(25 * 60));
    TimerSnapshot::from_state(&state.active)
}

pub(crate) fn do_switch_to_pomodoro(state: &mut AppState) -> TimerSnapshot {
    state.active = ActiveTimer::Pomodoro(PomodoroTimer::new(PomodoroConfig::default()));
    TimerSnapshot::from_state(&state.active)
}

pub(crate) fn do_get_snapshot(state: &AppState) -> TimerSnapshot {
    TimerSnapshot::from_state(&state.active)
}

// --- Tauri command wrappers ---

/// Emit snapshot to the window and update tray title immediately.
/// Must be called AFTER dropping the AppState lock.
fn emit_and_update_tray(app: &AppHandle, snapshot: TimerSnapshot) {
    let _ = app.emit("timer:tick", &snapshot);
    update_tray_title(app);
}

#[tauri::command]
pub fn start_timer(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut s = state.lock().unwrap();
        do_start(&mut s)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn pause_timer(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut s = state.lock().unwrap();
        do_pause(&mut s)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn reset_timer(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut s = state.lock().unwrap();
        do_reset(&mut s)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn set_duration(app: AppHandle, state: State<'_, Mutex<AppState>>, secs: u32) {
    let snapshot = {
        let mut s = state.lock().unwrap();
        do_set_duration(&mut s, secs)
    };
    if let Some(s) = snapshot {
        emit_and_update_tray(&app, s);
    }
}

#[tauri::command]
pub fn switch_to_basic(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut s = state.lock().unwrap();
        do_switch_to_basic(&mut s)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn switch_to_pomodoro(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut s = state.lock().unwrap();
        do_switch_to_pomodoro(&mut s)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, Mutex<AppState>>) -> TimerSnapshot {
    let s = state.lock().unwrap();
    do_get_snapshot(&s)
}

#[tauri::command]
pub fn toggle_always_on_top(window: tauri::Window) {
    if let Ok(is_on_top) = window.is_always_on_top() {
        let _ = window.set_always_on_top(!is_on_top);
    }
}

#[tauri::command]
pub fn dismiss_notification(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("notification") {
        let _ = win.close();
    }
}

#[tauri::command]
pub fn dismiss_overlay(app: tauri::AppHandle) {
    crate::runner::close_overlay_windows(&app);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::BasicTimer;

    // --- TimerSnapshot conversion tests ---

    #[test]
    fn from_basic_has_correct_fields() {
        let t = BasicTimer::new(300);
        let snap = TimerSnapshot::from_basic(&t);
        assert_eq!(snap.mode, "basic");
        assert!(!snap.is_running);
        assert!(!snap.is_finished);
        assert_eq!(snap.display, "05:00");
        assert_eq!(snap.remaining_secs, 300);
        assert_eq!(snap.total_secs, 300);
        assert_eq!(snap.tray_title, "⏱ 05:00");
        assert!(snap.phase.is_none());
        assert!(snap.session_display.is_none());
    }

    #[test]
    fn from_pomodoro_has_correct_fields() {
        let t = PomodoroTimer::new(PomodoroConfig::default());
        let snap = TimerSnapshot::from_pomodoro(&t);
        assert_eq!(snap.mode, "pomodoro");
        assert!(!snap.is_running);
        assert!(!snap.is_finished);
        assert_eq!(snap.remaining_secs, 25 * 60);
        assert_eq!(snap.total_secs, 25 * 60);
        assert_eq!(snap.phase, Some("Work".to_string()));
        assert_eq!(snap.session_display, Some("○ ○ ○ ○".to_string()));
        assert!(snap.tray_title.contains("🍅"));
    }

    #[test]
    fn from_state_dispatches_to_basic() {
        let active = ActiveTimer::Basic(BasicTimer::new(600));
        let snap = TimerSnapshot::from_state(&active);
        assert_eq!(snap.mode, "basic");
    }

    #[test]
    fn from_state_dispatches_to_pomodoro() {
        let active = ActiveTimer::Pomodoro(PomodoroTimer::new(PomodoroConfig::default()));
        let snap = TimerSnapshot::from_state(&active);
        assert_eq!(snap.mode, "pomodoro");
    }

    // --- State transition tests (do_* functions) ---

    #[test]
    fn do_start_sets_running() {
        let mut state = AppState {
            active: ActiveTimer::Basic(BasicTimer::new(300)),
        };
        let snap = do_start(&mut state);
        assert!(snap.is_running);
    }

    #[test]
    fn do_pause_stops_running() {
        let mut state = AppState {
            active: ActiveTimer::Basic(BasicTimer::new(300)),
        };
        do_start(&mut state);
        let snap = do_pause(&mut state);
        assert!(!snap.is_running);
    }

    #[test]
    fn do_reset_restores_full_duration() {
        let mut state = AppState {
            active: ActiveTimer::Basic(BasicTimer::new(300)),
        };
        do_start(&mut state);
        // Tick to reduce remaining
        if let ActiveTimer::Basic(t) = &mut state.active {
            t.tick();
        }
        let snap = do_reset(&mut state);
        assert_eq!(snap.remaining_secs, 300);
        assert_eq!(snap.total_secs, 300);
        assert!(!snap.is_running);
    }

    #[test]
    fn do_set_duration_on_basic_updates_duration() {
        let mut state = AppState {
            active: ActiveTimer::Basic(BasicTimer::new(300)),
        };
        let snap = do_set_duration(&mut state, 600);
        assert!(snap.is_some());
        let snap = snap.unwrap();
        assert_eq!(snap.remaining_secs, 600);
        assert_eq!(snap.total_secs, 600);
    }

    #[test]
    fn do_set_duration_on_pomodoro_returns_none() {
        let mut state = AppState::default();
        let snap = do_set_duration(&mut state, 600);
        assert!(snap.is_none());
    }

    #[test]
    fn do_switch_to_basic_creates_basic_timer() {
        let mut state = AppState::default(); // starts as pomodoro
        let snap = do_switch_to_basic(&mut state);
        assert_eq!(snap.mode, "basic");
        assert_eq!(snap.remaining_secs, 25 * 60);
    }

    #[test]
    fn do_switch_to_pomodoro_creates_pomodoro_timer() {
        let mut state = AppState {
            active: ActiveTimer::Basic(BasicTimer::new(300)),
        };
        let snap = do_switch_to_pomodoro(&mut state);
        assert_eq!(snap.mode, "pomodoro");
        assert_eq!(snap.remaining_secs, 25 * 60);
        assert_eq!(snap.phase, Some("Work".to_string()));
    }

    #[test]
    fn start_pause_reset_sequence() {
        let mut state = AppState {
            active: ActiveTimer::Basic(BasicTimer::new(300)),
        };
        let snap = do_start(&mut state);
        assert!(snap.is_running);

        let snap = do_pause(&mut state);
        assert!(!snap.is_running);

        let snap = do_reset(&mut state);
        assert!(!snap.is_running);
        assert_eq!(snap.remaining_secs, 300);
    }

    // --- AppState default test ---

    #[test]
    fn app_state_default_is_pomodoro() {
        let state = AppState::default();
        let snap = do_get_snapshot(&state);
        assert_eq!(snap.mode, "pomodoro");
    }
}
