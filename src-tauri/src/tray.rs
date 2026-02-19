use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Rect,
};

use crate::commands::AppState;

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    TrayIconBuilder::with_id("main")
        .title("⏱ 25:00")
        .tooltip("Timer")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                toggle_window(tray.app_handle(), rect);
            }
        })
        .build(app)?;

    Ok(())
}

pub fn toggle_window(app: &AppHandle, tray_rect: Rect) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            position_window_below_tray(&window, tray_rect);
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn position_window_below_tray(
    window: &tauri::WebviewWindow,
    tray_rect: Rect,
) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let win_size = match window.outer_size() {
        Ok(s) => s,
        Err(_) => return,
    };

    let tray_pos = tray_rect.position.to_physical::<f64>(scale);
    let tray_size = tray_rect.size.to_physical::<f64>(scale);

    // Center the window horizontally under the tray icon
    let tray_center_x = tray_pos.x + tray_size.width / 2.0;
    let x = tray_center_x - win_size.width as f64 / 2.0;
    // Place window just below the tray icon
    let y = tray_pos.y + tray_size.height;

    let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
}

pub fn update_tray_title(app: &AppHandle) {
    let state = app.state::<Mutex<AppState>>();
    let title = {
        let state = state.lock().unwrap();
        match &state.active {
            crate::commands::ActiveTimer::Basic(t) => format!("⏱ {}", t.display()),
            crate::commands::ActiveTimer::Pomodoro(t) => t.tray_title(),
        }
    };

    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(&title));
    }
}
