use directories::BaseDirs;
use std::{ffi::OsStr, path::Path};
use tauri::{AppHandle, Manager};

use crate::util::errors::UtilError;

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
pub fn get_user_home_dir() -> Result<String, UtilError> {
    let bd = match BaseDirs::new() {
        Some(bd) => bd,
        None => return Err(UtilError::HomeDirFS),
    };

    let bd = match bd.home_dir().to_str() {
        Some(bd) => bd,
        None => return Err(UtilError::HomeDirFS),
    };

    Ok(bd.to_string())
}

pub fn get_config_dir(app_handle: &AppHandle) -> Result<String, UtilError> {
    let pb = match app_handle.path().app_data_dir() {
        Ok(path) => path,
        Err(_) => return Err(UtilError::ConfigDirFS),
    };
    match pb.to_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(UtilError::ConfigDirFS),
    }
}

pub fn build_path_dir(path: &[&str]) -> String {
    #[cfg(target_os = "linux")]
    {
        format!("{}/{}/", path[0], path[1])
    }
    #[cfg(target_os = "windows")]
    {
        format!("{}\\{}\\", path[0], path[1])
    }
}

pub fn build_path_file(path: &[&str]) -> String {
    #[cfg(target_os = "linux")]
    {
        format!("{}/{}", path[0], path[1])
    }
    #[cfg(target_os = "windows")]
    {
        format!("{}\\{}", path[0], path[1])
    }
}
