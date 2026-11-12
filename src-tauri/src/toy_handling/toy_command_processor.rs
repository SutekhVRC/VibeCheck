use crate::toy_handling::toyops::LevelTweaks;
use crate::toy_handling::toyops::VCFeatureType;

use buttplug::client::ButtplugClientDevice;
use buttplug::client::RotateCommand::RotateMap;
use buttplug::client::ScalarCommand::ScalarMap;
use buttplug::core::message::ActuatorType;

use log::{error as logerr, info};
use vibecheck_core::{
    modules::{buttplug::device_level, levels::OutputLevels},
    SafetyLayer,
};

use std::collections::HashMap;

use std::sync::Arc;

/// The level sent to the toy for one value, in the same order as the toy
/// graph and output: Idle Speed and Range, then the master volume (safety
/// layer), then Flip Input.
#[inline]
pub(crate) fn clamp_and_flip(
    value: f64,
    flip: bool,
    levels: LevelTweaks,
    safety: &SafetyLayer,
) -> f64 {
    let level = OutputLevels {
        min: levels.minimum_level,
        max: levels.maximum_level,
        idle: levels.idle_level,
    }
    .apply(value);
    device_level(safety.limit(level), flip)
}

// Parse scalar levels and logic for level tweaks
#[inline]
pub async fn scalar_parse_levels_send_toy_cmd(
    dev: &Arc<ButtplugClientDevice>,
    scalar_level: f64,
    feature_index: u32,
    actuator_type: ActuatorType,
    flip_float: bool,
    feature_levels: LevelTweaks,
    safety: &SafetyLayer,
) {
    let new_level = clamp_and_flip(scalar_level, flip_float, feature_levels, safety);
    #[cfg(debug_assertions)]
    {
        let message_prefix = if scalar_level == 0.0 {
            "IDLE"
        } else {
            "SENDING"
        };
        info!(
            "{} FI[{}] AT[{}] SL[{}]",
            message_prefix, feature_index, actuator_type, new_level
        );
    }
    match dev
        .scalar(&ScalarMap(HashMap::from([(
            feature_index,
            (new_level, actuator_type),
        )])))
        .await
    {
        Ok(()) => {}
        Err(e) => {
            logerr!("Send scalar to device error: {}", e);
        }
    }
}

/*
 * Sends commands to toys
 */
/// Used by the Test button. Applies the master volume like the toy graph does.
pub async fn command_toy(
    dev: Arc<ButtplugClientDevice>,
    feature_type: VCFeatureType,
    float_level: f64,
    feature_index: u32,
    flip_float: bool,
    feature_levels: LevelTweaks,
    safety: SafetyLayer,
) {
    match feature_type {
        VCFeatureType::Vibrator => {
            scalar_parse_levels_send_toy_cmd(
                &dev,
                float_level,
                feature_index,
                ActuatorType::Vibrate,
                flip_float,
                feature_levels,
                &safety,
            )
            .await;
        }
        // We handle Rotator differently because it is not included in the Scalar feature set
        VCFeatureType::Rotator => {
            let new_level = clamp_and_flip(float_level, flip_float, feature_levels, &safety);
            let _ = dev
                .rotate(&RotateMap(HashMap::from([(
                    feature_index,
                    (new_level, true),
                )])))
                .await;
        }
        VCFeatureType::Constrict => {
            scalar_parse_levels_send_toy_cmd(
                &dev,
                float_level,
                feature_index,
                ActuatorType::Constrict,
                flip_float,
                feature_levels,
                &safety,
            )
            .await;
        }
        VCFeatureType::Oscillate => {
            scalar_parse_levels_send_toy_cmd(
                &dev,
                float_level,
                feature_index,
                ActuatorType::Oscillate,
                flip_float,
                feature_levels,
                &safety,
            )
            .await;
        }
        VCFeatureType::Position => {
            scalar_parse_levels_send_toy_cmd(
                &dev,
                float_level,
                feature_index,
                ActuatorType::Position,
                flip_float,
                feature_levels,
                &safety,
            )
            .await;
        }
        VCFeatureType::Inflate => {
            scalar_parse_levels_send_toy_cmd(
                &dev,
                float_level,
                feature_index,
                ActuatorType::Inflate,
                flip_float,
                feature_levels,
                &safety,
            )
            .await;
        }
        // We handle Linear differently because it is not included in the Scalar feature set
        VCFeatureType::Linear => {
            let new_level = clamp_and_flip(float_level, flip_float, feature_levels, &safety);
            let _ = dev
                .linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from(
                    [(
                        feature_index,
                        (feature_levels.linear_position_speed, new_level),
                    )],
                )))
                .await;
        }
        VCFeatureType::ScalarRotator => {
            scalar_parse_levels_send_toy_cmd(
                &dev,
                float_level,
                feature_index,
                ActuatorType::Rotate,
                flip_float,
                feature_levels,
                &safety,
            )
            .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_level_is_range_then_volume_then_flip() {
        let mut levels = LevelTweaks::default();
        levels.maximum_level = 0.8;
        let safety = SafetyLayer::default();

        // Full volume, no flip: unchanged from before
        assert_eq!(clamp_and_flip(0.5, false, levels, &safety), 0.5);
        // Range caps at 0.8 before the volume halves it, then flip
        safety.set_volume(0.5);
        assert!((clamp_and_flip(1.0, false, levels, &safety) - 0.4).abs() < 1e-9);
        assert_eq!(clamp_and_flip(1.0, true, levels, &safety), 0.6);
    }
}
