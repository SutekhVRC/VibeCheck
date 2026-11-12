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

use std::collections::HashMap;
use std::sync::Arc;

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
    Level { feature: u32, output: OutputType },
    /// Move to a position over the command's duration, in milliseconds
    /// (linear toys / strokers).
    Position { feature: u32 },
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
    pub actuator: Actuator,
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

pub struct ButtplugOutput {
    device: Arc<ButtplugClientDevice>,
    actuators: HashMap<Arc<str>, ActuatorSpec>,
    events: Option<(PortId, EventPlane)>,
    error_state: ErrorState,
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

    /// Publish error and recovery events as `module`, on `events`.
    pub fn with_events(mut self, module: ModuleInstanceId, events: EventPlane) -> Self {
        self.events = Some((PortId::new(module, ERROR_PORT), events));
        self
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
        ports
    }

    /// Sends every command, even if an earlier one fails, and returns the
    /// first error.
    async fn apply(&mut self, cmds: &[ClampedCmd]) -> Result<()> {
        let mut first_error = None;
        for cmd in cmds {
            let result = match self.actuators.get(&cmd.target().port) {
                Some(&spec) => self.send(spec, cmd).await,
                None => Err(Error::UnknownPort(cmd.target().clone())),
            };
            if let Err(e) = result {
                first_error.get_or_insert(e);
            }
        }
        let result = first_error.map_or(Ok(()), Err);
        if let Some((port, events)) = &self.events {
            if let Some(value) = self.error_state.update(&result) {
                events.publish(port.clone(), EventKind::OutputError, value);
            }
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
}
