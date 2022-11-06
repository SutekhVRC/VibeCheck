#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

//use crate::util::load_icon;

use config::config_load;

use parking_lot::Mutex;

mod config;
mod vcupdate;
mod handling;
mod frontend_native;
mod vcore;
mod util;
mod toyops;
mod lovense;

fn main() {

    tauri::Builder::default()
    .manage(
        vcore::VCStateMutex(
            Mutex::new(
                vcore::VibeCheckState::new(
                    config::config_load()
    ))))
    .invoke_handler(
        tauri::generate_handler![
            frontend_native::vibecheck_version,
            frontend_native::vibecheck_enable,
            frontend_native::vibecheck_disable,
            frontend_native::get_vibecheck_config,
            ]
    )
    .run(tauri::generate_context!())
    .expect("Failed to generate Tauri context");
}