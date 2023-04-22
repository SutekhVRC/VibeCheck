use std::path::Path;
use directories::BaseDirs;
use tauri::{api::path::{resolve_path, BaseDirectory}, Env};

pub fn path_exists(p: &String) -> bool {
    Path::new(&p).is_dir()
}

pub fn file_exists(p: &String) -> bool {
    Path::new(&p).is_file()
}

/*
 * Old method for config path
 * Still used for clearing OSC avatar configs
*/
pub fn get_user_home_dir() -> String {
    let bd = BaseDirs::new().expect("[-] Could not get user's directories.");
    let bd = bd
        .home_dir()
        .to_str()
        .expect("[-] Failed to get user's home directory.");
    bd.to_string()
}

pub fn get_config_dir() -> String {
    let context_gen = tauri::generate_context!();
    resolve_path(
        context_gen.config(),
        context_gen.package_info(),
        &Env::default(),
        "VibeCheck",
        Some(BaseDirectory::AppConfig),
    ).unwrap().to_str().unwrap().to_string()
}