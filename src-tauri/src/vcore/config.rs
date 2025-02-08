use log::{error as logerr, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    net::{Ipv4Addr, SocketAddrV4},
};

use crate::{
    frontend::frontend_types::FeOSCNetworking,
    util::fs::{file_exists, get_config_dir, path_exists},
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OSCNetworking {
    pub bind: SocketAddrV4,
    pub remote: SocketAddrV4,
    pub osc_query_enabled: bool,
}

impl Default for OSCNetworking {
    fn default() -> Self {
        Self {
            bind: SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 9001),
            remote: SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 9000),
            osc_query_enabled: true,
        }
    }
}

impl OSCNetworking {
    pub fn to_fe(&self) -> FeOSCNetworking {
        FeOSCNetworking {
            bind: self.bind.to_string(),
            remote: self.remote.to_string(),
            osc_query_enabled: self.osc_query_enabled,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VibeCheckConfig {
    // Change networking to an enum between OSCQuery and setting bind and remote.
    pub networking: OSCNetworking,
    pub scan_on_disconnect: bool,
    pub minimize_on_exit: bool,
    pub desktop_notifications: bool,
    //pub lc_override: Option<Ipv4Addr>,
    pub show_toy_advanced: bool,
    pub show_feature_advanced: bool,
}

pub fn config_load() -> VibeCheckConfig {
    let vc_root_dir = get_config_dir();

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
                //lc_override: None,
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
                /*
                if let Some(h) = o.lc_override {
                    std::env::set_var("VCLC_HOST_PORT", format!("{}:20010", h).as_str());
                    info!("Setting VCLC_HOST_PORT: {}", format!("{}:20010", h));
                }*/
                o
            }
            Err(_e) => {
                logerr!(
                    "Failed to parse json from file: {} [{}]",
                    vc_config_file,
                    _e
                );
                warn!("Resetting to default config.");

                let def_conf = VibeCheckConfig {
                    networking: OSCNetworking::default(),
                    scan_on_disconnect: false,
                    minimize_on_exit: false,
                    desktop_notifications: false,
                    //lc_override: None,
                    show_toy_advanced: false,
                    show_feature_advanced: false,
                };

                fs::write(&vc_config_file, serde_json::to_string(&def_conf).unwrap()).unwrap();
                trace!("Wrote VibeCheck config file");
                // If fail to parse config overwrite with new default
                def_conf
            }
        },
        Err(_e) => {
            logerr!(
                "Could not parse bytes from file: {} [{}].. Skipping..",
                vc_config_file,
                _e
            );
            warn!("[*] Resetting to default config.");
            let def_conf = VibeCheckConfig {
                networking: OSCNetworking::default(),
                scan_on_disconnect: false,
                minimize_on_exit: false,
                desktop_notifications: false,
                //lc_override: None,
                show_toy_advanced: false,
                show_feature_advanced: false
            };
            fs::write(&vc_config_file, serde_json::to_string(&def_conf).unwrap()).unwrap();
            trace!("Wrote VibeCheck config file");
            def_conf
        }
    }
}

pub mod toy {

    use crate::{
        frontend::frontend_types::FeVCToyAnatomy,
        toy_handling::toyops::VCToyFeatures,
        util::fs::{file_exists, get_config_dir},
        vcore::vcerror,
    };
    use log::{debug, error as logerr, info, warn};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
    pub enum VCToyAnatomy {
        Anus,
        //Any, I forgot what the point of this was?
        Breasts,
        Buttocks,
        Chest,
        Clitoris,
        Face,
        Feet,
        FootL,
        FootR,
        HandLeft,
        HandRight,
        Hands,
        Labia,
        Mouth,
        #[default]
        NA,
        Nipples,
        Penis,
        Perineum,
        Testicles,
        Thighs,
        Vagina,
        Vulva,
        Wrist,
    }

    impl VCToyAnatomy {
        pub fn get_anatomy(token: &String) -> Self {
            match token.to_lowercase().as_str() {
                "anus" => Self::Anus,
                //"any" => Self::Any,
                "breasts" => Self::Breasts,
                "butt" | "buttocks" => Self::Buttocks,
                "chest" => Self::Chest,
                "clitoris" => Self::Clitoris,
                "face" => Self::Face,
                "feet" => Self::Feet,
                "footl" => Self::FootL,
                "footr" => Self::FootR,
                "hands" => Self::Hands,
                "handl" => Self::HandLeft,
                "handr" => Self::HandRight,
                "labia" => Self::Labia,
                "mouth" => Self::Mouth,
                "na" => Self::NA,
                "nipples" => Self::Nipples,
                "penis" => Self::Penis,
                "perineum" => Self::Perineum,
                "testicles" => Self::Testicles,
                "thighs" => Self::Thighs,
                "vagina" => Self::Vagina,
                "vulva" => Self::Vulva,
                "wrist" => Self::Wrist,
                _ => {
                    warn!(
                        "Got \"{}\" for anatomy token.. Defaulting to Self::NA",
                        token
                    );
                    Self::NA
                }
            }
        }

        pub fn to_fe(&self) -> FeVCToyAnatomy {
            match self {
                Self::Anus => FeVCToyAnatomy::Anus,
                Self::Breasts => FeVCToyAnatomy::Breasts,
                Self::Buttocks => FeVCToyAnatomy::Buttocks,
                Self::Chest => FeVCToyAnatomy::Chest,
                Self::Clitoris => FeVCToyAnatomy::Clitoris,
                Self::Face => FeVCToyAnatomy::Face,
                Self::Feet => FeVCToyAnatomy::Feet,
                Self::FootL => FeVCToyAnatomy::FootL,
                Self::FootR => FeVCToyAnatomy::FootR,
                Self::HandLeft => FeVCToyAnatomy::HandLeft,
                Self::HandRight => FeVCToyAnatomy::HandRight,
                Self::Hands => FeVCToyAnatomy::Hands,
                Self::Labia => FeVCToyAnatomy::Labia,
                Self::Mouth => FeVCToyAnatomy::Mouth,
                Self::NA => FeVCToyAnatomy::NA,
                Self::Nipples => FeVCToyAnatomy::Nipples,
                Self::Penis => FeVCToyAnatomy::Penis,
                Self::Perineum => FeVCToyAnatomy::Perineum,
                Self::Testicles => FeVCToyAnatomy::Testicles,
                Self::Thighs => FeVCToyAnatomy::Thighs,
                Self::Vagina => FeVCToyAnatomy::Vagina,
                Self::Vulva => FeVCToyAnatomy::Vulva,
                Self::Wrist => FeVCToyAnatomy::Wrist,
            }
        }

        pub fn from_fe(&mut self, anatomy_type: FeVCToyAnatomy) {
            match anatomy_type {
                FeVCToyAnatomy::Anus => *self = Self::Anus,
                FeVCToyAnatomy::Breasts => *self = Self::Breasts,
                FeVCToyAnatomy::Buttocks => *self = Self::Buttocks,
                FeVCToyAnatomy::Chest => *self = Self::Chest,
                FeVCToyAnatomy::Clitoris => *self = Self::Clitoris,
                FeVCToyAnatomy::Face => *self = Self::Face,
                FeVCToyAnatomy::Feet => *self = Self::Feet,
                FeVCToyAnatomy::FootL => *self = Self::FootL,
                FeVCToyAnatomy::FootR => *self = Self::FootR,
                FeVCToyAnatomy::HandLeft => *self = Self::HandLeft,
                FeVCToyAnatomy::HandRight => *self = Self::HandRight,
                FeVCToyAnatomy::Hands => *self = Self::Hands,
                FeVCToyAnatomy::Labia => *self = Self::Labia,
                FeVCToyAnatomy::Mouth => *self = Self::Mouth,
                FeVCToyAnatomy::NA => *self = Self::NA,
                FeVCToyAnatomy::Nipples => *self = Self::Nipples,
                FeVCToyAnatomy::Penis => *self = Self::Penis,
                FeVCToyAnatomy::Perineum => *self = Self::Perineum,
                FeVCToyAnatomy::Testicles => *self = Self::Testicles,
                FeVCToyAnatomy::Thighs => *self = Self::Thighs,
                FeVCToyAnatomy::Vagina => *self = Self::Vagina,
                FeVCToyAnatomy::Vulva => *self = Self::Vulva,
                FeVCToyAnatomy::Wrist => *self = Self::Wrist,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone, Default)]
    pub struct VCToyConfig {
        pub toy_name: String,
        pub features: VCToyFeatures,
        pub osc_data: bool,
        pub anatomy: VCToyAnatomy,
    }

    impl VCToyConfig {
        pub fn load_offline_toy_config(
            toy_name: String,
        ) -> Result<VCToyConfig, vcerror::backend::VibeCheckToyConfigError> {
            // Generate config path
            // - Transform Lovense Connect toys to load lovense configs

            let config_path = format!(
                "{}\\ToyConfigs\\{}.json",
                get_config_dir(),
                toy_name/*.replace("Lovense Connect ", "Lovense ")*/,
            );

            if !file_exists(&config_path) {
                Err(vcerror::backend::VibeCheckToyConfigError::OfflineToyConfigNotFound)
            } else {
                let con = std::fs::read_to_string(config_path).unwrap();

                let config: VCToyConfig = match serde_json::from_str(&con) {
                    Ok(vc_toy_config) => vc_toy_config,
                    Err(_) => {
                        return Err(vcerror::backend::VibeCheckToyConfigError::DeserializeError);
                    }
                };
                debug!("Loaded & parsed toy config successfully!");
                Ok(config)
            }
        }

        pub fn save_offline_toy_config(&self) {
            let config_path = format!(
                "{}\\ToyConfigs\\{}.json",
                get_config_dir(),
                self.toy_name/*.replace("Lovense Connect ", "Lovense ")*/,
            );

            info!("Saving toy config to: {}", config_path);

            if let Ok(json_string) = serde_json::to_string(self) {
                match std::fs::write(&config_path, json_string) {
                    Ok(()) => {
                        info!("Saved toy config: {}", self.toy_name);
                    }
                    Err(e) => {
                        logerr!("Failed to write to file: {}", e);
                    }
                }
            } else {
                warn!("Failed to serialize config to json");
            }
        }
    }
}
