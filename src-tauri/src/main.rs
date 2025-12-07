#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use log::{error as logerr, info, trace, warn};
use parking_lot::Mutex;
use tauri::{Manager, SystemTrayMenu};

use crate::{
    frontend::frontend_native,
    vcore::config::{self, app::config_load},
};
//use env_logger;

mod error_signal_handler;
mod frontend;
mod mock;
mod osc;
mod osc_api;
mod toy_handling;
mod util;
mod vcore;

fn main() {
    #[cfg(debug_assertions)]
    let mock_toys_enabled = std::env::var("VC_MOCK_TOYS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    #[cfg(not(debug_assertions))]
    let mock_toys_enabled = false;

    #[cfg(debug_assertions)]
    {
        //tracing_subscriber::fmt::init();
        let mut log_builder = env_logger::builder();
        log_builder.filter(None, log::LevelFilter::Trace);
        log_builder.init();
    }

    let vibecheck_config = match config_load() {
        Ok(config_dir) => config_dir,
        Err(e) => {
            logerr!("Failed config_load(): {:?}", e);
            return;
        }
    };

    let vibecheck_state_pointer = Arc::new(Mutex::new(vcore::state::VibeCheckState::new(
        vibecheck_config,
        mock_toys_enabled,
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
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            warn!(
                "Another {} process mutex created.. Showing already running app.",
                app.package_info().name
            );
            let window = app.get_window("main").expect("Failed to get window main");
            window.show().expect("Failed to show window");
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
                    let window = app.get_window("main").expect("Failed to get window main");
                    window.hide().expect("Failed to hide window");
                }
                "show" => {
                    let window = app.get_window("main").expect("Failed to get window main");
                    window.show().expect("Failed to show window");
                }
                _ => {}
            },
            tauri::SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").expect("Failed to get window main");
                trace!("Opening window: {}", window.label());
                window.show().expect("Failed to show window");
            }
            _ => {}
        })
        .manage(vcore::state::VCStateMutex(vibecheck_state_pointer.clone()))
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
        vc_state.global_msg_handler_start().unwrap();
        trace!("Global message handler started");
        vc_state.init_toy_manager().unwrap();
        trace!("ToyManager initialized");
        vc_state.identifier = identifier;
        trace!("App Identifier set");
        if mock_toys_enabled {
            info!("Mock toy mode enabled; skipping hardware initialization");
        } else {
            vc_state.start_tmh().unwrap();
            trace!("Started TMH");
            vc_state.init_ceh().unwrap();
            trace!("Started CEH");
            vc_state.start_disabled_listener().unwrap();
            trace!("Started DOL");
        }
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
                        .state::<vcore::state::VCStateMutex>()
                        .0
                        .lock()
                        .config
                        .minimize_on_exit
                };

                if minimize_on_exit {
                    let window = _app_handle
                        .get_window(&label)
                        .expect("Failed to get window to minimize");
                    trace!("Closing window: {}", window.label());
                    window.hide().expect("Failed to hide window for minimize");
                    api.prevent_close();
                } else {
                    // Let exit
                }
            }
            tauri::RunEvent::ExitRequested { .. } => {
                // On exit
            }
            tauri::RunEvent::MainEventsCleared => {}
            tauri::RunEvent::Ready => {
                info!("App Ready");
            }
            tauri::RunEvent::Updater(updater_event) => match updater_event {
                tauri::UpdaterEvent::Error(err) => {
                    log::error!("Update error: {}", err);
                }
                tauri::UpdaterEvent::UpdateAvailable {
                    body: _,
                    date: _,
                    version,
                } => {
                    info!("Update available: {}", version);
                }
                _ => {}
            },
            _ => {}
        }
    });
}
