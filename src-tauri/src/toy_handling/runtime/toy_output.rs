/*
 * Naming and the Buttplug output for toys.
 *
 * Each toy's graph (toy_graph.rs) sends commands for its motors through the
 * safety layer into the toy's command slots. The toy's emitter task drains
 * the slots at the toy's update rate and hands them to ButtplugOutput.
 */

use log::warn;
use vibecheck_core::{
    modules::buttplug::{Actuator, ActuatorSpec, ButtplugOutput, LegacyLayout},
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

/// The actuator a feature drives. Features are numbered like Buttplug's
/// message spec v3 (see `LegacyLayout`): rotators and linears have their own
/// numbering, and every other type is a scalar. `None` if the toy doesn't
/// have it (e.g. a config saved for a different version of the toy).
pub fn actuator_for(
    layout: &LegacyLayout,
    feature_type: VCFeatureType,
    index: u32,
) -> Option<Actuator> {
    match feature_type {
        VCFeatureType::Rotator => layout.rotator(index),
        VCFeatureType::Linear => layout.linear(index),
        VCFeatureType::Vibrator
        | VCFeatureType::Oscillate
        | VCFeatureType::Constrict
        | VCFeatureType::Inflate
        | VCFeatureType::Position
        | VCFeatureType::ScalarRotator => layout.scalar(index),
    }
}

/// Each feature's port and how to drive it, including its Flip Input.
/// Features the toy doesn't have are left out (and logged).
pub fn toy_actuators(toy: &VCToy) -> Vec<(String, ActuatorSpec)> {
    toy.parsed_toy_features
        .features
        .iter()
        .filter_map(|f| {
            let port = feature_port_name(f.feature_type, f.feature_index);
            let Some(actuator) = actuator_for(&toy.layout, f.feature_type, f.feature_index) else {
                warn!("{}: the toy has no actuator for {}", toy.toy_name, port);
                return None;
            };
            Some((
                port,
                ActuatorSpec {
                    actuator,
                    flip: f.flip_input_float,
                },
            ))
        })
        .collect()
}

/// The Buttplug output for a toy, with one port per feature.
pub fn buttplug_output(toy: &VCToy) -> ButtplugOutput {
    ButtplugOutput::new(toy.device_handle.clone(), toy_actuators(toy))
}
