#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{Manager, SystemTrayMenu};

mod config;
//mod vcupdate;
mod handling;
mod frontend_native;
mod vcore;
mod util;
mod toyops;
//mod lovense;
mod bluetooth;
mod frontend_types;

fn main() {

    //tracing_subscriber::fmt::init();

    let vibecheck_state_pointer = Arc::new(
        Mutex::new(
            vcore::VibeCheckState::new(
                config::config_load())));

    let quit = tauri::CustomMenuItem::new("quit".to_string(), "Quit");
    let restart = tauri::CustomMenuItem::new("restart".to_string(), "Restart");
    let hide_app = tauri::CustomMenuItem::new("hide".to_string(), "Hide");
    let show_app = tauri::CustomMenuItem::new("show".to_string(), "Show");
    let enable_osc = tauri::CustomMenuItem::new("enable_osc".to_string(), "Enable");
    let disable_osc = tauri::CustomMenuItem::new("disable_osc".to_string(), "Disable");

    let tray_menu = SystemTrayMenu::new()
    .add_item(enable_osc)
    .add_item(disable_osc)
    .add_native_item(tauri::SystemTrayMenuItem::Separator)
    .add_item(hide_app)
    .add_item(show_app)
    .add_native_item(tauri::SystemTrayMenuItem::Separator)
    .add_item(restart)
    .add_item(quit);

    let app = tauri::Builder::default()
    .system_tray(tauri::SystemTray::new().with_menu(tray_menu))
    .on_system_tray_event(|app, event| match event {
        tauri::SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "quit" => {
                    app.exit(0);
                },
                "restart" => {
                    app.restart();
                },
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                },
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                },
                "enable_osc" => {
                    let vc_lock = app.state::<vcore::VCStateMutex>();
                    let _ = vcore::native_vibecheck_enable(vc_lock);
                },
                "disable_osc" => {
                    let vc_lock = app.state::<vcore::VCStateMutex>();
                    let _ = tauri::async_runtime::block_on(async move {vcore::native_vibecheck_disable(vc_lock).await});
                }
                _ => {},
            }
        },
        _ => {}
    })
    .manage(
        vcore::VCStateMutex(vibecheck_state_pointer.clone()))
    .invoke_handler(
        tauri::generate_handler![
            frontend_native::vibecheck_version,
            frontend_native::vibecheck_enable,
            frontend_native::vibecheck_disable,
            frontend_native::get_vibecheck_config,
            frontend_native::set_vibecheck_config,
            frontend_native::vibecheck_start_bt_scan,
            frontend_native::vibecheck_stop_bt_scan,
            frontend_native::get_toys,
            frontend_native::alter_toy,
            frontend_native::get_connection_modes,
            ]
    )
    .build(tauri::generate_context!())
    .expect("Failed to generate Tauri context");

    let identifier = app.config().tauri.bundle.identifier.clone();
    let handling_pointer = vibecheck_state_pointer.clone();
    {
        let lock = vibecheck_state_pointer.lock();
        lock.async_rt.spawn(handling::message_handling(handling_pointer, identifier));
    }



    app.run(|_app_handle, event| {

        match event {
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

            /*
            let state = _app_handle.state::<vcore::VCStateMutex>();
            let vc_lock = state.0.lock();
            
            // Handle inter-thread data
            // Problem: This does not continuously execute (When app is hidden does not execute)
            handling::message_handling(vc_lock);*/
            //println!("[+] State MainEventsCleared.");
        },
        tauri::RunEvent::Ready => {
            println!("[+] App Ready");
        }
        _ => {}
    }});
}