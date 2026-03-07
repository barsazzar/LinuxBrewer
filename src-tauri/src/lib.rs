mod brew;
mod stream;
mod tray;
mod types;

use types::BrewState;
use stream::CancelRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(BrewState::new())
        .manage(CancelRegistry::new())
        .setup(tray::setup_tray)
        .invoke_handler(tauri::generate_handler![
            brew::brew_status,
            brew::brew_list_installed,
            brew::brew_outdated,
            brew::brew_install,
            brew::brew_uninstall,
            brew::brew_upgrade_all,
            brew::brew_upgrade_single,
            brew::brew_search,
            brew::brew_tap_list,
            brew::brew_info,
            brew::brew_doctor,
            brew::set_brew_path,
            stream::brew_run_stream,
            stream::cancel_brew_stream,
            tray::update_tray,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
