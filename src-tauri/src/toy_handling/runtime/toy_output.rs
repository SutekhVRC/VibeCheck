/*
 * Bridge between the existing per-toy processing and the vibecheck-core output side.
 *
 * Levels arrive already processed by the toy's pipeline (mode, then idle,
 * Range and Flip Input). They go through the safety layer into one
 * latest-command slot per motor. The toy's
 * emitter task drains the slots at the toy's update rate and hands the
 * commands to ButtplugOutput.
 */

use buttplug::core::message::ActuatorType;
use vibecheck_core::{
    modules::buttplug::{Actuator, ButtplugOutput},
    CommandSlots, ModuleInstanceId, PortId, SafetyLayer,
};

use crate::toy_handling::toyops::{VCFeatureType, VCToy};

/// Module id for a toy, e.g. `toy:lovense_lush:0`. Same name form as the OSC
/// battery address, and stable across reconnects (no Buttplug device index).
pub fn toy_module_id(toy: &VCToy) -> ModuleInstanceId {
    ModuleInstanceId::new(format!(
        "toy:{}:{}",
        toy.toy_name.replace(' ', "_").to_lowercase(),
        toy.sub_id
    ))
}

/// Port name for one feature, e.g. `Vibrator_0`. Scalar features share one
/// index space, and rotators and linears each have their own, so the
/// type + index pair is unique within a toy.
pub fn feature_port_name(feature_type: VCFeatureType, feature_index: u32) -> String {
    format!("{:?}_{}", feature_type, feature_index)
}

fn actuator_for(feature_type: VCFeatureType, index: u32) -> Actuator {
    let scalar = |actuator| Actuator::Scalar { index, actuator };
    match feature_type {
        VCFeatureType::Vibrator => scalar(ActuatorType::Vibrate),
        VCFeatureType::Rotator => Actuator::Rotate { index },
        VCFeatureType::Linear => Actuator::Linear { index },
        VCFeatureType::Oscillate => scalar(ActuatorType::Oscillate),
        VCFeatureType::Constrict => scalar(ActuatorType::Constrict),
        VCFeatureType::Inflate => scalar(ActuatorType::Inflate),
        VCFeatureType::Position => scalar(ActuatorType::Position),
        VCFeatureType::ScalarRotator => scalar(ActuatorType::Rotate),
    }
}

/// The Buttplug output for a toy, with one port per feature.
pub fn buttplug_output(toy: &VCToy) -> ButtplugOutput {
    ButtplugOutput::new(
        toy.device_handle.clone(),
        toy.parsed_toy_features.features.iter().map(|f| {
            (
                feature_port_name(f.feature_type, f.feature_index),
                actuator_for(f.feature_type, f.feature_index),
            )
        }),
    )
}

/// Where the toy task puts processed levels.
pub struct ToyCommandSink {
    module: ModuleInstanceId,
    safety: SafetyLayer,
    slots: CommandSlots,
}

impl ToyCommandSink {
    pub fn new(toy: &VCToy, slots: CommandSlots) -> Self {
        Self {
            module: toy_module_id(toy),
            safety: SafetyLayer::default(),
            slots,
        }
    }

    /// A sink for an explicit module id, for tests where there's no device.
    #[cfg(test)]
    pub fn for_module(module: ModuleInstanceId, slots: CommandSlots) -> Self {
        Self {
            module,
            safety: SafetyLayer::default(),
            slots,
        }
    }

    /// Queue a fully processed level for one feature. Replaces any command
    /// for the same feature that hasn't been sent yet; other features are
    /// unaffected. `linear_speed_ms` is only used by Linear features.
    pub fn send(
        &self,
        feature_type: VCFeatureType,
        feature_index: u32,
        linear_speed_ms: u32,
        level: f64,
    ) {
        let duration_ms = match feature_type {
            VCFeatureType::Linear => Some(linear_speed_ms),
            _ => None,
        };
        let port = PortId::new(
            self.module.clone(),
            feature_port_name(feature_type, feature_index),
        );
        self.slots
            .write(self.safety.clamp(port, level, duration_ms));
    }
}
