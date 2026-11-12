/*
 * Runs a toy's SPS/TPS input processors on OSC messages.
 *
 * Input processors are still message-driven and live on the feature config
 * (they are due a rewrite). Their output for each feature is written to the
 * signal plane, where the toy's graph picks it up (see toy_graph.rs).
 *
 * Plain OSC parameters don't come through here: OscInput writes them to the
 * signal plane directly.
 */

use rosc::{OscMessage, OscType};
use vibecheck_core::{ModuleInstanceId, Signal, SignalPlane};

use crate::toy_handling::runtime::toy_graph::input_processor_port;
use crate::toy_handling::toyops::VCToyFeatures;

use super::mode_processor::core::ModeProcessorInputType;

/// OSC floats are rounded to hundredths, then widened to f64.
fn round_osc_float(lvl: f32) -> f64 {
    ((lvl * 100.0).round() / 100.0) as f64
}

pub fn run_input_processors(
    msg: &mut OscMessage,
    vc_toy_features: &mut VCToyFeatures,
    signals: &SignalPlane,
    toy: &ModuleInstanceId,
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

    for feature in &mut vc_toy_features.features {
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
            signals.write(&input_processor_port(toy, feature), Signal::Float(value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toy_handling::{
        input_processor::{penetration_systems::PenetrationSystemType, InputProcessor},
        toyops::{VCFeatureType, VCToyFeature},
    };

    /// Accepts /test/... addresses and halves float input.
    #[derive(Clone, Debug)]
    struct Halver;

    impl InputProcessor for Halver {
        fn process(&mut self, _: &str, input: ModeProcessorInputType) -> Option<f64> {
            input.try_float().map(|f| f / 2.0)
        }
        fn is_parameter(&self, param: &str) -> bool {
            param.starts_with("/test/")
        }
        fn input_filter(&self, param: &str, filters: &Vec<String>) -> bool {
            filters.iter().any(|f| param.contains(f))
        }
    }

    fn with_processor(index: u32, filter: Option<Vec<String>>) -> VCToyFeature {
        let mut f = VCToyFeature::new(vec![], index, VCFeatureType::Vibrator);
        f.penetration_system.pen_system_type = PenetrationSystemType::Sps;
        f.penetration_system.pen_system = Some(Box::new(Halver));
        f.penetration_system.pen_system_input_filter = filter;
        f
    }

    #[test]
    fn writes_output_for_each_accepting_feature() {
        let toy = ModuleInstanceId::new("toy:test:0");
        let mut disabled = with_processor(2, None);
        disabled.feature_enabled = false;
        let mut features = VCToyFeatures {
            features: vec![
                with_processor(0, None),
                with_processor(1, Some(vec!["Mouth".into()])),
                disabled,
            ],
        };
        let signals = SignalPlane::new();

        let mut msg = OscMessage {
            addr: "/test/Hand".into(),
            args: vec![OscType::Float(0.8)],
        };
        run_input_processors(&mut msg, &mut features, &signals, &toy);

        let read = |i: usize| signals.read(&input_processor_port(&toy, &features.features[i]));
        // 0.8 arrives rounded as f32, then widened
        assert_eq!(read(0), Some(Signal::Float(0.8f32 as f64 / 2.0)));
        assert_eq!(read(1), None, "filtered out");
        assert_eq!(read(2), None, "disabled");
    }
}
