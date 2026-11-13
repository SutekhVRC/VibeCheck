/*
 * Finds and clears VRChat's saved avatar OSC configs (Settings >
 * Troubleshooting > Reset VRChat OSC configs).
 *
 * VRChat keeps one folder per user, `usr_<id>`, inside its OSC folder, holding
 * a JSON config per avatar. Deleting the `usr_*` folders makes VRChat write
 * them again the next time an avatar loads. Nothing else is touched.
 *
 * Where the OSC folder is:
 * - Windows: `%USERPROFILE%\AppData\LocalLow\VRChat\VRChat\OSC`
 * - Linux: VRChat runs through Proton, so it's inside VRChat's Proton prefix
 *   (Steam app 438100), in whichever Steam library VRChat is installed in:
 *   `<library>/steamapps/compatdata/438100/pfx/drive_c/users/steamuser/AppData/LocalLow/VRChat/VRChat/OSC`.
 *   Steam can be a normal install, Flatpak or Snap, and every library it
 *   knows about is listed in `steamapps/libraryfolders.vdf`.
 */

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use log::{info, warn};

use crate::vcore::errors::backend::VibeCheckFSError;

/// VRChat's Steam app id
const VRCHAT_APP_ID: &str = "438100";

/// The OSC folder, relative to a Windows user profile
const OSC_IN_PROFILE: [&str; 5] = ["AppData", "LocalLow", "VRChat", "VRChat", "OSC"];

/// Where Steam keeps its own folder on Linux, relative to the home folder
const STEAM_ROOTS: [&str; 4] = [
    ".local/share/Steam",
    ".steam/steam",
    ".var/app/com.valvesoftware.Steam/.local/share/Steam",
    "snap/steam/common/.local/share/Steam",
];

/// Clears every VRChat OSC folder found. Returns the folders that were
/// cleared, or `VRChatOscNotFound` if there's none.
pub fn clear_osc_configs() -> Result<Vec<String>, VibeCheckFSError> {
    let home = directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .ok_or(VibeCheckFSError::ReadDirFailure)?;
    let dirs = osc_dirs(&home);
    if dirs.is_empty() {
        warn!("No VRChat OSC folder found");
        return Err(VibeCheckFSError::VRChatOscNotFound);
    }
    for dir in &dirs {
        clear_user_folders(dir)?;
    }
    Ok(dirs.iter().map(|d| d.display().to_string()).collect())
}

/// Every VRChat OSC folder that exists for this user
fn osc_dirs(home: &Path) -> Vec<PathBuf> {
    let candidates = if cfg!(target_os = "windows") {
        vec![OSC_IN_PROFILE
            .iter()
            .fold(home.to_path_buf(), |p, s| p.join(s))]
    } else {
        steam_libraries(home)
            .into_iter()
            .map(|library| proton_osc_dir(&library))
            .collect()
    };

    // The same folder can be reached through different links (~/.steam/steam
    // usually points at ~/.local/share/Steam), so keep each one once
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|dir| dir.is_dir())
        .filter(|dir| seen.insert(fs::canonicalize(dir).unwrap_or_else(|_| dir.clone())))
        .collect()
}

/// The OSC folder inside VRChat's Proton prefix in a Steam library
fn proton_osc_dir(library: &Path) -> PathBuf {
    let prefix = library
        .join("steamapps/compatdata")
        .join(VRCHAT_APP_ID)
        .join("pfx/drive_c/users/steamuser");
    OSC_IN_PROFILE.iter().fold(prefix, |p, s| p.join(s))
}

/// Every Steam library: each Steam folder itself, plus the libraries listed
/// in its libraryfolders.vdf
fn steam_libraries(home: &Path) -> Vec<PathBuf> {
    let mut libraries = Vec::new();
    for root in STEAM_ROOTS.iter().map(|r| home.join(r)) {
        if !root.is_dir() {
            continue;
        }
        libraries.push(root.clone());
        let vdf = root.join("steamapps/libraryfolders.vdf");
        if let Ok(text) = fs::read_to_string(&vdf) {
            libraries.extend(library_paths(&text).into_iter().map(PathBuf::from));
        }
    }
    libraries
}

/// The `"path"` entries of a libraryfolders.vdf
fn library_paths(vdf: &str) -> Vec<String> {
    vdf.lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("\"path\"")?;
            let value = rest.trim().strip_prefix('"')?.strip_suffix('"')?;
            Some(value.replace("\\\\", "\\"))
        })
        .collect()
}

/// Deletes the `usr_*` folders in an OSC folder. Links are skipped, so nothing
/// outside the OSC folder can be deleted.
fn clear_user_folders(osc_dir: &Path) -> Result<(), VibeCheckFSError> {
    let entries = fs::read_dir(osc_dir).map_err(|_| VibeCheckFSError::ReadDirFailure)?;
    for entry in entries {
        let entry = entry.map_err(|_| VibeCheckFSError::ReadDirPathFailure)?;
        let is_user_folder = entry.file_name().to_string_lossy().starts_with("usr_");
        let is_real_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_user_folder && is_real_dir {
            info!("Clearing {}", entry.path().display());
            fs::remove_dir_all(entry.path()).map_err(|_| VibeCheckFSError::RemoveDirsFailure)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, empty folder for one test
    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("vibecheck-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn reads_library_paths_from_libraryfolders_vdf() {
        let vdf = r#""libraryfolders"
{
	"0"
	{
		"path"		"/home/user/.local/share/Steam"
		"label"		""
	}
	"1"
	{
		"path"		"/mnt/games/SteamLibrary"
		"apps"
		{
			"438100"		"123"
		}
	}
}"#;
        assert_eq!(
            library_paths(vdf),
            vec!["/home/user/.local/share/Steam", "/mnt/games/SteamLibrary"]
        );
    }

    #[test]
    fn clears_only_user_folders() {
        let osc = scratch("clear");
        fs::create_dir_all(osc.join("usr_abc/Avatars")).unwrap();
        fs::write(osc.join("usr_abc/Avatars/avtr_1.json"), "{}").unwrap();
        fs::create_dir_all(osc.join("other")).unwrap();
        fs::write(osc.join("usr_file"), "not a folder").unwrap();

        clear_user_folders(&osc).unwrap();
        assert!(!osc.join("usr_abc").exists());
        assert!(osc.join("other").exists());
        assert!(osc.join("usr_file").exists());
        fs::remove_dir_all(&osc).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn finds_vrchat_in_the_main_and_extra_steam_libraries() {
        let home = scratch("find");
        let steam = home.join(".local/share/Steam");
        let extra = home.join("games/SteamLibrary");
        fs::create_dir_all(steam.join("steamapps")).unwrap();
        fs::write(
            steam.join("steamapps/libraryfolders.vdf"),
            format!(
                "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n}}\n",
                steam.display(),
                extra.display()
            ),
        )
        .unwrap();
        // VRChat is installed in the extra library
        fs::create_dir_all(proton_osc_dir(&extra)).unwrap();

        assert_eq!(osc_dirs(&home), vec![proton_osc_dir(&extra)]);

        // Nothing found anywhere else
        let empty = scratch("find-empty");
        assert!(osc_dirs(&empty).is_empty());
        fs::remove_dir_all(&home).unwrap();
        fs::remove_dir_all(&empty).unwrap();
    }
}
