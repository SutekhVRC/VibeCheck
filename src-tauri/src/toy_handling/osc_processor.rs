use log::{debug, info};
use rosc::{OscMessage, OscType};
use vibecheck_core::{modules::levels::OutputLevels, process_one, Processor, Signal};

use crate::toy_handling::runtime::toy_output::ToyCommandSink;
use crate::toy_handling::runtime::toy_pipeline::{FeaturePipeline, ToyPipeline};
use crate::toy_handling::toyops::{VCToyFeature, VCToyFeatures};

use super::mode_processor::core::ModeProcessorInputType;

/// Runs a value through a mode processor, then the feature's output levels,
/// and queues the result for the toy. Nothing is sent if the mode holds the
/// value back (e.g. Smooth still collecting).
fn process_and_send(
    command_sink: &ToyCommandSink,
    feature: &VCToyFeature,
    mode: &mut dyn Processor,
    levels: &OutputLevels,
    input: Signal,
) {
    if let Some(level) = process_one(mode, input, 0.0) {
        command_sink.send(
            feature.feature_type,
            feature.feature_index,
            feature.feature_levels.linear_position_speed,
            levels.apply(level.as_level()),
        );
    }
}

/// OSC floats are rounded to hundredths, then widened to f64.
fn round_osc_float(lvl: f32) -> f64 {
    ((lvl * 100.0).round() / 100.0) as f64
}

/// Processes one OSC message for one toy.
///
/// Features whose input processor (SPS/TPS) accepts the address go first,
/// then features with a matching parameter, each in feature order.
pub fn parse_osc_message(
    command_sink: &ToyCommandSink,
    msg: &mut OscMessage,
    vc_toy_features: &mut VCToyFeatures,
    pipeline: &mut ToyPipeline,
) {
    // A message without arguments carries no value to act on
    let Some(newest_msg_val) = msg.args.pop() else {
        return;
    };

    let input = match newest_msg_val {
        OscType::Float(lvl) => ModeProcessorInputType::Float(round_osc_float(lvl)),
        OscType::Bool(b) => ModeProcessorInputType::Boolean(b),
        _ => return, // Unsupported OSC type
    };

    // Input processors (SPS/TPS)
    for (feature, pipe) in vc_toy_features
        .features
        .iter_mut()
        .zip(pipeline.features.iter_mut())
    {
        if !feature.input_processor_accepts(&msg.addr) {
            continue;
        }
        // input_processor_accepts guarantees there is an input processor
        let processed = feature
            .penetration_system
            .pen_system
            .as_mut()
            .unwrap()
            .process(msg.addr.as_str(), input);
        if let Some(value) = processed {
            let FeaturePipeline {
                input_processor_mode,
                levels,
                ..
            } = pipe;
            process_and_send(
                command_sink,
                feature,
                input_processor_mode.as_mut(),
                levels,
                Signal::Float(value),
            );
        }
    }

    // Parameters
    let signal = match input {
        ModeProcessorInputType::Float(f) => {
            debug!("Received and cast float lvl: {:.5}", f);
            Signal::Float(f)
        }
        ModeProcessorInputType::Boolean(b) => {
            info!("Got a Bool! {} = {}", msg.addr, b);
            Signal::Bool(b)
        }
    };
    for (feature, pipe) in vc_toy_features
        .features
        .iter()
        .zip(pipeline.features.iter_mut())
    {
        // Only the first parameter with this address counts
        let Some(param_index) = feature.enabled_param_index(&msg.addr) else {
            continue;
        };
        let FeaturePipeline { params, levels, .. } = pipe;
        process_and_send(
            command_sink,
            feature,
            params[param_index].as_mut(),
            levels,
            signal,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toy_handling::toyops::{ProcessingMode, ToyParameter, VCFeatureType};
    use vibecheck_core::{CommandSlots, ModuleInstanceId};

    const BOOP: &str = "/avatar/parameters/Boop";

    fn param(addr: &str, mode: ProcessingMode) -> ToyParameter {
        ToyParameter {
            parameter: addr.into(),
            processing_mode: mode,
        }
    }

    fn feature(index: u32, kind: VCFeatureType, params: Vec<ToyParameter>) -> VCToyFeature {
        VCToyFeature::new(params, index, kind)
    }

    struct Harness {
        sink: ToyCommandSink,
        slots: CommandSlots,
        features: VCToyFeatures,
        pipeline: ToyPipeline,
    }

    impl Harness {
        fn new(features: Vec<VCToyFeature>) -> Self {
            let slots = CommandSlots::new();
            let features = VCToyFeatures { features };
            Self {
                sink: ToyCommandSink::for_module(
                    ModuleInstanceId::new("toy:test:0"),
                    slots.clone(),
                ),
                pipeline: ToyPipeline::new(&features),
                slots,
                features,
            }
        }

        /// Sends one OSC value and returns (port, level, duration) for every
        /// command queued.
        fn send(&mut self, addr: &str, arg: OscType) -> Vec<(String, f64, Option<u32>)> {
            let mut msg = OscMessage {
                addr: addr.into(),
                args: vec![arg],
            };
            parse_osc_message(&self.sink, &mut msg, &mut self.features, &mut self.pipeline);
            self.slots
                .take_changed()
                .iter()
                .map(|c| (c.target().port.to_string(), c.level(), c.duration_ms()))
                .collect()
        }
    }

    #[test]
    fn modes_from_config_then_levels() {
        let mut constant = feature(
            0,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Constant)],
        );
        constant.feature_levels.constant_level = 0.65;
        let mut ranged = feature(
            1,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Raw)],
        );
        ranged.feature_levels.maximum_level = 0.5;
        ranged.feature_levels.idle_level = 0.1;
        let mut h = Harness::new(vec![constant, ranged]);

        // One message drives both motors, each with its own mode and levels
        assert_eq!(
            h.send(BOOP, OscType::Bool(true)),
            vec![
                ("Vibrator_0".into(), 0.65, None),
                ("Vibrator_1".into(), 0.5, None), // Raw true = 1.0, capped by Range
            ]
        );
        // Raw 0 becomes Idle Speed; Constant off is 0 with no idle set
        assert_eq!(
            h.send(BOOP, OscType::Float(0.0)),
            vec![
                ("Vibrator_0".into(), 0.0, None),
                ("Vibrator_1".into(), 0.1, None)
            ]
        );
    }

    #[test]
    fn smooth_holds_values_back_until_its_window_fills() {
        let mut smooth = feature(
            0,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Smooth)],
        );
        smooth.feature_levels.smooth_rate = 2.0;
        let mut h = Harness::new(vec![smooth]);

        assert!(h.send(BOOP, OscType::Float(0.2)).is_empty());
        assert!(h.send(BOOP, OscType::Float(0.4)).is_empty());
        let sent = h.send(BOOP, OscType::Float(0.9));
        assert_eq!(sent.len(), 1);
        assert!((sent[0].1 - 0.3).abs() < 1e-9);
    }

    #[test]
    fn first_matching_parameter_wins_and_disabled_features_are_skipped() {
        let first_wins = feature(
            0,
            VCFeatureType::Vibrator,
            vec![
                param(BOOP, ProcessingMode::Constant),
                param(BOOP, ProcessingMode::Raw),
            ],
        );
        let mut disabled = feature(
            1,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Raw)],
        );
        disabled.feature_enabled = false;
        let mut h = Harness::new(vec![first_wins, disabled]);

        // Constant (default level 0.5), not Raw's 0.8; the disabled motor gets nothing
        assert_eq!(
            h.send(BOOP, OscType::Float(0.8)),
            vec![("Vibrator_0".into(), 0.5, None)]
        );
    }

    #[test]
    fn linear_gets_its_speed_and_other_values_are_ignored() {
        let mut linear = feature(
            0,
            VCFeatureType::Linear,
            vec![param(BOOP, ProcessingMode::Raw)],
        );
        linear.feature_levels.linear_position_speed = 250;
        let mut h = Harness::new(vec![linear]);

        assert_eq!(
            h.send(BOOP, OscType::Float(0.4)),
            // OSC floats are rounded as f32, so 0.4 arrives as 0.4f32 widened
            vec![("Linear_0".into(), 0.4f32 as f64, Some(250))]
        );
        assert!(h
            .send("/avatar/parameters/Other", OscType::Float(0.4))
            .is_empty());
        assert!(h.send(BOOP, OscType::Int(1)).is_empty());
    }
}
