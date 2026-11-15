/*
 * Problems found by background work (toy handling, OSC, OSCQuery, Bluetooth)
 * that the user should hear about.
 *
 * A task reports a Notice through its ErrorReporter. The notice is logged
 * straight away. The notice handler then adds it to the diagnostics log and
 * shows it in the app, skipping repeats of the same problem for a while so a
 * failure inside a loop doesn't flood the screen.
 *
 * Errors from a command the frontend called go back as that command's
 * result instead, so they aren't shown twice.
 */

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use log::{error as logerr, info, warn};
use tauri::AppHandle;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use vibecheck_core::{Diagnostics, ModuleInstanceId};

use crate::{frontend::error::FrontendError, vcore::ipc::emit_plane::emit_error};

/// The same problem (module and code) is shown at most once per this long.
/// It's still logged every time.
const REPEAT_WINDOW: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    fn label(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// A problem to tell the user about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub severity: Severity,
    /// Where it happened, e.g. "oscquery" or "toys". Also the module shown
    /// in the diagnostics log.
    pub module: &'static str,
    /// Names the problem, e.g. "battery_socket". Repeats are recognised by
    /// module and code, so the message can carry details that change.
    pub code: &'static str,
    /// What the user sees.
    pub message: String,
}

/// Reports notices to the notice handler. Cheap to clone; give one to each
/// task that can run into problems.
#[derive(Clone, Debug)]
pub struct ErrorReporter {
    tx: UnboundedSender<Notice>,
}

impl ErrorReporter {
    /// A reporter and the receiver to hand to [`notice_handler`]. Notices
    /// reported before the handler starts wait for it.
    pub fn channel() -> (Self, UnboundedReceiver<Notice>) {
        let (tx, rx) = unbounded_channel();
        (Self { tx }, rx)
    }

    pub fn report(&self, notice: Notice) {
        let line = format!("[{}] {}: {}", notice.module, notice.code, notice.message);
        match notice.severity {
            Severity::Error => logerr!("{}", line),
            Severity::Warning => warn!("{}", line),
            Severity::Info => info!("{}", line),
        }
        // Only fails once the app is shutting down
        let _ = self.tx.send(notice);
    }

    pub fn error(&self, module: &'static str, code: &'static str, message: impl Into<String>) {
        self.notice(Severity::Error, module, code, message.into());
    }

    pub fn warning(&self, module: &'static str, code: &'static str, message: impl Into<String>) {
        self.notice(Severity::Warning, module, code, message.into());
    }

    pub fn info(&self, module: &'static str, code: &'static str, message: impl Into<String>) {
        self.notice(Severity::Info, module, code, message.into());
    }

    fn notice(
        &self,
        severity: Severity,
        module: &'static str,
        code: &'static str,
        message: String,
    ) {
        self.report(Notice {
            severity,
            module,
            code,
            message,
        });
    }
}

/// Decides which notices are shown: the first of each module and code, then
/// again once [`REPEAT_WINDOW`] has passed.
#[derive(Default)]
struct RepeatFilter {
    last_shown: HashMap<(&'static str, &'static str), Instant>,
}

impl RepeatFilter {
    fn should_show(&mut self, notice: &Notice, now: Instant) -> bool {
        let key = (notice.module, notice.code);
        match self.last_shown.get(&key) {
            Some(last) if now.saturating_duration_since(*last) < REPEAT_WINDOW => false,
            _ => {
                self.last_shown.insert(key, now);
                true
            }
        }
    }
}

/// Adds every notice to the diagnostics log (while it's on) and shows it in
/// the app, skipping recent repeats. Runs for the life of the app.
pub async fn notice_handler(
    app_handle: AppHandle,
    mut notices: UnboundedReceiver<Notice>,
    diagnostics: Diagnostics,
) {
    let mut filter = RepeatFilter::default();
    while let Some(notice) = notices.recv().await {
        diagnostics.log(
            &ModuleInstanceId::new(notice.module),
            format!("{} ({})", notice.severity.label(), notice.code),
            || notice.message.clone(),
        );
        if !filter.should_show(&notice, Instant::now()) {
            continue;
        }
        let shown = match notice.severity {
            Severity::Error => FrontendError::Error(notice.message),
            Severity::Warning => FrontendError::Warning(notice.message),
            Severity::Info => FrontendError::Info(notice.message),
        };
        emit_error(&app_handle, shown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notice(module: &'static str, code: &'static str) -> Notice {
        Notice {
            severity: Severity::Error,
            module,
            code,
            message: "something broke".to_owned(),
        }
    }

    #[test]
    fn repeats_are_shown_once_per_window() {
        let mut filter = RepeatFilter::default();
        let start = Instant::now();
        let battery = notice("toys", "battery_socket");

        assert!(filter.should_show(&battery, start));
        assert!(!filter.should_show(&battery, start + Duration::from_secs(5)));
        assert!(filter.should_show(&battery, start + REPEAT_WINDOW));
    }

    #[test]
    fn different_problems_are_each_shown() {
        let mut filter = RepeatFilter::default();
        let now = Instant::now();

        assert!(filter.should_show(&notice("toys", "battery_socket"), now));
        assert!(filter.should_show(&notice("toys", "save_config"), now));
        assert!(filter.should_show(&notice("oscquery", "battery_socket"), now));
    }

    #[tokio::test]
    async fn reporting_before_the_handler_starts_keeps_the_notice() {
        let (reporter, mut rx) = ErrorReporter::channel();
        reporter.warning("bluetooth", "no_adapter", "No Bluetooth adapter found");

        let got = rx.recv().await.unwrap();
        assert_eq!(got.severity, Severity::Warning);
        assert_eq!(got.code, "no_adapter");
    }
}
