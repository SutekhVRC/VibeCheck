use std::fs;
use serde::{Deserialize, Serialize};

use crate::{util::{
    file_exists,
    path_exists,
    get_user_home_dir,
}, toyops::FeatureParamMap};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OSCNetworking {
    pub bind: (String, String),
    //vrchat: (String, String),
}

impl Default for OSCNetworking {
    fn default() -> Self {
        Self {
            bind: ("127.0.0.1".to_string(), "9001".to_string()),
            //vrchat: ("127.0.0.1".to_string(), "9000".to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VibeCheckConfig {
    pub networking: OSCNetworking,
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
        println!("[+] Created VibeCheck root directory.");
    } else {
        println!("[*] VibeCheck root directory exists.");
    }

    if !path_exists(&vc_toy_config_dir) {
        fs::create_dir(&vc_toy_config_dir).expect("[-] Cannot create VibeCheck toy directory.");
        println!("[+] Created VibeCheck toy config directory.");
    } else {
        println!("[*] VibeCheck toy config directory.");
    }

    if !file_exists(&vc_config_file) {
        fs::write(
            &vc_config_file,
            serde_json::to_string(&VibeCheckConfig {
                networking: OSCNetworking::default(),
            })
            .unwrap(),
        )
        .unwrap();
        println!("[+] Created VibeCheck config.");
    } else {
        println!("[*] VibeCheck config exists.");
    }

    match fs::read_to_string(&vc_config_file) {
        Ok(fc) => match serde_json::from_str(&fc) {
            Ok(o) => {
                println!("[*] Config Loaded Successfully!");
                return o;
            },
            Err(_e) => {
                println!(
                    "[-] Failed to parse json from file: {} [{}]",
                    vc_config_file, _e
                );
                println!("[*] Resetting to default config.");

                let def_conf = VibeCheckConfig {
                    networking: OSCNetworking::default(),
                };

                fs::write(
                    &vc_config_file,
                    serde_json::to_string(&def_conf).unwrap(),
                )
                .unwrap();
                // If fail to parse config overwrite with new default
                return def_conf;
            }
        },
        Err(_e) => {
            println!(
                "[-] Could not parse bytes from file: {} [{}].. Skipping..",
                vc_config_file, _e
            );
            println!("[*] Resetting to default config.");
            let def_conf = VibeCheckConfig {
                networking: OSCNetworking::default(),
            };
            fs::write(
                &vc_config_file,
                serde_json::to_string(&def_conf).unwrap(),
            )
            .unwrap();
            return def_conf;
        }
    }
}

pub fn load_toy_config(toy_name: &String) -> Option<FeatureParamMap> {

    let config_path = format!(
        "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck\\ToyConfigs\\{}.json",
        get_user_home_dir(),
        toy_name
    );

    if !file_exists(&config_path) {
        // Config doesn't exist
        return None;
    } else {
        // Try to read as 
        if let Ok(con) = fs::read_to_string(config_path) {
            let feature_param_map: FeatureParamMap = match serde_json::from_str(&con) {
                Ok(fpm) => fpm,
                Err(_) => {
                    return None;
                }
            };
            return Some(feature_param_map);
        }
        return None;
    }
}

// Save Toy config by name
pub fn save_toy_config(toy_name: &String, feature_param_map: FeatureParamMap) {
    let config_path = format!(
        "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck\\ToyConfigs\\{}.json",
        get_user_home_dir(),
        toy_name
    );

    if let Ok(json_string) = serde_json::to_string(&feature_param_map) {
        let _ = fs::write(
            &config_path,
            json_string,
        );
    }
}