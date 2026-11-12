/*
 * Naming and the Buttplug output for toys.
 *
 * Each toy's graph (toy_graph.rs) sends commands for its motors through the
 * safety layer into the toy's command slots. The toy's emitter task drains
 * the slots at the toy's update rate and hands them to ButtplugOutput.
 */

use buttplug::core::message::ActuatorType;
use vibecheck_core::{
    modules::buttplug::{Actuator, ActuatorSpec, ButtplugOutput},
    ModuleInstanceId,
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

/// Each feature's port and how to drive it, including its Flip Input.
pub fn toy_actuators(toy: &VCToy) -> Vec<(String, ActuatorSpec)> {
    toy.parsed_toy_features
        .features
        .iter()
        .map(|f| {
            (
                feature_port_name(f.feature_type, f.feature_index),
                ActuatorSpec {
                    actuator: actuator_for(f.feature_type, f.feature_index),
                    flip: f.flip_input_float,
                },
            )
        })
        .collect()
}

/// The Buttplug output for a toy, with one port per feature.
pub fn buttplug_output(toy: &VCToy) -> ButtplugOutput {
    ButtplugOutput::new(toy.device_handle.clone(), toy_actuators(toy))
}
