use crate::util::toy_service::{SavedAddress, ToyConnection};
use log::{error as logerr, info, warn};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tauri::AppHandle;

use crate::{
    error_signal_handler::reporter::ErrorReporter,
    osc::OSCNetworking,
    util::fs::{
        build_path_dir, build_path_file, file_exists, get_config_dir, path_exists,
        set_aside_bad_file,
    },
    vcore::errors::backend::VibeCheckConfigError,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VibeCheckConfig {
    // Change networking to an enum between OSCQuery and setting bind and remote.
    pub networking: OSCNetworking,
    pub scan_on_disconnect: bool,
    pub minimize_on_exit: bool,
    pub desktop_notifications: bool,
    pub show_toy_advanced: bool,
    pub show_feature_advanced: bool,
    /// Master intensity, 0.0 to 1.0: scales every toy's level. Defaults to full
    /// intensity, including for config files saved before it existed. Also
    /// reads `master_volume`, its name in early development builds.
    #[serde(default = "full_intensity", alias = "master_volume")]
    pub master_intensity: f64,
    /// How long a Bluetooth scan runs before it stops itself, in seconds. 0
    /// scans until stopped. Defaults to 30 s, including for config files
    /// saved before it existed.
    #[serde(default = "default_scan_length")]
    pub scan_length: u32,
    /// The app's theme id. An id that doesn't exist (e.g. a removed theme)
    /// shows the default theme. Missing in older config files.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Pressing the scan button turns VibeCheck on first
    #[serde(default)]
    pub auto_enable_on_scan: bool,
    /// The sidebar is collapsed to icons
    #[serde(default)]
    pub sidebar_collapsed: bool,
    /// Where toys are reached: VibeCheck's own Bluetooth, or Intiface
    /// Central (util/toy_service.rs). Local in older config files.
    #[serde(default)]
    pub toy_connection: ToyConnection,
    /// Intiface addresses that connected before, newest first, with the
    /// names the user gave them (util/toy_service.rs `remember`). Kept by
    /// Reset VibeCheck settings. Empty in older config files.
    #[serde(default)]
    pub intiface_addresses: Vec<SavedAddress>,
}

/// The frontend's DEFAULT_THEME_ID
fn default_theme() -> String {
    "vibecheck".to_owned()
}

fn full_intensity() -> f64 {
    1.0
}

pub const DEFAULT_SCAN_LENGTH: u32 = 30;
/// Longest scan length that can be set, in seconds (0, until stopped, aside)
pub const MAX_SCAN_LENGTH: u32 = 600;

fn default_scan_length() -> u32 {
    DEFAULT_SCAN_LENGTH
}

impl Default for VibeCheckConfig {
    fn default() -> Self {
        Self {
            networking: OSCNetworking::default(),
            scan_on_disconnect: false,
            minimize_on_exit: false,
            desktop_notifications: false,
            show_toy_advanced: false,
            show_feature_advanced: false,
            master_intensity: full_intensity(),
            scan_length: default_scan_length(),
            theme: default_theme(),
            auto_enable_on_scan: false,
            sidebar_collapsed: false,
            toy_connection: ToyConnection::Local,
            intiface_addresses: Vec::new(),
        }
    }
}

/// Writes the default config. A failure is only logged: VibeCheck runs on
/// the defaults anyway, and saving settings later reports its own error.
fn write_default_config(path: &str) {
    let json = serde_json::to_string(&VibeCheckConfig::default())
        .expect("the default config always serializes");
    if let Err(e) = fs::write(path, json) {
        logerr!("Couldn't write the default config to {}: {}", path, e);
    }
}

/// Loads VibeCheck's settings, creating its folders and a default config the
/// first time. Problems with the file itself are reported through
/// `reporter`, and VibeCheck runs on the defaults; only a missing config
/// folder is an error.
pub fn config_load(
    app_handle: &AppHandle,
    reporter: &ErrorReporter,
) -> Result<VibeCheckConfig, VibeCheckConfigError> {
    let vc_root_dir = match get_config_dir(app_handle) {
        Ok(d) => d,
        Err(_) => return Err(VibeCheckConfigError::ConfigDirFail),
    };

    let vc_config_file = build_path_file(&[&vc_root_dir, "Config.json"]);
    let vc_toy_config_dir = build_path_dir(&[&vc_root_dir, "ToyConfigs"]);

    info!("vc_config_file: {}", vc_config_file);
    info!("vc_toy_config_dir: {}", vc_toy_config_dir);

    if !path_exists(&vc_root_dir) {
        if let Err(e) = fs::create_dir_all(&vc_root_dir) {
            logerr!("Cannot create VibeCheck root directory: {}", e);
            return Err(VibeCheckConfigError::ConfigDirFail);
        }
        info!("Created VibeCheck root directory.");
    } else {
        info!("VibeCheck root directory exists.");
    }

    info!("Attempting creation of : {}", vc_toy_config_dir);
    if !path_exists(&vc_toy_config_dir) {
        if let Err(e) = fs::create_dir(&vc_toy_config_dir) {
            logerr!("Cannot create VibeCheck toy directory: {}", e);
            return Err(VibeCheckConfigError::ConfigDirFail);
        }
        info!("Created VibeCheck toy config directory.");
    } else {
        info!("VibeCheck toy config directory.");
    }

    if !file_exists(&vc_config_file) {
        write_default_config(&vc_config_file);
        info!("Created VibeCheck config.");
    } else {
        info!("VibeCheck config exists.");
    }

    Ok(load_config_file(Path::new(&vc_config_file), reporter))
}

/// Reads the config at `path`. If it can't be used, VibeCheck runs on the
/// defaults and says so:
/// - a file that isn't valid settings is renamed to `Config.json.bad` and
///   replaced with the defaults, so the user can still look at it;
/// - a file that can't be read is left alone (it may be a passing problem,
///   like permissions), but the next settings change will overwrite it.
fn load_config_file(path: &Path, reporter: &ErrorReporter) -> VibeCheckConfig {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            reporter.warning(
                "config",
                "config_unreadable",
                format!(
                    "Couldn't open your settings ({e}), so VibeCheck is using its defaults. Changing a setting will replace the file at {}.",
                    path.display()
                ),
            );
            return VibeCheckConfig::default();
        }
    };

    match serde_json::from_str::<VibeCheckConfig>(&contents) {
        Ok(config) => {
            info!("Config Loaded Successfully!");
            config
        }
        Err(e) => {
            logerr!("Failed to parse {}: {}", path.display(), e);
            let kept = match set_aside_bad_file(path) {
                Ok(bad) => format!(" The old file was kept as {}.", bad.display()),
                Err(e) => {
                    warn!("Couldn't keep the unreadable config: {}", e);
                    String::new()
                }
            };
            write_default_config(&path.to_string_lossy());
            reporter.warning(
                "config",
                "config_corrupt",
                format!("Your settings file couldn't be read, so VibeCheck went back to its default settings.{kept}"),
            );
            VibeCheckConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_config(name: &str, contents: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("vibecheck-config-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Config.json");
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn a_corrupt_config_is_kept_and_replaced_with_defaults() {
        let path = temp_config("corrupt", "{ not json");
        let (reporter, mut notices) = ErrorReporter::channel();

        let config = load_config_file(&path, &reporter);
        assert_eq!(config.theme, VibeCheckConfig::default().theme);

        let bad = path.with_file_name("Config.json.bad");
        assert_eq!(fs::read_to_string(&bad).unwrap(), "{ not json");
        let rewritten: VibeCheckConfig =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(rewritten.theme, VibeCheckConfig::default().theme);

        let notice = notices.try_recv().unwrap();
        assert_eq!(notice.code, "config_corrupt");
        assert!(notice.message.contains("Config.json.bad"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_good_config_loads_without_a_notice() {
        let mut saved = VibeCheckConfig::default();
        saved.theme = "other".to_owned();
        let path = temp_config("good", &serde_json::to_string(&saved).unwrap());
        let (reporter, mut notices) = ErrorReporter::channel();

        assert_eq!(load_config_file(&path, &reporter).theme, "other");
        assert!(notices.try_recv().is_err());

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    fn config_json_with(key: Option<(&str, f64)>) -> String {
        let mut value = serde_json::to_value(VibeCheckConfig::default()).unwrap();
        let map = value.as_object_mut().unwrap();
        map.remove("master_intensity");
        if let Some((name, level)) = key {
            map.insert(name.to_string(), serde_json::json!(level));
        }
        value.to_string()
    }

    #[test]
    fn master_intensity_loads_from_new_old_and_missing_keys() {
        let load = |json: String| -> f64 {
            serde_json::from_str::<VibeCheckConfig>(&json)
                .unwrap()
                .master_intensity
        };
        assert_eq!(load(config_json_with(Some(("master_intensity", 0.4)))), 0.4);
        assert_eq!(load(config_json_with(Some(("master_volume", 0.6)))), 0.6);
        assert_eq!(load(config_json_with(None)), 1.0);
    }

    #[test]
    fn frontend_preferences_default_when_missing() {
        let mut value = serde_json::to_value(VibeCheckConfig::default()).unwrap();
        let map = value.as_object_mut().unwrap();
        for key in ["theme", "auto_enable_on_scan", "sidebar_collapsed"] {
            map.remove(key);
        }
        let config: VibeCheckConfig = serde_json::from_value(value).unwrap();
        assert_eq!(config.theme, "vibecheck");
        assert!(!config.auto_enable_on_scan);
        assert!(!config.sidebar_collapsed);
    }

    #[test]
    fn scan_length_defaults_to_30_seconds_when_missing() {
        let mut value = serde_json::to_value(VibeCheckConfig::default()).unwrap();
        value.as_object_mut().unwrap().remove("scan_length");
        let config: VibeCheckConfig = serde_json::from_value(value).unwrap();
        assert_eq!(config.scan_length, 30);
    }
}
