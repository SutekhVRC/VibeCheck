//! Latest-command slots in front of an output.
//!
//! Whatever produces commands writes them here as fast as it likes. The output
//! drains the slots at its own rate (e.g. a toy's Bluetooth update rate) and
//! only sends what changed.
//!
//! There is one slot **per target port**, so commands for different motors on
//! the same toy never overwrite each other. Commands for the *same* motor do:
//! only the newest is sent.

use std::collections::BTreeMap;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::address::PortId;
use crate::safety::ClampedCmd;

#[derive(Default)]
struct Slot {
    cmd: Option<ClampedCmd>,
    dirty: bool,
}

/// Cheap to clone; clones share the same slots.
#[derive(Clone, Default)]
pub struct CommandSlots {
    slots: Arc<Mutex<BTreeMap<PortId, Slot>>>,
}

impl CommandSlots {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the pending command for the command's target.
    ///
    /// Writing the same value again still counts as a change, so it is sent
    /// again on the next drain.
    pub fn write(&self, cmd: ClampedCmd) {
        let mut slots = self.slots.lock();
        let slot = slots.entry(cmd.target().clone()).or_default();
        slot.cmd = Some(cmd);
        slot.dirty = true;
    }

    /// Every command written since the last drain, at most one per target,
    /// ordered by target.
    pub fn take_changed(&self) -> Vec<ClampedCmd> {
        let mut slots = self.slots.lock();
        slots
            .values_mut()
            .filter(|slot| slot.dirty)
            .filter_map(|slot| {
                slot.dirty = false;
                slot.cmd.clone()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::ModuleInstanceId;
    use crate::safety::SafetyLayer;

    fn motor(n: &str) -> PortId {
        PortId::new(ModuleInstanceId::new("toy:lovense_edge:0"), n)
    }

    #[test]
    fn motors_on_one_toy_do_not_overwrite_each_other() {
        let safety = SafetyLayer::default();
        let slots = CommandSlots::new();

        // One OSC message driving two motors within the same tick.
        slots.write(safety.clamp(motor("Vibrator_0"), 0.4, None).unwrap());
        slots.write(safety.clamp(motor("Vibrator_1"), 0.4, None).unwrap());

        let sent = slots.take_changed();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0].target(), &motor("Vibrator_0"));
        assert_eq!(sent[1].target(), &motor("Vibrator_1"));
    }

    #[test]
    fn newest_command_per_motor_wins_and_is_sent_once() {
        let safety = SafetyLayer::default();
        let slots = CommandSlots::new();

        slots.write(safety.clamp(motor("Vibrator_0"), 0.2, None).unwrap());
        slots.write(safety.clamp(motor("Vibrator_0"), 0.7, None).unwrap());

        let sent = slots.take_changed();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].level(), 0.7);

        assert!(slots.take_changed().is_empty());

        // Re-sending the same value counts as a change.
        slots.write(safety.clamp(motor("Vibrator_0"), 0.7, None).unwrap());
        assert_eq!(slots.take_changed().len(), 1);
    }
}
