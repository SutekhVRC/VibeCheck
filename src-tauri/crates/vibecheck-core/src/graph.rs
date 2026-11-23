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
//! - [`add_event_sink`](GraphBuilder::add_event_sink) turns the events a node
//!   emits on one of its event ports into commands for a target, for outputs
//!   that act on one-off actions rather than a level (see step 4 below).
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
//! 4. each event a node emits on a port wired to an event sink becomes a
//!    command for the sink's target, every time it's emitted (not only on
//!    change, so the same action can repeat). The level comes from the
//!    event's value: a `Float` is the level, `Bool(true)` and `None` are 1.0,
//!    and `Bool(false)` or `Text` send nothing. It goes through the safety
//!    layer like any command, and carries the event as its
//!    [`cause`](ClampedCmd::cause). Event sink commands come before signal
//!    sink commands, in the order the events were emitted.
//!
//! Events a processor emits ([`EventOut`](crate::module::EventOut)) are published on the
//! [`EventPlane`] with the node's port as their source, e.g.
//! `proc:avg/above_0.7`.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::address::{ModuleInstanceId, PortId};
use crate::error::{Error, Result};
use crate::event::{EventId, EventPlane, Value};
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
    /// Ports the node publishes events on.
    event_outputs: Vec<Arc<str>>,
    /// Its event inputs, each with the event ports wired to it.
    event_inputs: Vec<(Arc<str>, Vec<PortId>)>,
}

struct SinkSpec {
    from: PortId,
    target: PortId,
    duration_ms: Option<u32>,
}

/// Builds a [`Graph`]: add processors, wire them, add sinks, then
/// [`build`](GraphBuilder::build). See the module docs for an example.
#[derive(Default)]
pub struct GraphBuilder {
    nodes: Vec<NodeSpec>,
    index: HashMap<ModuleInstanceId, usize>,
    sinks: Vec<SinkSpec>,
    event_sinks: Vec<SinkSpec>,
}

fn graph_err(why: impl Into<String>) -> Error {
    Error::Graph(why.into())
}

impl GraphBuilder {
    /// An empty graph.
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
        let event_ports = |direction| -> Vec<Arc<str>> {
            processor
                .ports()
                .into_iter()
                .filter(|p| p.kind == PortKind::Event && p.direction == direction)
                .map(|p| Arc::from(p.name))
                .collect()
        };
        let event_outputs = event_ports(PortDirection::Out);
        let event_inputs = event_ports(PortDirection::In)
            .into_iter()
            .map(|name| (name, Vec::new()))
            .collect();
        let watches = processor.watches();
        self.index.insert(id.clone(), self.nodes.len());
        self.nodes.push(NodeSpec {
            id,
            wiring: vec![None; inputs.len()],
            watches,
            processor,
            inputs,
            outputs,
            event_outputs,
            event_inputs,
        });
        Ok(())
    }

    /// Wires the event port `from` (a node's event output) into the event
    /// input `to`. An event input can take any number of wires: events from
    /// all of them arrive, in the order they're emitted.
    pub fn connect_event(&mut self, from: PortId, to: PortId) -> Result<()> {
        let &source = self
            .index
            .get(&from.module)
            .ok_or_else(|| graph_err(format!("{from}: no node {}", from.module)))?;
        if !self.nodes[source].event_outputs.contains(&from.port) {
            return Err(graph_err(format!(
                "{from} is not an event output of node {}",
                from.module
            )));
        }
        let &node = self
            .index
            .get(&to.module)
            .ok_or_else(|| graph_err(format!("{to}: no node {}", to.module)))?;
        let input = self.nodes[node]
            .event_inputs
            .iter_mut()
            .find(|(name, _)| *name == to.port)
            .ok_or_else(|| {
                graph_err(format!("{to} is not an event input of node {}", to.module))
            })?;
        if !input.1.contains(&from) {
            input.1.push(from);
        }
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

    /// Sends each event the node emits on its event port `from` to `target`
    /// as a command, every time (see the module docs for how the event's
    /// value becomes a level), with an optional duration for timed actions.
    pub fn add_event_sink(
        &mut self,
        from: PortId,
        target: PortId,
        duration_ms: Option<u32>,
    ) -> Result<()> {
        let &node = self.index.get(&from.module).ok_or_else(|| {
            graph_err(format!("event sink source {from}: no node {}", from.module))
        })?;
        if !self.nodes[node].event_outputs.contains(&from.port) {
            return Err(graph_err(format!(
                "event sink source {from} is not an event port of node {}",
                from.module
            )));
        }
        self.event_sinks.push(SinkSpec {
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
            // Event wires order nodes too: an emitter runs before what it feeds
            for (_, sources) in &spec.event_inputs {
                for source in sources {
                    node_deps.push(self.index[&source.module]);
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
        let specs_events: Vec<Vec<(Arc<str>, Vec<PortId>)>> = self
            .nodes
            .iter()
            .map(|spec| spec.event_inputs.clone())
            .collect();
        let mut specs: Vec<Option<NodeSpec>> = self.nodes.into_iter().map(Some).collect();
        let mut nodes = order
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
                    event_inputs: Vec::new(),
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
        let event_sinks = self
            .event_sinks
            .into_iter()
            .map(|sink| EventSink {
                node: index[&sink.from.module],
                port: sink.from.port,
                target: sink.target,
                duration_ms: sink.duration_ms,
            })
            .collect();
        let mut event_wires = Vec::new();
        let mut event_inputs = vec![Vec::new(); nodes.len()];
        for (old, spec) in specs_events.into_iter().enumerate() {
            let to = position[old];
            for (input, (name, sources)) in spec.into_iter().enumerate() {
                for source in &sources {
                    event_wires.push(EventWire {
                        from: index[&source.module],
                        port: source.port.clone(),
                        to,
                        input,
                    });
                }
                event_inputs[to].push((name, sources));
            }
        }
        for (node, inputs) in nodes.iter_mut().zip(event_inputs) {
            node.event_inputs = inputs;
        }

        Ok(Graph {
            nodes,
            sinks,
            event_sinks,
            event_wires,
            event_counts: HashMap::new(),
        })
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
    /// Its event inputs and what's wired to each (for `shape`).
    event_inputs: Vec<(Arc<str>, Vec<PortId>)>,
}

/// An event port wired to a node's event input.
struct EventWire {
    /// The emitting node and its port.
    from: usize,
    port: Arc<str>,
    /// The receiving node and which of its event inputs.
    to: usize,
    input: usize,
}

struct Sink {
    node: usize,
    port: usize,
    target: PortId,
    duration_ms: Option<u32>,
    last: Option<f64>,
}

struct EventSink {
    node: usize,
    port: Arc<str>,
    target: PortId,
    duration_ms: Option<u32>,
}

/// The level an event sink sends for an event's value, if any.
fn event_level(value: &Value) -> Option<f64> {
    match value {
        Value::Float(level) => Some(*level),
        Value::Bool(true) | Value::None => Some(1.0),
        Value::Bool(false) | Value::Text(_) => None,
    }
}

/// A built graph. See the [module docs](self).
pub struct Graph {
    nodes: Vec<Node>,
    sinks: Vec<Sink>,
    event_sinks: Vec<EventSink>,
    event_wires: Vec<EventWire>,
    /// How many events each node's event port has emitted (by node index
    /// and port), for views ([`GraphSnapshot::events`]).
    event_counts: HashMap<(usize, Arc<str>), u64>,
}

impl Graph {
    /// Runs every node once, then returns a command for every event sent to
    /// an event sink and every sink whose value changed. `dt` is the time
    /// since the last tick, in seconds. Events processors emit are published
    /// on `events`.
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
        let mut cmds = Vec::new();
        // Events on their way to each node's event inputs this tick: (input,
        // value, the event's id). Emitters run first, so it's filled in time.
        let mut inbox: Vec<Vec<(usize, Value, EventId)>> = vec![Vec::new(); self.nodes.len()];
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

            // Events first too, even if the node can't run yet
            let mut cause = None;
            for (input, value, id) in inbox[i].drain(..) {
                node.processor.event(input, &value);
                cause = Some(id);
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
                let count = self
                    .event_counts
                    .entry((i, event.port.clone()))
                    .or_default();
                *count = count.wrapping_add(1);
                let level = event_level(&event.value);
                let value = event.value.clone();
                let source = PortId::new(node.id.clone(), event.port.as_ref());
                let id = match cause {
                    // A response to an event: part of its chain
                    Some(cause) => {
                        events.publish_caused_by_id(cause, source, event.kind, event.value)
                    }
                    None => events.publish(source, event.kind, event.value),
                };
                // On to the nodes wired to this port (they run later)
                for wire in &self.event_wires {
                    if wire.from == i && wire.port == event.port {
                        inbox[wire.to].push((wire.input, value.clone(), id));
                    }
                }
                let Some(level) = level else { continue };
                for sink in &self.event_sinks {
                    if sink.node != i || sink.port != event.port {
                        continue;
                    }
                    // Refused (panic, not armed, rate limit): dropped, never
                    // replayed
                    if let Some(cmd) = safety.clamp(sink.target.clone(), level, sink.duration_ms) {
                        cmds.push(cmd.with_cause(id));
                    }
                }
            }
            for (slot, new) in node.outputs.iter_mut().zip(out) {
                if new.is_some() {
                    *slot = new;
                }
            }
        }

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

    /// Every port the graph sends commands to, with the duration its
    /// commands carry. A port fed by several chains appears once per chain.
    pub fn sink_targets(&self) -> impl Iterator<Item = (&PortId, Option<u32>)> {
        self.sinks.iter().map(|s| (&s.target, s.duration_ms))
    }

    /// Node ids in run order.
    pub fn node_ids(&self) -> impl Iterator<Item = &ModuleInstanceId> {
        self.nodes.iter().map(|n| &n.id)
    }

    /// What the graph is made of: nodes, wiring and sinks. For views of
    /// the graph; pair it with [`snapshot`](Self::snapshot) for its values.
    pub fn shape(&self) -> GraphShape {
        let output = |node: usize, port: usize| {
            let node = &self.nodes[node];
            PortId::new(node.id.clone(), &node.output_names[port])
        };
        let nodes = self
            .nodes
            .iter()
            .map(|node| {
                let ports = node.processor.ports();
                let names = |direction: PortDirection, kind: PortKind| -> Vec<Arc<str>> {
                    ports
                        .iter()
                        .filter(|p| p.direction == direction && p.kind == kind)
                        .map(|p| Arc::from(p.name.as_str()))
                        .collect()
                };
                let input_names = names(PortDirection::In, PortKind::Signal);
                NodeShape {
                    id: node.id.clone(),
                    kind: node.processor.kind(),
                    inputs: input_names
                        .into_iter()
                        .zip(&node.inputs)
                        .map(|(name, input)| {
                            let from = match input {
                                InputRef::Node { node, port } => output(*node, *port),
                                InputRef::External(port) => port.clone(),
                            };
                            (name, from)
                        })
                        .collect(),
                    outputs: node.output_names.clone(),
                    event_outputs: names(PortDirection::Out, PortKind::Event),
                    event_inputs: node.event_inputs.clone(),
                    watches: node.watches.iter().map(|(w, _)| w.clone()).collect(),
                }
            })
            .collect();
        let sinks = self
            .sinks
            .iter()
            .map(|sink| SinkShape {
                from: output(sink.node, sink.port),
                target: sink.target.clone(),
                duration_ms: sink.duration_ms,
            })
            .collect();
        let event_sinks = self
            .event_sinks
            .iter()
            .map(|sink| SinkShape {
                from: PortId::new(self.nodes[sink.node].id.clone(), &sink.port),
                target: sink.target.clone(),
                duration_ms: sink.duration_ms,
            })
            .collect();
        GraphShape {
            nodes,
            sinks,
            event_sinks,
        }
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
        let mut events: Vec<(PortId, u64)> = self
            .event_counts
            .iter()
            .map(|((node, port), count)| {
                (
                    PortId::new(self.nodes[*node].id.clone(), port.as_ref()),
                    *count,
                )
            })
            .collect();
        events.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));
        GraphSnapshot {
            outputs,
            sinks,
            events,
        }
    }
}

/// What a graph is made of, for views of it ([`Graph::shape`]).
#[derive(Clone, Debug, PartialEq)]
pub struct GraphShape {
    /// Each node, in run order.
    pub nodes: Vec<NodeShape>,
    /// Each sink, in the order they were added.
    pub sinks: Vec<SinkShape>,
    /// Each event sink, in the order they were added.
    pub event_sinks: Vec<SinkShape>,
}

/// A node in a [`GraphShape`].
#[derive(Clone, Debug, PartialEq)]
pub struct NodeShape {
    /// The node's id in the graph.
    pub id: ModuleInstanceId,
    /// What kind of node it is ([`Processor::kind`]).
    pub kind: String,
    /// Each signal input and what it reads: another node's output, or a
    /// port on the signal plane (e.g. an avatar parameter).
    pub inputs: Vec<(Arc<str>, PortId)>,
    /// Its signal outputs.
    pub outputs: Vec<Arc<str>>,
    /// The ports it publishes events on.
    pub event_outputs: Vec<Arc<str>>,
    /// Its event inputs, each with the event ports wired to it.
    pub event_inputs: Vec<(Arc<str>, Vec<PortId>)>,
    /// Signal-plane ports it reads besides its inputs.
    pub watches: Vec<Watch>,
}

/// A sink or event sink in a [`GraphShape`].
#[derive(Clone, Debug, PartialEq)]
pub struct SinkShape {
    /// The node output (or event port) it sends.
    pub from: PortId,
    /// Where it sends it, e.g. a motor.
    pub target: PortId,
    /// The duration it sends with, for timed outputs.
    pub duration_ms: Option<u32>,
}

/// What [`Graph::snapshot`] returns.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphSnapshot {
    /// Each node output, in run order.
    pub outputs: Vec<(PortId, Option<Signal>)>,
    /// Each sink, in the order they were added.
    pub sinks: Vec<SinkValue>,
    /// How many events each node's event port has emitted since the graph
    /// was built (wrapping), for ports that have emitted any, sorted by
    /// port. A view that polls can tell an event passed when a count goes
    /// up, e.g. to flash the wires from that port.
    pub events: Vec<(PortId, u64)>,
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

    /// Emits its input as an event on `fire` every tick, as the value it's
    /// given: a float stays a float, a bool stays a bool.
    struct Fire;

    impl Processor for Fire {
        fn ports(&self) -> Vec<PortSpec> {
            vec![PortSpec::signal_in("in"), PortSpec::event_out("fire")]
        }
        fn tick(
            &mut self,
            ins: &[Signal],
            _: f32,
            _: &mut [Option<Signal>],
            events: &mut Vec<EventOut>,
        ) {
            let value = match ins[0] {
                Signal::Float(v) => Value::Float(v),
                Signal::Bool(b) => Value::Bool(b),
            };
            events.push(EventOut::new("fire", EventKind::Threshold, value));
        }
    }

    /// A graph: osc/in -> Fire, its events to dev/zap for 300 ms.
    fn fire_graph() -> Graph {
        let mut b = GraphBuilder::new();
        b.add_processor(id("fire"), Box::new(Fire)).unwrap();
        b.connect(port("osc", "in"), port("fire", "in")).unwrap();
        b.add_event_sink(port("fire", "fire"), port("dev", "zap"), Some(300))
            .unwrap();
        b.build().unwrap()
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

    /// Emits `fired` (value: its input) whenever its input is 0.5 or more.
    struct Emit;

    impl Processor for Emit {
        fn ports(&self) -> Vec<PortSpec> {
            vec![PortSpec::signal_in("in"), PortSpec::event_out("fired")]
        }

        fn tick(
            &mut self,
            ins: &[Signal],
            _: f32,
            _: &mut [Option<Signal>],
            ev: &mut Vec<EventOut>,
        ) {
            if ins[0].as_level() >= 0.5 {
                ev.push(EventOut::new(
                    "fired",
                    EventKind::Threshold,
                    Value::Float(ins[0].as_level()),
                ));
            }
        }
    }

    /// Counts events on `add` in quarters (levels are capped at 1), starts
    /// over on `reset`; outputs the count and emits `counted` when it grows.
    #[derive(Default)]
    struct Count {
        n: f64,
        added: bool,
    }

    impl Processor for Count {
        fn ports(&self) -> Vec<PortSpec> {
            vec![
                PortSpec::event_in("add"),
                PortSpec::event_in("reset"),
                PortSpec::signal_out("count"),
                PortSpec::event_out("counted"),
            ]
        }

        fn event(&mut self, input: usize, _: &Value) {
            if input == 0 {
                self.n += 0.25;
                self.added = true;
            } else {
                self.n = 0.0;
            }
        }

        fn tick(
            &mut self,
            _: &[Signal],
            _: f32,
            out: &mut [Option<Signal>],
            ev: &mut Vec<EventOut>,
        ) {
            out[0] = Some(Signal::Float(self.n));
            if std::mem::take(&mut self.added) {
                ev.push(EventOut::new(
                    "counted",
                    EventKind::Threshold,
                    Value::Float(self.n),
                ));
            }
        }
    }

    fn counting_graph() -> Graph {
        let mut b = GraphBuilder::new();
        // Added in the "wrong" order: the wiring decides who runs first
        b.add_processor(id("count"), Box::new(Count::default()))
            .unwrap();
        b.add_processor(id("a"), Box::new(Emit)).unwrap();
        b.add_processor(id("b"), Box::new(Emit)).unwrap();
        b.connect(port("osc", "A"), port("a", "in")).unwrap();
        b.connect(port("osc", "B"), port("b", "in")).unwrap();
        b.connect_event(port("a", "fired"), port("count", "add"))
            .unwrap();
        b.connect_event(port("b", "fired"), port("count", "add"))
            .unwrap();
        b.add_sink(port("count", "count"), port("toy", "Vibrator_0"), None)
            .unwrap();
        b.build().unwrap()
    }

    #[test]
    fn events_reach_the_inputs_wired_to_them_in_the_same_tick() {
        let mut g = counting_graph();
        let signals = SignalPlane::new();
        let (events, mut received) = EventPlane::new();
        let safety = SafetyLayer::default();
        signals.write(&port("osc", "A"), Signal::Float(0.0));
        signals.write(&port("osc", "B"), Signal::Float(0.0));
        let levels = |cmds: Vec<ClampedCmd>| cmds.iter().map(|c| c.level()).collect::<Vec<_>>();
        assert_eq!(levels(g.tick(&signals, 0.01, &safety, &events)), vec![0.0]);

        // Both inputs fire: both events reach the one input this tick
        signals.write(&port("osc", "A"), Signal::Float(1.0));
        signals.write(&port("osc", "B"), Signal::Float(0.8));
        assert_eq!(levels(g.tick(&signals, 0.01, &safety, &events)), vec![0.5]);

        // The counter's own event is part of the chain that caused it
        let mut fired = Vec::new();
        while let Some(e) = received.try_recv() {
            fired.push(e);
        }
        let counted = fired
            .iter()
            .find(|e| e.source == port("count", "counted"))
            .unwrap();
        assert_eq!(counted.depth, 1);
        assert!(events.chain_includes(counted.id, &id("b")));
    }

    #[test]
    fn event_wiring_is_checked_and_ordered() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("count"), Box::new(Count::default()))
            .unwrap();
        b.add_processor(id("a"), Box::new(Emit)).unwrap();
        assert!(b
            .connect_event(port("a", "nope"), port("count", "add"))
            .is_err());
        assert!(b
            .connect_event(port("a", "fired"), port("count", "count"))
            .is_err());
        assert!(b
            .connect_event(port("count", "counted"), port("count", "add"))
            .is_ok());
        assert!(b.build().is_err(), "a node feeding itself is a cycle");

        let shape = counting_graph().shape();
        let count = shape.nodes.iter().find(|n| n.id == id("count")).unwrap();
        assert_eq!(
            count.event_inputs,
            vec![
                (
                    Arc::from("add"),
                    vec![port("a", "fired"), port("b", "fired")]
                ),
                (Arc::from("reset"), vec![]),
            ]
        );
        assert_eq!(
            *shape.nodes.last().map(|n| &n.id).unwrap(),
            id("count"),
            "runs last"
        );
    }

    #[test]
    fn shape_shows_nodes_wiring_and_sinks() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("add"), Box::new(Add(0.25))).unwrap();
        b.add_processor(id("more"), Box::new(Add(0.5))).unwrap();
        b.connect(port("osc", "Boop"), port("add", "in")).unwrap();
        b.connect(port("add", "out"), port("more", "in")).unwrap();
        b.add_sink(port("more", "out"), port("toy", "Vibrator_0"), Some(100))
            .unwrap();
        let shape = b.build().unwrap().shape();

        assert_eq!(shape.nodes.len(), 2);
        let add = &shape.nodes[0];
        assert_eq!(add.id, id("add"));
        assert_eq!(add.kind, "Add", "the type's name by default");
        assert_eq!(
            add.inputs,
            vec![(Arc::from("in"), port("osc", "Boop"))],
            "an outside input"
        );
        assert_eq!(add.outputs, vec![Arc::from("out")]);
        assert_eq!(
            shape.nodes[1].inputs,
            vec![(Arc::from("in"), port("add", "out"))],
            "wired from the first node"
        );
        assert_eq!(
            shape.sinks,
            vec![SinkShape {
                from: port("more", "out"),
                target: port("toy", "Vibrator_0"),
                duration_ms: Some(100),
            }]
        );
        assert!(shape.event_sinks.is_empty());
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
                events: vec![],
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
                events: vec![],
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
        // Counted for views
        assert_eq!(g.snapshot().events, vec![(port("proc:avg", "crossed"), 1)]);

        // No change, no event
        g.tick(&signals, 0.01, &safety, &events);
        assert!(rx.try_recv().is_none());
        assert_eq!(g.snapshot().events, vec![(port("proc:avg", "crossed"), 1)]);
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

    #[test]
    fn event_sinks_send_every_event_with_its_cause() {
        let mut g = fire_graph();
        let signals = SignalPlane::new();
        let (events, mut received) = EventPlane::new();
        let safety = SafetyLayer::default();
        let mut tick = |value| {
            signals.write(&port("osc", "in"), value);
            g.tick(&signals, 0.01, &safety, &events)
        };

        // The same level twice: two commands, unlike a signal sink
        for _ in 0..2 {
            let cmds = tick(Signal::Float(0.4));
            assert_eq!(cmds.len(), 1);
            assert_eq!(cmds[0].target(), &port("dev", "zap"));
            assert_eq!(cmds[0].level(), 0.4);
            assert_eq!(cmds[0].duration_ms(), Some(300));
            // Caused by the event the processor emitted
            let event = received.try_recv().unwrap();
            assert_eq!(event.source, port("fire", "fire"));
            assert_eq!(cmds[0].cause(), Some(event.id));
        }

        // true is full level, false sends nothing (but is still published)
        assert_eq!(tick(Signal::Bool(true))[0].level(), 1.0);
        assert!(tick(Signal::Bool(false)).is_empty());
        assert_eq!(
            received.try_recv().map(|e| e.value),
            Some(Value::Bool(true))
        );
        assert_eq!(
            received.try_recv().map(|e| e.value),
            Some(Value::Bool(false))
        );
    }

    #[test]
    fn event_sinks_go_through_the_safety_layer_and_never_replay() {
        let mut g = fire_graph();
        let signals = SignalPlane::new();
        let events = EventPlane::new().0;
        let safety = SafetyLayer::default();
        signals.write(&port("osc", "in"), Signal::Float(0.8));

        safety.panic();
        assert!(g.tick(&signals, 0.01, &safety, &events).is_empty());
        safety.resume();
        // Only the new event is sent, nothing queued from the panic
        assert_eq!(g.tick(&signals, 0.01, &safety, &events).len(), 1);

        // A device that needs arming gets nothing until armed
        safety.set_policy(
            id("dev"),
            crate::safety::TargetPolicy {
                requires_arming: true,
                ..Default::default()
            },
        );
        assert!(g.tick(&signals, 0.01, &safety, &events).is_empty());
        assert!(safety.arm(id("dev")));
        assert_eq!(g.tick(&signals, 0.01, &safety, &events).len(), 1);
    }

    #[test]
    fn event_sinks_need_a_node_event_port() {
        let mut b = GraphBuilder::new();
        b.add_processor(id("fire"), Box::new(Fire)).unwrap();
        b.add_processor(id("add"), Box::new(Add(0.1))).unwrap();
        // Unknown node, a signal port, a port that doesn't exist
        assert!(b
            .add_event_sink(port("nope", "fire"), port("dev", "zap"), None)
            .is_err());
        assert!(b
            .add_event_sink(port("add", "out"), port("dev", "zap"), None)
            .is_err());
        assert!(b
            .add_event_sink(port("fire", "missing"), port("dev", "zap"), None)
            .is_err());
        assert!(b
            .add_event_sink(port("fire", "fire"), port("dev", "zap"), None)
            .is_ok());
    }
}
