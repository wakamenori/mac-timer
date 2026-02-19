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

/// Emit snapshot to the window and update tray title immediately.
/// Must be called AFTER dropping the AppState lock.
fn emit_and_update_tray(app: &AppHandle, snapshot: TimerSnapshot) {
    let _ = app.emit("timer:tick", &snapshot);
    update_tray_title(app);
}

#[tauri::command]
pub fn start_timer(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut state = state.lock().unwrap();
        match &mut state.active {
            ActiveTimer::Basic(t) => t.start(),
            ActiveTimer::Pomodoro(t) => t.start(),
        }
        TimerSnapshot::from_state(&state.active)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn pause_timer(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut state = state.lock().unwrap();
        match &mut state.active {
            ActiveTimer::Basic(t) => t.pause(),
            ActiveTimer::Pomodoro(t) => t.pause(),
        }
        TimerSnapshot::from_state(&state.active)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn reset_timer(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut state = state.lock().unwrap();
        match &mut state.active {
            ActiveTimer::Basic(t) => t.reset(),
            ActiveTimer::Pomodoro(t) => t.reset(),
        }
        TimerSnapshot::from_state(&state.active)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn set_duration(app: AppHandle, state: State<'_, Mutex<AppState>>, secs: u32) {
    let snapshot = {
        let mut state = state.lock().unwrap();
        if let ActiveTimer::Basic(t) = &mut state.active {
            t.set_duration(secs);
            Some(TimerSnapshot::from_basic(t))
        } else {
            None
        }
    };
    if let Some(s) = snapshot {
        emit_and_update_tray(&app, s);
    }
}

#[tauri::command]
pub fn switch_to_basic(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut state = state.lock().unwrap();
        state.active = ActiveTimer::Basic(BasicTimer::new(25 * 60));
        TimerSnapshot::from_state(&state.active)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn switch_to_pomodoro(app: AppHandle, state: State<'_, Mutex<AppState>>) {
    let snapshot = {
        let mut state = state.lock().unwrap();
        state.active = ActiveTimer::Pomodoro(PomodoroTimer::new(PomodoroConfig::default()));
        TimerSnapshot::from_state(&state.active)
    };
    emit_and_update_tray(&app, snapshot);
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, Mutex<AppState>>) -> TimerSnapshot {
    let state = state.lock().unwrap();
    match &state.active {
        ActiveTimer::Basic(t) => TimerSnapshot::from_basic(t),
        ActiveTimer::Pomodoro(t) => TimerSnapshot::from_pomodoro(t),
    }
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
