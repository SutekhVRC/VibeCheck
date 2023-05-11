use std::{fs, net::{Ipv4Addr, SocketAddrV4}};
use serde::{Deserialize, Serialize};
use log::{info, trace, error as logerr, warn};

use crate::{util::{
    file_exists,
    path_exists,
    get_config_dir,
}};


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
    pub minimize_on_exit: bool,
    pub desktop_notifications: bool,
    pub lc_override: Option<Ipv4Addr>,
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
                lc_override: None,
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
                if let Some(h) = o.lc_override {
                    std::env::set_var("VCLC_HOST_PORT", format!("{}:20010", h.to_string()).as_str());
                    info!("Setting VCLC_HOST_PORT: {}", format!("{}:20010", h.to_string()));
                }
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
                    minimize_on_exit: false,
                    desktop_notifications: false,
                    lc_override: None,
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
                minimize_on_exit: false,
                desktop_notifications: false,
                lc_override: None,
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

    use log::warn;
    use serde::{Serialize, Deserialize};
    use crate::{toyops::FeatureParamMap, frontend_types::FeVCToyAnatomy};

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
                "vagina" => Self::Vagina,
                "vulva" => Self::Vulva,
                "wrist" => Self::Wrist,
                _ => {
                    warn!("Got \"{}\" for anatomy token.. Defaulting to Self::NA", token);
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
                FeVCToyAnatomy::Vagina => *self = Self::Vagina,
                FeVCToyAnatomy::Vulva => *self = Self::Vulva,
                FeVCToyAnatomy::Wrist => *self = Self::Wrist,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone, Default)]
    pub struct VCToyConfig {
        pub toy_name: String,
        pub features: FeatureParamMap,
        pub osc_data: bool,
        pub anatomy: VCToyAnatomy,
    }
}