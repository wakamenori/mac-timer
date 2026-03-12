use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Rect,
};

use crate::commands::{ActiveTimer, AppState};
use crate::pomodoro::Phase;

/// Tracks whether the tray menu currently includes the "Show Overlay" item.
static TRAY_HAS_OVERLAY_ITEM: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayIcon {
    Timer,
    Tomato,
    Coffee,
}

impl TrayIcon {
    fn bytes(self) -> &'static [u8] {
        match self {
            TrayIcon::Timer => include_bytes!("../icons/tray-timer.png"),
            TrayIcon::Tomato => include_bytes!("../icons/tray-tomato.png"),
            TrayIcon::Coffee => include_bytes!("../icons/tray-coffee.png"),
        }
    }
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    let icon = Image::from_bytes(TrayIcon::Timer.bytes())?;

    TrayIconBuilder::with_id("main")
        .icon(icon)
        .icon_as_template(true)
        .title("25:00")
        .tooltip("Timer")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id() == "quit" {
                app.exit(0);
            } else if event.id() == "show_overlay" {
                show_overlay_from_tray(app);
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

fn position_window_below_tray(window: &tauri::WebviewWindow, tray_rect: Rect) {
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
    let (title, icon, is_break) = {
        let state = state.lock().unwrap();
        match &state.active {
            ActiveTimer::Basic(t) => (t.display(), TrayIcon::Timer, false),
            ActiveTimer::Pomodoro(t) => {
                let is_break = matches!(t.phase(), Phase::ShortBreak | Phase::LongBreak);
                (t.display(), t.tray_icon(), is_break)
            }
        }
    };

    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(&title));
        if let Ok(img) = Image::from_bytes(icon.bytes()) {
            let _ = tray.set_icon(Some(img));
        }

        // Show "オーバーレイを表示" only when in break AND overlay is not open
        let overlay_exists = app
            .webview_windows()
            .keys()
            .any(|label| label.starts_with("overlay-"));
        let should_show_item = is_break && !overlay_exists;
        let had_item = TRAY_HAS_OVERLAY_ITEM.load(Ordering::Relaxed);
        if should_show_item != had_item {
            TRAY_HAS_OVERLAY_ITEM.store(should_show_item, Ordering::Relaxed);
            if let Ok(menu) = build_tray_menu(app, should_show_item) {
                let _ = tray.set_menu(Some(menu));
            }
        }
    }
}

fn build_tray_menu(app: &AppHandle, include_show_overlay: bool) -> tauri::Result<Menu<tauri::Wry>> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    if include_show_overlay {
        let show_overlay =
            MenuItem::with_id(app, "show_overlay", "オーバーレイを表示", true, None::<&str>)?;
        Menu::with_items(app, &[&show_overlay, &quit])
    } else {
        Menu::with_items(app, &[&quit])
    }
}

fn show_overlay_from_tray(app: &AppHandle) {
    // Check if overlay windows already exist
    let windows = app.webview_windows();
    let overlay_exists = windows.keys().any(|label| label.starts_with("overlay-"));
    if overlay_exists {
        return;
    }

    // Read current break phase from state
    let break_phase = {
        let state = app.state::<Mutex<AppState>>();
        let state = state.lock().unwrap();
        match &state.active {
            ActiveTimer::Pomodoro(t) => match t.phase() {
                Phase::ShortBreak => Some("ShortBreak"),
                Phase::LongBreak => Some("LongBreak"),
                Phase::Work => None,
            },
            ActiveTimer::Basic(_) => None,
        }
    };

    if let Some(phase) = break_phase {
        crate::runner::open_overlay_windows(app, phase);
    }
}
