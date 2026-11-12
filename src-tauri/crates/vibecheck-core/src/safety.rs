//! The safety layer: the only way a command reaches an output.
//!
//! [`ClampedCmd`] has private fields and no public constructor, so outside
//! this crate the only way to get one is through [`SafetyLayer::clamp`]. Output
//! modules therefore can't receive, or invent, a command that skipped the
//! user's caps.
//!
//! Levels here are **intensities**: 0 is off and 1 is full power, before any
//! per-actuator Flip Input (outputs apply that themselves), so caps mean the
//! same thing for every toy.
//!
//! So far: master volume, level and duration caps, and the causal depth
//! limit. Arming, rate limits and panic stop come later.
//!
//! A [`SafetyLayer`] is cheap to clone and clones share their caps, so the app
//! can hand one to every part of a session and change the caps (e.g. the
//! master volume from Settings) while toys are running.

use std::sync::Arc;

use parking_lot::RwLock;

use crate::address::PortId;
use crate::event::Event;

/// User-set limits applied to every command, after every rule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SafetyCaps {
    /// Master volume, 0.0 to 1.0: every level is scaled by this, so the full
    /// range is kept but quieter (0.6 turns 100% into 60% and 50% into 30%).
    pub volume: f64,
    /// Highest level any command may request after the volume, 0.0 to 1.0.
    pub max_level: f64,
    /// Longest duration any command may request, in milliseconds.
    pub max_duration_ms: u32,
    /// Longest chain of events a command may result from. Root events are
    /// depth 0, so with 3 a rule may react to a rule's output at most twice.
    pub max_event_depth: u8,
}

impl Default for SafetyCaps {
    fn default() -> Self {
        Self {
            volume: 1.0,
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
    level: f64,
    duration_ms: Option<u32>,
}

impl ClampedCmd {
    pub fn target(&self) -> &PortId {
        &self.target
    }

    /// Intensity from 0.0 to the user's cap, after the master volume.
    pub fn level(&self) -> f64 {
        self.level
    }

    /// For one-shot actions (e.g. a timed vibrate). `None` for continuous outputs.
    pub fn duration_ms(&self) -> Option<u32> {
        self.duration_ms
    }
}

/// Clones share the same caps.
#[derive(Clone, Debug, Default)]
pub struct SafetyLayer {
    caps: Arc<RwLock<SafetyCaps>>,
}

/// A finite value clamped to 0..=1; anything else (NaN, infinity) becomes 0.
fn unit(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

impl SafetyLayer {
    pub fn new(caps: SafetyCaps) -> Self {
        Self {
            caps: Arc::new(RwLock::new(caps)),
        }
    }

    pub fn caps(&self) -> SafetyCaps {
        *self.caps.read()
    }

    /// Replaces the caps for this layer and every clone of it.
    pub fn set_caps(&self, caps: SafetyCaps) {
        *self.caps.write() = caps;
    }

    /// Sets the master volume (0.0 to 1.0) for this layer and every clone.
    pub fn set_volume(&self, volume: f64) {
        self.caps.write().volume = unit(volume);
    }

    /// The intensity actually allowed for a requested one: clamped to 0..=1,
    /// scaled by the master volume, then capped at `max_level`. Non-finite
    /// levels (NaN or infinity) become 0.0 so a bad input can never drive an
    /// output.
    pub fn limit(&self, level: f64) -> f64 {
        let caps = self.caps();
        (unit(level) * unit(caps.volume)).min(unit(caps.max_level))
    }

    /// Limit a requested level (and optional duration) for `target`.
    pub fn clamp(&self, target: PortId, level: f64, duration_ms: Option<u32>) -> ClampedCmd {
        ClampedCmd {
            target,
            level: self.limit(level),
            duration_ms: duration_ms.map(|d| d.min(self.caps().max_duration_ms)),
        }
    }

    /// Clamp a command requested in response to `cause`, refusing it if the
    /// chain of events behind it is too long.
    pub fn clamp_caused_by(
        &self,
        cause: &Event,
        target: PortId,
        level: f64,
        duration_ms: Option<u32>,
    ) -> Option<ClampedCmd> {
        if cause.depth >= self.caps().max_event_depth {
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
            volume: 1.0,
            max_level: 0.5,
            max_duration_ms: 1000,
            max_event_depth: 3,
        });

        let cmd = safety.clamp(target(), 0.9, Some(5000));
        assert_eq!(cmd.level(), 0.5);
        assert_eq!(cmd.duration_ms(), Some(1000));

        assert_eq!(safety.clamp(target(), -1.0, None).level(), 0.0);
        assert_eq!(safety.clamp(target(), f64::NAN, None).level(), 0.0);
        assert_eq!(safety.clamp(target(), f64::INFINITY, None).level(), 0.0);
    }

    #[test]
    fn full_volume_changes_nothing() {
        let safety = SafetyLayer::default();
        for level in [0.0, 0.01, 0.37, 0.5, 0.99, 1.0] {
            assert_eq!(safety.limit(level), level);
        }
    }

    #[test]
    fn volume_scales_the_whole_range() {
        let safety = SafetyLayer::default();
        safety.set_volume(0.6);
        assert!((safety.limit(1.0) - 0.6).abs() < 1e-9);
        assert!((safety.limit(0.5) - 0.3).abs() < 1e-9);
        assert_eq!(safety.limit(0.0), 0.0);
        // Out-of-range volumes are clamped
        safety.set_volume(3.0);
        assert_eq!(safety.limit(0.5), 0.5);
        safety.set_volume(f64::NAN);
        assert_eq!(safety.limit(0.5), 0.0);
    }

    #[test]
    fn clones_share_the_volume() {
        let safety = SafetyLayer::default();
        let in_a_running_session = safety.clone();
        safety.set_volume(0.5);
        assert!((in_a_running_session.limit(1.0) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn refuses_chains_past_max_depth() {
        let safety = SafetyLayer::default();
        assert!(safety
            .clamp_caused_by(&event_at_depth(2), target(), 0.3, None)
            .is_some());
        assert!(safety
            .clamp_caused_by(&event_at_depth(3), target(), 0.3, None)
            .is_none());
    }
}
