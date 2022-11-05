#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

//use crate::util::load_icon;

mod config;
mod vcupdate;
mod handling;
mod ui;
mod util;
mod toyops;
mod lovense;

#[tauri::command]
fn vibecheck_version() -> String {
    vcupdate::VERSION.to_string()
}

fn main() {

    /*
    // Native UI Options
    let mut native_opts = NativeOptions::default();
    native_opts.initial_window_size = Some(Vec2::new(450., 500.));
    
    let icon_bytes = include_bytes!("../images/vibecheck-ico32x32.ico");
    native_opts.icon_data = Some(load_icon(icon_bytes.to_vec()));

    run_native(
        "VibeCheck",
        native_opts,
        Box::new(|cc| Box::new(ui::VibeCheckGUI::new(config::config_load(), cc))));
    */

    //let _js_handler = ;


    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![vibecheck_version])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

