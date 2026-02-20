use std::sync::Mutex;
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Emitter, Manager};
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
    // Work → Break transitions get a fullscreen overlay instead of a small notification
    if from == "Work" && (to == "ShortBreak" || to == "LongBreak") {
        open_overlay_windows(app, to);
        return;
    }

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

fn open_overlay_windows(app: &AppHandle, to: &str) {
    // Close any existing overlay windows first
    close_overlay_windows(app);

    let monitors = match app.available_monitors() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to enumerate monitors: {}", e);
            return;
        }
    };

    for (i, monitor) in monitors.iter().enumerate() {
        let label = format!("overlay-{}", i);
        let path = format!("overlay.html?to={}", to);
        let url = tauri::WebviewUrl::App(path.into());

        let pos = monitor.position();
        let size = monitor.size();
        let scale = monitor.scale_factor();

        let logical_width = size.width as f64 / scale;
        let logical_height = size.height as f64 / scale;
        let logical_x = pos.x as f64 / scale;
        let logical_y = pos.y as f64 / scale;

        let builder = WebviewWindowBuilder::new(app, &label, url)
            .title("Break Overlay")
            .position(logical_x, logical_y)
            .inner_size(logical_width, logical_height)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false);

        match builder.build() {
            Ok(_) => {}
            Err(e) => eprintln!("Failed to create overlay window {}: {}", label, e),
        }
    }
}

pub fn close_overlay_windows(app: &AppHandle) {
    let windows = app.webview_windows();
    for (label, win) in &windows {
        if label.starts_with("overlay-") {
            let _ = win.close();
        }
    }
}
