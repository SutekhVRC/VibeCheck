#![windows_subsystem = "windows"]

use directories::BaseDirs;
use eframe::epaint::Vec2;
use eframe::{run_native, NativeOptions};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::Path;

mod handling;
mod ui;

use ui::VibeCheckGUI;

fn default_socket() -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::new(127,0,0,1), 9001)
}

fn default_intiface() -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::new(0,0,0,0), 6969)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VibeCheckConfig {
    networking: SocketAddrV4,
    intiface_config: SocketAddrV4,
}

pub struct VibeCheckUserConfig {
    networking_addr: String,
    networking_port: String,
    intiface_port: String,
}

impl VibeCheckConfig {
    fn as_user_config(self) -> VibeCheckUserConfig {
        return VibeCheckUserConfig {
            networking_addr: self.networking.ip().to_string(),
            networking_port: self.networking.port().to_string(),
            intiface_port: self.intiface_config.port().to_string(),
        };
    }
}


fn config_load() -> VibeCheckConfig {
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
                networking: default_socket(),
                intiface_config: default_intiface(),
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
                    networking: default_socket(),
                    intiface_config: default_intiface(),
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
                networking: default_socket(),
                intiface_config: default_intiface(),
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

fn main() {
    let mut native_opts = NativeOptions::default();

    native_opts.initial_window_size = Some(Vec2::new(450., 500.));

    run_native("VibeCheck", native_opts, Box::new(|cc| Box::new(VibeCheckGUI::new(config_load(), cc))));
}

fn path_exists(p: &String) -> bool {
    Path::new(&p).is_dir()
}

fn file_exists(p: &String) -> bool {
    Path::new(&p).is_file()
}

fn get_user_home_dir() -> String {
    let bd = BaseDirs::new().expect("[-] Could not get user's directories.");
    let bd = bd
        .home_dir()
        .to_str()
        .expect("[-] Failed to get user's home directory.");
    bd.to_string()
}
