//! The signal plane: continuous values where the latest value wins.
//!
//! OSC can deliver hundreds of values per second. Queuing them would only add
//! latency, so each port holds a single slot. Writers overwrite it; readers
//! (the graph runner, once per tick) see whatever is newest.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::address::PortId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Signal {
    Float(f32),
    Bool(bool),
}

impl Signal {
    /// The signal as a level from 0.0 to 1.0. Booleans become 0.0 or 1.0.
    pub fn as_level(self) -> f32 {
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

/// Latest-value slots for every port, shared between writers and the graph runner.
///
/// Cheap to clone; clones share the same slots.
#[derive(Clone, Default)]
pub struct SignalPlane {
    slots: Arc<RwLock<HashMap<PortId, Signal>>>,
}

impl SignalPlane {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overwrite the port's slot. Older values are discarded, never queued.
    pub fn write(&self, port: &PortId, signal: Signal) {
        self.slots.write().insert(port.clone(), signal);
    }

    /// The newest value written to the port, if any.
    pub fn read(&self, port: &PortId) -> Option<Signal> {
        self.slots.read().get(port).copied()
    }

    /// Forget a port's value, e.g. when its module is removed.
    pub fn clear(&self, port: &PortId) {
        self.slots.write().remove(port);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::ModuleInstanceId;

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
    fn clones_share_slots() {
        let plane = SignalPlane::new();
        let writer = plane.clone();
        let port = PortId::new(ModuleInstanceId::new("osc"), "Boop");

        writer.write(&port, Signal::Bool(true));
        assert_eq!(plane.read(&port).map(Signal::as_level), Some(1.0));
    }
}
