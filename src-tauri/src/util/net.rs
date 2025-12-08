use std::net::{TcpListener, UdpSocket};

pub enum InterfaceL4Proto {
    Udp(String),
    Tcp(String),
}

fn is_port_available(interface_proto: InterfaceL4Proto, port: u16) -> bool {
    match interface_proto {
        InterfaceL4Proto::Tcp(interface) => TcpListener::bind((interface, port)).is_ok(),
        InterfaceL4Proto::Udp(interface) => UdpSocket::bind((interface, port)).is_ok(),
    }
}

pub fn find_available_tcp_port(interface_addr: String) -> Option<u16> {
    (10000..11000)
        .find(|port| is_port_available(InterfaceL4Proto::Tcp(interface_addr.to_owned()), *port))
}

pub fn find_available_udp_port(interface_addr: String) -> Option<u16> {
    (10000..11000)
        .find(|port| is_port_available(InterfaceL4Proto::Udp(interface_addr.to_owned()), *port))
}
