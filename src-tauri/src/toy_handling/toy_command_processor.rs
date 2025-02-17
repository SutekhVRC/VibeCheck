
use crate::toy_handling::toyops::LevelTweaks;
use crate::toy_handling::toyops::VCFeatureType;

use buttplug::client::ButtplugClientDevice;
use buttplug::client::RotateCommand::RotateMap;
use buttplug::client::ScalarCommand::ScalarMap;
use buttplug::core::message::ActuatorType;

use log::{error as logerr, info};


use std::collections::HashMap;

use std::sync::Arc;


#[inline]
fn clamp_and_flip(value: f64, flip: bool, levels: LevelTweaks) -> f64 {
    let mut new_value;
    if value == 0.0 {
        new_value = levels.idle_level;
    } else {
        new_value = value.clamp(levels.minimum_level, levels.maximum_level);
    }
    if flip {
        new_value = flip_float64(new_value)
    }
    new_value
}

#[inline]
pub fn flip_float64(orig: f64) -> f64 {
    //1.00 - orig
    ((1.00 - orig) * 100.0).round() / 100.0
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
) {
    let new_level = clamp_and_flip(scalar_level, flip_float, feature_levels);
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
pub async fn command_toy(
    dev: Arc<ButtplugClientDevice>,
    feature_type: VCFeatureType,
    float_level: f64,
    feature_index: u32,
    flip_float: bool,
    feature_levels: LevelTweaks,
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
            )
            .await;
        }
        // We handle Rotator differently because it is not included in the Scalar feature set
        VCFeatureType::Rotator => {
            let new_level = clamp_and_flip(float_level, flip_float, feature_levels);
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
            )
            .await;
        }
        // We handle Linear differently because it is not included in the Scalar feature set
        VCFeatureType::Linear => {
            let new_level = clamp_and_flip(float_level, flip_float, feature_levels);
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
            )
            .await;
        }
    }
}