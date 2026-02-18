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
                open_notification_window(app);
            }
        }
        ActiveTimer::Pomodoro(timer) => {
            let transition = timer.tick();
            let snapshot = TimerSnapshot::from_pomodoro(timer);
            let _ = app.emit("timer:tick", &snapshot);
            if let Some(t) = transition {
                let _ = app.emit(
                    "timer:phase-change",
                    PhaseChangePayload {
                        from: format!("{:?}", t.from),
                        to: format!("{:?}", t.to),
                    },
                );
                open_notification_window(app);
            }
        }
    }

    drop(state);
    update_tray_title(app);
}

fn open_notification_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("notification") {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }

    let url = tauri::WebviewUrl::App("notification.html".into());
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
