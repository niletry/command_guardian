mod commands;
mod models;
mod state;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::new(app.handle()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_task,
            commands::get_tasks,
            commands::delete_task,
            commands::start_task,
            commands::stop_task,
            commands::update_task,
            commands::resize_pty,
            commands::write_to_pty,
            commands::get_log_history,
            commands::clear_log_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
