#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use crate::{frontend::frontend_native, vcore::config};

use log::{info, trace, warn};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{
    menu::{CheckMenuItem, IconMenuItem, Menu, MenuBuilder, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_updater::UpdaterExt;

mod frontend;
mod osc;
mod osc_api;
mod toy_handling;
mod util;
mod vcore;

fn main() {
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

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            warn!(
                "Another {} process mutex created.. Showing already running app.",
                app.package_info().name
            );
            let window = app.get_webview_window("main").unwrap();
            window.show().unwrap();
        }))
        .setup(|app| {
            let handle = app.handle();
            let menu = MenuBuilder::new(handle)
                .quit()
                .items(&[&CheckMenuItem::new(
                    handle,
                    "Restart",
                    true,
                    true,
                    None::<&str>,
                )?])
                .hide()
                .show_all()
                .build()?;
            app.set_menu(menu);
            TrayIconBuilder::new().on_tray_icon_event(|app, event| {
                tauri_plugin_positioner::on_tray_event(app.app_handle(), &event);
            });
            app.updater();
            Ok(())
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

    // let identifier = app.config().tauri.bundle.identifier.clone();
    // info!("Got bundle id: {}", identifier);

    let vc_state_pointer = vibecheck_state_pointer.clone();
    {
        let mut vc_state = vibecheck_state_pointer.lock();
        vc_state.set_state_pointer(vc_state_pointer);
        trace!("State pointer set");
        vc_state.set_app_handle(app.handle().to_owned());
        trace!("App handle set");
        vc_state.init_toy_manager();
        trace!("ToyManager initialized");
        // vc_state.identifier = identifier;
        // trace!("App Identifier set");
        vc_state.start_tmh();
        trace!("Started TMH");
        vc_state.init_ceh();
        trace!("Started CEH");
        vc_state.start_disabled_listener();
        trace!("Started DOL");
    }

    app.run(|_app_handle, event| match event {
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
                let window = _app_handle.get_webview_window(&label).unwrap();
                trace!("Closing window: {}", window.label());
                window.hide().unwrap();
                api.prevent_close();
            }
        }
        tauri::RunEvent::ExitRequested { .. } => {}
        tauri::RunEvent::MainEventsCleared => {}
        tauri::RunEvent::Ready => {
            info!("App Ready");
        }
        _ => {}
    });
}
