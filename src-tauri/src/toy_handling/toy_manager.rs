use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error_signal_handler::{ErrorSource, VibeCheckError};
use crate::frontend::ToFrontend;
use crate::util::fs::build_path_dir;
use crate::{
    config::toy::VCToyConfig,
    frontend::frontend_types::FeVCToy,
    toy_handling::toyops::VCToy,
    util::fs::{file_exists, get_config_dir},
};
use log::{debug, info, trace, warn};
use tauri::AppHandle;

use super::errors::ToyHandlingError;

#[derive(Clone)]
pub struct ToyManager {
    pub configs: HashMap<String, VCToyConfig>,
    pub online_toys: HashMap<u32, VCToy>,
    _app_handle: AppHandle,
}

impl ToyManager {
    pub fn new(app_handle: AppHandle) -> Result<Self, VibeCheckError> {
        /*
         * Read all toy configs
         * Send update to frontend
         *
         * Create methods:
         * Method for a toy disconnecting
         * Method for a toy connecting
         *
         * This struct is the new toys handler / object
         */

        let mut ot = Self {
            configs: HashMap::new(),
            online_toys: HashMap::new(),
            _app_handle: app_handle,
        };

        match ot.populate_configs() {
            Ok(()) => {
                trace!("ToyManager config population complete!");
                Ok(ot)
            }
            Err(e) => Err(VibeCheckError::new(
                ErrorSource::ToyHandling(e),
                Some("Config population failure"),
            )),
        }
    }

    pub fn populate_configs(&mut self) -> Result<(), ToyHandlingError> {
        let config_dir = match get_config_dir(&self._app_handle) {
            Ok(d) => d,
            Err(_) => return Err(ToyHandlingError::PopulateConfigFailure),
        };

        let toy_config_dir = match fs::read_dir(build_path_dir(&[&config_dir, "ToyConfigs"])) {
            Ok(config_paths) => config_paths,
            // Doesn't populate
            Err(_e) => return Err(ToyHandlingError::PopulateConfigFailure),
        };

        let config_files = toy_config_dir.filter_map(|dir| dir.ok());

        for f in config_files {
            if !file_exists(&f.path()) {
                continue;
            }

            let con = match std::fs::read_to_string(f.path()) {
                Ok(contents) => contents,
                Err(_e) => continue,
            };

            let config: VCToyConfig = match serde_json::from_str(&con) {
                Ok(vc_toy_config) => vc_toy_config,
                Err(_) => {
                    continue;
                }
            };

            trace!(
                "Loaded & parsed toy config [{}] successfully!",
                config.toy_name
            );
            let toy_name = config.toy_name.clone();
            self.configs.insert(toy_name, config);
        }

        debug!("Loaded {} Offline toy configs!", self.configs.len());
        Ok(())
    }

    pub fn sync_frontend(&mut self, refresh_toys: bool) -> Result<Vec<FeVCToy>, ToyHandlingError> {
        if refresh_toys {
            info!("Clearing toy manager configs map and repopulating from disk..");
            self.configs.clear();
            self.populate_configs()?;
        }

        trace!("Generating offline toy sync..");
        Ok(self.fetoy_vec_from_offline_toys())
    }

    /// Forgets an offline toy: deletes its saved settings and drops it from
    /// the offline list. A connected toy can't be removed, because it would
    /// be saved again straight away.
    pub fn remove_offline_toy(&mut self, toy_name: &str) -> Result<(), ToyHandlingError> {
        if self.check_toy_online(&toy_name.to_owned()) {
            return Err(ToyHandlingError::ToyOnline);
        }

        let config_dir =
            get_config_dir(&self._app_handle).map_err(|_| ToyHandlingError::RemoveFailure)?;
        let toy_config_dir = build_path_dir(&[&config_dir, "ToyConfigs"]);

        match remove_toy_configs(Path::new(&toy_config_dir), toy_name) {
            Ok(0) => Err(ToyHandlingError::ToyNotFound),
            Ok(removed) => {
                info!("Removed toy [{}] ({} config file(s))", toy_name, removed);
                self.configs.remove(toy_name);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to remove toy [{}]: {}", toy_name, e);
                Err(ToyHandlingError::RemoveFailure)
            }
        }
    }

    fn check_toy_online(&self, config_toy_name: &String) -> bool {
        for online_toy in self.online_toys.iter() {
            if *config_toy_name == online_toy.1.toy_name {
                return true;
            }
        }
        false
    }

    fn fetoy_vec_from_offline_toys(&self) -> Vec<FeVCToy> {
        let mut offline_toy_vec = Vec::new();

        for (_toy_key, config) in self.configs.iter() {
            if self.check_toy_online(&config.toy_name) {
                continue;
            }

            offline_toy_vec.push(FeVCToy {
                toy_id: None,
                toy_name: config.toy_name.clone(),
                toy_anatomy: config.anatomy.to_fe(),
                toy_power: super::ToyPower::Offline,
                toy_connected: false,
                features: config.features.features.to_frontend(),
                listening: false,
                osc_data: config.osc_data,
                bt_update_rate: config.bt_update_rate,
                sub_id: 255,
            });
        }

        offline_toy_vec
    }
}

/// Deletes every config in `dir` saved for `toy_name`, and returns how many
/// went. Files are matched by the name inside them rather than by building a
/// path from the name, so a name can never point outside the folder.
fn remove_toy_configs(dir: &Path, toy_name: &str) -> std::io::Result<usize> {
    let mut removed = 0;
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(config) = serde_json::from_str::<VCToyConfig>(&contents) else {
            continue;
        };
        if config.toy_name == toy_name {
            fs::remove_file(&path)?;
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "vibecheck-toy-test-{}-{}",
            name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_config(dir: &Path, file: &str, toy_name: &str) {
        let config = VCToyConfig {
            toy_name: toy_name.to_owned(),
            ..Default::default()
        };
        fs::write(dir.join(file), serde_json::to_string(&config).unwrap()).unwrap();
    }

    #[test]
    fn removes_only_the_named_toy() {
        let dir = temp_dir("named");
        write_config(&dir, "Lovense Hush.json", "Lovense Hush");
        write_config(&dir, "Lovense Lush.json", "Lovense Lush");
        fs::write(dir.join("notes.txt"), "not a config").unwrap();

        assert_eq!(remove_toy_configs(&dir, "Lovense Hush").unwrap(), 1);
        assert!(!dir.join("Lovense Hush.json").exists());
        assert!(dir.join("Lovense Lush.json").exists());
        assert!(dir.join("notes.txt").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn matches_by_the_saved_name_not_the_file_name() {
        let dir = temp_dir("saved-name");
        write_config(&dir, "renamed.json", "Lovense Hush");

        assert_eq!(remove_toy_configs(&dir, "Lovense Hush").unwrap(), 1);
        assert!(!dir.join("renamed.json").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_name_cannot_reach_outside_the_folder() {
        let dir = temp_dir("outside");
        let inner = dir.join("ToyConfigs");
        fs::create_dir_all(&inner).unwrap();
        write_config(&dir, "Config.json", "Config");

        assert_eq!(remove_toy_configs(&inner, "../Config").unwrap(), 0);
        assert!(dir.join("Config.json").exists());

        fs::remove_dir_all(&dir).unwrap();
    }
}
