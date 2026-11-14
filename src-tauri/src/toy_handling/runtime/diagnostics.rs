/*
 * Sends live values to the diagnostics view.
 *
 * One task per listening session. While diagnostics are on (the view is
 * open), ten times a second it gathers:
 * - values on the signal plane that changed (every input's values, e.g. each
 *   OSC parameter), with how often each was written, and whether it was read
 *   from OSCQuery rather than sent over OSC;
 * - the values inside every toy's graph, copied out by the graph runner;
 * - new diagnostics log entries (e.g. every OSC message, including ones
 *   nothing uses),
 * and emits them as one `fe_diagnostics` event.
 *
 * The first batch of a session, and the first after the plane was cleared
 * (avatar change), has `reset` set and carries every value, so the view
 * never shows values from a previous session or avatar.
 */

use std::time::Duration;

use log::error as logerr;
use tauri::{AppHandle, Emitter};
use tokio::time::{interval, MissedTickBehavior};
use vibecheck_core::{Diagnostics, GraphSnapshot, Signal, SignalPlane, SignalSeq};

use crate::{
    frontend::frontend_types::{
        FeDiagLogEntry, FeDiagPort, FeDiagSent, FeDiagSignal, FeDiagToy, FeDiagnostics,
    },
    osc::avatar_query::AvatarQuery,
    toy_handling::runtime::{graph_runner::GraphValues, toy_graph::OSC_MODULE},
};

/// How often batches are sent.
const SEND_EVERY: Duration = Duration::from_millis(100);

/// What a session's diagnostics task has already sent.
#[derive(Default)]
struct Sent {
    /// Whether the next batch must start over.
    reset: bool,
    signals: SignalSeq,
    clears: u64,
    log: u64,
}

pub async fn diagnostics_task(
    diagnostics: Diagnostics,
    signals: SignalPlane,
    graph_values: GraphValues,
    avatar_query: AvatarQuery,
    app_handle: AppHandle,
) {
    let mut ticker = interval(SEND_EVERY);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut sent = Sent {
        reset: true,
        ..Default::default()
    };
    loop {
        ticker.tick().await;
        if !diagnostics.enabled() {
            // Start over when the view opens again
            sent = Sent {
                reset: true,
                ..Default::default()
            };
            continue;
        }
        let from_oscquery = |addr: &str| avatar_query.was_filled(addr);
        let batch = next_batch(
            &mut sent,
            &diagnostics,
            &signals,
            &graph_values,
            &from_oscquery,
        );
        if let Err(e) = app_handle.emit("fe_diagnostics", &batch) {
            logerr!("Emit diagnostics failed: {}", e);
        }
    }
}

/// The next batch to send, updating what's been sent. `filled` says whether
/// an OSC address was filled in from OSCQuery.
fn next_batch(
    sent: &mut Sent,
    diagnostics: &Diagnostics,
    signals: &SignalPlane,
    graph_values: &GraphValues,
    filled: &dyn Fn(&str) -> bool,
) -> FeDiagnostics {
    let clears = signals.clears();
    let reset = sent.reset || clears != sent.clears;
    if reset {
        sent.signals = SignalSeq::default();
    }
    sent.reset = false;
    sent.clears = clears;

    let (changed, seq) = signals.changed_since_all(sent.signals);
    sent.signals = seq;
    let (log, log_seq) = diagnostics.entries_since(sent.log);
    sent.log = log_seq;

    let mut toys: Vec<FeDiagToy> = graph_values
        .latest()
        .into_iter()
        .map(|(toy_id, snapshot)| fe_toy(toy_id, snapshot))
        .collect();
    toys.sort_by_key(|toy| toy.toy_id);

    FeDiagnostics {
        reset,
        signals: changed
            .into_iter()
            .map(|v| FeDiagSignal {
                module: v.port.module.to_string(),
                port: v.port.port.to_string(),
                value: v.signal.as_level(),
                is_bool: matches!(v.signal, Signal::Bool(_)),
                writes: v.writes,
                // A filled-in value is written once; any later write came
                // from VRChat over OSC
                from_oscquery: v.writes == 1
                    && v.port.module.as_str() == OSC_MODULE
                    && filled(&v.port.port),
            })
            .collect(),
        toys,
        log: log
            .into_iter()
            .map(|e| FeDiagLogEntry {
                seq: e.seq,
                at_ms: e.at_ms,
                module: e.module.to_string(),
                label: e.label,
                value: e.value,
            })
            .collect(),
    }
}

fn fe_toy(toy_id: u32, snapshot: GraphSnapshot) -> FeDiagToy {
    FeDiagToy {
        toy_id,
        ports: snapshot
            .outputs
            .into_iter()
            .map(|(port, value)| FeDiagPort {
                name: port.to_string(),
                value: value.map(Signal::as_level),
            })
            .collect(),
        sent: snapshot
            .sinks
            .into_iter()
            .map(|sink| FeDiagSent {
                from: sink.from.to_string(),
                motor: sink.target.to_string(),
                value: sink.last,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibecheck_core::{ModuleInstanceId, PortId};

    fn osc(addr: &str) -> PortId {
        PortId::new(ModuleInstanceId::new("osc"), addr)
    }

    #[test]
    fn sends_everything_first_then_only_changes_and_starts_over_after_a_clear() {
        let diagnostics = Diagnostics::default();
        diagnostics.set_enabled(true);
        let signals = SignalPlane::new();
        let graphs = GraphValues::default();
        let mut sent = Sent {
            reset: true,
            ..Default::default()
        };
        let filled = |addr: &str| addr == "/b";
        signals.write(&osc("/a"), Signal::Float(0.25));
        signals.write(&osc("/b"), Signal::Bool(true));
        diagnostics.log(&ModuleInstanceId::new("osc"), "/a", || {
            "0.25 (float)".into()
        });

        let first = next_batch(&mut sent, &diagnostics, &signals, &graphs, &filled);
        assert!(first.reset);
        assert_eq!(first.signals.len(), 2);
        assert!(first.signals[1].is_bool);
        assert!(!first.signals[0].from_oscquery);
        assert!(first.signals[1].from_oscquery);
        assert_eq!(first.log.len(), 1);

        signals.write(&osc("/a"), Signal::Float(0.5));
        let second = next_batch(&mut sent, &diagnostics, &signals, &graphs, &filled);
        assert!(!second.reset);
        assert_eq!(second.signals.len(), 1);
        assert_eq!(second.signals[0].value, 0.5);
        assert_eq!(second.signals[0].writes, 2);

        // VRChat sends the filled-in value itself: it's no longer marked
        signals.write(&osc("/b"), Signal::Bool(false));
        let sent_by_vrchat = next_batch(&mut sent, &diagnostics, &signals, &graphs, &filled);
        assert!(!sent_by_vrchat.signals[0].from_oscquery);
        assert!(second.log.is_empty());

        // Avatar change: the old values are gone, so the view starts over
        signals.clear_module(&ModuleInstanceId::new("osc"));
        signals.write(&osc("/c"), Signal::Float(1.0));
        let third = next_batch(&mut sent, &diagnostics, &signals, &graphs, &filled);
        assert!(third.reset);
        assert_eq!(third.signals.len(), 1);
        assert_eq!(third.signals[0].port, "/c");
    }
}
