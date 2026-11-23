use crate::util::errors::UtilError;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

/// Renames a settings file that couldn't be read to `<path>.bad`
/// (replacing an older one), so it's kept for the user instead of being
/// overwritten with defaults. Returns the new path.
pub fn set_aside_bad_file(path: &Path) -> std::io::Result<PathBuf> {
    let mut bad = path.as_os_str().to_owned();
    bad.push(".bad");
    let bad = PathBuf::from(bad);
    std::fs::rename(path, &bad)?;
    Ok(bad)
}

pub fn path_exists(p: &String) -> bool {
    Path::new(&p).is_dir()
}

pub fn file_exists<P>(p: &P) -> bool
where
    P: AsRef<OsStr> + ?Sized,
{
    Path::new(&p).is_file()
}

pub fn get_config_dir(app_handle: &AppHandle) -> Result<String, UtilError> {
    let pb = match app_handle.path().app_config_dir() {
        Ok(path) => path,
        Err(_) => return Err(UtilError::ConfigDirFS),
    };

    match pb.to_str() {
        Some(config_dir) => Ok(config_dir.to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_file_is_kept_next_to_the_original() {
        let dir = std::env::temp_dir().join(format!("vibecheck-bad-file-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Config.json");

        std::fs::write(dir.join("Config.json.bad"), "older").unwrap();
        std::fs::write(&path, "not json").unwrap();

        let bad = set_aside_bad_file(&path).unwrap();
        assert_eq!(bad, dir.join("Config.json.bad"));
        assert!(!path.exists());
        assert_eq!(std::fs::read_to_string(&bad).unwrap(), "not json");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
