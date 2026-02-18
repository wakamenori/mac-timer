pub mod commands;
pub mod pomodoro;
pub mod runner;
pub mod timer;
pub mod tray;

use std::sync::Mutex;

use commands::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::start_timer,
            commands::pause_timer,
            commands::reset_timer,
            commands::set_duration,
            commands::switch_to_basic,
            commands::switch_to_pomodoro,
            commands::get_snapshot,
            commands::toggle_always_on_top,
        ])
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            runner::start_tick_loop(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
