//! Diagnostics: an on/off switch and a log for live troubleshooting views.
//!
//! The signal plane already shows every value an input writes (see
//! [`SignalPlane::changed_since_all`](crate::signal::SignalPlane::changed_since_all)).
//! This log is for what never reaches the plane: messages nothing uses,
//! argument types the plane doesn't keep, connection changes and so on. Any
//! input can add entries through [`InputCtx::log`](crate::module::InputCtx::log).
//!
//! Everything is off until a view turns it on, and logging checks the switch
//! before formatting anything, so it costs next to nothing the rest of the
//! time.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;

use crate::address::ModuleInstanceId;

/// How many entries the log keeps by default.
pub const DEFAULT_LOG_CAPACITY: usize = 2000;

/// One log entry.
#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    /// Position in the log, starting at 1. Pass the last one seen to
    /// [`Diagnostics::entries_since`] to get only newer entries.
    pub seq: u64,
    /// Milliseconds since diagnostics were turned on.
    pub at_ms: u64,
    /// The module that logged it.
    pub module: ModuleInstanceId,
    /// What happened, e.g. an OSC address.
    pub label: String,
    /// Its value as text, e.g. `f 0.5` or `s avtr_123`.
    pub value: String,
}

struct Log {
    entries: VecDeque<LogEntry>,
    capacity: usize,
    seq: u64,
    started: Instant,
}

struct Inner {
    enabled: AtomicBool,
    log: Mutex<Log>,
}

/// Shared diagnostics switch and log. Cheap to clone; clones share
/// everything.
#[derive(Clone)]
pub struct Diagnostics {
    inner: Arc<Inner>,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new(DEFAULT_LOG_CAPACITY)
    }
}

impl Diagnostics {
    /// Off, keeping at most `capacity` log entries once on.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                enabled: AtomicBool::new(false),
                log: Mutex::new(Log {
                    entries: VecDeque::new(),
                    capacity: capacity.max(1),
                    seq: 0,
                    started: Instant::now(),
                }),
            }),
        }
    }

    pub fn enabled(&self) -> bool {
        self.inner.enabled.load(Ordering::Relaxed)
    }

    /// Turning on starts an empty log; turning off empties it.
    pub fn set_enabled(&self, enabled: bool) {
        let mut log = self.inner.log.lock();
        if enabled != self.enabled() {
            log.entries.clear();
            log.started = Instant::now();
        }
        self.inner.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Adds an entry if diagnostics are on. `value` is only called then, so
    /// formatting costs nothing while off. The oldest entry is dropped when
    /// the log is full.
    pub fn log(
        &self,
        module: &ModuleInstanceId,
        label: impl Into<String>,
        value: impl FnOnce() -> String,
    ) {
        if !self.enabled() {
            return;
        }
        let mut log = self.inner.log.lock();
        log.seq += 1;
        let entry = LogEntry {
            seq: log.seq,
            at_ms: log.started.elapsed().as_millis() as u64,
            module: module.clone(),
            label: label.into(),
            value: value(),
        };
        if log.entries.len() == log.capacity {
            log.entries.pop_front();
        }
        log.entries.push_back(entry);
    }

    /// Entries newer than `since` (0 for all), oldest first, and the seq to
    /// pass next time.
    pub fn entries_since(&self, since: u64) -> (Vec<LogEntry>, u64) {
        let log = self.inner.log.lock();
        let entries = log
            .entries
            .iter()
            .filter(|e| e.seq > since)
            .cloned()
            .collect();
        (entries, log.seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module() -> ModuleInstanceId {
        ModuleInstanceId::new("osc")
    }

    #[test]
    fn logs_nothing_and_formats_nothing_while_off() {
        let d = Diagnostics::default();
        d.log(&module(), "/a", || panic!("formatted while off"));
        assert!(d.entries_since(0).0.is_empty());
    }

    #[test]
    fn returns_only_newer_entries() {
        let d = Diagnostics::default();
        d.set_enabled(true);
        d.log(&module(), "/a", || "f 0.1".into());
        let (first, seq) = d.entries_since(0);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].label, "/a");
        assert_eq!(first[0].value, "f 0.1");

        d.log(&module(), "/b", || "T".into());
        let (next, _) = d.entries_since(seq);
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].label, "/b");
    }

    #[test]
    fn drops_the_oldest_entry_when_full() {
        let d = Diagnostics::new(2);
        d.set_enabled(true);
        for label in ["/a", "/b", "/c"] {
            d.log(&module(), label, String::new);
        }
        let labels: Vec<_> = d.entries_since(0).0.into_iter().map(|e| e.label).collect();
        assert_eq!(labels, vec!["/b", "/c"]);
    }

    #[test]
    fn turning_off_empties_the_log() {
        let d = Diagnostics::default();
        d.set_enabled(true);
        d.log(&module(), "/a", String::new);
        d.set_enabled(false);
        d.set_enabled(true);
        assert!(d.entries_since(0).0.is_empty());
    }
}
