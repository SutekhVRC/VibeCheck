#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use parking_lot::Mutex;
use tauri::{Manager, SystemTrayMenu};

mod config;
mod vcupdate;
mod handling;
mod frontend_native;
mod vcore;
mod util;
mod toyops;
mod lovense;

fn main() {

    let quit = tauri::CustomMenuItem::new("quit".to_string(), "Quit");
    let hide_app = tauri::CustomMenuItem::new("hide".to_string(), "Hide");
    let show_app = tauri::CustomMenuItem::new("show".to_string(), "Show");

    let tray_menu = SystemTrayMenu::new()
    .add_item(hide_app)
    .add_item(show_app)
    .add_native_item(tauri::SystemTrayMenuItem::Separator)
    .add_item(quit);

    let app = tauri::Builder::default()
    .system_tray(tauri::SystemTray::new().with_menu(tray_menu))
    .on_system_tray_event(|app, event| match event {
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "quit" => {
                    app.exit(0);
                },
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                },
                "show" => {
                    app.windows().iter().for_each(|w| {
                        println!("{}", w.0);
                    });
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                }
                _ => {},
            }
        },
        _ => {}
    })
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
        tauri::RunEvent::WindowEvent { label, event, .. } => {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let window = _app_handle.get_window(&label).unwrap();
                    window.hide().unwrap();
                    api.prevent_close();
                }, _ => {}
            }
        },
        tauri::RunEvent::ExitRequested { .. } => {
            // On exit
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