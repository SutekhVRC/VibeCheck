/*
 * Starts the webview fresh whenever VibeCheck runs as a different version
 * than last time (an update, a downgrade, or switching builds).
 *
 * The webview keeps its cache and local storage in the app's local data
 * folder (Tauri's app_local_data_dir):
 * - Linux: ~/.local/share/com.vibecheck (WebKitGTK cache and storage)
 * - Windows: %LOCALAPPDATA%\com.vibecheck (WebView2's EBWebView folder)
 *
 * Old cached pages and old local storage could clash with the new version's
 * frontend, so on a version change everything in that folder is deleted
 * before the window opens. Local storage only holds frontend preferences
 * (theme, Turn on when scanning, sidebar collapsed), which go back to their
 * defaults. VibeCheck's config and toy configs are in the config folder and
 * aren't touched.
 *
 * The version that last ran is kept in a small file in the same folder. If
 * it's missing but the folder has data, the data came from a version before
 * this check existed, so it's cleared too.
 *
 * Failures are logged and ignored: this must never stop the app starting.
 */

use std::fs;
use std::path::{Path, PathBuf};

use log::{info, warn};

/// Records the version that last ran
const VERSION_FILE: &str = "vibecheck-version";

/// Clears the webview's data if `version` differs from the last run. Call
/// before any window is created.
pub fn reset_on_version_change(identifier: &str, version: &str) {
    let Some(dir) = webview_data_dir(identifier) else {
        warn!("Couldn't find the webview data folder; not checking for a version change");
        return;
    };
    match reset_if_changed(&dir, version) {
        Ok(true) => info!(
            "VibeCheck is now {}: cleared the webview's data in {}",
            version,
            dir.display()
        ),
        Ok(false) => {}
        Err(e) => warn!(
            "Couldn't reset the webview's data in {}: {}",
            dir.display(),
            e
        ),
    }
}

/// The folder Tauri gives the webview: the app's local data folder
fn webview_data_dir(identifier: &str) -> Option<PathBuf> {
    directories::BaseDirs::new().map(|dirs| dirs.data_local_dir().join(identifier))
}

/// Empties `dir` and records `version` if the last run was a different
/// version. Returns whether anything was cleared.
fn reset_if_changed(dir: &Path, version: &str) -> std::io::Result<bool> {
    let version_file = dir.join(VERSION_FILE);
    let last = fs::read_to_string(&version_file).ok();
    if last.as_deref().map(str::trim) == Some(version) {
        return Ok(false);
    }

    let mut cleared = false;
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            // Folders are removed with their contents; links are removed
            // themselves, never followed
            if entry.file_type()?.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
            // The old version file doesn't count as data
            cleared |= path != version_file;
        }
    }

    fs::create_dir_all(dir)?;
    fs::write(&version_file, version)?;
    Ok(cleared)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vibecheck-test-webview-{}-{}",
            name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn first_run_only_records_the_version() {
        let dir = scratch("first");
        assert!(!reset_if_changed(&dir, "0.6.5").unwrap());
        assert_eq!(fs::read_to_string(dir.join(VERSION_FILE)).unwrap(), "0.6.5");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn same_version_keeps_everything() {
        let dir = scratch("same");
        reset_if_changed(&dir, "0.6.5").unwrap();
        fs::create_dir_all(dir.join("localstorage")).unwrap();
        assert!(!reset_if_changed(&dir, "0.6.5").unwrap());
        assert!(dir.join("localstorage").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_new_version_clears_cache_and_storage() {
        let dir = scratch("new");
        reset_if_changed(&dir, "0.6.5").unwrap();
        fs::create_dir_all(dir.join("WebKitCache/Version 16")).unwrap();
        fs::create_dir_all(dir.join("localstorage")).unwrap();
        fs::write(dir.join("cookies"), "").unwrap();

        assert!(reset_if_changed(&dir, "0.6.6").unwrap());
        let left: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(left, vec![std::ffi::OsString::from(VERSION_FILE)]);
        assert_eq!(fs::read_to_string(dir.join(VERSION_FILE)).unwrap(), "0.6.6");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn data_from_before_this_check_is_cleared() {
        // Upgrading from a version that never wrote the version file
        let dir = scratch("old");
        fs::create_dir_all(dir.join("EBWebView/Default/Local Storage")).unwrap();
        assert!(reset_if_changed(&dir, "0.6.5").unwrap());
        assert!(!dir.join("EBWebView").exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
