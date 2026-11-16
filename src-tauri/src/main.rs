#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    vibecheck_lib::run(
        tauri::generate_context!(),
        vibecheck_lib::Extensions::default(),
    );
}
