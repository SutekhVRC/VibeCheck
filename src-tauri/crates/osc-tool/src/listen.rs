//! `osc-tool listen`: print OSC messages arriving on a port.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use crate::format::{flatten, format_message, parse_socket_addr, stamp};

const DEFAULT_ADDR: &str = "127.0.0.1:9001";
const QUIET_HINT_AFTER: Duration = Duration::from_secs(5);

pub const USAGE: &str = "\
osc-tool listen: print OSC messages arriving on a UDP port

USAGE:
    osc-tool listen [ADDRESS] [--filter TEXT]... [--stats]

ARGS:
    ADDRESS          host:port or just a port to listen on [default: 127.0.0.1:9001]

OPTIONS:
    --filter TEXT    Only show addresses containing TEXT (case-insensitive). Repeat to allow several.
    --stats          Print one summary line per second instead of every message.
    -h, --help       Show this help.

Close VibeCheck first: it holds the same port, even while switched off.";

#[derive(Debug, PartialEq)]
pub struct ListenOptions {
    pub addr: SocketAddr,
    pub filters: Vec<String>,
    pub stats: bool,
}

/// `Ok(None)` means help was asked for.
pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<ListenOptions>, String> {
    let mut addr = None;
    let mut filters = Vec::new();
    let mut stats = false;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--stats" => stats = true,
            "--filter" => {
                let text = args.next().ok_or("--filter needs a value")?;
                filters.push(text.to_lowercase());
            }
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            value => {
                if addr.is_some() {
                    return Err(format!("unexpected argument {value}"));
                }
                addr = Some(parse_socket_addr(value)?);
            }
        }
    }

    Ok(Some(ListenOptions {
        addr: addr.unwrap_or_else(|| DEFAULT_ADDR.parse().unwrap()),
        filters,
        stats,
    }))
}

fn matches(filters: &[String], addr: &str) -> bool {
    filters.is_empty() || {
        let addr = addr.to_lowercase();
        filters.iter().any(|f| addr.contains(f))
    }
}

fn bind(addr: SocketAddr) -> Result<UdpSocket, String> {
    UdpSocket::bind(addr).map_err(|e| match e.kind() {
        ErrorKind::AddrInUse => format!(
            "{addr} is already in use.\n\
             VibeCheck holds this port while it's open, even when switched off. Close it fully \
             (including the tray icon) and try again."
        ),
        ErrorKind::AddrNotAvailable => format!(
            "{addr} isn't an address on this computer. Use 127.0.0.1 or one of this \
             computer's own IP addresses."
        ),
        _ => format!("couldn't listen on {addr}: {e}"),
    })
}

fn print_quiet_hint(addr: SocketAddr) {
    eprintln!(
        "\nNo OSC received on {addr} yet. Things to check:\n\
         - OSC is enabled in VRChat: Action Menu > Options > OSC > Enabled.\n\
         - VRChat is sending to this port. It sends to port 9001 by default; the --osc launch option can change that.\n\
         - Something else isn't already receiving on this port (another OSC app or router).\n\
         - Move a little or touch a contact: VRChat only sends parameters when they change.\n"
    );
}

pub fn run(opts: ListenOptions) -> Result<(), String> {
    let socket = bind(opts.addr)?;
    socket
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;

    eprintln!("Listening for OSC on {}. Press Ctrl+C to stop.", opts.addr);
    if !opts.filters.is_empty() {
        eprintln!("Showing addresses containing: {}", opts.filters.join(", "));
    }

    let start = Instant::now();
    let mut buf = [0u8; rosc::decoder::MTU];
    let mut messages = Vec::new();

    let mut total: u64 = 0;
    let mut hinted = false;

    // --stats counters for the current second
    let mut window_start = Instant::now();
    let mut window_count: u64 = 0;
    let mut window_addrs = BTreeSet::new();

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, from)) => match rosc::decoder::decode_udp(&buf[..len]) {
                Ok((_, packet)) => {
                    messages.clear();
                    flatten(packet, &mut messages);
                    for msg in messages.drain(..) {
                        total += 1;
                        if !matches(&opts.filters, &msg.addr) {
                            continue;
                        }
                        if opts.stats {
                            window_count += 1;
                            window_addrs.insert(msg.addr);
                        } else {
                            if msg.addr == "/avatar/change" {
                                println!("{}  ---- avatar changed ----", stamp(start));
                            }
                            println!("{}  {}", stamp(start), format_message(&msg));
                        }
                    }
                }
                Err(e) => eprintln!(
                    "{}  {len} bytes from {from} that aren't valid OSC: {e:?}",
                    stamp(start)
                ),
            },
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(e) => return Err(format!("receive failed: {e}")),
        }

        if total == 0 && !hinted && start.elapsed() >= QUIET_HINT_AFTER {
            print_quiet_hint(opts.addr);
            hinted = true;
        }

        if opts.stats && window_start.elapsed() >= Duration::from_secs(1) {
            println!(
                "{}  {} msg/s across {} addresses ({} total)",
                stamp(start),
                window_count,
                window_addrs.len(),
                total
            );
            window_start = Instant::now();
            window_count = 0;
            window_addrs.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_to_vrchat_send_port() {
        let opts = parse_args(args(&[])).unwrap().unwrap();
        assert_eq!(opts.addr, "127.0.0.1:9001".parse().unwrap());
        assert!(opts.filters.is_empty());
        assert!(!opts.stats);
    }

    #[test]
    fn parses_port_address_and_flags() {
        let opts = parse_args(args(&["9002", "--filter", "Boop", "--stats"]))
            .unwrap()
            .unwrap();
        assert_eq!(opts.addr, "127.0.0.1:9002".parse().unwrap());
        assert_eq!(opts.filters, vec!["boop"]);
        assert!(opts.stats);

        assert_eq!(parse_args(args(&["--help"])).unwrap(), None);
        assert!(parse_args(args(&["--filter"])).is_err());
        assert!(parse_args(args(&["nope"])).is_err());
        assert!(parse_args(args(&["9001", "9002"])).is_err());
    }

    #[test]
    fn filters_are_case_insensitive_substrings() {
        let filters = vec!["boop".to_string()];
        assert!(matches(&filters, "/avatar/parameters/Boop"));
        assert!(!matches(&filters, "/avatar/parameters/Tail"));
        assert!(matches(&[], "/anything"));
    }
}
