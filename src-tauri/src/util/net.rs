use std::net::{TcpListener, UdpSocket};

pub enum InterfaceL4Proto {
    UDP(String),
    TCP(String),
}

fn is_port_available(interface_proto: InterfaceL4Proto, port: u16) -> bool {
    match interface_proto {
        InterfaceL4Proto::TCP(interface) => match TcpListener::bind((interface, port)) {
            Ok(_) => true,
            _ => false,
        },
        InterfaceL4Proto::UDP(interface) => match UdpSocket::bind((interface, port)) {
            Ok(_) => true,
            _ => false,
        },
    }
}

pub fn find_available_tcp_port(interface_addr: String) -> Option<u16> {
    (10000..11000)
        .find(|port| is_port_available(InterfaceL4Proto::TCP(interface_addr.to_owned()), *port))
}

pub fn find_available_udp_port(interface_addr: String) -> Option<u16> {
    (10000..11000)
        .find(|port| is_port_available(InterfaceL4Proto::UDP(interface_addr.to_owned()), *port))
}
