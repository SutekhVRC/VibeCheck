//! The signal plane: continuous values where the latest value wins.
//!
//! OSC can deliver hundreds of values per second. Queuing them would only add
//! latency, so each port holds a single slot. Writers overwrite it; readers
//! (the graph runner, once per tick) see whatever is newest.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::address::{ModuleInstanceId, PortId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Signal {
    Float(f64),
    Bool(bool),
}

impl Signal {
    /// The signal as a level from 0.0 to 1.0. Booleans become 0.0 or 1.0.
    pub fn as_level(self) -> f64 {
        match self {
            Signal::Float(v) => v,
            Signal::Bool(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

/// A value on the signal plane, with when it was written relative to other
/// writes (see [`SignalPlane::changed_since`]).
#[derive(Clone, Copy)]
struct Slot {
    signal: Signal,
    seq: u64,
}

#[derive(Default)]
struct Slots {
    ports: HashMap<PortId, Slot>,
    /// Sequence number of the newest write.
    seq: u64,
}

/// Latest-value slots for every port, shared between writers and the graph runner.
///
/// Cheap to clone; clones share the same slots.
#[derive(Clone, Default)]
pub struct SignalPlane {
    slots: Arc<RwLock<Slots>>,
}

/// A position in the plane's write order, returned by
/// [`SignalPlane::changed_since`]. Pass it back in to get only newer writes.
/// [`Default`] is "before the first write".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignalSeq(u64);

impl SignalPlane {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overwrite the port's slot. Older values are discarded, never queued.
    pub fn write(&self, port: &PortId, signal: Signal) {
        let mut slots = self.slots.write();
        slots.seq += 1;
        let seq = slots.seq;
        slots.ports.insert(port.clone(), Slot { signal, seq });
    }

    /// The newest value written to the port, if any.
    pub fn read(&self, port: &PortId) -> Option<Signal> {
        self.slots.read().ports.get(port).map(|slot| slot.signal)
    }

    /// Ports of `module` whose name starts with `prefix` and that were written
    /// after `since`, with their latest value, oldest write first. Also
    /// returns where to continue from next time.
    ///
    /// A port written several times since `since` appears once, with its
    /// latest value. Cleared ports don't appear.
    pub fn changed_since(
        &self,
        module: &ModuleInstanceId,
        prefix: &str,
        since: SignalSeq,
    ) -> (Vec<(PortId, Signal)>, SignalSeq) {
        let slots = self.slots.read();
        let mut changed: Vec<(u64, PortId, Signal)> = slots
            .ports
            .iter()
            .filter(|(port, slot)| {
                slot.seq > since.0 && port.module == *module && port.port.starts_with(prefix)
            })
            .map(|(port, slot)| (slot.seq, port.clone(), slot.signal))
            .collect();
        changed.sort_unstable_by_key(|(seq, _, _)| *seq);
        let changed = changed
            .into_iter()
            .map(|(_, port, signal)| (port, signal))
            .collect();
        (changed, SignalSeq(slots.seq))
    }

    /// Forget a port's value, e.g. when its module is removed.
    pub fn clear(&self, port: &PortId) {
        self.slots.write().ports.remove(port);
    }

    /// Forget every value written to `module`'s ports (e.g. all OSC parameters
    /// when the avatar changes).
    pub fn clear_module(&self, module: &ModuleInstanceId) {
        self.slots
            .write()
            .ports
            .retain(|port, _| port.module != *module);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_value_wins() {
        let plane = SignalPlane::new();
        let port = PortId::new(ModuleInstanceId::new("osc"), "Contact");

        assert_eq!(plane.read(&port), None);
        plane.write(&port, Signal::Float(0.82));
        plane.write(&port, Signal::Float(0.85));
        assert_eq!(plane.read(&port), Some(Signal::Float(0.85)));

        plane.clear(&port);
        assert_eq!(plane.read(&port), None);
    }

    #[test]
    fn clear_module_only_clears_that_module() {
        let plane = SignalPlane::new();
        let osc = PortId::new(ModuleInstanceId::new("osc"), "Boop");
        let toy = PortId::new(
            ModuleInstanceId::new("toy:x:0"),
            "Vibrator_0/input_processor",
        );
        plane.write(&osc, Signal::Float(0.5));
        plane.write(&toy, Signal::Float(0.5));

        plane.clear_module(&ModuleInstanceId::new("osc"));
        assert_eq!(plane.read(&osc), None);
        assert_eq!(plane.read(&toy), Some(Signal::Float(0.5)));
    }

    #[test]
    fn clones_share_slots() {
        let plane = SignalPlane::new();
        let writer = plane.clone();
        let port = PortId::new(ModuleInstanceId::new("osc"), "Boop");

        writer.write(&port, Signal::Bool(true));
        assert_eq!(plane.read(&port).map(Signal::as_level), Some(1.0));
    }

    #[test]
    fn changed_since_returns_each_new_value_once_in_write_order() {
        let plane = SignalPlane::new();
        let osc = ModuleInstanceId::new("osc");
        let port = |name: &str| PortId::new(osc.clone(), name);

        plane.write(&port("/ogb/a"), Signal::Float(0.1));
        plane.write(&port("/ogb/b"), Signal::Float(0.2));
        plane.write(&port("/other"), Signal::Float(0.3));
        plane.write(&port("/ogb/a"), Signal::Float(0.4));
        plane.write(
            &PortId::new(ModuleInstanceId::new("toy"), "/ogb/a"),
            Signal::Float(0.5),
        );

        let (changed, seq) = plane.changed_since(&osc, "/ogb/", SignalSeq::default());
        assert_eq!(
            changed,
            vec![
                (port("/ogb/b"), Signal::Float(0.2)),
                (port("/ogb/a"), Signal::Float(0.4)),
            ]
        );

        // Nothing new
        assert!(plane.changed_since(&osc, "/ogb/", seq).0.is_empty());

        plane.write(&port("/ogb/b"), Signal::Bool(true));
        let (changed, _) = plane.changed_since(&osc, "/ogb/", seq);
        assert_eq!(changed, vec![(port("/ogb/b"), Signal::Bool(true))]);
    }
}
