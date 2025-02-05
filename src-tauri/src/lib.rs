#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use log::{info, trace, warn};
use parking_lot::Mutex;
use tauri::{Manager, SystemTrayMenu};

use crate::{frontend::frontend_native, vcore::config};
//use env_logger;

mod frontend;
mod osc;
mod osc_api;
mod toy_handling;
mod util;
mod vcore;

fn run() {
    //tracing_subscriber::fmt::init();
    #[cfg(debug_assertions)]
    {
        let mut log_builder = env_logger::builder();
        log_builder.filter(None, log::LevelFilter::Trace);
        log_builder.init();
    }

    let vibecheck_state_pointer = Arc::new(Mutex::new(vcore::core::VibeCheckState::new(
        config::config_load(),
    )));
    trace!("VibeCheckState created");

    let quit = tauri::CustomMenuItem::new("quit".to_string(), "Quit");
    let restart = tauri::CustomMenuItem::new("restart".to_string(), "Restart");
    let hide_app = tauri::CustomMenuItem::new("hide".to_string(), "Hide");
    let show_app = tauri::CustomMenuItem::new("show".to_string(), "Show");
    //let enable_osc = tauri::CustomMenuItem::new("enable_osc".to_string(), "Enable");
    //let disable_osc = tauri::CustomMenuItem::new("disable_osc".to_string(), "Disable");

    let tray_menu = SystemTrayMenu::new()
        //.add_item(enable_osc)
        //.add_item(disable_osc)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(hide_app)
        .add_item(show_app)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(restart)
        .add_item(quit);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            warn!(
                "Another {} process mutex created.. Showing already running app.",
                app.package_info().name
            );
            let window = app.get_window("main").unwrap();
            window.show().unwrap();
        }))
        .setup(|_app| Ok(()))
        .system_tray(tauri::SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(|app, event| match event {
            tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    app.exit(0);
                }
                "restart" => {
                    app.restart();
                }
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            tauri::SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                trace!("Opening window: {}", window.label());
                window.show().unwrap();
            }
            _ => {}
        })
        .manage(vcore::core::VCStateMutex(vibecheck_state_pointer.clone()))
        .invoke_handler(tauri::generate_handler![
            frontend_native::vibecheck_version,
            frontend_native::vibecheck_enable,
            frontend_native::vibecheck_disable,
            frontend_native::get_vibecheck_config,
            frontend_native::set_vibecheck_config,
            frontend_native::vibecheck_start_bt_scan,
            frontend_native::vibecheck_stop_bt_scan,
            frontend_native::alter_toy,
            frontend_native::open_default_browser,
            frontend_native::clear_osc_config,
            frontend_native::simulate_device_feature,
            frontend_native::sync_offline_toys,
            frontend_native::osc_query_start,
            frontend_native::osc_query_stop,
            frontend_native::osc_query_attempt_force_connect,
            //frontend_native::simulate_feature_osc_input,
        ])
        .build(tauri::generate_context!())
        .expect("Failed to generate Tauri context");
    trace!("Tauri app built");

    let identifier = app.config().tauri.bundle.identifier.clone();
    info!("Got bundle id: {}", identifier);

    let vc_state_pointer = vibecheck_state_pointer.clone();
    {
        let mut vc_state = vibecheck_state_pointer.lock();
        vc_state.set_state_pointer(vc_state_pointer);
        trace!("State pointer set");
        vc_state.set_app_handle(app.app_handle());
        trace!("App handle set");
        vc_state.init_toy_manager();
        trace!("ToyManager initialized");
        vc_state.identifier = identifier;
        trace!("App Identifier set");
        vc_state.start_tmh();
        trace!("Started TMH");
        vc_state.init_ceh();
        trace!("Started CEH");
        vc_state.start_disabled_listener();
        trace!("Started DOL");
    }

    app.run(|_app_handle, event| {
        match event {
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } => {
                let minimize_on_exit = {
                    _app_handle
                        .state::<vcore::core::VCStateMutex>()
                        .0
                        .lock()
                        .config
                        .minimize_on_exit
                };

                if minimize_on_exit {
                    let window = _app_handle.get_window(&label).unwrap();
                    trace!("Closing window: {}", window.label());
                    window.hide().unwrap();
                    api.prevent_close();
                } else {
                    // Let exit
                }
            }
            tauri::RunEvent::ExitRequested { .. } => {
                // On exit
            }
            tauri::RunEvent::MainEventsCleared => {

                /*
                let state = _app_handle.state::<vcore::VCStateMutex>();
                let vc_lock = state.0.lock();

                // Handle inter-thread data
                // Problem: This does not continuously execute (When app is hidden does not execute)
                handling::message_handling(vc_lock);*/
                //info!("[+] State MainEventsCleared.");
            }
            tauri::RunEvent::Ready => {
                info!("App Ready");

                // Sync offline toys to frontend
                //_app_handle.state::<vcore::VCStateMutex>().0.lock().core_toy_manager.as_ref().unwrap().sync_frontend();
            }
            _ => {}
        }
    });
}
