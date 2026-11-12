use crate::toy_handling::runtime::toy_output::actuator_for;
use crate::toy_handling::toyops::LevelTweaks;
use crate::toy_handling::toyops::VCFeatureType;

use buttplug_client::ButtplugClientDevice;

use log::{debug, error as logerr, warn};
use vibecheck_core::{
    modules::{
        buttplug::{device_level, send_to_actuator, LegacyLayout},
        levels::OutputLevels,
    },
    SafetyLayer,
};

use std::sync::Arc;

/// The level sent to the toy for one value, in the same order as the toy
/// graph and output: Idle Speed and Range, then the master intensity (safety
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

/// Used by the Test button: sends one level to one feature, after the same
/// steps as the toy graph and output (Idle and Range, master intensity,
/// Flip Input).
pub async fn command_toy(
    dev: Arc<ButtplugClientDevice>,
    layout: LegacyLayout,
    feature_type: VCFeatureType,
    float_level: f64,
    feature_index: u32,
    flip_float: bool,
    feature_levels: LevelTweaks,
    safety: SafetyLayer,
) {
    let Some(actuator) = actuator_for(&layout, feature_type, feature_index) else {
        warn!(
            "Test: the toy has no actuator for {:?} {}",
            feature_type, feature_index
        );
        return;
    };
    let level = clamp_and_flip(float_level, flip_float, feature_levels, &safety);
    debug!(
        "Test: sending {} to {:?} {}",
        level, feature_type, feature_index
    );
    let duration = Some(feature_levels.linear_position_speed);
    if let Err(e) = send_to_actuator(&dev, actuator, level, duration).await {
        logerr!("Send to device error: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_level_is_range_then_intensity_then_flip() {
        let mut levels = LevelTweaks::default();
        levels.maximum_level = 0.8;
        let safety = SafetyLayer::default();

        // Full intensity, no flip: unchanged from before
        assert_eq!(clamp_and_flip(0.5, false, levels, &safety), 0.5);
        // Range caps at 0.8 before the intensity halves it, then flip
        safety.set_intensity(0.5);
        assert!((clamp_and_flip(1.0, false, levels, &safety) - 0.4).abs() < 1e-9);
        assert_eq!(clamp_and_flip(1.0, true, levels, &safety), 0.6);
    }
}
