use std::{fs, net::{Ipv4Addr, SocketAddrV4}};
use serde::{Deserialize, Serialize};
use log::{info, trace, error as logerr, warn};

use crate::{util::{
    file_exists,
    path_exists,
    get_user_home_dir,
}, toyops::FeatureParamMap};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OSCNetworking {
    pub bind: SocketAddrV4,
    pub remote: SocketAddrV4,
}

impl Default for OSCNetworking {
    fn default() -> Self {
        Self {
            bind: SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 9001),
            remote: SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 9000),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VibeCheckConfig {
    // Change networking to an enum between OSCQuery and setting bind and remote.
    pub networking: OSCNetworking,
    pub scan_on_disconnect: bool,
}

pub fn config_load() -> VibeCheckConfig {
    let vc_root_dir = format!(
        "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck",
        get_user_home_dir()
    );
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
            })
            .unwrap(),
        )
        .unwrap();
        info!("Created VibeCheck config.");
    } else {
        info!("VibeCheck config exists.");
    }

    match fs::read_to_string(&vc_config_file) {
        Ok(fc) => match serde_json::from_str(&fc) {
            Ok(o) => {
                info!("Config Loaded Successfully!");
                return o;
            },
            Err(_e) => {
                logerr!(
                    "Failed to parse json from file: {} [{}]",
                    vc_config_file, _e
                );
                warn!("Resetting to default config.");

                let def_conf = VibeCheckConfig {
                    networking: OSCNetworking::default(),
                    scan_on_disconnect: false,
                };

                fs::write(
                    &vc_config_file,
                    serde_json::to_string(&def_conf).unwrap(),
                )
                .unwrap();
                trace!("Wrote VibeCheck config file");
                // If fail to parse config overwrite with new default
                return def_conf;
            }
        },
        Err(_e) => {
            logerr!(
                "Could not parse bytes from file: {} [{}].. Skipping..",
                vc_config_file, _e
            );
            warn!("[*] Resetting to default config.");
            let def_conf = VibeCheckConfig {
                networking: OSCNetworking::default(),
                scan_on_disconnect: false,
            };
            fs::write(
                &vc_config_file,
                serde_json::to_string(&def_conf).unwrap(),
            )
            .unwrap();
            trace!("Wrote VibeCheck config file");
            return def_conf;
        }
    }
}


pub mod toy {
    use std::fs;

    use serde::{Serialize, Deserialize};

    use crate::{util::{get_user_home_dir, file_exists}, toyops::FeatureParamMap};


    #[derive(Debug, Serialize, Deserialize, Clone, Default)]
    pub struct VCToyConfig {
        pub toy_name: String,
        pub features: FeatureParamMap,
        pub osc_data: bool,
    }
}