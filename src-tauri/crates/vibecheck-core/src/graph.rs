//! The graph runner: processors wired by ports, run on a fixed tick.
//!
//! Build a [`Graph`] with a [`GraphBuilder`]:
//!
//! - [`add_processor`](GraphBuilder::add_processor) adds a node. Its id is a
//!   [`ModuleInstanceId`] and its ports are whatever the processor declares.
//! - [`connect`](GraphBuilder::connect) wires a source to a node's input. The
//!   source is either another node's output port, or any other port, which is
//!   read from the [`SignalPlane`] (e.g. an OSC parameter written by an input).
//! - a processor can also [watch](crate::module::Watch) a prefix of signal-plane
//!   ports instead of having each one wired, for inputs that aren't known up
//!   front (e.g. every SPS parameter an avatar sends).
//! - [`add_sink`](GraphBuilder::add_sink) sends a node's output to a target,
//!   such as a toy's motor port.
//!
//! On every [`tick`](Graph::tick):
//!
//! 1. nodes run in dependency order, each reading the latest value of its
//!    inputs. Watching nodes are first given the watched values written
//!    since their last tick. A node runs only once all of its inputs have a
//!    value, so nothing happens before the first input arrives (a node with
//!    only watches has no inputs to wait for, and runs every tick).
//! 2. a node's output port keeps its last value until the node writes a new
//!    one (`None` from a processor means "unchanged").
//! 3. each sink whose value changed since it last fired goes through the
//!    [`SafetyLayer`] and comes back as a [`ClampedCmd`], unless the safety
//!    layer refuses it (panic, not armed, rate limit). Unchanged sinks send
//!    nothing, so an idle graph doesn't flood its outputs, and a refused
//!    value isn't retried: the sink waits for its next change.
//!
//! Events a processor emits ([`EventOut`](crate::module::EventOut)) are published on the
//! [`EventPlane`] with the node's port as their source, e.g.
//! `proc:avg/above_0.7`.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::address::{ModuleInstanceId, PortId};
use crate::error::{Error, Result};
use crate::event::EventPlane;
use crate::module::{PortDirection, PortKind, Processor, Watch};
use crate::safety::{ClampedCmd, SafetyLayer};
use crate::signal::{Signal, SignalPlane, SignalSeq};

struct NodeSpec {
    id: ModuleInstanceId,
    processor: Box<dyn Processor>,
    inputs: Vec<Arc<str>>,
    outputs: Vec<Arc<str>>,
    wiring: Vec<Option<PortId>>,
    watches: Vec<Watch>,
}

struct SinkSpec {
    from: PortId,
    target: PortId,
    duration_ms: Option<u32>,
}

#[derive(Default)]
pub struct GraphBuilder {
    nodes: Vec<NodeSpec>,
    index: HashMap<ModuleInstanceId, usize>,
    sinks: Vec<SinkSpec>,
}

fn graph_err(why: impl Into<String>) -> Error {
    Error::Graph(why.into())
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a processor as node `id`.
    pub fn add_processor(
        &mut self,
        id: ModuleInstanceId,
        processor: Box<dyn Processor>,
    ) -> Result<()> {
        if self.index.contains_key(&id) {
            return Err(graph_err(format!("node {id} added twice")));
        }
        let signal_ports = |direction| -> Vec<Arc<str>> {
            processor
                .ports()
                .into_iter()
                .filter(|p| p.kind == PortKind::Signal && p.direction == direction)
                .map(|p| Arc::from(p.name))
                .collect()
        };
        let inputs = signal_ports(PortDirection::In);
        let outputs = signal_ports(PortDirection::Out);
        let watches = processor.watches();
        self.index.insert(id.clone(), self.nodes.len());
        self.nodes.push(NodeSpec {
            id,
            wiring: vec![None; inputs.len()],
            watches,
            processor,
            inputs,
            outputs,
        });
        Ok(())
    }

    fn output_index(&self, port: &PortId) -> Result<Option<(usize, usize)>> {
        let Some(&node) = self.index.get(&port.module) else {
            return Ok(None); // Not a node: read from the signal plane
        };
        let out = self.nodes[node]
            .outputs
            .iter()
            .position(|p| *p == port.port)
            .ok_or_else(|| graph_err(format!("{port} is not an output of node {}", port.module)))?;
        Ok(Some((node, out)))
    }

    /// Wires `from` into the input port `to`. Each input takes exactly one source.
    pub fn connect(&mut self, from: PortId, to: PortId) -> Result<()> {
        self.output_index(&from)?;
        let &node = self
            .index
            .get(&to.module)
            .ok_or_else(|| graph_err(format!("{to}: no node {}", to.module)))?;
        let spec = &mut self.nodes[node];
        let input = spec
            .inputs
            .iter()
            .position(|p| *p == to.port)
            .ok_or_else(|| graph_err(format!("{to} is not an input of node {}", to.module)))?;
        if spec.wiring[input].is_some() {
            return Err(graph_err(format!("{to} is already connected")));
        }
        spec.wiring[input] = Some(from);
        Ok(())
    }

    /// Sends the node output `from` to `target` (e.g. a motor port), with an
    /// optional duration for one-shot or linear outputs.
    pub fn add_sink(
        &mut self,
        from: PortId,
        target: PortId,
        duration_ms: Option<u32>,
    ) -> Result<()> {
        if self.output_index(&from)?.is_none() {
            return Err(graph_err(format!(
                "sink source {from} is not a node output"
            )));
        }
        self.sinks.push(SinkSpec {
            from,
            target,
            duration_ms,
        });
        Ok(())
    }

    /// Checks every input is connected and there are no cycles, and orders the
    /// nodes so each runs after the nodes it reads from.
    pub fn build(self) -> Result<Graph> {
        // Resolve wiring
        let mut deps: Vec<Vec<usize>> = Vec::with_capacity(self.nodes.len());
        let mut inputs: Vec<Vec<InputRef>> = Vec::with_capacity(self.nodes.len());
        for spec in &self.nodes {
            let mut node_deps = Vec::new();
            let mut node_inputs = Vec::new();
            for (port, source) in spec.inputs.iter().zip(&spec.wiring) {
                let source = source.as_ref().ok_or_else(|| {
                    graph_err(format!("input {}/{port} is not connected", spec.id))
                })?;
                match self.output_index(source)? {
                    Some((node, out)) => {
                        node_deps.push(node);
                        node_inputs.push(InputRef::Node { node, port: out });
                    }
                    None => node_inputs.push(InputRef::External(source.clone())),
                }
            }
            deps.push(node_deps);
            inputs.push(node_inputs);
        }

        // Topological order (Kahn), stable in insertion order
        let n = self.nodes.len();
        let mut remaining: Vec<usize> = deps.iter().map(Vec::len).collect();
        let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (node, node_deps) in deps.iter().enumerate() {
            for &d in node_deps {
                dependents[d].push(node);
            }
        }
        let mut ready: VecDeque<usize> = (0..n).filter(|&i| remaining[i] == 0).collect();
        let mut order = Vec::with_capacity(n);
        while let Some(node) = ready.pop_front() {
            order.push(node);
            for &dep in &dependents[node] {
                remaining[dep] -= 1;
                if remaining[dep] == 0 {
                    ready.push_back(dep);
                }
            }
        }
        if order.len() != n {
            return Err(graph_err("the graph has a cycle"));
        }

        // Renumber nodes into run order
        let mut position = vec![0; n];
        for (pos, &old) in order.iter().enumerate() {
            position[old] = pos;
        }
        let renumber = |r: &InputRef| match r {
            InputRef::Node { node, port } => InputRef::Node {
                node: position[*node],
                port: *port,
            },
            InputRef::External(p) => InputRef::External(p.clone()),
        };
        let mut specs: Vec<Option<NodeSpec>> = self.nodes.into_iter().map(Some).collect();
        let nodes = order
            .iter()
            .map(|&old| {
                let spec = specs[old].take().unwrap();
                Node {
                    outputs: vec![None; spec.outputs.len()],
                    watches: spec
                        .watches
                        .into_iter()
                        .map(|w| (w, SignalSeq::default()))
                        .collect(),
                    inputs: inputs[old].iter().map(renumber).collect(),
                    processor: spec.processor,
                    id: spec.id,
                    output_names: spec.outputs,
                }
            })
            .collect::<Vec<_>>();

        let index: HashMap<ModuleInstanceId, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.id.clone(), i))
            .collect();
        let sinks = self
            .sinks
            .into_iter()
            .map(|sink| {
                let node = index[&sink.from.module];
                let port = nodes[node]
                    .output_names
                    .iter()
                    .position(|p| *p == sink.from.port)
                    .unwrap();
                Sink {
                    node,
                    port,
                    target: sink.target,
                    duration_ms: sink.duration_ms,
                    last: None,
                }
            })
            .collect();

        Ok(Graph { nodes, sinks })
    }
}

#[derive(Clone)]
enum InputRef {
    Node { node: usize, port: usize },
    External(PortId),
}

struct Node {
    id: ModuleInstanceId,
    processor: Box<dyn Processor>,
    inputs: Vec<InputRef>,
    output_names: Vec<Arc<str>>,
    outputs: Vec<Option<Signal>>,
    /// Each watch, with how far the node has seen.
    watches: Vec<(Watch, SignalSeq)>,
}

struct Sink {
    node: usize,
    port: usize,
    target: PortId,
    duration_ms: Option<u32>,
    last: Option<f64>,
}

/// A built graph. See the [module docs](self).
pub struct Graph {
    nodes: Vec<Node>,
    sinks: Vec<Sink>,
}

impl Graph {
    /// Runs every node once, then returns a command for every sink whose
    /// value changed. `dt` is the time since the last tick, in seconds.
    /// Events processors emit are published on `events`.
    pub fn tick(
        &mut self,
        signals: &SignalPlane,
        dt: f32,
        safety: &SafetyLayer,
        events: &EventPlane,
    ) -> Vec<ClampedCmd> {
        let mut ins = Vec::new();
        let mut emitted = Vec::new();
        let mut changed = Vec::new();
        for i in 0..self.nodes.len() {
            // Nodes only read from nodes before them (run order)
            let (before, rest) = self.nodes.split_at_mut(i);
            let node = &mut rest[0];

            // Watched values first, even if the node isn't ready to run, so
            // none are missed
            if !node.watches.is_empty() {
                changed.clear();
                for (watch, seen) in &mut node.watches {
                    let (new, seq) = signals.changed_since(&watch.module, &watch.prefix, *seen);
                    *seen = seq;
                    changed.extend(new);
                }
                if !changed.is_empty() {
                    node.processor.watched(&changed);
                }
            }

            ins.clear();
            let mut ready = true;
            for input in &node.inputs {
                let value = match input {
                    InputRef::Node { node, port } => before[*node].outputs[*port],
                    InputRef::External(port) => signals.read(port),
                };
                match value {
                    Some(v) => ins.push(v),
                    None => {
                        ready = false;
                        break;
                    }
                }
            }
            if !ready {
                continue;
            }

            let mut out = vec![None; node.outputs.len()];
            node.processor.tick(&ins, dt, &mut out, &mut emitted);
            for event in emitted.drain(..) {
                events.publish(
                    PortId::new(node.id.clone(), event.port),
                    event.kind,
                    event.value,
                );
            }
            for (slot, new) in node.outputs.iter_mut().zip(out) {
                if new.is_some() {
                    *slot = new;
                }
            }
        }

        let mut cmds = Vec::new();
        for sink in &mut self.sinks {
            let Some(value) = self.nodes[sink.node].outputs[sink.port] else {
                continue;
            };
            let level = value.as_level();
            if sink.last == Some(level) {
                continue;
            }
            // Recorded even if the safety layer refuses it (panic, not
            // armed, rate limit), so nothing is replayed later: after a
            // resume or arming, the sink waits for its next change
            sink.last = Some(level);
            if let Some(cmd) = safety.clamp(sink.target.clone(), level, sink.duration_ms) {
                cmds.push(cmd);
            }
        }
        cmds
    }

    /// Starts over as if the graph were new, keeping its wiring and
    /// settings: every processor forgets its state, node outputs are
    /// cleared, and sinks forget the last value they sent, so the next value
    /// is sent even if it's the same as before (the device may have been
    /// stopped in between). Watching nodes are given every watched value
    /// still on the plane again on the next tick, like a new graph, so a
    /// reset that runs late doesn't lose values written just before it
    /// (clear the plane first for values that should be forgotten).
    pub fn reset(&mut self) {
        for node in &mut self.nodes {
            node.processor.reset();
            node.outputs.iter_mut().for_each(|out| *out = None);
            for (_, seen) in &mut node.watches {
                *seen = SignalSeq::default();
            }
        }
        for sink in &mut self.sinks {
            sink.last = None;
        }
    }

    /// Node ids in run order.
    pub fn node_ids(&self) -> impl Iterator<Item = &ModuleInstanceId> {
        self.nodes.iter().map(|n| &n.id)
    }

    /// Every node output's latest value, and the last value each sink sent,
    /// for views that show what's going on inside (diagnostics, a graph
    /// editor). `None` means nothing yet, e.g. a node that hasn't had input.
    pub fn snapshot(&self) -> GraphSnapshot {
        let outputs = self
            .nodes
            .iter()
            .flat_map(|node| {
                node.output_names
                    .iter()
                    .zip(&node.outputs)
                    .map(|(name, value)| (PortId::new(node.id.clone(), name), *value))
            })
            .collect();
        let sinks = self
            .sinks
            .iter()
            .map(|sink| {
                let node = &self.nodes[sink.node];
                SinkValue {
                    from: PortId::new(node.id.clone(), &node.output_names[sink.port]),
                    target: sink.target.clone(),
                    last: sink.last,
                }
            })
            .collect();
        GraphSnapshot { outputs, sinks }
    }
}

/// What [`Graph::snapshot`] returns.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphSnapshot {
    /// Each node output, in run order.
    pub outputs: Vec<(PortId, Option<Signal>)>,
    /// Each sink, in the order they were added.
    pub sinks: Vec<SinkValue>,
}

/// A sink in a [`GraphSnapshot`].
#[derive(Clone, Debug, PartialEq)]
pub struct SinkValue {
    /// The node output it sends.
    pub from: PortId,
    /// Where it sends it, e.g. a motor. Several sinks can share a target.
    pub target: PortId,
    /// The last level it sent, before the safety layer (so master intensity
    /// isn't applied yet).
    pub last: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, Value};
    use crate::module::{EventOut, PortSpec};

    /// Adds a constant to its input.
    struct Add(f64);

    impl Processor for Add {
        fn ports(&self) -> Vec<PortSpec> {
            vec![PortSpec::signal_in("in"), PortSpec::signal_out("out")]
        }
        fn tick(
            &mut self,
            ins: &[Signal],
            _: f32,
            out: &mut [Option<Signal>],
            _: &mut Vec<EventOut>,
        ) {
            out[0] = Some(Signal::Float(ins[0].as_level() + self.0));
        }
    }

    /// Only outputs when its input is above 0.5.
    struct Gate;

    impl Processor for Gate {
        fn ports(&self) -> Vec<PortSpec> {
            vec![PortSpec::signal_in("in"), PortSpec::signal_out("out")]
        }
        fn tick(
            &mut self,
            ins: &[Signal],
            _: f32,
            out: &mut [Option<Signal>],
            _: &mut Vec<EventOut>,
        ) {
            if ins[0].as_level() > 0.5 {
                out[0] = Some(ins[0]);
            }
        }
    }

    fn id(s: &str) -> ModuleInstanceId {
        ModuleInstanceId::new(s)
    }

    fn port(m: &str, p: &str) -> PortId {
        PortId::new(id(m), p)
    }

    fn tick(g: &mut Graph, signals: &SignalPlane) -> Vec<(String, f64)> {
        g.tick(signals, 0.01, &SafetyLayer::default(), &EventPlane::new().0)
            .iter()
            .map(|c| (c.target().to_string(), c.level()))
            .collect()
    }

    #[test]
    fn snapshot_shows_node_outputs_and_what_sinks_sent() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("add"), Box::new(Add(0.25))).unwrap();
        b.connect(port("osc", "Boop"), port("add", "in")).unwrap();
        b.add_sink(port("add", "out"), port("toy", "Vibrator_0"), None)
            .unwrap();
        let mut g = b.build().unwrap();

        // Nothing yet before any input
        assert_eq!(
            g.snapshot(),
            GraphSnapshot {
                outputs: vec![(port("add", "out"), None)],
                sinks: vec![SinkValue {
                    from: port("add", "out"),
                    target: port("toy", "Vibrator_0"),
                    last: None
                }],
            }
        );

        let signals = SignalPlane::new();
        signals.write(&port("osc", "Boop"), Signal::Float(0.25));
        tick(&mut g, &signals);
        assert_eq!(
            g.snapshot(),
            GraphSnapshot {
                outputs: vec![(port("add", "out"), Some(Signal::Float(0.5)))],
                sinks: vec![SinkValue {
                    from: port("add", "out"),
                    target: port("toy", "Vibrator_0"),
                    last: Some(0.5)
                }],
            }
        );
    }

    #[test]
    fn runs_in_dependency_order_regardless_of_insertion_order() {
        let mut b = GraphBuilder::new();
        // Added downstream-first on purpose
        b.add_processor(id("second"), Box::new(Add(0.25))).unwrap();
        b.add_processor(id("first"), Box::new(Add(0.25))).unwrap();
        b.connect(port("osc", "Boop"), port("first", "in")).unwrap();
        b.connect(port("first", "out"), port("second", "in"))
            .unwrap();
        b.add_sink(port("second", "out"), port("toy", "Vibrator_0"), None)
            .unwrap();
        let mut g = b.build().unwrap();
        assert_eq!(
            g.node_ids().map(|i| i.as_str()).collect::<Vec<_>>(),
            ["first", "second"]
        );

        let signals = SignalPlane::new();
        // No input yet: nothing runs, nothing is sent
        assert!(tick(&mut g, &signals).is_empty());

        signals.write(&port("osc", "Boop"), Signal::Float(0.1));
        assert_eq!(tick(&mut g, &signals), vec![("toy/Vibrator_0".into(), 0.6)]);
        // Unchanged: nothing sent
        assert!(tick(&mut g, &signals).is_empty());

        signals.write(&port("osc", "Boop"), Signal::Float(0.2));
        assert_eq!(tick(&mut g, &signals), vec![("toy/Vibrator_0".into(), 0.7)]);
    }

    #[test]
    fn outputs_hold_their_last_value() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("gate"), Box::new(Gate)).unwrap();
        b.add_processor(id("add"), Box::new(Add(0.0))).unwrap();
        b.connect(port("osc", "Boop"), port("gate", "in")).unwrap();
        b.connect(port("gate", "out"), port("add", "in")).unwrap();
        b.add_sink(port("add", "out"), port("toy", "m"), None)
            .unwrap();
        let mut g = b.build().unwrap();
        let signals = SignalPlane::new();

        // Gate holds back: downstream has no input yet
        signals.write(&port("osc", "Boop"), Signal::Float(0.2));
        assert!(tick(&mut g, &signals).is_empty());

        signals.write(&port("osc", "Boop"), Signal::Float(0.9));
        assert_eq!(tick(&mut g, &signals), vec![("toy/m".into(), 0.9)]);

        // Gate holds back again: its output keeps 0.9, so nothing changes
        signals.write(&port("osc", "Boop"), Signal::Float(0.3));
        assert!(tick(&mut g, &signals).is_empty());
    }

    #[test]
    fn sinks_carry_duration_and_pass_the_safety_layer() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("add"), Box::new(Add(5.0))).unwrap();
        b.connect(port("osc", "Boop"), port("add", "in")).unwrap();
        b.add_sink(port("add", "out"), port("toy", "Linear_0"), Some(250))
            .unwrap();
        let mut g = b.build().unwrap();
        let signals = SignalPlane::new();
        signals.write(&port("osc", "Boop"), Signal::Float(0.0));

        let cmds = g.tick(
            &signals,
            0.01,
            &SafetyLayer::default(),
            &EventPlane::new().0,
        );
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].level(), 1.0); // 5.0 clamped by the safety layer
        assert_eq!(cmds[0].duration_ms(), Some(250));
    }

    /// Emits an event when its input crosses 0.5.
    struct Threshold {
        above: bool,
    }

    impl Processor for Threshold {
        fn ports(&self) -> Vec<PortSpec> {
            vec![PortSpec::signal_in("in"), PortSpec::event_out("crossed")]
        }
        fn tick(
            &mut self,
            ins: &[Signal],
            _: f32,
            _: &mut [Option<Signal>],
            events: &mut Vec<EventOut>,
        ) {
            let above = ins[0].as_level() > 0.5;
            if above != self.above {
                self.above = above;
                events.push(EventOut::new(
                    "crossed",
                    EventKind::Threshold,
                    Value::Bool(above),
                ));
            }
        }
    }

    #[test]
    fn publishes_processor_events_from_their_node() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("proc:avg"), Box::new(Threshold { above: false }))
            .unwrap();
        b.connect(port("osc", "Boop"), port("proc:avg", "in"))
            .unwrap();
        let mut g = b.build().unwrap();
        let signals = SignalPlane::new();
        let (events, mut rx) = EventPlane::new();
        let safety = SafetyLayer::default();

        signals.write(&port("osc", "Boop"), Signal::Float(0.9));
        g.tick(&signals, 0.01, &safety, &events);
        let event = rx.try_recv().unwrap();
        assert_eq!(event.source, port("proc:avg", "crossed"));
        assert_eq!(event.kind, EventKind::Threshold);
        assert_eq!(event.value, Value::Bool(true));

        // No change, no event
        g.tick(&signals, 0.01, &safety, &events);
        assert!(rx.try_recv().is_none());
    }

    #[test]
    fn values_refused_during_a_panic_are_not_replayed_after_resume() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("add"), Box::new(Add(0.0))).unwrap();
        b.connect(port("osc", "Boop"), port("add", "in")).unwrap();
        b.add_sink(port("add", "out"), port("toy", "m"), None)
            .unwrap();
        let mut g = b.build().unwrap();
        let signals = SignalPlane::new();
        let safety = SafetyLayer::default();
        let events = EventPlane::new().0;
        let tick = |g: &mut Graph| g.tick(&signals, 0.01, &safety, &events).len();

        safety.panic();
        signals.write(&port("osc", "Boop"), Signal::Float(0.8));
        assert_eq!(tick(&mut g), 0, "refused while panicked");

        safety.resume();
        assert_eq!(tick(&mut g), 0, "0.8 isn't replayed after resume");
        signals.write(&port("osc", "Boop"), Signal::Float(0.5));
        assert_eq!(tick(&mut g), 1, "the next change goes through");
    }

    #[test]
    fn rejects_bad_wiring() {
        let add = || -> Box<dyn Processor> { Box::new(Add(0.0)) };

        let mut b = GraphBuilder::new();
        b.add_processor(id("a"), add()).unwrap();
        assert!(b.add_processor(id("a"), add()).is_err(), "duplicate node");
        assert!(
            b.connect(port("osc", "x"), port("a", "nope")).is_err(),
            "unknown input"
        );
        assert!(
            b.connect(port("osc", "x"), port("missing", "in")).is_err(),
            "unknown node"
        );
        assert!(
            b.connect(port("a", "nope"), port("a", "in")).is_err(),
            "unknown output"
        );
        b.connect(port("osc", "x"), port("a", "in")).unwrap();
        assert!(
            b.connect(port("osc", "y"), port("a", "in")).is_err(),
            "input wired twice"
        );
        assert!(
            b.add_sink(port("osc", "x"), port("toy", "m"), None)
                .is_err(),
            "sink from non-node"
        );

        let mut b = GraphBuilder::new();
        b.add_processor(id("a"), add()).unwrap();
        assert!(b.build().is_err(), "unconnected input");

        let mut b = GraphBuilder::new();
        b.add_processor(id("a"), add()).unwrap();
        b.add_processor(id("b"), add()).unwrap();
        b.connect(port("a", "out"), port("b", "in")).unwrap();
        b.connect(port("b", "out"), port("a", "in")).unwrap();
        assert!(b.build().is_err(), "cycle");
    }

    /// Sums every value under a prefix, keeping the latest per port.
    #[derive(Default)]
    struct SumWatched(HashMap<PortId, f64>);

    impl Processor for SumWatched {
        fn ports(&self) -> Vec<PortSpec> {
            vec![PortSpec::signal_out("out")]
        }
        fn watches(&self) -> Vec<Watch> {
            vec![Watch::new(ModuleInstanceId::new("osc"), "/sum/")]
        }
        fn watched(&mut self, changed: &[(PortId, Signal)]) {
            for (port, value) in changed {
                self.0.insert(port.clone(), value.as_level());
            }
        }
        fn tick(
            &mut self,
            _: &[Signal],
            _: f32,
            out: &mut [Option<Signal>],
            _: &mut Vec<EventOut>,
        ) {
            if !self.0.is_empty() {
                out[0] = Some(Signal::Float(self.0.values().sum()));
            }
        }
        fn reset(&mut self) {
            self.0.clear();
        }
    }

    #[test]
    fn watching_nodes_see_ports_that_were_never_wired() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("sum"), Box::<SumWatched>::default())
            .unwrap();
        b.add_processor(id("add"), Box::new(Add(0.0))).unwrap();
        b.connect(port("sum", "out"), port("add", "in")).unwrap();
        b.add_sink(port("add", "out"), port("toy", "Vibrator_0"), None)
            .unwrap();
        let mut g = b.build().unwrap();
        let signals = SignalPlane::new();
        let osc = |name: &str| PortId::new(ModuleInstanceId::new("osc"), name);

        // Nothing watched yet: nothing to send
        assert!(tick(&mut g, &signals).is_empty());

        signals.write(&osc("/sum/a"), Signal::Float(0.25));
        signals.write(&osc("/other"), Signal::Float(0.5));
        assert_eq!(
            tick(&mut g, &signals),
            vec![("toy/Vibrator_0".into(), 0.25)]
        );

        // A port nobody knew about when the graph was built
        signals.write(&osc("/sum/b"), Signal::Float(0.5));
        assert_eq!(
            tick(&mut g, &signals),
            vec![("toy/Vibrator_0".into(), 0.75)]
        );
        assert!(tick(&mut g, &signals).is_empty());
    }

    #[test]
    fn a_late_reset_replays_what_is_still_on_the_plane() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("sum"), Box::<SumWatched>::default())
            .unwrap();
        b.add_sink(port("sum", "out"), port("toy", "Vibrator_0"), None)
            .unwrap();
        let mut g = b.build().unwrap();
        let signals = SignalPlane::new();
        let osc = |name: &str| PortId::new(ModuleInstanceId::new("osc"), name);

        // The old avatar's value
        signals.write(&osc("/sum/old"), Signal::Float(0.5));
        tick(&mut g, &signals);

        // Avatar change: the input clears the plane at once, the new avatar's
        // value arrives and is seen, and only then does the reset run
        signals.clear_module(&ModuleInstanceId::new("osc"));
        signals.write(&osc("/sum/new"), Signal::Float(0.25));
        tick(&mut g, &signals);
        g.reset();

        // The new value is given again; the cleared old one isn't
        assert_eq!(
            tick(&mut g, &signals),
            vec![("toy/Vibrator_0".into(), 0.25)]
        );
    }

    /// Counts its ticks; the count is its output.
    #[derive(Default)]
    struct Counter(u32);

    impl Processor for Counter {
        fn ports(&self) -> Vec<PortSpec> {
            vec![PortSpec::signal_in("in"), PortSpec::signal_out("out")]
        }
        fn tick(
            &mut self,
            _: &[Signal],
            _: f32,
            out: &mut [Option<Signal>],
            _: &mut Vec<EventOut>,
        ) {
            self.0 += 1;
            out[0] = Some(Signal::Float(self.0 as f64 / 10.0));
        }
        fn reset(&mut self) {
            self.0 = 0;
        }
    }

    #[test]
    fn reset_forgets_state_and_resends_the_same_value() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("count"), Box::<Counter>::default())
            .unwrap();
        b.connect(port("osc", "Boop"), port("count", "in")).unwrap();
        b.add_sink(port("count", "out"), port("toy", "Vibrator_0"), None)
            .unwrap();
        let mut g = b.build().unwrap();
        let signals = SignalPlane::new();
        signals.write(&port("osc", "Boop"), Signal::Float(1.0));

        assert_eq!(tick(&mut g, &signals), vec![("toy/Vibrator_0".into(), 0.1)]);
        assert_eq!(tick(&mut g, &signals), vec![("toy/Vibrator_0".into(), 0.2)]);

        g.reset();
        // Counting starts over, and 0.1 is sent again although it was sent before
        assert_eq!(tick(&mut g, &signals), vec![("toy/Vibrator_0".into(), 0.1)]);

        // With no input, a reset graph sends nothing
        g.reset();
        signals.clear(&port("osc", "Boop"));
        assert!(tick(&mut g, &signals).is_empty());
    }
}
