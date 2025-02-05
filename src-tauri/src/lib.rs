use log::{info, warn};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use crate::{frontend::frontend_native, vcore::config};
//use env_logger;

mod frontend;
mod osc;
mod osc_api;
mod toy_handling;
mod util;
mod vcore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn run() {
    //tracing_subscriber::fmt::init();
    #[cfg(debug_assertions)]
    {
        let mut log_builder = env_logger::builder();
        log_builder.filter(None, log::LevelFilter::Trace);
        log_builder.init();
    }

    tauri::Builder::default()
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
            let window = app.get_webview_window("main").unwrap();
            window.show().unwrap();
        }))
        .setup(|app| {
            let toggle = MenuItemBuilder::with_id("toggle", "Toggle").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let restart = MenuItemBuilder::with_id("restart", "Restart").build(app)?;
            let hide_app = MenuItemBuilder::with_id("hide_app", "Hide App").build(app)?;
            let show_app = MenuItemBuilder::with_id("show_app", "Show App").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&toggle, &quit, &restart, &hide_app, &show_app])
                .build()?;
            let _ = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "toggle" => {
                        println!("toggle clicked");
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    "restart" => {
                        app.restart();
                    }
                    "hide_app" => {
                        app.get_webview_window("main").unwrap().hide().unwrap();
                    }
                    "show_app" => {
                        app.get_webview_window("main").unwrap().show().unwrap();
                    }
                    _ => (),
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(webview_window) = app.get_webview_window("main") {
                            let _ = webview_window.show();
                            let _ = webview_window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let identifier = app.config().identifier.clone();
            info!("Got bundle id: {}", identifier);

            Ok(())
        })
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
        .run(tauri::generate_context!())
        .expect("error while running vibecheck");
}
