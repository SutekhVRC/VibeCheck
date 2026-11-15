/*
 * Reports panics. A panic in a background task only ends that task, so
 * without this a part of VibeCheck (the graph runner, OSC input) could stop
 * with nothing on screen. Each panic is logged and reported like any other
 * problem (see reporter.rs): shown in the app and added to the diagnostics
 * log, with where it happened.
 */

use std::{any::Any, panic::Location, sync::OnceLock};

use log::error as logerr;

use super::reporter::ErrorReporter;

static REPORTER: OnceLock<ErrorReporter> = OnceLock::new();

/// Reports every panic from now on through `reporter`, then runs the
/// previous hook (which prints to stderr). Call once, at startup.
pub fn install(reporter: ErrorReporter) {
    if REPORTER.set(reporter).is_err() {
        return; // Already installed
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let what = describe(info.payload(), info.location());
        logerr!("VibeCheck panicked: {}", what);
        // So the panic is in the log file even if the app goes down with it
        log::logger().flush();
        if let Some(reporter) = REPORTER.get() {
            reporter.error(
                "app",
                "panic",
                format!(
                    "Something went wrong inside VibeCheck ({what}). If something stops working, restart VibeCheck, and please report this on the Discord or GitHub."
                ),
            );
        }
        previous(info);
    }));
}

/// The panic's message and where it happened, e.g.
/// "index out of bounds at src/osc/logic.rs:42".
fn describe(payload: &dyn Any, location: Option<&Location<'_>>) -> String {
    let message = payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown error".to_owned());
    match location {
        Some(l) => format!("{message} at {}:{}", l.file(), l.line()),
        None => message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_str_and_string_payloads() {
        let here = Location::caller();
        let text = describe(&"boom", Some(here));
        assert!(text.starts_with("boom at "));
        assert!(text.ends_with(&format!(":{}", here.line())));

        assert_eq!(describe(&String::from("bang"), None), "bang");
        assert_eq!(describe(&42, None), "unknown error");
    }
}
