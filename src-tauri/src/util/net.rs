use std::net::{Ipv4Addr, UdpSocket};

/// Ports VibeCheck picks from for OSCQuery mode. Below the ephemeral ranges
/// (Linux 32768+, Windows 49152+), so the OS won't hand the port out for
/// outgoing traffic later.
const PORT_RANGE: std::ops::Range<u16> = 10000..11000;

/// Whether a UDP port can be bound on `ip` right now.
pub fn udp_port_available(ip: Ipv4Addr, port: u16) -> bool {
    UdpSocket::bind((ip, port)).is_ok()
}

/// A UDP port in [`PORT_RANGE`] that can be bound on `ip` right now.
pub fn find_available_udp_port(ip: Ipv4Addr) -> Option<u16> {
    PORT_RANGE
        .clone()
        .find(|port| udp_port_available(ip, *port))
}
