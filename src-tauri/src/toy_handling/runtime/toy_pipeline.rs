/*
 * Per-toy processing built from the toy's config.
 *
 * For every feature: one mode processor per OSC parameter, one for the
 * feature's input processor (SPS/TPS) output, and the output level tweaks.
 * All are vibecheck-core processors.
 *
 * The pipeline is rebuilt whenever the toy's config changes (alter_toy),
 * which resets processor state, as the old per-parameter state did.
 * The SPS/TPS input processors themselves still live on the feature config.
 */

use vibecheck_core::{
    modules::{
        levels::OutputLevels,
        modes::{self, Mode, ModeSettings},
    },
    Processor,
};

use crate::toy_handling::toyops::{ProcessingMode, VCToyFeature, VCToyFeatures};

fn to_core_mode(mode: &ProcessingMode) -> Mode {
    match mode {
        ProcessingMode::Raw => Mode::Raw,
        ProcessingMode::Smooth => Mode::Smooth,
        ProcessingMode::Rate => Mode::Rate,
        ProcessingMode::Constant => Mode::Constant,
    }
}

pub struct FeaturePipeline {
    /// Mode processors, in the same order as the feature's `osc_parameters`.
    pub params: Vec<Box<dyn Processor>>,
    /// Mode processor for the input processor's (SPS/TPS) output.
    pub input_processor_mode: Box<dyn Processor>,
    /// Idle Speed, Range and Flip Input.
    pub levels: OutputLevels,
}

impl FeaturePipeline {
    fn new(feature: &VCToyFeature) -> Self {
        let levels = feature.feature_levels;
        let settings = ModeSettings {
            smooth_size: levels.smooth_rate as usize,
            rate_decay: levels.rate_tune,
            constant_level: levels.constant_level,
            flip: feature.flip_input_float,
        };
        Self {
            params: feature
                .osc_parameters
                .iter()
                .map(|p| modes::build(to_core_mode(&p.processing_mode), settings))
                .collect(),
            input_processor_mode: modes::build(
                to_core_mode(&feature.penetration_system.pen_system_processing_mode),
                settings,
            ),
            levels: OutputLevels {
                min: levels.minimum_level,
                max: levels.maximum_level,
                idle: levels.idle_level,
                flip: feature.flip_input_float,
            },
        }
    }
}

/// One `FeaturePipeline` per feature, in the same order as the toy's features.
pub struct ToyPipeline {
    pub features: Vec<FeaturePipeline>,
}

impl ToyPipeline {
    pub fn new(features: &VCToyFeatures) -> Self {
        Self {
            features: features.features.iter().map(FeaturePipeline::new).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toy_handling::toyops::{ToyParameter, VCFeatureType};
    use vibecheck_core::{process_one, Signal};

    fn param(mode: ProcessingMode) -> ToyParameter {
        ToyParameter {
            parameter: "/avatar/parameters/Boop".into(),
            processing_mode: mode,
        }
    }

    #[test]
    fn builds_processors_from_saved_modes() {
        let mut feature = VCToyFeature::new(
            vec![param(ProcessingMode::Constant), param(ProcessingMode::Raw)],
            0,
            VCFeatureType::Vibrator,
        );
        feature.feature_levels.constant_level = 0.65;
        let mut pipeline = ToyPipeline::new(&VCToyFeatures {
            features: vec![feature],
        });

        let f = &mut pipeline.features[0];
        assert_eq!(f.params.len(), 2);
        // Constant mode straight from config, without any alter_toy first
        assert_eq!(
            process_one(f.params[0].as_mut(), Signal::Bool(true), 0.0),
            Some(Signal::Float(0.65))
        );
        assert_eq!(
            process_one(f.params[1].as_mut(), Signal::Float(0.3), 0.0),
            Some(Signal::Float(0.3))
        );
    }
}
