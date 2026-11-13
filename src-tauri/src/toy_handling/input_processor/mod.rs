/*
 * Input processors: turn a family of OSC parameters into one motor level.
 *
 * A plain parameter drives a motor directly. An input processor reads every
 * parameter under some address prefixes, including ones that aren't known
 * until the avatar sends them (for example one set per SPS socket), and
 * works out a single level from all of them. SPS and TPS are input
 * processors.
 *
 * Each feature with an input processor gets its own instance, as a node in
 * the toy's graph (see runtime/toy_graph.rs):
 *
 *   OSC parameters under the prefixes
 *     -> input filter (the feature's tags)
 *     -> InputProcessor::update, once per changed parameter
 *     -> InputProcessor::tick, every graph tick (100 Hz)
 *     -> processing mode -> Idle/Range -> motor
 *
 * A new instance is made whenever the toy's settings change, so any state
 * an input processor keeps (such as learned lengths) starts over then. On
 * avatar change, InputProcessor::reset is called instead.
 *
 * To add an input processor: implement InputProcessor, add a variant to
 * PenetrationSystemType and return the new processor from
 * PenetrationSystemType::processor.
 */

use vibecheck_core::{EventOut, ModuleInstanceId, PortId, PortSpec, Processor, Signal, Watch};

use crate::toy_handling::runtime::toy_graph::OSC_MODULE;

pub mod penetration_systems;

pub trait InputProcessor: Send {
    /// The OSC address prefixes this processor reads, e.g.
    /// `/avatar/parameters/OGB/`. Asked once, when the graph is built.
    fn prefixes(&self) -> Vec<String>;

    /// Called with each parameter under the prefixes that changed since the
    /// last tick (and passed the input filter), oldest change first. If a
    /// parameter changed several times within one tick, only its latest
    /// value is seen.
    ///
    /// `addr` is the full OSC address, e.g. `/avatar/parameters/OGB/Orf/Mouth/PenOthers`.
    /// Values are as VRChat sent them: floats aren't rounded.
    fn update(&mut self, addr: &str, value: Signal);

    /// Called every graph tick, after any updates. `dt` is the time since
    /// the last tick, in seconds. Return the motor level (0.0 to 1.0) to set
    /// it, or `None` to leave it where it is.
    fn tick(&mut self, dt: f32) -> Option<f64>;

    /// Forget everything learned from past input, as if new. Called when
    /// the avatar changes, after its parameters have been cleared, so
    /// nothing from the old avatar (zones, learned lengths, flags) carries
    /// over. Until new input arrives, `tick` should return `None`.
    fn reset(&mut self);

    /// Whether `addr` passes the feature's input filter tags. Only asked when
    /// the feature has tags. Default: the address contains any of the tags.
    fn input_filter(&self, addr: &str, tags: &[String]) -> bool {
        tags.iter().any(|tag| addr.contains(tag.as_str()))
    }
}

/// Runs an input processor as a graph node: watches its prefixes on the
/// signal plane, applies the input filter and outputs its level on `out`.
pub struct InputProcessorNode {
    processor: Box<dyn InputProcessor>,
    tags: Vec<String>,
}

impl InputProcessorNode {
    /// `tags` is the feature's input filter. No tags means everything passes.
    pub fn new(processor: Box<dyn InputProcessor>, tags: Vec<String>) -> Self {
        Self { processor, tags }
    }
}

impl Processor for InputProcessorNode {
    fn ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::signal_out("out")]
    }

    fn watches(&self) -> Vec<Watch> {
        let osc = ModuleInstanceId::new(OSC_MODULE);
        self.processor
            .prefixes()
            .into_iter()
            .map(|prefix| Watch::new(osc.clone(), prefix))
            .collect()
    }

    fn watched(&mut self, changed: &[(PortId, Signal)]) {
        for (port, value) in changed {
            let addr = &*port.port;
            if self.tags.is_empty() || self.processor.input_filter(addr, &self.tags) {
                self.processor.update(addr, *value);
            }
        }
    }

    fn tick(&mut self, _: &[Signal], dt: f32, out: &mut [Option<Signal>], _: &mut Vec<EventOut>) {
        out[0] = self.processor.tick(dt).map(Signal::Float);
    }

    fn reset(&mut self) {
        self.processor.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibecheck_core::{graph::GraphBuilder, EventPlane, SafetyLayer, SignalPlane};

    /// Outputs the latest value it was given, halved.
    #[derive(Default)]
    struct Halver(Option<f64>);

    impl InputProcessor for Halver {
        fn prefixes(&self) -> Vec<String> {
            vec!["/avatar/parameters/Half/".into()]
        }
        fn update(&mut self, _: &str, value: Signal) {
            self.0 = Some(value.as_level() / 2.0);
        }
        fn tick(&mut self, _: f32) -> Option<f64> {
            self.0.take()
        }
        fn reset(&mut self) {
            self.0 = None;
        }
    }

    fn run(tags: Vec<String>, writes: &[(&str, f64)]) -> Vec<f64> {
        let mut b = GraphBuilder::new();
        let node = ModuleInstanceId::new("ip");
        b.add_processor(
            node.clone(),
            Box::new(InputProcessorNode::new(Box::<Halver>::default(), tags)),
        )
        .unwrap();
        b.add_sink(
            PortId::new(node, "out"),
            PortId::new(ModuleInstanceId::new("toy"), "Vibrator_0"),
            None,
        )
        .unwrap();
        let mut graph = b.build().unwrap();
        let signals = SignalPlane::new();
        let events = EventPlane::new().0;
        let mut sent = Vec::new();
        for (addr, value) in writes {
            signals.write(
                &PortId::new(ModuleInstanceId::new(OSC_MODULE), *addr),
                Signal::Float(*value),
            );
            sent.extend(
                graph
                    .tick(&signals, 0.01, &SafetyLayer::default(), &events)
                    .iter()
                    .map(|c| c.level()),
            );
        }
        sent
    }

    #[test]
    fn reads_parameters_under_its_prefixes_only() {
        let sent = run(
            vec![],
            &[
                ("/avatar/parameters/Half/Mouth", 0.8),
                ("/avatar/parameters/Other", 0.6),
                ("/avatar/parameters/Half/Anus", 0.2),
            ],
        );
        assert_eq!(sent, vec![0.4, 0.1]);
    }

    #[test]
    fn input_filter_tags_narrow_it_down() {
        let sent = run(
            vec!["Mouth".into()],
            &[
                ("/avatar/parameters/Half/Mouth", 0.8),
                ("/avatar/parameters/Half/Anus", 0.2),
            ],
        );
        assert_eq!(sent, vec![0.4]);
    }
}
