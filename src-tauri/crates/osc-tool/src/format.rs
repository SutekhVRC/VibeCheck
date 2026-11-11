//! Shared helpers: addresses, values and how messages are printed.

use std::net::SocketAddr;
use std::time::Instant;

use rosc::{OscMessage, OscPacket, OscType};

pub const AVATAR_PARAMS: &str = "/avatar/parameters/";

/// Accepts `host:port`, or a bare port meaning `127.0.0.1:port`.
pub fn parse_socket_addr(value: &str) -> Result<SocketAddr, String> {
    if let Ok(port) = value.parse::<u16>() {
        return Ok(SocketAddr::from(([127, 0, 0, 1], port)));
    }
    value
        .parse()
        .map_err(|_| format!("{value} is not a port or host:port"))
}

/// A full OSC address. Anything not starting with `/` is treated as an avatar
/// parameter name, so `Boop` becomes `/avatar/parameters/Boop`.
pub fn osc_address(name: &str) -> String {
    if name.starts_with('/') {
        name.to_string()
    } else {
        format!("{AVATAR_PARAMS}{name}")
    }
}

/// Parses a value for sending.
///
/// - `true` / `false` → bool
/// - a number → float (VRChat avatar parameters are floats, not ints)
/// - `f:0.5`, `i:3`, `b:true`, `s:text` → that type explicitly
pub fn parse_value(value: &str) -> Result<OscType, String> {
    let typed = |kind: &str, raw: &str| -> Result<OscType, String> {
        let bad = || format!("{raw:?} is not a valid {kind}");
        match kind {
            "f" => raw.parse().map(OscType::Float).map_err(|_| bad()),
            "i" => raw.parse().map(OscType::Int).map_err(|_| bad()),
            "b" => raw.parse().map(OscType::Bool).map_err(|_| bad()),
            "s" => Ok(OscType::String(raw.to_string())),
            _ => unreachable!(),
        }
    };

    if let Some((kind @ ("f" | "i" | "b" | "s"), raw)) = value.split_once(':') {
        return typed(kind, raw);
    }
    match value {
        "true" => Ok(OscType::Bool(true)),
        "false" => Ok(OscType::Bool(false)),
        _ => value.parse().map(OscType::Float).map_err(|_| {
            format!("{value:?} is not a number or true/false (use s:{value} to send it as text)")
        }),
    }
}

/// Every message in a packet, with bundles unpacked.
pub fn flatten(packet: OscPacket, out: &mut Vec<OscMessage>) {
    match packet {
        OscPacket::Message(msg) => out.push(msg),
        OscPacket::Bundle(bundle) => {
            for packet in bundle.content {
                flatten(packet, out);
            }
        }
    }
}

pub fn format_arg(arg: &OscType) -> String {
    match arg {
        OscType::Float(v) => format!("{v} (float)"),
        OscType::Double(v) => format!("{v} (double)"),
        OscType::Int(v) => format!("{v} (int)"),
        OscType::Long(v) => format!("{v} (long)"),
        OscType::Bool(v) => format!("{v} (bool)"),
        OscType::String(v) => format!("{v:?} (string)"),
        other => format!("{other:?}"),
    }
}

pub fn format_message(msg: &OscMessage) -> String {
    let args: Vec<String> = msg.args.iter().map(format_arg).collect();
    if args.is_empty() {
        msg.addr.clone()
    } else {
        format!("{}  {}", msg.addr, args.join(", "))
    }
}

/// Seconds since `start`, for timestamps.
pub fn stamp(start: Instant) -> String {
    format!("{:>9.3}s", start.elapsed().as_secs_f64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rosc::{OscBundle, OscTime};

    #[test]
    fn short_names_become_avatar_parameters() {
        assert_eq!(osc_address("Boop"), "/avatar/parameters/Boop");
        assert_eq!(
            osc_address("vibecheck/state"),
            "/avatar/parameters/vibecheck/state"
        );
        assert_eq!(osc_address("/avatar/change"), "/avatar/change");
    }

    #[test]
    fn parses_values() {
        assert_eq!(parse_value("0.5"), Ok(OscType::Float(0.5)));
        assert_eq!(parse_value("1"), Ok(OscType::Float(1.0)));
        assert_eq!(parse_value("-0.25"), Ok(OscType::Float(-0.25)));
        assert_eq!(parse_value("true"), Ok(OscType::Bool(true)));
        assert_eq!(parse_value("false"), Ok(OscType::Bool(false)));
        assert_eq!(parse_value("i:3"), Ok(OscType::Int(3)));
        assert_eq!(parse_value("f:2"), Ok(OscType::Float(2.0)));
        assert_eq!(parse_value("b:false"), Ok(OscType::Bool(false)));
        assert_eq!(
            parse_value("s:avtr_x"),
            Ok(OscType::String("avtr_x".into()))
        );
        assert!(parse_value("hello").is_err());
        assert!(parse_value("i:nope").is_err());
    }

    #[test]
    fn parses_socket_addresses() {
        assert_eq!(
            parse_socket_addr("9002"),
            Ok("127.0.0.1:9002".parse().unwrap())
        );
        assert_eq!(
            parse_socket_addr("0.0.0.0:9001"),
            Ok("0.0.0.0:9001".parse().unwrap())
        );
        assert!(parse_socket_addr("nope").is_err());
    }

    #[test]
    fn unpacks_nested_bundles() {
        let msg = |addr: &str| {
            OscPacket::Message(OscMessage {
                addr: addr.into(),
                args: vec![],
            })
        };
        let bundle = |content| {
            OscPacket::Bundle(OscBundle {
                timetag: OscTime::from((0, 1)),
                content,
            })
        };
        let packet = bundle(vec![msg("/a"), bundle(vec![msg("/b"), msg("/c")])]);

        let mut out = Vec::new();
        flatten(packet, &mut out);
        let addrs: Vec<_> = out.iter().map(|m| m.addr.as_str()).collect();
        assert_eq!(addrs, ["/a", "/b", "/c"]);
    }

    #[test]
    fn formats_message_with_typed_args() {
        let msg = OscMessage {
            addr: "/avatar/parameters/Boop".into(),
            args: vec![OscType::Float(0.5), OscType::Bool(true)],
        };
        assert_eq!(
            format_message(&msg),
            "/avatar/parameters/Boop  0.5 (float), true (bool)"
        );
    }
}
