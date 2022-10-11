use std::net::Ipv4Addr;
use std::path::Path;
use directories::BaseDirs;

pub fn check_valid_port(port: &String) -> bool {
    if let Ok(p) = port.parse::<u16>() {
        // Dont need to check for >= 0 bc of type limits unsigned 16 bit int
        if p < 65535 {
            true
        } else {
            false
        }
    } else {
        false
    }
}

pub fn check_valid_ipv4(ip: &String) -> bool {
    !ip.parse::<Ipv4Addr>().is_err()
}

pub fn path_exists(p: &String) -> bool {
    Path::new(&p).is_dir()
}

pub fn file_exists(p: &String) -> bool {
    Path::new(&p).is_file()
}

pub fn get_user_home_dir() -> String {
    let bd = BaseDirs::new().expect("[-] Could not get user's directories.");
    let bd = bd
        .home_dir()
        .to_str()
        .expect("[-] Failed to get user's home directory.");
    bd.to_string()
}