mod cards;
mod commands;
mod db;
mod error;
mod pm3;
mod state;

use std::sync::Mutex;

use commands::firmware::FlashState;
use pm3::connection::HfOperationState;
use state::WizardMachine;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let database =
                db::Database::open(data_dir).expect("failed to open database");
            app.manage(database);
            app.manage(Mutex::new(WizardMachine::new()));
            app.manage(FlashState::new());
            app.manage(HfOperationState::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::wizard::get_wizard_state,
            commands::wizard::wizard_action,
            commands::device::detect_device,
            commands::blank::detect_blank,
            commands::scan::scan_card,
            commands::write::write_clone,
            commands::write::write_clone_with_data,
            commands::write::verify_clone,
            commands::history::get_history,
            commands::history::save_clone_record,
            commands::firmware::check_firmware_version,
            commands::firmware::flash_firmware,
            commands::firmware::cancel_flash,
            commands::erase::detect_chip,
            commands::erase::wipe_chip,
            commands::saved::save_card,
            commands::saved::get_saved_cards,
            commands::saved::delete_saved_card,
            commands::raw::run_raw_command,
            commands::hf_clone::hf_autopwn,
            commands::hf_clone::hf_write_clone,
            commands::hf_clone::hf_dump,
            commands::hf_clone::hf_verify_clone,
            commands::hf_clone::cancel_hf_operation,
        ])
        .run(tauri::generate_context!())
        .expect("error running Phosphor");
}
