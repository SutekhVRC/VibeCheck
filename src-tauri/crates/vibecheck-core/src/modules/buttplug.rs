//! Output module for toys connected through Buttplug IO.
//!
//! Each actuator (motor) on the toy is one signal-in port. Port names are
//! stable across reconnects, e.g. `Vibrator_0` or `Linear_0`; Buttplug's
//! device index is only used internally.
//!
//! Buttplug (from the version 12 release, message spec v4) describes a device
//! as numbered *features*, each with one or more *outputs* (vibrate, rotate,
//! position with duration, ...). An [`Actuator`] names a feature and what to
//! send it. [`LegacyLayout`] rebuilds the older view, where scalar, rotate
//! and linear actuators were numbered separately, which is how VibeCheck's
//! saved toy configs number their features.
//!
//! Commands arrive as intensities (0 = off). Actuators marked `flip` (Flip
//! Input, for toys whose motors run backwards) get `1 - level`, applied here,
//! after the safety layer.
//!
//! With [`with_events`](ButtplugOutput::with_events), the output publishes an
//! [`EventKind::OutputError`] event on its `error` port when sending starts
//! failing (value: the error as text) and again when it works again (value:
//! `None`). It only fires when that state changes, not on every failed send.
//!
//! It also publishes [`EventKind::Activity`] on its `activity` port: `true`
//! as soon as any actuator is sent a level above 0, `false` once every
//! actuator has been at 0 for [`STOP_AFTER`]. The delay keeps patterns that
//! pulse through 0 from firing on every pulse. Call
//! [`update_activity`](ButtplugOutput::update_activity) regularly (the
//! emitter does it every tick) so the stop is noticed even when nothing
//! new is sent. Levels are intensities, before Flip Input, so 0 is off
//! whichever way the motor runs.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use buttplug_client::device::{ClientDeviceCommandValue, ClientDeviceOutputCommand};
use buttplug_client::ButtplugClientDevice;
use buttplug_core::message::{DeviceFeature, DeviceFeatureOutput, OutputType};

use crate::address::{ModuleInstanceId, PortId};
use crate::error::{Error, Result};
use crate::event::{EventKind, EventPlane, Value};
use crate::module::{Output, PortSpec};
use crate::modules::levels::flip;
use crate::safety::ClampedCmd;

/// Event port for send failures and recoveries.
pub const ERROR_PORT: &str = "error";

/// Event port for the toy starting and stopping.
pub const ACTIVITY_PORT: &str = "activity";

/// How long every actuator must be at 0 before the toy counts as stopped.
pub const STOP_AFTER: Duration = Duration::from_secs(1);

/// Tracks whether any actuator is running, so activity events fire only
/// when that changes (see the module docs).
#[derive(Default)]
struct ActivityState {
    /// The last intensity sent to each actuator.
    levels: HashMap<Arc<str>, f64>,
    active: bool,
    /// When every actuator went to 0, while still counted as active.
    zero_since: Option<Instant>,
}

impl ActivityState {
    fn record(&mut self, port: &Arc<str>, level: f64) {
        self.levels.insert(port.clone(), level);
    }

    /// Forget every level, e.g. after a panic stopped the device outside
    /// this output. The toy then counts as stopped.
    fn clear(&mut self) {
        self.levels.clear();
    }

    /// The event value to publish at `now`, if the state changed.
    fn update(&mut self, now: Instant) -> Option<bool> {
        if self.levels.values().any(|level| *level > 0.0) {
            self.zero_since = None;
            if !self.active {
                self.active = true;
                return Some(true);
            }
            return None;
        }
        if !self.active {
            return None;
        }
        let since = *self.zero_since.get_or_insert(now);
        if now.duration_since(since) >= STOP_AFTER {
            self.active = false;
            self.zero_since = None;
            return Some(false);
        }
        None
    }
}

/// Tracks whether an output is failing, so error events fire only when that
/// changes.
#[derive(Default)]
struct ErrorState {
    failing: bool,
}

impl ErrorState {
    /// The event value to publish for this result, if the state changed.
    fn update(&mut self, result: &Result<()>) -> Option<Value> {
        match (result, self.failing) {
            (Err(e), false) => {
                self.failing = true;
                Some(Value::Text(e.to_string().into()))
            }
            (Ok(()), true) => {
                self.failing = false;
                Some(Value::None)
            }
            _ => None,
        }
    }
}

/// How to drive one actuator: a Buttplug device feature, and what to send it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Actuator {
    /// Set a level on one of the feature's outputs (vibrate, rotate,
    /// oscillate, constrict, position). Rotation is always clockwise.
    Level {
        /// The device feature's index.
        feature: u32,
        /// Which of its outputs.
        output: OutputType,
    },
    /// Move to a position over the command's duration, in milliseconds
    /// (linear toys / strokers).
    Position {
        /// The device feature's index.
        feature: u32,
    },
}

/// Outputs VibeCheck drives with a level, in the order they're numbered
/// within a feature. Temperature, LED and spray outputs aren't used.
const LEVEL_OUTPUTS: [OutputType; 5] = [
    OutputType::Vibrate,
    OutputType::Rotate,
    OutputType::Oscillate,
    OutputType::Constrict,
    OutputType::Position,
];

/// A device's actuators as Buttplug's message spec v3 numbered them: scalar,
/// rotate and linear actuators each counted from 0. VibeCheck's toy configs
/// number their features this way, so keeping the layout keeps saved
/// configs matching their toys.
///
/// - **scalars**: every level output of every feature, in feature order,
///   except rotators that can turn both ways;
/// - **rotators**: features whose rotate output can turn both ways;
/// - **linears**: features with a position-with-duration output.
///
/// This matches how Buttplug 9 listed devices: a rotate feature was either
/// a scalar or a rotator, never both, and its rotators are exactly the
/// features Buttplug 12 lets turn both ways (checked against every rotating
/// device in both versions' device lists). Buttplug 12's own v3
/// compatibility lists two-way rotators as scalars too, which would show
/// one motor twice.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LegacyLayout {
    /// Feature and output for each scalar actuator.
    pub scalars: Vec<(u32, OutputType)>,
    /// Feature for each rotate actuator.
    pub rotators: Vec<u32>,
    /// Feature for each linear actuator.
    pub linears: Vec<u32>,
}

impl LegacyLayout {
    /// The layout of a connected device.
    pub fn of(device: &ButtplugClientDevice) -> Self {
        Self::from_features(device.device_features().values().map(|f| f.feature()))
    }

    /// From a device's features, in feature order.
    pub fn from_features<'a>(features: impl IntoIterator<Item = &'a DeviceFeature>) -> Self {
        let mut layout = Self::default();
        for feature in features {
            let index = feature.feature_index();
            let two_way_rotator = matches!(
                feature.get_output(OutputType::Rotate),
                Some(DeviceFeatureOutput::Rotate(rotate)) if rotate.value().start() < 0
            );
            for output in LEVEL_OUTPUTS {
                if output == OutputType::Rotate && two_way_rotator {
                    continue;
                }
                if feature.contains_output(output) {
                    layout.scalars.push((index, output));
                }
            }
            if two_way_rotator {
                layout.rotators.push(index);
            }
            if feature.contains_output(OutputType::HwPositionWithDuration) {
                layout.linears.push(index);
            }
        }
        layout
    }

    /// The actuator for scalar `index`, if the device has it.
    pub fn scalar(&self, index: u32) -> Option<Actuator> {
        self.scalars
            .get(index as usize)
            .map(|&(feature, output)| Actuator::Level { feature, output })
    }

    /// The actuator for rotator `index`, if the device has it.
    pub fn rotator(&self, index: u32) -> Option<Actuator> {
        self.rotators
            .get(index as usize)
            .map(|&feature| Actuator::Level {
                feature,
                output: OutputType::Rotate,
            })
    }

    /// The actuator for linear `index`, if the device has it.
    pub fn linear(&self, index: u32) -> Option<Actuator> {
        self.linears
            .get(index as usize)
            .map(|&feature| Actuator::Position { feature })
    }
}

/// Sends `level` (0.0 to 1.0, already flipped if needed) to one actuator.
/// `duration_ms` is only used by [`Actuator::Position`].
pub async fn send_to_actuator(
    device: &ButtplugClientDevice,
    actuator: Actuator,
    level: f64,
    duration_ms: Option<u32>,
) -> Result<()> {
    let unavailable = |e: &dyn std::fmt::Display| Error::OutputUnavailable(e.to_string());
    let (feature, command) = match actuator {
        Actuator::Level { feature, output } => (
            feature,
            ClientDeviceOutputCommand::from_command_value(
                output,
                &ClientDeviceCommandValue::Percent(level),
            )
            .map_err(|e| unavailable(&e))?,
        ),
        Actuator::Position { feature } => (
            feature,
            ClientDeviceOutputCommand::HwPositionWithDuration(
                ClientDeviceCommandValue::Percent(level),
                duration_ms.unwrap_or(0),
            ),
        ),
    };
    let feature = device
        .device_features()
        .get(&feature)
        .ok_or_else(|| unavailable(&format!("the device has no feature {feature}")))?;
    feature
        .run_output(&command)
        .await
        .map_err(|e| unavailable(&e))
}

/// An actuator plus whether its level is flipped before sending.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActuatorSpec {
    /// What to drive.
    pub actuator: Actuator,
    /// Flip Input: send `1 - level`, for motors that run backwards.
    pub flip: bool,
}

/// The level actually sent to the device for an intensity.
pub fn device_level(level: f64, flipped: bool) -> f64 {
    if flipped {
        flip(level)
    } else {
        level
    }
}

/// An [`Output`] for one Buttplug device: one signal-in port per actuator
/// (see the module docs).
pub struct ButtplugOutput {
    device: Arc<ButtplugClientDevice>,
    actuators: HashMap<Arc<str>, ActuatorSpec>,
    events: Option<(ModuleInstanceId, EventPlane)>,
    error_state: ErrorState,
    activity: ActivityState,
}

impl ButtplugOutput {
    /// `actuators` maps each port name to the actuator it drives.
    pub fn new(
        device: Arc<ButtplugClientDevice>,
        actuators: impl IntoIterator<Item = (String, ActuatorSpec)>,
    ) -> Self {
        Self {
            device,
            actuators: Self::actuator_map(actuators),
            events: None,
            error_state: ErrorState::default(),
            activity: ActivityState::default(),
        }
    }

    fn actuator_map(
        actuators: impl IntoIterator<Item = (String, ActuatorSpec)>,
    ) -> HashMap<Arc<str>, ActuatorSpec> {
        actuators
            .into_iter()
            .map(|(port, spec)| (Arc::from(port), spec))
            .collect()
    }

    /// Replaces the actuator mapping, e.g. when Flip Input changes.
    pub fn set_actuators(&mut self, actuators: impl IntoIterator<Item = (String, ActuatorSpec)>) {
        self.actuators = Self::actuator_map(actuators);
    }

    /// Publish error, recovery and activity events as `module`, on `events`.
    pub fn with_events(mut self, module: ModuleInstanceId, events: EventPlane) -> Self {
        self.events = Some((module, events));
        self
    }

    fn publish(&self, port: &str, kind: EventKind, value: Value) {
        if let Some((module, events)) = &self.events {
            events.publish(PortId::new(module.clone(), port), kind, value);
        }
    }

    /// Publishes an activity event if the toy started or stopped. Call
    /// regularly. `stopped_outside` means the device was stopped without
    /// going through this output (a panic stop): the levels it was last
    /// sent no longer apply.
    pub fn update_activity(&mut self, now: Instant, stopped_outside: bool) {
        if stopped_outside {
            self.activity.clear();
        }
        if let Some(active) = self.activity.update(now) {
            self.publish(ACTIVITY_PORT, EventKind::Activity, Value::Bool(active));
        }
    }

    async fn send(&self, spec: ActuatorSpec, cmd: &ClampedCmd) -> Result<()> {
        let level = device_level(cmd.level(), spec.flip);
        send_to_actuator(&self.device, spec.actuator, level, cmd.duration_ms()).await
    }
}

#[async_trait]
impl Output for ButtplugOutput {
    fn ports(&self) -> Vec<PortSpec> {
        let mut ports: Vec<PortSpec> = self
            .actuators
            .keys()
            .map(|port| PortSpec::signal_in(port.as_ref()))
            .collect();
        ports.sort_by(|a, b| a.name.cmp(&b.name));
        ports.push(PortSpec::event_out(ERROR_PORT));
        ports.push(PortSpec::event_out(ACTIVITY_PORT));
        ports
    }

    /// Sends every command, even if an earlier one fails, and returns the
    /// first error.
    async fn apply(&mut self, cmds: &[ClampedCmd]) -> Result<()> {
        let mut first_error = None;
        for cmd in cmds {
            let port = &cmd.target().port;
            let result = match self.actuators.get(port) {
                Some(&spec) => {
                    self.activity.record(port, cmd.level());
                    self.send(spec, cmd).await
                }
                None => Err(Error::UnknownPort(cmd.target().clone())),
            };
            if let Err(e) = result {
                first_error.get_or_insert(e);
            }
        }
        let result = first_error.map_or(Ok(()), Err);
        if let Some(value) = self.error_state.update(&result) {
            self.publish(ERROR_PORT, EventKind::OutputError, value);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use buttplug_core::message::{
        DeviceFeatureOutputHwPositionWithDurationProperties, DeviceFeatureOutputValueProperties,
    };
    use buttplug_core::util::range::RangeInclusive;

    fn feature(index: u32, outputs: Vec<DeviceFeatureOutput>) -> DeviceFeature {
        use buttplug_core::util::small_vec_enum_map::SmallVecEnumMap;
        DeviceFeature::new(
            index,
            "",
            &SmallVecEnumMap(outputs.into_iter().collect()),
            &SmallVecEnumMap(Default::default()),
        )
    }

    fn level(start: i32, end: i32) -> DeviceFeatureOutputValueProperties {
        DeviceFeatureOutputValueProperties::new(RangeInclusive::new(start, end))
    }

    #[test]
    fn legacy_layout_numbers_actuators_like_spec_v3() {
        use DeviceFeatureOutput as O;
        let features = [
            // A battery-only feature has no actuators
            feature(0, vec![]),
            feature(1, vec![O::Vibrate(level(0, 20))]),
            feature(2, vec![O::Rotate(level(-20, 20))]),
            feature(3, vec![O::Rotate(level(0, 20))]),
            feature(
                4,
                vec![O::HwPositionWithDuration(
                    DeviceFeatureOutputHwPositionWithDurationProperties::new(
                        RangeInclusive::new(0, 100),
                        RangeInclusive::new(0, 1000),
                    ),
                )],
            ),
            feature(5, vec![O::Led(level(0, 1)), O::Vibrate(level(0, 20))]),
        ];
        let layout = LegacyLayout::from_features(&features);
        assert_eq!(
            layout,
            LegacyLayout {
                // The two-way rotator is only a rotator, the one-way one
                // only a scalar (as Buttplug 9 listed them)
                scalars: vec![
                    (1, OutputType::Vibrate),
                    (3, OutputType::Rotate),
                    (5, OutputType::Vibrate),
                ],
                rotators: vec![2],
                linears: vec![4],
            }
        );
        assert_eq!(
            layout.scalar(2),
            Some(Actuator::Level {
                feature: 5,
                output: OutputType::Vibrate
            })
        );
        assert_eq!(layout.linear(0), Some(Actuator::Position { feature: 4 }));
        assert_eq!(layout.rotator(1), None);
    }

    #[test]
    fn flipped_actuators_get_the_inverted_level() {
        assert_eq!(device_level(0.3, false), 0.3);
        assert_eq!(device_level(0.3, true), 0.7);
        assert_eq!(device_level(0.0, true), 1.0);
    }

    #[test]
    fn error_events_fire_only_when_the_state_changes() {
        let mut state = ErrorState::default();
        let failed = || Err(Error::OutputUnavailable("disconnected".into()));

        assert_eq!(state.update(&Ok(())), None);
        assert_eq!(
            state.update(&failed()),
            Some(Value::Text("output unavailable: disconnected".into()))
        );
        assert_eq!(state.update(&failed()), None, "still failing");
        assert_eq!(state.update(&Ok(())), Some(Value::None), "recovered");
        assert_eq!(state.update(&Ok(())), None);
    }

    #[test]
    fn activity_starts_at_once_and_stops_after_a_second_at_zero() {
        let mut state = ActivityState::default();
        let (a, b): (Arc<str>, Arc<str>) = ("Vibrator_0".into(), "Vibrator_1".into());
        let t0 = Instant::now();
        let at = |ms: u64| t0 + Duration::from_millis(ms);

        assert_eq!(state.update(at(0)), None, "nothing sent yet");
        state.record(&a, 0.0);
        assert_eq!(state.update(at(0)), None, "0 is off");
        state.record(&a, 0.4);
        assert_eq!(state.update(at(10)), Some(true), "started at once");
        state.record(&b, 0.2);
        assert_eq!(state.update(at(20)), None, "already started");

        // A pulse through 0 shorter than STOP_AFTER isn't a stop
        state.record(&a, 0.0);
        state.record(&b, 0.0);
        assert_eq!(state.update(at(100)), None);
        assert_eq!(state.update(at(600)), None);
        state.record(&b, 0.3);
        assert_eq!(state.update(at(700)), None, "back on before a second");

        // One actuator still running keeps it active
        state.record(&a, 0.0);
        assert_eq!(state.update(at(5000)), None);

        state.record(&b, 0.0);
        assert_eq!(state.update(at(5100)), None, "the second starts now");
        assert_eq!(state.update(at(6099)), None);
        assert_eq!(state.update(at(6100)), Some(false), "stopped");
        assert_eq!(state.update(at(9000)), None, "already stopped");
    }

    #[test]
    fn a_stop_from_outside_counts_as_stopped() {
        let mut state = ActivityState::default();
        let port: Arc<str> = "Vibrator_0".into();
        let t0 = Instant::now();
        state.record(&port, 0.8);
        assert_eq!(state.update(t0), Some(true));

        // Panic: the device was stopped without a 0 being sent through here
        state.clear();
        assert_eq!(state.update(t0), None);
        assert_eq!(state.update(t0 + STOP_AFTER), Some(false));
        // After resume nothing is replayed, so it stays stopped
        assert_eq!(state.update(t0 + STOP_AFTER * 3), None);
    }
}
