use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri::webview::WebviewWindowBuilder;
use tokio::time::{interval, Duration};

use crate::commands::{ActiveTimer, AppState, TimerSnapshot};
use crate::tray::update_tray_title;

#[derive(Clone, serde::Serialize)]
struct PhaseChangePayload {
    from: String,
    to: String,
}

pub fn start_tick_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut tick_interval = interval(Duration::from_secs(1));
        loop {
            tick_interval.tick().await;
            tick_once(&app);
        }
    });
}

fn tick_once(app: &AppHandle) {
    let state = app.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();

    match &mut state.active {
        ActiveTimer::Basic(timer) => {
            let was_finished = timer.is_finished();
            timer.tick();
            let snapshot = TimerSnapshot::from_basic(timer);
            let _ = app.emit("timer:tick", &snapshot);
            if timer.is_finished() && !was_finished {
                let _ = app.emit(
                    "timer:phase-change",
                    PhaseChangePayload {
                        from: "timer".to_string(),
                        to: "finished".to_string(),
                    },
                );
                open_notification_window(app, "timer", "finished");
            }
        }
        ActiveTimer::Pomodoro(timer) => {
            let transition = timer.tick();
            let snapshot = TimerSnapshot::from_pomodoro(timer);
            let _ = app.emit("timer:tick", &snapshot);
            if let Some(t) = transition {
                let from = format!("{:?}", t.from);
                let to = format!("{:?}", t.to);
                let _ = app.emit(
                    "timer:phase-change",
                    PhaseChangePayload {
                        from: from.clone(),
                        to: to.clone(),
                    },
                );
                open_notification_window(app, &from, &to);
            }
        }
    }

    drop(state);
    update_tray_title(app);
}

fn open_notification_window(app: &AppHandle, from: &str, to: &str) {
    // Close existing notification window so a fresh one opens with the new params
    if let Some(win) = app.get_webview_window("notification") {
        let _ = win.close();
    }

    let path = format!("notification.html?from={}&to={}", from, to);
    let url = tauri::WebviewUrl::App(path.into());
    let builder = WebviewWindowBuilder::new(app, "notification", url)
        .title("Notification")
        .inner_size(300.0, 120.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .center()
        .resizable(false);

    match builder.build() {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to create notification window: {}", e),
    }
}
