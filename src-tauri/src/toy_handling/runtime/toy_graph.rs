/*
 * Builds each toy's vibecheck-core Graph from its config.
 *
 * For every enabled feature (motor), one chain per source:
 *
 *   OSC parameter:   osc/<address> -> round -> mode -> levels -> motor
 *   SPS/TPS output:  toy/<Motor>/input_processor -> mode -> levels -> motor
 *
 * - "round" rounds OSC floats to hundredths as f32, exactly as VibeCheck
 *   always has.
 * - "mode" is the parameter's (or the SPS/TPS output's) processing mode.
 * - "levels" is Idle Speed, Range and Flip Input.
 *
 * Only the first parameter with a given address counts within a feature.
 * SPS/TPS chains come before parameter chains, so if both change in the same
 * tick the parameter wins, as it did when messages were processed one by one.
 *
 * The graph is rebuilt whenever the toy's config changes.
 */

use std::{collections::HashSet, time::Duration};

use vibecheck_core::{
    graph::{Graph, GraphBuilder},
    modules::{
        levels::OutputLevels,
        modes::{self, Mode, ModeSettings},
    },
    Event, ModuleInstanceId, PortId, PortSpec, Processor, Result, Signal,
};

use crate::toy_handling::{
    runtime::toy_output::feature_port_name,
    toyops::{ProcessingMode, VCFeatureType, VCToyFeature, VCToyFeatures},
};

/// The module id OscInput writes parameter values under.
pub const OSC_MODULE: &str = "osc";

/// Smooth Level is this many milliseconds of averaging per step.
pub const SMOOTH_MS_PER_LEVEL: u64 = 50;

fn to_core_mode(mode: &ProcessingMode) -> Mode {
    match mode {
        ProcessingMode::Raw => Mode::Raw,
        ProcessingMode::Smooth => Mode::Smooth,
        ProcessingMode::Rate => Mode::Rate,
        ProcessingMode::Constant => Mode::Constant,
    }
}

/// Where a feature's SPS/TPS output is written on the signal plane.
pub fn input_processor_port(toy: &ModuleInstanceId, feature: &VCToyFeature) -> PortId {
    PortId::new(
        toy.clone(),
        format!(
            "{}/input_processor",
            feature_port_name(feature.feature_type, feature.feature_index)
        ),
    )
}

/// Rounds OSC floats to hundredths in f32 before widening, as VibeCheck
/// always has (so 0.4 arrives as 0.4f32 widened). Booleans pass through.
struct OscRounding;

impl Processor for OscRounding {
    fn ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::signal_in("in"), PortSpec::signal_out("out")]
    }

    fn tick(&mut self, ins: &[Signal], _: f32, out: &mut [Option<Signal>], _: &mut Vec<Event>) {
        out[0] = Some(match ins[0] {
            Signal::Float(v) => Signal::Float((((v as f32) * 100.0).round() / 100.0) as f64),
            other => other,
        });
    }
}

struct Chain<'a> {
    prefix: String,
    source: PortId,
    round: bool,
    mode: Mode,
    settings: ModeSettings,
    levels: OutputLevels,
    motor: &'a PortId,
    duration_ms: Option<u32>,
}

fn add_chain(b: &mut GraphBuilder, chain: Chain) -> Result<()> {
    let node = |name: &str| ModuleInstanceId::new(format!("{}/{}", chain.prefix, name));
    let mut upstream = chain.source;

    if chain.round {
        b.add_processor(node("round"), Box::new(OscRounding))?;
        b.connect(upstream, PortId::new(node("round"), "in"))?;
        upstream = PortId::new(node("round"), "out");
    }

    b.add_processor(node("mode"), modes::build(chain.mode, chain.settings))?;
    b.connect(upstream, PortId::new(node("mode"), "in"))?;

    b.add_processor(node("levels"), Box::new(chain.levels))?;
    b.connect(
        PortId::new(node("mode"), "out"),
        PortId::new(node("levels"), "in"),
    )?;

    b.add_sink(
        PortId::new(node("levels"), "out"),
        chain.motor.clone(),
        chain.duration_ms,
    )
}

/// The graph for one toy. `toy` is the toy's module id (see `toy_module_id`).
pub fn build_toy_graph(toy: &ModuleInstanceId, features: &VCToyFeatures) -> Result<Graph> {
    let mut b = GraphBuilder::new();
    let osc = ModuleInstanceId::new(OSC_MODULE);

    for feature in features.features.iter().filter(|f| f.feature_enabled) {
        let motor_name = feature_port_name(feature.feature_type, feature.feature_index);
        let motor = PortId::new(toy.clone(), &motor_name);
        let prefix = format!("{toy}/{motor_name}");

        let l = feature.feature_levels;
        let settings = ModeSettings {
            smooth_window: Duration::from_millis(
                SMOOTH_MS_PER_LEVEL * l.smooth_rate.max(1.0) as u64,
            ),
            rate_decay: l.rate_tune,
            constant_level: l.constant_level,
            flip: feature.flip_input_float,
        };
        let levels = OutputLevels {
            min: l.minimum_level,
            max: l.maximum_level,
            idle: l.idle_level,
            flip: feature.flip_input_float,
        };
        let duration_ms = match feature.feature_type {
            VCFeatureType::Linear => Some(l.linear_position_speed),
            _ => None,
        };

        if feature.penetration_system.pen_system.is_some() {
            add_chain(
                &mut b,
                Chain {
                    prefix: format!("{prefix}/input_processor"),
                    source: input_processor_port(toy, feature),
                    round: false,
                    mode: to_core_mode(&feature.penetration_system.pen_system_processing_mode),
                    settings,
                    levels,
                    motor: &motor,
                    duration_ms,
                },
            )?;
        }

        let mut seen = HashSet::new();
        for (i, param) in feature.osc_parameters.iter().enumerate() {
            if !seen.insert(param.parameter.as_str()) {
                continue; // Only the first parameter with an address counts
            }
            add_chain(
                &mut b,
                Chain {
                    prefix: format!("{prefix}/param{i}"),
                    source: PortId::new(osc.clone(), &param.parameter),
                    round: true,
                    mode: to_core_mode(&param.processing_mode),
                    settings,
                    levels,
                    motor: &motor,
                    duration_ms,
                },
            )?;
        }
    }

    b.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toy_handling::{
        input_processor::penetration_systems::{tps::TPSProcessor, PenetrationSystemType},
        toyops::ToyParameter,
    };
    use vibecheck_core::{SafetyLayer, SignalPlane};

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
        toy: ModuleInstanceId,
        graph: Graph,
        signals: SignalPlane,
    }

    impl Harness {
        fn new(features: Vec<VCToyFeature>) -> Self {
            let toy = ModuleInstanceId::new("toy:test:0");
            let graph = build_toy_graph(&toy, &VCToyFeatures { features }).unwrap();
            Self {
                toy,
                graph,
                signals: SignalPlane::new(),
            }
        }

        fn osc(&self, addr: &str, signal: Signal) {
            self.signals.write(
                &PortId::new(ModuleInstanceId::new(OSC_MODULE), addr),
                signal,
            );
        }

        /// One 10 ms tick; returns (motor, level, duration) for each command.
        fn tick(&mut self) -> Vec<(String, f64, Option<u32>)> {
            self.graph
                .tick(
                    &self.signals,
                    0.01,
                    &SafetyLayer::default(),
                    &mut Vec::new(),
                )
                .iter()
                .map(|c| (c.target().port.to_string(), c.level(), c.duration_ms()))
                .collect()
        }
    }

    #[test]
    fn nothing_is_sent_before_the_first_input() {
        let mut idle = feature(
            0,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Raw)],
        );
        idle.feature_levels.idle_level = 0.2;
        let mut h = Harness::new(vec![idle]);
        assert!(h.tick().is_empty());

        h.osc(BOOP, Signal::Float(0.0));
        assert_eq!(h.tick(), vec![("Vibrator_0".into(), 0.2, None)]);
    }

    #[test]
    fn modes_from_config_then_levels_for_each_motor() {
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

        // One parameter drives both motors, each with its own mode and levels
        h.osc(BOOP, Signal::Bool(true));
        assert_eq!(
            h.tick(),
            vec![
                ("Vibrator_0".into(), 0.65, None),
                ("Vibrator_1".into(), 0.5, None), // Raw true = 1.0, capped by Range
            ]
        );
        // Unchanged: nothing more is sent
        assert!(h.tick().is_empty());

        // Raw 0 becomes Idle Speed; Constant off is 0 with no idle set
        h.osc(BOOP, Signal::Float(0.0));
        assert_eq!(
            h.tick(),
            vec![
                ("Vibrator_0".into(), 0.0, None),
                ("Vibrator_1".into(), 0.1, None)
            ]
        );
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
        h.osc(BOOP, Signal::Float(0.8));
        assert_eq!(h.tick(), vec![("Vibrator_0".into(), 0.5, None)]);
    }

    #[test]
    fn linear_gets_its_speed_and_floats_are_rounded_as_before() {
        let mut linear = feature(
            0,
            VCFeatureType::Linear,
            vec![param(BOOP, ProcessingMode::Raw)],
        );
        linear.feature_levels.linear_position_speed = 250;
        let mut h = Harness::new(vec![linear]);

        // OSC floats arrive as f32 and are rounded as f32: 0.4 becomes 0.4f32 widened
        h.osc(BOOP, Signal::Float(0.401f32 as f64));
        assert_eq!(
            h.tick(),
            vec![("Linear_0".into(), 0.4f32 as f64, Some(250))]
        );

        h.osc("/avatar/parameters/Other", Signal::Float(0.9));
        assert!(h.tick().is_empty());
    }

    #[test]
    fn rate_winds_down_after_the_input_stops() {
        let mut rate = feature(
            0,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Rate)],
        );
        rate.feature_levels.idle_level = 0.05;
        let mut h = Harness::new(vec![rate]);

        h.osc(BOOP, Signal::Float(0.8));
        let first = h.tick();
        assert!(first[0].1 > 0.7);

        // The value stays at 0.8 (VRChat sends nothing new): the level keeps
        // falling and ends at Idle Speed
        let mut last = first;
        for _ in 0..100 {
            let sent = h.tick();
            if !sent.is_empty() {
                last = sent;
            }
        }
        assert_eq!(last, vec![("Vibrator_0".into(), 0.05, None)]);
    }

    #[test]
    fn smooth_follows_over_its_window() {
        let mut smooth = feature(
            0,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Smooth)],
        );
        smooth.feature_levels.smooth_rate = 2.0; // 100 ms
        let mut h = Harness::new(vec![smooth]);

        h.osc(BOOP, Signal::Float(0.2));
        for _ in 0..10 {
            h.tick();
        }
        h.osc(BOOP, Signal::Float(0.8));
        let mut levels = Vec::new();
        for _ in 0..12 {
            levels.extend(h.tick().into_iter().map(|c| c.1));
        }
        // Rises step by step rather than jumping, and settles on 0.8
        assert!(levels.len() > 3, "{levels:?}");
        assert!(levels.windows(2).all(|w| w[1] >= w[0]), "{levels:?}");
        assert_eq!(*levels.last().unwrap(), 0.8);
    }

    #[test]
    fn input_processor_output_has_its_own_chain() {
        let mut tps = feature(0, VCFeatureType::Vibrator, vec![]);
        tps.penetration_system.pen_system_type = PenetrationSystemType::Tps;
        tps.penetration_system.pen_system = Some(Box::<TPSProcessor>::default());
        tps.feature_levels.maximum_level = 0.6;
        let mut h = Harness::new(vec![tps.clone()]);

        h.signals
            .write(&input_processor_port(&h.toy, &tps), Signal::Float(0.9));
        assert_eq!(h.tick(), vec![("Vibrator_0".into(), 0.6, None)]);
    }
}
