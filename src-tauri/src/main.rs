#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use parking_lot::Mutex;
use tauri::Manager;

mod config;
mod vcupdate;
mod handling;
mod frontend_native;
mod vcore;
mod util;
mod toyops;
mod lovense;

fn main() {

    let app = tauri::Builder::default()
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
            frontend_native::vibecheck_start_bt_scan,
            frontend_native::vibecheck_stop_bt_scan,
            ]
    )
    .build(tauri::generate_context!())
    .expect("Failed to generate Tauri context");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { .. } => {
            println!("Exit Request!");
        },
        tauri::RunEvent::MainEventsCleared => {
            let state = _app_handle.state::<vcore::VCStateMutex>();
            let vc_lock = state.0.lock();
            
            // Handle inter-thread data
            vcore::message_handling(vc_lock);
        },
        _ => {}
    });
}