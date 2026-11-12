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
//! What it enforces:
//!
//! - **Master intensity** and a level cap ([`SafetyCaps`]).
//! - **Panic stop**: [`panic`](SafetyLayer::panic) refuses every new command,
//!   invalidates commands already created (outputs check
//!   [`admits`](SafetyLayer::admits) before sending), and disarms everything,
//!   until [`resume`](SafetyLayer::resume).
//! - **Arming**: targets whose [`TargetPolicy`] says `requires_arming` only
//!   get commands while armed. Nothing is armed by default, and arming lasts
//!   until disarmed or a panic.
//! - **Rate limits**: a per-target minimum gap between commands.
//! - **Feedback loops**: an action taken in response to an event is refused if
//!   the chain behind that event is too long, or if the module asking is
//!   already part of it, i.e. it would be reacting to the result of its own
//!   earlier action ([`clamp_in_response`](SafetyLayer::clamp_in_response)).
//!
//! A [`SafetyLayer`] is cheap to clone and clones share all of this, so the
//! app can hand one to every part of a session and change it (master intensity,
//! panic, arming) while toys are running.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use crate::address::{ModuleInstanceId, PortId};
use crate::event::{Event, EventId, EventKind, EventPlane, Value};

/// Port an `Acted` event is published from on the requesting module.
pub const ACTED_PORT: &str = "acted";

/// User-set limits applied to every command, after every rule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SafetyCaps {
    /// Master intensity, 0.0 to 1.0: every level is scaled by this, so the full
    /// range is kept but gentler (0.6 turns 100% into 60% and 50% into 30%).
    pub intensity: f64,
    /// Highest level any command may request after the intensity, 0.0 to 1.0.
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
            intensity: 1.0,
            max_level: 1.0,
            max_duration_ms: 10_000,
            max_event_depth: 3,
        }
    }
}

/// Extra rules for one output module (e.g. a device that needs arming). Modules without a
/// policy (toys) only get the caps and panic stop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TargetPolicy {
    /// Commands are refused unless the module has been armed.
    pub requires_arming: bool,
    /// Commands to the same target port closer together than this are refused.
    pub min_interval: Option<Duration>,
}

/// A command that has passed the safety layer.
#[derive(Clone, Debug, PartialEq)]
pub struct ClampedCmd {
    target: PortId,
    level: f64,
    duration_ms: Option<u32>,
    /// Panic count when the command was made; older commands are stale.
    epoch: u64,
    /// The `Acted` event recorded for this command, if it was a response to
    /// an event.
    cause: Option<EventId>,
}

impl ClampedCmd {
    pub fn target(&self) -> &PortId {
        &self.target
    }

    /// Intensity from 0.0 to the user's cap, after the master intensity.
    pub fn level(&self) -> f64 {
        self.level
    }

    /// For one-shot actions (e.g. a timed vibrate). `None` for continuous outputs.
    pub fn duration_ms(&self) -> Option<u32> {
        self.duration_ms
    }

    /// For commands made by [`SafetyLayer::clamp_in_response`]: the `Acted`
    /// event recorded for it. An output reporting what it did should publish
    /// its event caused by this (`EventPlane::publish_caused_by_id`), so the
    /// chain leads back through whoever asked.
    pub fn cause(&self) -> Option<EventId> {
        self.cause
    }
}

#[derive(Debug, Default)]
struct State {
    caps: SafetyCaps,
    panicked: bool,
    /// Goes up by one on every panic.
    epoch: u64,
    policies: HashMap<ModuleInstanceId, TargetPolicy>,
    armed: HashSet<ModuleInstanceId>,
    last_sent: HashMap<PortId, Instant>,
}

impl State {
    fn requires_arming(&self, module: &ModuleInstanceId) -> bool {
        self.policies.get(module).is_some_and(|p| p.requires_arming)
    }

    fn may_send_to(&self, module: &ModuleInstanceId) -> bool {
        !self.panicked && (!self.requires_arming(module) || self.armed.contains(module))
    }
}

/// Clones share the same state.
#[derive(Clone, Debug, Default)]
pub struct SafetyLayer {
    state: Arc<Mutex<State>>,
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
        let layer = Self::default();
        layer.set_caps(caps);
        layer
    }

    pub fn caps(&self) -> SafetyCaps {
        self.state.lock().caps
    }

    /// Replaces the caps for this layer and every clone of it.
    pub fn set_caps(&self, caps: SafetyCaps) {
        self.state.lock().caps = caps;
    }

    /// Sets the master intensity (0.0 to 1.0) for this layer and every clone.
    pub fn set_intensity(&self, intensity: f64) {
        self.state.lock().caps.intensity = unit(intensity);
    }

    /// Panic stop: refuse every new command, make every command already
    /// created stale, and disarm everything, until [`resume`](Self::resume).
    /// Stopping the devices themselves is up to the caller.
    pub fn panic(&self) {
        let mut state = self.state.lock();
        state.panicked = true;
        state.epoch += 1;
        state.armed.clear();
    }

    /// Lift a panic stop. Nothing is re-armed, and nothing already refused
    /// is replayed: outputs wait for their next new command.
    pub fn resume(&self) {
        self.state.lock().panicked = false;
    }

    pub fn is_panicked(&self) -> bool {
        self.state.lock().panicked
    }

    /// Sets the policy for an output module, e.g. one that requires arming.
    pub fn set_policy(&self, module: ModuleInstanceId, policy: TargetPolicy) {
        self.state.lock().policies.insert(module, policy);
    }

    /// Arms a module. Refused (returns `false`) while panicked.
    pub fn arm(&self, module: ModuleInstanceId) -> bool {
        let mut state = self.state.lock();
        if state.panicked {
            return false;
        }
        state.armed.insert(module);
        true
    }

    pub fn disarm(&self, module: &ModuleInstanceId) {
        self.state.lock().armed.remove(module);
    }

    pub fn disarm_all(&self) {
        self.state.lock().armed.clear();
    }

    pub fn is_armed(&self, module: &ModuleInstanceId) -> bool {
        self.state.lock().armed.contains(module)
    }

    /// The intensity allowed for a requested one: 0 while panicked; otherwise
    /// clamped to 0..=1, scaled by the master intensity, then capped at
    /// `max_level`. Non-finite levels (NaN or infinity) become 0.0 so a bad
    /// input can never drive an output.
    pub fn limit(&self, level: f64) -> f64 {
        let state = self.state.lock();
        if state.panicked {
            return 0.0;
        }
        (unit(level) * unit(state.caps.intensity)).min(unit(state.caps.max_level))
    }

    /// Makes a command for `target`, or refuses it (`None`) while panicked,
    /// while the target's module requires arming and isn't armed, or if the
    /// target's minimum interval hasn't passed since its last command.
    pub fn clamp(
        &self,
        target: PortId,
        level: f64,
        duration_ms: Option<u32>,
    ) -> Option<ClampedCmd> {
        let level = self.limit(level);
        let mut state = self.state.lock();
        if !state.may_send_to(&target.module) {
            return None;
        }
        if let Some(interval) = state
            .policies
            .get(&target.module)
            .and_then(|p| p.min_interval)
        {
            let now = Instant::now();
            if let Some(last) = state.last_sent.get(&target) {
                if now.saturating_duration_since(*last) < interval {
                    return None;
                }
            }
            state.last_sent.insert(target.clone(), now);
        }
        Some(ClampedCmd {
            level,
            duration_ms: duration_ms.map(|d| d.min(state.caps.max_duration_ms)),
            epoch: state.epoch,
            cause: None,
            target,
        })
    }

    /// Like [`clamp`](Self::clamp) for a command requested in response to
    /// `cause`, also refusing it if the chain of events behind it is too long.
    ///
    /// Prefer [`clamp_in_response`](Self::clamp_in_response), which also
    /// stops a module reacting to the result of its own action.
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
        self.clamp(target, level, duration_ms)
    }

    /// The loop guard for any module (a rule, a processor) that acts because
    /// of an event. `trigger` is the event it's reacting to and `requester`
    /// is the module asking.
    ///
    /// Refused (`None`) if:
    /// - the chain behind `trigger` is already `max_event_depth` long;
    /// - `requester` appears anywhere in that chain: it would be reacting to
    ///   the result of its own earlier action, which is how feedback loops
    ///   start;
    /// - or for any reason [`clamp`](Self::clamp) refuses.
    ///
    /// When admitted, an [`EventKind::Acted`] event is published from
    /// `requester` (caused by `trigger`) and the command carries it as its
    /// [`cause`](ClampedCmd::cause). That's what lets the loop be seen later:
    /// whatever the output reports chains back through the requester.
    pub fn clamp_in_response(
        &self,
        events: &EventPlane,
        trigger: &Event,
        requester: &ModuleInstanceId,
        target: PortId,
        level: f64,
        duration_ms: Option<u32>,
    ) -> Option<ClampedCmd> {
        if trigger.depth >= self.caps().max_event_depth {
            return None;
        }
        if events.chain_includes(trigger.id, requester) || trigger.source.module == *requester {
            return None;
        }
        let mut cmd = self.clamp(target, level, duration_ms)?;
        cmd.cause = Some(events.publish_caused_by_id(
            trigger.id,
            PortId::new(requester.clone(), ACTED_PORT),
            EventKind::Acted,
            Value::None,
        ));
        Some(cmd)
    }

    /// Whether an output may still send `cmd` now. Outputs check this right
    /// before sending: a panic (or disarm) since the command was made means
    /// it must be dropped.
    pub fn admits(&self, cmd: &ClampedCmd) -> bool {
        let state = self.state.lock();
        cmd.epoch == state.epoch && state.may_send_to(&cmd.target.module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventId, EventKind, Value};

    fn target() -> PortId {
        PortId::new(ModuleInstanceId::new("output:0"), "vibrate")
    }

    fn toy() -> PortId {
        PortId::new(ModuleInstanceId::new("toy:lush:0"), "Vibrator_0")
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
            intensity: 1.0,
            max_level: 0.5,
            max_duration_ms: 1000,
            max_event_depth: 3,
        });

        let cmd = safety.clamp(target(), 0.9, Some(5000)).unwrap();
        assert_eq!(cmd.level(), 0.5);
        assert_eq!(cmd.duration_ms(), Some(1000));

        assert_eq!(safety.limit(-1.0), 0.0);
        assert_eq!(safety.limit(f64::NAN), 0.0);
        assert_eq!(safety.limit(f64::INFINITY), 0.0);
    }

    #[test]
    fn full_intensity_changes_nothing() {
        let safety = SafetyLayer::default();
        for level in [0.0, 0.01, 0.37, 0.5, 0.99, 1.0] {
            assert_eq!(safety.limit(level), level);
        }
    }

    #[test]
    fn intensity_scales_the_whole_range() {
        let safety = SafetyLayer::default();
        safety.set_intensity(0.6);
        assert!((safety.limit(1.0) - 0.6).abs() < 1e-9);
        assert!((safety.limit(0.5) - 0.3).abs() < 1e-9);
        assert_eq!(safety.limit(0.0), 0.0);
        // Out-of-range intensities are clamped
        safety.set_intensity(3.0);
        assert_eq!(safety.limit(0.5), 0.5);
        safety.set_intensity(f64::NAN);
        assert_eq!(safety.limit(0.5), 0.0);
    }

    #[test]
    fn clones_share_the_intensity() {
        let safety = SafetyLayer::default();
        let in_a_running_session = safety.clone();
        safety.set_intensity(0.5);
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

    #[test]
    fn panic_refuses_new_commands_and_invalidates_old_ones() {
        let safety = SafetyLayer::default();
        let in_flight = safety.clamp(toy(), 0.8, None).unwrap();
        assert!(safety.admits(&in_flight));

        let in_a_session = safety.clone();
        in_a_session.panic();
        assert!(safety.is_panicked());
        assert!(!safety.admits(&in_flight), "made before the panic");
        assert!(safety.clamp(toy(), 0.8, None).is_none());
        assert_eq!(safety.limit(0.8), 0.0);

        safety.resume();
        assert!(!safety.is_panicked());
        assert!(!safety.admits(&in_flight), "still stale after resume");
        let fresh = safety.clamp(toy(), 0.8, None).unwrap();
        assert!(safety.admits(&fresh));
    }

    #[test]
    fn risky_targets_need_arming_and_panic_disarms() {
        let safety = SafetyLayer::default();
        let risky = target().module;
        safety.set_policy(
            risky.clone(),
            TargetPolicy {
                requires_arming: true,
                min_interval: None,
            },
        );

        assert!(
            safety.clamp(target(), 0.3, Some(500)).is_none(),
            "not armed"
        );
        assert!(
            safety.clamp(toy(), 0.3, None).is_some(),
            "toys need no arming"
        );

        assert!(safety.arm(risky.clone()));
        let cmd = safety.clamp(target(), 0.3, Some(500)).unwrap();
        assert!(safety.admits(&cmd));

        // Disarming drops commands not yet sent
        safety.disarm(&risky);
        assert!(!safety.admits(&cmd));

        safety.arm(risky.clone());
        safety.panic();
        assert!(!safety.is_armed(&risky));
        assert!(!safety.arm(risky.clone()), "can't arm while panicked");
        safety.resume();
        assert!(!safety.is_armed(&risky), "resume doesn't re-arm");
    }

    /// The loop from the architecture notes: a toy event triggers a rule,
    /// the rule makes an output act, the output reports it, and the rule
    /// sees that report. The rule's second reaction must be refused.
    #[test]
    fn a_module_cannot_react_to_the_result_of_its_own_action() {
        let safety = SafetyLayer::default();
        let (events, mut rx) = EventPlane::new();
        let rule = ModuleInstanceId::new("rule:a");
        let other_rule = ModuleInstanceId::new("rule:b");
        let out = PortId::new(ModuleInstanceId::new("output:0"), "vibrate");

        // 1. The toy does something
        events.publish(toy(), EventKind::Pattern, Value::None);
        let trigger = rx.try_recv().unwrap();

        // 2. The rule reacts: allowed, and an Acted event is recorded
        let cmd = safety
            .clamp_in_response(&events, &trigger, &rule, out.clone(), 0.3, Some(500))
            .expect("first reaction is fine");
        let acted = rx.try_recv().unwrap();
        assert_eq!(acted.kind, EventKind::Acted);
        assert_eq!(acted.source.module, rule);
        assert_eq!(acted.cause, Some(trigger.id));
        assert_eq!(cmd.cause(), Some(acted.id));

        // 3. The output reports what it did, caused by the command
        events.publish_caused_by_id(
            cmd.cause().unwrap(),
            PortId::new(ModuleInstanceId::new("output:0"), "acted"),
            EventKind::Custom("output_acted".into()),
            Value::None,
        );
        let report = rx.try_recv().unwrap();

        // 4. The same rule reacting to that report is a loop: refused
        assert!(safety
            .clamp_in_response(&events, &report, &rule, out.clone(), 0.3, Some(500))
            .is_none());
        assert!(rx.try_recv().is_none(), "nothing recorded for a refusal");

        // A different module may still react to it
        assert!(safety
            .clamp_in_response(&events, &report, &other_rule, out, 0.3, Some(500))
            .is_some());
    }

    #[test]
    fn a_module_cannot_react_to_its_own_event() {
        let safety = SafetyLayer::default();
        let (events, mut rx) = EventPlane::new();
        let rule = ModuleInstanceId::new("rule:a");
        events.publish(
            PortId::new(rule.clone(), "fired"),
            EventKind::RuleFired,
            Value::None,
        );
        let own = rx.try_recv().unwrap();
        assert!(safety
            .clamp_in_response(&events, &own, &rule, toy(), 0.3, None)
            .is_none());
    }

    #[test]
    fn long_chains_are_refused_even_between_different_modules() {
        let safety = SafetyLayer::default(); // max_event_depth 3
        let (events, mut rx) = EventPlane::new();
        let out = PortId::new(ModuleInstanceId::new("output:0"), "vibrate");

        events.publish(toy(), EventKind::Pattern, Value::None);
        let mut trigger = rx.try_recv().unwrap();
        // a, b, c each react to the previous one's Acted event
        for name in ["rule:a", "rule:b", "rule:c"] {
            let module = ModuleInstanceId::new(name);
            let allowed = safety
                .clamp_in_response(&events, &trigger, &module, out.clone(), 0.3, None)
                .is_some();
            if trigger.depth < 3 {
                assert!(allowed, "{name} at depth {}", trigger.depth);
                trigger = rx.try_recv().unwrap();
            } else {
                assert!(!allowed);
            }
        }
        // The fourth link is past the limit
        assert_eq!(trigger.depth, 3);
        assert!(safety
            .clamp_in_response(
                &events,
                &trigger,
                &ModuleInstanceId::new("rule:d"),
                out,
                0.3,
                None
            )
            .is_none());
    }

    #[test]
    fn min_interval_limits_the_rate_per_target() {
        let safety = SafetyLayer::default();
        safety.set_policy(
            target().module,
            TargetPolicy {
                requires_arming: false,
                min_interval: Some(Duration::from_secs(60)),
            },
        );
        assert!(safety.clamp(target(), 0.3, None).is_some());
        assert!(safety.clamp(target(), 0.3, None).is_none(), "too soon");
        let other_port = PortId::new(target().module, "shock");
        assert!(safety.clamp(other_port, 0.3, None).is_some(), "per port");
    }
}
