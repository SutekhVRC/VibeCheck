//! `osc-tool avatar`: save the current avatar's parameters from VRChat's
//! OSCQuery server to a file, e.g. as a test fixture.
//!
//! Only reads from VRChat (mDNS lookup and one HTTP request). It doesn't
//! advertise anything or open an OSC port, so it's safe while VRChat runs.
//!
//! The file is VRChat's reply exactly as sent, not re-encoded, so it shows
//! anything a parser might lose (e.g. a node with both a value and
//! children).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use vrcoscquery::discovery;

const DEFAULT_OUT: &str = "avatar-parameters.json";
const PARAMETERS_PATH: &str = "/avatar/parameters";
const TIMEOUT: Duration = Duration::from_secs(5);

pub const USAGE: &str = "\
osc-tool avatar: save the current avatar's parameters from VRChat's OSCQuery

USAGE:
    osc-tool avatar [--out FILE]

OPTIONS:
    --out FILE    Where to save the parameter tree (JSON) [default: avatar-parameters.json]
    -h, --help    Show this help.

VRChat must be running with OSC on. This only reads from VRChat, so it's safe
to run while VRChat and VibeCheck are open. The file holds parameter names,
types and current values; it doesn't include the avatar's id.";

#[derive(Debug, PartialEq)]
pub struct AvatarOptions {
    pub out: PathBuf,
}

/// `Ok(None)` means help was asked for.
pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<AvatarOptions>, String> {
    let mut out = PathBuf::from(DEFAULT_OUT);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--out" => out = args.next().ok_or("--out needs a file name")?.into(),
            other => return Err(format!("unexpected argument {other}")),
        }
    }
    Ok(Some(AvatarOptions { out }))
}

pub fn run(opts: AvatarOptions) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("couldn't start: {e}"))?;
    runtime.block_on(async {
        println!("Looking for VRChat's OSCQuery server...");
        let vrchat = discovery::find_vrchat(TIMEOUT)
            .await
            .map_err(|e| format!("looking for VRChat failed: {e}"))?
            .ok_or("VRChat wasn't found. Is it running, with OSC on?")?;
        let body = http_get(vrchat.addr, PARAMETERS_PATH)
            .map_err(|e| format!("reading the avatar's parameters failed: {e}"))?;
        let json: serde_json::Value =
            serde_json::from_slice(&body).map_err(|e| format!("VRChat's reply isn't JSON: {e}"))?;
        let (parameters, both) = count(&json);
        std::fs::write(&opts.out, &body)
            .map_err(|e| format!("couldn't write {}: {e}", opts.out.display()))?;
        println!("Saved {parameters} parameters to {}", opts.out.display());
        if both > 0 {
            println!("{both} of them also have parameters under them");
        }
        Ok(())
    })
}

/// The body of `GET path` from `addr`. A plain HTTP/1.0 request, so the
/// reply is saved exactly as VRChat sent it.
fn http_get(addr: SocketAddr, path: &str) -> Result<Vec<u8>, String> {
    let mut stream = TcpStream::connect_timeout(&addr, TIMEOUT).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    write!(
        stream,
        "GET {path} HTTP/1.0\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
    )
    .map_err(|e| e.to_string())?;
    let mut reply = Vec::new();
    stream.read_to_end(&mut reply).map_err(|e| e.to_string())?;
    let split = reply
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("the reply had no body")?;
    let status = String::from_utf8_lossy(&reply[..split]);
    if !status.starts_with("HTTP/1.1 200") && !status.starts_with("HTTP/1.0 200") {
        return Err(format!(
            "VRChat answered {}",
            status.lines().next().unwrap_or("")
        ));
    }
    Ok(reply[split + 4..].to_vec())
}

/// How many parameters (nodes with a value) are in an OSCQuery tree, and
/// how many of those also have children.
fn count(node: &serde_json::Value) -> (usize, usize) {
    let has_value = node.get("TYPE").is_some();
    let children = node.get("CONTENTS").and_then(|c| c.as_object());
    let (mut parameters, mut both) = (usize::from(has_value), 0);
    if let Some(children) = children {
        both += usize::from(has_value);
        for child in children.values() {
            let (p, b) = count(child);
            parameters += p;
            both += b;
        }
    }
    (parameters, both)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn counts_parameters_and_ones_with_children() {
        let tree: serde_json::Value = serde_json::from_str(
            r#"{ "FULL_PATH": "/avatar/parameters", "CONTENTS": {
                "Boop": { "TYPE": "f", "VALUE": [0.0] },
                "A": { "TYPE": "T", "VALUE": [true], "CONTENTS": {
                    "B": { "TYPE": "f", "VALUE": [1.0] }
                }}
            }}"#,
        )
        .unwrap();
        assert_eq!(count(&tree), (3, 1));
    }

    #[test]
    fn parses_out_and_help() {
        assert_eq!(
            parse_args(args(&[])).unwrap(),
            Some(AvatarOptions {
                out: DEFAULT_OUT.into()
            })
        );
        assert_eq!(
            parse_args(args(&["--out", "sps3.json"])).unwrap(),
            Some(AvatarOptions {
                out: "sps3.json".into()
            })
        );
        assert_eq!(parse_args(args(&["--help"])).unwrap(), None);
        assert!(parse_args(args(&["--out"])).is_err());
        assert!(parse_args(args(&["extra"])).is_err());
    }
}
