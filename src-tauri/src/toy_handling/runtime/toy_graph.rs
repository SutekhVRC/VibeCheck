/*
 * Builds each toy's vibecheck-core Graph from its config.
 *
 * For every enabled feature (motor), one chain per source:
 *
 *   OSC parameter:    osc/<address> -> round -> mode -> levels -> motor
 *   Input processor:  input_processor -> mode -> levels -> motor
 *
 * - "round" rounds OSC floats to hundredths as f32, exactly as VibeCheck
 *   always has.
 * - "input_processor" is the feature's SPS/TPS processor (see
 *   input_processor/mod.rs). It watches its OSC parameters itself, so it
 *   has no wired input.
 * - "mode" is the parameter's (or the input processor's) processing mode.
 * - "levels" is Idle Speed and Range. Flip Input is applied by the toy's
 *   output, after the safety layer (master intensity), so the graph and the
 *   safety layer work in intensities where 0 is always off.
 *
 * Only the first parameter with a given address counts within a feature.
 * Input processor chains come before parameter chains, so if both change in
 * the same tick the parameter wins.
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
    EventOut, ModuleInstanceId, PortId, PortSpec, Processor, Result, Signal,
};

use crate::toy_handling::{
    input_processor::InputProcessorNode,
    runtime::toy_output::feature_port_name,
    toyops::{ProcessingMode, VCFeatureType, VCToyFeatures},
};

/// The module id OscInput writes parameter values under.
pub const OSC_MODULE: &str = "osc";

/// Smooth window at Smooth Level 1, in seconds.
pub const SMOOTH_BASE_SECONDS: f64 = 0.25;
/// How the window grows with Smooth Level. Below 1, low levels get
/// relatively more smoothing and high levels relatively less.
pub const SMOOTH_CURVE: f64 = 0.7;

/// How long Smooth averages over for a Smooth Level:
/// `0.25 s × level^0.7`, i.e. about 0.25 s at 1, 0.4 s at 2, 0.75 s at 5,
/// 1.25 s at 10 and 2 s at 20. Levels below 1 count as 1.
pub fn smooth_window(level: f64) -> Duration {
    Duration::from_secs_f64(SMOOTH_BASE_SECONDS * level.max(1.0).powf(SMOOTH_CURVE))
}

fn to_core_mode(mode: &ProcessingMode) -> Mode {
    match mode {
        ProcessingMode::Raw => Mode::Raw,
        ProcessingMode::Smooth => Mode::Smooth,
        ProcessingMode::Rate => Mode::Rate,
        ProcessingMode::Constant => Mode::Constant,
    }
}

/// Rounds OSC floats to hundredths in f32 before widening, as VibeCheck
/// always has (so 0.4 arrives as 0.4f32 widened). Booleans pass through.
struct OscRounding;

impl Processor for OscRounding {
    fn ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::signal_in("in"), PortSpec::signal_out("out")]
    }

    fn tick(&mut self, ins: &[Signal], _: f32, out: &mut [Option<Signal>], _: &mut Vec<EventOut>) {
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
            smooth_window: smooth_window(l.smooth_rate),
            rate_decay: l.rate_tune,
            constant_level: l.constant_level,
            flip: feature.flip_input_float,
        };
        let levels = OutputLevels {
            min: l.minimum_level,
            max: l.maximum_level,
            idle: l.idle_level,
        };
        let duration_ms = match feature.feature_type {
            VCFeatureType::Linear => Some(l.linear_position_speed),
            _ => None,
        };

        let pen = &feature.penetration_system;
        if let Some(processor) = pen.pen_system_type.processor(feature.feature_type) {
            let node = ModuleInstanceId::new(format!("{prefix}/input_processor"));
            let tags = pen.pen_system_input_filter.clone().unwrap_or_default();
            b.add_processor(
                node.clone(),
                Box::new(InputProcessorNode::new(processor, tags)),
            )?;
            add_chain(
                &mut b,
                Chain {
                    prefix: format!("{prefix}/input_processor"),
                    source: PortId::new(node, "out"),
                    round: false,
                    mode: to_core_mode(&pen.pen_system_processing_mode),
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
        input_processor::penetration_systems::PenetrationSystemType,
        toyops::{ToyParameter, VCToyFeature},
    };
    use vibecheck_core::{EventPlane, SafetyLayer, SignalPlane};

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
        graph: Graph,
        signals: SignalPlane,
        events: EventPlane,
    }

    impl Harness {
        fn new(features: Vec<VCToyFeature>) -> Self {
            let toy = ModuleInstanceId::new("toy:test:0");
            let graph = build_toy_graph(&toy, &VCToyFeatures { features }).unwrap();
            Self {
                graph,
                signals: SignalPlane::new(),
                events: EventPlane::new().0,
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
                .tick(&self.signals, 0.01, &SafetyLayer::default(), &self.events)
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
    fn flip_input_is_left_to_the_output() {
        let mut flipped = feature(
            0,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Raw)],
        );
        flipped.flip_input_float = true;
        let mut h = Harness::new(vec![flipped]);

        // The graph sends the intensity; ButtplugOutput flips it to 0.7
        h.osc(BOOP, Signal::Float(0.3));
        assert_eq!(h.tick(), vec![("Vibrator_0".into(), 0.3f32 as f64, None)]);
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
    fn smooth_levels_follow_a_curve() {
        let secs = |level: f64| smooth_window(level).as_secs_f64();
        let close = |a: f64, b: f64| (a - b).abs() < 0.05;
        assert!(close(secs(1.0), 0.25));
        assert!(close(secs(2.0), 0.4));
        assert!(close(secs(5.0), 0.75));
        assert!(close(secs(10.0), 1.25));
        assert!(close(secs(20.0), 2.0));
        // Below 1 counts as 1
        assert_eq!(smooth_window(0.0), smooth_window(1.0));
    }

    #[test]
    fn smooth_follows_over_its_window() {
        let mut smooth = feature(
            0,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Smooth)],
        );
        smooth.feature_levels.smooth_rate = 2.0; // about 0.4 s
        let mut h = Harness::new(vec![smooth]);

        h.osc(BOOP, Signal::Float(0.2));
        for _ in 0..45 {
            h.tick();
        }
        h.osc(BOOP, Signal::Float(0.8));
        let mut levels = Vec::new();
        for _ in 0..50 {
            levels.extend(h.tick().into_iter().map(|c| c.1));
        }
        // Rises step by step rather than jumping, and settles on 0.8
        assert!(levels.len() > 3, "{levels:?}");
        assert!(levels.windows(2).all(|w| w[1] >= w[0]), "{levels:?}");
        assert_eq!(*levels.last().unwrap(), 0.8);
    }

    #[test]
    fn input_processors_get_their_own_chain_next_to_parameters() {
        let mut sps = feature(
            0,
            VCFeatureType::Vibrator,
            vec![param(BOOP, ProcessingMode::Raw)],
        );
        sps.penetration_system.pen_system_type = PenetrationSystemType::Sps;
        let mut h = Harness::new(vec![sps]);
        assert!(h
            .graph
            .node_ids()
            .any(|n| n.as_str() == "toy:test:0/Vibrator_0/input_processor"));

        // SPS parameters reach the motor through the processor, unrounded
        h.osc(
            "/avatar/parameters/OGB/Orf/Mouth/PenOthers",
            Signal::Float(0.7),
        );
        assert_eq!(h.tick(), vec![("Vibrator_0".into(), 0.7, None)]);

        // The feature's own parameters still work alongside
        h.osc(BOOP, Signal::Float(0.5));
        assert_eq!(h.tick(), vec![("Vibrator_0".into(), 0.5, None)]);

        // An avatar change resets the graph: the SPS zone is forgotten and
        // nothing is sent until new input arrives
        h.graph.reset();
        h.signals.clear_module(&ModuleInstanceId::new(OSC_MODULE));
        assert!(h.tick().is_empty());
        h.osc(
            "/avatar/parameters/OGB/Orf/Mouth/PenOthers",
            Signal::Float(0.5),
        );
        // Sent again although 0.5 was the last value before the reset
        assert_eq!(h.tick(), vec![("Vibrator_0".into(), 0.5, None)]);
    }
}
