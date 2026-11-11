//! Debug tool for VibeCheck's OSC input.
//!
//! - `listen`: print OSC arriving on a port, to check VRChat is sending to
//!   VibeCheck. VibeCheck must be closed while this runs, since both need the
//!   same port.
//! - `send` / `sweep`: send OSC to VibeCheck as if VRChat sent it, to test
//!   toy parameters and the OSC API without VRChat.
//!
//! ```text
//! cargo run -p osc-tool -- listen --filter Boop
//! cargo run -p osc-tool -- send Boop 0.5
//! cargo run -p osc-tool -- sweep Boop 0 1 --back
//! ```

mod format;
mod listen;
mod send;

use std::process::ExitCode;

const USAGE: &str = "\
osc-tool: test VibeCheck's OSC input

USAGE:
    osc-tool listen [ADDRESS] [--filter TEXT]... [--stats]
    osc-tool send PARAM VALUE... [--to ADDRESS] [--repeat N] [--every MS]
    osc-tool sweep PARAM FROM TO [--seconds S] [--rate HZ] [--back] [--to ADDRESS]

COMMANDS:
    listen    Print OSC arriving on a port (default 127.0.0.1:9001). Close VibeCheck first.
    send      Send a message to VibeCheck as if VRChat sent it.
    sweep     Ramp a float parameter, e.g. to test Smooth or Rate mode.

Run osc-tool COMMAND --help for details. With no command, osc-tool listens.";

/// Runs a subcommand: parse its args, show its help, or report its errors.
fn dispatch<T>(
    args: Vec<String>,
    usage: &str,
    parse: impl FnOnce(Vec<String>) -> Result<Option<T>, String>,
    run: impl FnOnce(T) -> Result<(), String>,
) -> ExitCode {
    match parse(args) {
        Ok(Some(opts)) => match run(opts) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        Ok(None) => {
            println!("{usage}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}\n\n{usage}");
            ExitCode::from(2)
        }
    }
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str);

    match command {
        Some("listen") | Some("send") | Some("sweep") => {
            let command = args.remove(0);
            match command.as_str() {
                "listen" => dispatch(args, listen::USAGE, listen::parse_args, listen::run),
                "send" => dispatch(
                    args,
                    send::SEND_USAGE,
                    send::parse_send_args,
                    send::run_send,
                ),
                _ => dispatch(
                    args,
                    send::SWEEP_USAGE,
                    send::parse_sweep_args,
                    send::run_sweep,
                ),
            }
        }
        Some("-h") | Some("--help") | Some("help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        // Not a command or an address: most likely a typo
        Some(other) if !other.starts_with('-') && format::parse_socket_addr(other).is_err() => {
            eprintln!("error: unknown command {other}\n\n{USAGE}");
            ExitCode::from(2)
        }
        // No command: listen, so `osc-tool 9001 --filter Boop` still works
        _ => dispatch(args, listen::USAGE, listen::parse_args, listen::run),
    }
}
