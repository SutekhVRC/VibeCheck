#![windows_subsystem = "windows"]

use eframe::epaint::Vec2;
use serde::{Deserialize, Serialize};
use eframe::{NativeOptions, run_native};
use std::path::Path;
use std::fs;
use directories::BaseDirs;
use std::net::Ipv4Addr;


mod ui;
mod handling;

use ui::VibeCheckGUI;


#[derive(Deserialize, Serialize, Debug)]
struct OSCNetworking {
    bind: (String, String),
    //vrchat: (String, String),
}

impl Default for OSCNetworking {
    fn default() -> Self {
        Self {
            bind: ("127.0.0.1".to_string(), "10069".to_string()),
            //vrchat: ("127.0.0.1".to_string(), "9000".to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct IntifaceConfig(String);

#[derive(Deserialize, Serialize, Debug)]
pub struct VibeCheckConfig {
    networking: OSCNetworking,
    intiface_config: IntifaceConfig,
}


fn config_load() -> VibeCheckConfig {
    
    let vc_root_dir = format!("{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck", get_user_home_dir());
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
        fs::write(&vc_config_file, serde_json::to_string(
            &VibeCheckConfig {
                networking: OSCNetworking::default(),
                intiface_config: IntifaceConfig("6969".to_string()),
            }
        ).unwrap()).unwrap();
        println!("[+] Created VibeCheck config.");
    } else {
        println!("[*] VibeCheck config exists.");
    }

    match fs::read_to_string(&vc_config_file) {
        Ok(fc) => {
            match serde_json::from_str(&fc) {
                Ok(o) => return o,
                Err(_e) => {
                    println!("[-] Failed to parse json from file: {} [{}]", vc_config_file, _e);
                    std::process::exit(0);
                }
            }
        },
        Err(_e) => {
            println!("[-] Could not parse bytes from file: {} [{}].. Skipping..", vc_config_file, _e);
            std::process::exit(0);
        }
    }
}


fn main() {

    let mut native_opts = NativeOptions::default();

    native_opts.initial_window_size = Some(Vec2::new(450., 500.));

    run_native(
        Box::new(
            VibeCheckGUI::new(config_load())), native_opts);
}

fn check_valid_port(port: &String) -> bool {
    if let Ok(p) = port.parse::<u64>() {
        if p > 0 && p < 65535 {
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn check_valid_ipv4(ip: &String) -> bool {

    if ip.parse::<Ipv4Addr>().is_err() {
        false
    } else {
        true
    }
}

fn path_exists(p: &String) -> bool {
    Path::new(&p).is_dir()
}

fn file_exists(p: &String) -> bool {
    Path::new(&p).is_file()
}

fn get_user_home_dir() -> String {
    let bd = BaseDirs::new().expect("[-] Could not get user's directories.");
    let bd = bd.home_dir().to_str().expect("[-] Failed to get user's home directory.");
    bd.to_string()
}