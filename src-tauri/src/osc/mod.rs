use std::net::{Ipv4Addr, SocketAddrV4};

use serde::{Deserialize, Serialize};

use crate::frontend::frontend_types::FeOSCNetworking;

pub mod logic;

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
