use directories::BaseDirs;
use directories::ProjectDirs;
use std::{ffi::OsStr, path::Path};

pub fn path_exists(p: &String) -> bool {
    Path::new(&p).is_dir()
}

pub fn file_exists<P>(p: &P) -> bool
where
    P: AsRef<OsStr> + ?Sized,
{
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
    return ProjectDirs::from("com", "vibecheck", "Vibecheck")
        .unwrap()
        .config_dir()
        .to_str()
        .unwrap()
        .to_string();
}
