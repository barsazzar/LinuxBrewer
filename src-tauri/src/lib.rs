mod brew;
mod nvm;
mod stream;
mod tray;
mod types;

use types::{BrewState, NvmState};
use stream::CancelRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(BrewState::new())
        .manage(NvmState::new())
        .manage(CancelRegistry::new())
        .setup(tray::setup_tray)
        .invoke_handler(tauri::generate_handler![
            brew::brew_status,
            brew::brew_list_installed,
            brew::brew_list_pinned,
            brew::brew_outdated,
            brew::brew_search,
            brew::brew_tap_list,
            brew::set_brew_path,
            stream::brew_run_stream,
            stream::cancel_brew_stream,
            tray::update_tray,
            nvm::nvm_status,
            nvm::nvm_list_installed,
            nvm::nvm_lts_slots,
            nvm::nvm_refresh,
            nvm::nvm_reset_cache,
            nvm::nvm_which,
            nvm::nvm_fetch_remote_lts,
            nvm::nvm_fetch_all_versions,
            nvm::nvm_fetch_versions_for_major,
            nvm::nvm_set_default,
            nvm::nvm_read_project,
            nvm::nvm_write_project,
            nvm::nvm_run_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
