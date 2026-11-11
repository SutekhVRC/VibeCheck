//! The safety layer: the only way a command reaches an output.
//!
//! [`ClampedCmd`] has private fields and no public constructor, so outside
//! this crate the only way to get one is through [`SafetyLayer::clamp`]. Output
//! modules therefore can't receive, or invent, a command that skipped the
//! user's caps.
//!
//! This is the first slice of the safety layer: level and duration caps, plus
//! the causal depth limit. Arming, rate limits and panic stop come later.

use crate::address::PortId;
use crate::event::Event;

/// User-set limits applied to every command, after every rule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SafetyCaps {
    /// Highest level any command may request, from 0.0 to 1.0.
    pub max_level: f32,
    /// Longest duration any command may request, in milliseconds.
    pub max_duration_ms: u32,
    /// Longest chain of events a command may result from. Root events are
    /// depth 0, so with 3 a rule may react to a rule's output at most twice.
    pub max_event_depth: u8,
}

impl Default for SafetyCaps {
    fn default() -> Self {
        Self {
            max_level: 1.0,
            max_duration_ms: 10_000,
            max_event_depth: 3,
        }
    }
}

/// A command that has passed the safety layer.
#[derive(Clone, Debug, PartialEq)]
pub struct ClampedCmd {
    target: PortId,
    level: f32,
    duration_ms: Option<u32>,
}

impl ClampedCmd {
    pub fn target(&self) -> &PortId {
        &self.target
    }

    /// Level from 0.0 to the user's cap.
    pub fn level(&self) -> f32 {
        self.level
    }

    /// For one-shot actions (e.g. a timed vibrate). `None` for continuous outputs.
    pub fn duration_ms(&self) -> Option<u32> {
        self.duration_ms
    }
}

#[derive(Clone, Debug, Default)]
pub struct SafetyLayer {
    caps: SafetyCaps,
}

impl SafetyLayer {
    pub fn new(caps: SafetyCaps) -> Self {
        Self { caps }
    }

    pub fn caps(&self) -> SafetyCaps {
        self.caps
    }

    pub fn set_caps(&mut self, caps: SafetyCaps) {
        self.caps = caps;
    }

    /// Clamp a requested level (and optional duration) for `target`.
    ///
    /// Levels are clamped to 0.0..=max_level. Non-finite levels (NaN or
    /// infinity) become 0.0 so a bad input can never drive an output.
    pub fn clamp(&self, target: PortId, level: f32, duration_ms: Option<u32>) -> ClampedCmd {
        let max = self.caps.max_level.clamp(0.0, 1.0);
        let level = if level.is_finite() {
            level.clamp(0.0, max)
        } else {
            0.0
        };
        ClampedCmd {
            target,
            level,
            duration_ms: duration_ms.map(|d| d.min(self.caps.max_duration_ms)),
        }
    }

    /// Clamp a command requested in response to `cause`, refusing it if the
    /// chain of events behind it is too long.
    pub fn clamp_caused_by(
        &self,
        cause: &Event,
        target: PortId,
        level: f32,
        duration_ms: Option<u32>,
    ) -> Option<ClampedCmd> {
        if cause.depth >= self.caps.max_event_depth {
            return None;
        }
        Some(self.clamp(target, level, duration_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::ModuleInstanceId;
    use crate::event::{EventId, EventKind, Value};

    fn target() -> PortId {
        PortId::new(ModuleInstanceId::new("output:0"), "vibrate")
    }

    fn event_at_depth(depth: u8) -> Event {
        Event {
            id: EventId(1),
            source: target(),
            kind: EventKind::RuleFired,
            value: Value::None,
            ts_ms: 0,
            cause: None,
            depth,
        }
    }

    #[test]
    fn clamps_level_and_duration_to_caps() {
        let safety = SafetyLayer::new(SafetyCaps {
            max_level: 0.5,
            max_duration_ms: 1000,
            max_event_depth: 3,
        });

        let cmd = safety.clamp(target(), 0.9, Some(5000));
        assert_eq!(cmd.level(), 0.5);
        assert_eq!(cmd.duration_ms(), Some(1000));

        assert_eq!(safety.clamp(target(), -1.0, None).level(), 0.0);
        assert_eq!(safety.clamp(target(), f32::NAN, None).level(), 0.0);
        assert_eq!(safety.clamp(target(), f32::INFINITY, None).level(), 0.0);
    }

    #[test]
    fn refuses_chains_past_max_depth() {
        let safety = SafetyLayer::default();
        assert!(safety.clamp_caused_by(&event_at_depth(2), target(), 0.3, None).is_some());
        assert!(safety.clamp_caused_by(&event_at_depth(3), target(), 0.3, None).is_none());
    }
}
