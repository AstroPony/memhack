pub mod commands;
pub mod memory;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            // Process
            commands::list_processes,
            commands::attach_process,
            commands::detach_process,
            // Scanning
            commands::first_scan,
            commands::next_scan,
            commands::get_scan_results,
            commands::reset_scan,
            // Writing
            commands::write_value,
            commands::freeze_value,
            commands::unfreeze_value,
            // Tables
            commands::save_address_table,
            commands::load_address_table,
            commands::list_saved_tables,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MemHack");
}
