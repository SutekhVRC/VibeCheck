//! The graph runner: processors wired by ports, run on a fixed tick.
//!
//! Build a [`Graph`] with a [`GraphBuilder`]:
//!
//! - [`add_processor`](GraphBuilder::add_processor) adds a node. Its id is a
//!   [`ModuleInstanceId`] and its ports are whatever the processor declares.
//! - [`connect`](GraphBuilder::connect) wires a source to a node's input. The
//!   source is either another node's output port, or any other port, which is
//!   read from the [`SignalPlane`] (e.g. an OSC parameter written by an input).
//! - [`add_sink`](GraphBuilder::add_sink) sends a node's output to a target,
//!   such as a toy's motor port.
//!
//! On every [`tick`](Graph::tick):
//!
//! 1. nodes run in dependency order, each reading the latest value of its
//!    inputs. A node runs only once all of its inputs have a value, so
//!    nothing happens before the first input arrives.
//! 2. a node's output port keeps its last value until the node writes a new
//!    one (`None` from a processor means "unchanged").
//! 3. each sink whose value changed since it last fired goes through the
//!    [`SafetyLayer`] and comes back as a [`ClampedCmd`]. Unchanged sinks send
//!    nothing, so an idle graph doesn't flood its outputs.
//!
//! Events a processor emits ([`EventOut`](crate::module::EventOut)) are published on the
//! [`EventPlane`] with the node's port as their source, e.g.
//! `proc:avg/above_0.7`.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::address::{ModuleInstanceId, PortId};
use crate::error::{Error, Result};
use crate::event::EventPlane;
use crate::module::{PortDirection, PortKind, Processor};
use crate::safety::{ClampedCmd, SafetyLayer};
use crate::signal::{Signal, SignalPlane};

struct NodeSpec {
    id: ModuleInstanceId,
    processor: Box<dyn Processor>,
    inputs: Vec<Arc<str>>,
    outputs: Vec<Arc<str>>,
    wiring: Vec<Option<PortId>>,
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
        self.index.insert(id.clone(), self.nodes.len());
        self.nodes.push(NodeSpec {
            id,
            wiring: vec![None; inputs.len()],
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
        for i in 0..self.nodes.len() {
            // Nodes only read from nodes before them (run order)
            let (before, rest) = self.nodes.split_at_mut(i);
            let node = &mut rest[0];

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
            sink.last = Some(level);
            cmds.push(safety.clamp(sink.target.clone(), level, sink.duration_ms));
        }
        cmds
    }

    /// Node ids in run order.
    pub fn node_ids(&self) -> impl Iterator<Item = &ModuleInstanceId> {
        self.nodes.iter().map(|n| &n.id)
    }
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
}
