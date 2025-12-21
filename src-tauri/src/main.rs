#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use log::{error as logerr, info, trace, warn};
use parking_lot::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

use crate::{
    frontend::frontend_native,
    vcore::config::{
        self,
        app::{config_load, VibeCheckConfig},
    },
};
//use env_logger;

mod error_signal_handler;
mod frontend;
mod osc;
mod osc_api;
mod toy_handling;
mod util;
mod vcore;

fn main() {
    #[cfg(debug_assertions)]
    {
        //tracing_subscriber::fmt::init();
        let mut log_builder = env_logger::builder();
        log_builder.filter(None, log::LevelFilter::Debug);
        log_builder.init();
    }

    let vibecheck_state_pointer = Arc::new(Mutex::new(vcore::state::VibeCheckState::new(
        VibeCheckConfig::default(),
    )));
    trace!("VibeCheckState created");

    let app = tauri::Builder::default()
        //.plugin(tauri_plugin_os::init())
        //.plugin(tauri_plugin_process::init())
        //.plugin(tauri_plugin_fs::init())
        //.plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        //.plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            warn!(
                "Another {} process mutex created.. Showing already running app.",
                app.package_info().name
            );
            let window = app
                .get_webview_window("main")
                .expect("Failed to get window main");
            window.show().expect("Failed to show window");
        }))
        .setup(|app| {
            // System Tray Initialization
            //let app_handle = app.handle();
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
            let restart = MenuItem::with_id(app, "restart", "Restart", true, None::<&str>).unwrap();
            let hide_app = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>).unwrap();
            let show_app = MenuItem::with_id(app, "show", "Show", true, None::<&str>).unwrap();
            let menu = Menu::with_items(app, &[&quit, &restart, &hide_app, &show_app]).unwrap();

            TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "restart" => {
                        app.restart();
                    }
                    "hide" => {
                        let window = app
                            .get_webview_window("main")
                            .expect("Failed to get window main");
                        window.hide().expect("Failed to hide window");
                    }
                    "show" => {
                        let window = app
                            .get_webview_window("main")
                            .expect("Failed to get window main");
                        window.show().expect("Failed to show window");
                    }
                    _ => {}
                })
                .build(app)
                .unwrap();

            Ok(())
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

    let identifier = app.config().identifier.clone();
    info!("Got bundle id: {}", identifier);

    let vc_state_pointer = vibecheck_state_pointer.clone();
    {
        let mut vc_state = vibecheck_state_pointer.lock();

        vc_state.config = match config_load(app.app_handle()) {
            Ok(config_dir) => config_dir,
            Err(e) => {
                logerr!("Failed config_load(): {:?}", e);
                std::process::exit(-1);
            }
        };

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

        vc_state.start_tmh().unwrap();
        trace!("Started TMH");
        vc_state.init_ceh().unwrap();
        trace!("Started CEH");
        vc_state.start_disabled_listener().unwrap();
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
                        .state::<vcore::state::VCStateMutex>()
                        .0
                        .lock()
                        .config
                        .minimize_on_exit
                };

                if minimize_on_exit {
                    let window = _app_handle
                        .get_webview_window(&label)
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
            _ => {}
        }
    });
}
