#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::epaint::Vec2;
use eframe::{run_native, NativeOptions};

mod config;
mod vcupdate;
mod handling;
mod ui;
mod util;
mod toyops;

fn main() {

    // Native UI Options
    let mut native_opts = NativeOptions::default();
    native_opts.initial_window_size = Some(Vec2::new(450., 500.));

    run_native(
        "VibeCheck",
        native_opts,
        Box::new(|cc| Box::new(ui::VibeCheckGUI::new(config::config_load(), cc))));
}