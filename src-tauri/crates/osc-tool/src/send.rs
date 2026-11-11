//! `osc-tool send` and `osc-tool sweep`: send OSC to VibeCheck as if it came
//! from VRChat.
//!
//! Sending uses a temporary local port, so it works while VibeCheck is running.

use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, Instant};

use rosc::{OscMessage, OscPacket, OscType};

use crate::format::{format_message, osc_address, parse_socket_addr, parse_value, stamp};

const DEFAULT_TO: &str = "127.0.0.1:9001";

pub const SEND_USAGE: &str = "\
osc-tool send: send one OSC message, as if VRChat sent it

USAGE:
    osc-tool send PARAM VALUE... [--to ADDRESS] [--repeat N] [--every MS]

ARGS:
    PARAM            Parameter name (Boop) or full address (/avatar/parameters/Boop).
                     Names without a leading / get /avatar/parameters/ in front.
    VALUE            true / false, a number (sent as a float), or typed: f:0.5 i:3 b:true s:text

OPTIONS:
    --to ADDRESS     host:port or port to send to [default: 127.0.0.1:9001, VibeCheck's OSC Bind]
    --repeat N       Send N times [default: 1]
    --every MS       Milliseconds between repeats [default: 1000]
    -h, --help       Show this help.

EXAMPLES:
    osc-tool send Boop true                 drive a Constant-mode parameter
    osc-tool send Boop 0.5                  drive a Raw-mode parameter at 50%
    osc-tool send vibecheck/state true      turn VibeCheck on (OSC API)
    osc-tool send vibecheck/api/state false turn VibeCheck off (OSC API)
    osc-tool send vibecheck/api/anatomy/Vagina/enabled false
                                            disable toys tagged Vagina (OSC API)";

pub const SWEEP_USAGE: &str = "\
osc-tool sweep: ramp a float parameter from one value to another

USAGE:
    osc-tool sweep PARAM FROM TO [--seconds S] [--rate HZ] [--to ADDRESS] [--back]

ARGS:
    PARAM            Parameter name or full address, as for send.
    FROM, TO         Start and end values, e.g. 0 1.

OPTIONS:
    --seconds S      How long the ramp takes [default: 2]
    --rate HZ        Messages per second [default: 20]
    --back           Ramp back down to FROM afterwards.
    --to ADDRESS     host:port or port to send to [default: 127.0.0.1:9001]
    -h, --help       Show this help.

EXAMPLES:
    osc-tool sweep Boop 0 1                 slow ramp, to check Range and Smooth
    osc-tool sweep Boop 0 1 --seconds 0.3 --back
                                            fast stroke, to check Rate mode";

#[derive(Debug, PartialEq)]
pub struct SendOptions {
    pub to: SocketAddr,
    pub addr: String,
    pub args: Vec<OscType>,
    pub repeat: u32,
    pub every: Duration,
}

#[derive(Debug, PartialEq)]
pub struct SweepOptions {
    pub to: SocketAddr,
    pub addr: String,
    pub from: f32,
    pub until: f32,
    pub seconds: f32,
    pub rate: f32,
    pub back: bool,
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag} needs a value"))
}

fn parse_number<T: std::str::FromStr>(value: &str, flag: &str) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{flag} expects a number, got {value:?}"))
}

/// `Ok(None)` means help was asked for.
pub fn parse_send_args(
    args: impl IntoIterator<Item = String>,
) -> Result<Option<SendOptions>, String> {
    let mut to = None;
    let mut repeat = 1;
    let mut every = Duration::from_secs(1);
    let mut positional = Vec::new();

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--to" => to = Some(parse_socket_addr(&next_value(&mut args, "--to")?)?),
            "--repeat" => repeat = parse_number(&next_value(&mut args, "--repeat")?, "--repeat")?,
            "--every" => {
                every = Duration::from_millis(parse_number(
                    &next_value(&mut args, "--every")?,
                    "--every",
                )?)
            }
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            _ => positional.push(arg),
        }
    }

    let mut positional = positional.into_iter();
    let addr = osc_address(&positional.next().ok_or("missing PARAM")?);
    let args = positional
        .map(|v| parse_value(&v))
        .collect::<Result<Vec<_>, _>>()?;
    if args.is_empty() {
        return Err("missing VALUE".into());
    }
    if repeat == 0 {
        return Err("--repeat must be at least 1".into());
    }

    Ok(Some(SendOptions {
        to: to.unwrap_or_else(|| DEFAULT_TO.parse().unwrap()),
        addr,
        args,
        repeat,
        every,
    }))
}

/// `Ok(None)` means help was asked for.
pub fn parse_sweep_args(
    args: impl IntoIterator<Item = String>,
) -> Result<Option<SweepOptions>, String> {
    let mut to = None;
    let mut seconds: f32 = 2.0;
    let mut rate: f32 = 20.0;
    let mut back = false;
    let mut positional = Vec::new();

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--to" => to = Some(parse_socket_addr(&next_value(&mut args, "--to")?)?),
            "--seconds" => {
                seconds = parse_number(&next_value(&mut args, "--seconds")?, "--seconds")?
            }
            "--rate" => rate = parse_number(&next_value(&mut args, "--rate")?, "--rate")?,
            "--back" => back = true,
            // Only `--` starts a flag, so negative numbers like -1 are values
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            _ => positional.push(arg),
        }
    }

    let [param, from, until]: [String; 3] = positional
        .try_into()
        .map_err(|_| "expected PARAM FROM TO".to_string())?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err("--seconds must be more than 0".into());
    }
    if !rate.is_finite() || rate <= 0.0 || rate > 1000.0 {
        return Err("--rate must be between 0 and 1000".into());
    }

    Ok(Some(SweepOptions {
        to: to.unwrap_or_else(|| DEFAULT_TO.parse().unwrap()),
        addr: osc_address(&param),
        from: parse_number(&from, "FROM")?,
        until: parse_number(&until, "TO")?,
        seconds,
        rate,
        back,
    }))
}

/// The values a sweep sends, in order, ending exactly on `until` (and on
/// `from` again with `back`). Values are rounded to hundredths, like VibeCheck
/// does on receipt, so the printout shows what VibeCheck actually sees.
pub fn sweep_values(opts: &SweepOptions) -> Vec<f32> {
    let steps = ((opts.seconds * opts.rate).round() as usize).max(1);
    let ramp = |a: f32, b: f32| -> Vec<f32> {
        (0..=steps)
            .map(|i| a + (b - a) * (i as f32 / steps as f32))
            .map(|v| (v * 100.0).round() / 100.0)
            .collect()
    };
    let mut values = ramp(opts.from, opts.until);
    if opts.back {
        values.extend(ramp(opts.until, opts.from).into_iter().skip(1));
    }
    values
}

struct Sender {
    socket: UdpSocket,
    to: SocketAddr,
}

impl Sender {
    fn new(to: SocketAddr) -> Result<Self, String> {
        let local: SocketAddr = if to.is_ipv4() {
            "0.0.0.0:0".parse().unwrap()
        } else {
            "[::]:0".parse().unwrap()
        };
        let socket = UdpSocket::bind(local).map_err(|e| format!("couldn't open a socket: {e}"))?;
        Ok(Self { socket, to })
    }

    fn send(&self, msg: OscMessage) -> Result<OscMessage, String> {
        let buf = rosc::encoder::encode(&OscPacket::Message(msg.clone()))
            .map_err(|e| format!("couldn't encode OSC: {e:?}"))?;
        self.socket
            .send_to(&buf, self.to)
            .map_err(|e| format!("couldn't send to {}: {e}", self.to))?;
        Ok(msg)
    }
}

pub fn run_send(opts: SendOptions) -> Result<(), String> {
    let sender = Sender::new(opts.to)?;
    let start = Instant::now();
    for i in 0..opts.repeat {
        if i > 0 {
            thread::sleep(opts.every);
        }
        let sent = sender.send(OscMessage {
            addr: opts.addr.clone(),
            args: opts.args.clone(),
        })?;
        println!(
            "{}  -> {}  {}",
            stamp(start),
            opts.to,
            format_message(&sent)
        );
    }
    print_delivery_note();
    Ok(())
}

pub fn run_sweep(opts: SweepOptions) -> Result<(), String> {
    let sender = Sender::new(opts.to)?;
    let values = sweep_values(&opts);
    let interval = Duration::from_secs_f32(1.0 / opts.rate);
    eprintln!(
        "Sweeping {} from {} to {}{} over {}s at {} Hz ({} messages) -> {}",
        opts.addr,
        opts.from,
        opts.until,
        if opts.back { " and back" } else { "" },
        opts.seconds,
        opts.rate,
        values.len(),
        opts.to
    );

    let start = Instant::now();
    for (i, value) in values.into_iter().enumerate() {
        // Pace against the start time so delays don't add up
        let due = interval * i as u32;
        if let Some(wait) = due.checked_sub(start.elapsed()) {
            thread::sleep(wait);
        }
        let sent = sender.send(OscMessage {
            addr: opts.addr.clone(),
            args: vec![OscType::Float(value)],
        })?;
        println!("{}  {}", stamp(start), format_message(&sent));
    }
    print_delivery_note();
    Ok(())
}

fn print_delivery_note() {
    eprintln!(
        "\nOSC over UDP has no delivery receipt: \"sent\" doesn't mean anything was listening.\n\
         If VibeCheck didn't react, check it's switched on, the toy has a feature using this \
         parameter, and --to matches VibeCheck's OSC Bind."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_send() {
        let opts = parse_send_args(args(&["Boop", "0.5"])).unwrap().unwrap();
        assert_eq!(opts.to, "127.0.0.1:9001".parse().unwrap());
        assert_eq!(opts.addr, "/avatar/parameters/Boop");
        assert_eq!(opts.args, vec![OscType::Float(0.5)]);
        assert_eq!(opts.repeat, 1);

        let opts = parse_send_args(args(&[
            "/avatar/change",
            "s:avtr_x",
            "--to",
            "9002",
            "--repeat",
            "3",
            "--every",
            "250",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(opts.addr, "/avatar/change");
        assert_eq!(opts.to, "127.0.0.1:9002".parse().unwrap());
        assert_eq!(opts.repeat, 3);
        assert_eq!(opts.every, Duration::from_millis(250));

        assert!(parse_send_args(args(&[])).is_err());
        assert!(parse_send_args(args(&["Boop"])).is_err());
        assert!(parse_send_args(args(&["Boop", "1", "--repeat", "0"])).is_err());
        assert_eq!(parse_send_args(args(&["--help"])).unwrap(), None);
    }

    #[test]
    fn parses_sweep_including_negative_values() {
        let opts = parse_sweep_args(args(&["Boop", "-1", "1", "--seconds", "1", "--back"]))
            .unwrap()
            .unwrap();
        assert_eq!(opts.addr, "/avatar/parameters/Boop");
        assert_eq!((opts.from, opts.until), (-1.0, 1.0));
        assert_eq!(opts.seconds, 1.0);
        assert!(opts.back);

        assert!(parse_sweep_args(args(&["Boop", "0"])).is_err());
        assert!(parse_sweep_args(args(&["Boop", "0", "1", "--seconds", "0"])).is_err());
        assert!(parse_sweep_args(args(&["Boop", "0", "1", "--rate", "0"])).is_err());
    }

    #[test]
    fn sweep_hits_both_ends() {
        let mut opts = parse_sweep_args(args(&["Boop", "0", "1", "--seconds", "1", "--rate", "4"]))
            .unwrap()
            .unwrap();
        assert_eq!(sweep_values(&opts), vec![0.0, 0.25, 0.5, 0.75, 1.0]);

        opts.back = true;
        assert_eq!(
            sweep_values(&opts),
            vec![0.0, 0.25, 0.5, 0.75, 1.0, 0.75, 0.5, 0.25, 0.0]
        );
    }
}
