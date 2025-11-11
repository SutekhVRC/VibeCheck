use log::{error as logerr, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    osc::OSCNetworking,
    util::fs::{file_exists, get_config_dir, path_exists},
    vcore::errors::backend::{VibeCheckConfigError, VibeCheckFSError},
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
}

pub fn config_load() -> Result<VibeCheckConfig, VibeCheckConfigError> {
    let vc_root_dir = match get_config_dir() {
        Ok(d) => d,
        Err(_) => return Err(VibeCheckConfigError::ConfigDirFail),
    };

    let vc_config_file = format!("{}\\Config.json", vc_root_dir);
    let vc_toy_config_dir = format!("{}\\ToyConfigs", vc_root_dir);

    if !path_exists(&vc_root_dir) {
        fs::create_dir_all(&vc_root_dir).expect("[-] Cannot create VibeCheck root directory.");
        info!("Created VibeCheck root directory.");
    } else {
        info!("VibeCheck root directory exists.");
    }

    if !path_exists(&vc_toy_config_dir) {
        fs::create_dir(&vc_toy_config_dir).expect("[-] Cannot create VibeCheck toy directory.");
        info!("Created VibeCheck toy config directory.");
    } else {
        info!("VibeCheck toy config directory.");
    }

    if !file_exists(&vc_config_file) {
        fs::write(
            &vc_config_file,
            serde_json::to_string(&VibeCheckConfig {
                networking: OSCNetworking::default(),
                scan_on_disconnect: false,
                minimize_on_exit: false,
                desktop_notifications: false,
                show_toy_advanced: false,
                show_feature_advanced: false,
            })
            .unwrap(),
        )
        .unwrap();
        info!("Created VibeCheck config.");
    } else {
        info!("VibeCheck config exists.");
    }

    match fs::read_to_string(&vc_config_file) {
        Ok(fc) => match serde_json::from_str::<VibeCheckConfig>(&fc) {
            Ok(o) => {
                info!("Config Loaded Successfully!");
                Ok(o)
            }
            Err(_e) => {
                logerr!(
                    "Failed to parse json from file: {} [{}]",
                    vc_config_file,
                    _e
                );
                warn!("Resetting to default config.");

                let default_conf = VibeCheckConfig {
                    networking: OSCNetworking::default(),
                    scan_on_disconnect: false,
                    minimize_on_exit: false,
                    desktop_notifications: false,
                    show_toy_advanced: false,
                    show_feature_advanced: false,
                };

                fs::write(
                    &vc_config_file,
                    serde_json::to_string(&default_conf).unwrap(),
                )
                .unwrap();
                trace!("Wrote VibeCheck config file");
                // If fail to parse config overwrite with new default
                Ok(default_conf)
            }
        },
        Err(_e) => {
            logerr!(
                "Could not parse bytes from file: {} [{}].. Skipping..",
                vc_config_file,
                _e
            );
            warn!("[*] Resetting to default config.");
            let default_conf = VibeCheckConfig {
                networking: OSCNetworking::default(),
                scan_on_disconnect: false,
                minimize_on_exit: false,
                desktop_notifications: false,
                show_toy_advanced: false,
                show_feature_advanced: false,
            };
            fs::write(
                &vc_config_file,
                serde_json::to_string(&default_conf).unwrap(),
            )
            .unwrap();
            trace!("Wrote VibeCheck config file");
            Ok(default_conf)
        }
    }
}
