/*
 * Ticks every toy's graph, and any graphs an app added (see extensions.rs),
 * at a fixed rate.
 *
 * Commands are routed by their target's module: each output registers its
 * module with its CommandSlots (every toy does; apps register theirs), so a
 * graph can drive any of them, e.g. a toy's motor and an avatar parameter
 * at once. A command for a module nothing registered goes to the graph's
 * own slots.
 *
 * One task per listening session. Each tick it runs every graph against the
 * session's signal plane and writes the resulting commands into that graph's
 * CommandSlots, which its output sends at its own rate (a toy's emitter at
 * the toy's update rate).
 */

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use log::info;
use parking_lot::Mutex;
use tokio::{
    sync::{mpsc::UnboundedReceiver, oneshot, watch},
    time::{interval, Instant, MissedTickBehavior},
};
use vibecheck_core::{
    graph::Graph, ClampedCmd, CommandSlots, Diagnostics, EventPlane, GraphShape, GraphSnapshot,
    ModuleInstanceId, PortId, SafetyLayer, SignalPlane, Watch,
};

/// How often graphs run.
pub const GRAPH_TICK_HZ: u64 = 100;

/// Longest gap a single tick accounts for, so a stall (e.g. the machine
/// sleeping) doesn't make time-based processors jump.
const MAX_DT: f32 = 0.1;

/// While diagnostics are on, graph values are copied out every this many
/// ticks (10 times a second at 100 Hz), which is as often as they're shown.
const SNAPSHOT_EVERY: u32 = 10;

/// The latest values inside every toy's graph, for the diagnostics view.
/// Only filled while diagnostics are on.
#[derive(Clone, Default)]
pub struct GraphValues {
    diagnostics: Diagnostics,
    latest: Arc<Mutex<HashMap<u32, GraphSnapshot>>>,
}

impl GraphValues {
    pub fn new(diagnostics: Diagnostics) -> Self {
        Self {
            diagnostics,
            latest: Arc::default(),
        }
    }

    /// Each toy's latest snapshot, by toy id.
    pub fn latest(&self) -> HashMap<u32, GraphSnapshot> {
        self.latest.lock().clone()
    }

    fn update(&self, graphs: &HashMap<GraphKey, ToyGraph>) {
        let mut latest = self.latest.lock();
        latest.clear();
        if self.diagnostics.enabled() {
            latest.extend(graphs.iter().filter_map(|(key, toy)| match key {
                GraphKey::Toy(id) => Some((*id, toy.graph.snapshot())),
                GraphKey::App(_) => None,
            }));
        }
    }
}

/// Which graph: a toy's, or one an app added to the session (by the name
/// it chose; see extensions.rs).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GraphKey {
    Toy(u32),
    App(String),
}

pub enum GraphControl {
    /// Run `graph` from now on, replacing any previous graph with this key.
    Set {
        key: GraphKey,
        graph: Graph,
        slots: CommandSlots,
    },
    Remove(GraphKey),
    /// Start every graph over (see `Graph::reset`) and send 0 to every
    /// output, e.g. on avatar change. The 0 also replaces any command still
    /// waiting to be sent, and lets each output see its toy stop.
    ResetAll,
    /// Reply with every graph running now: its shape and current values.
    Describe(oneshot::Sender<Vec<GraphView>>),
    /// Commands for `module`'s ports go to `slots`, from any graph.
    SetOutput {
        module: ModuleInstanceId,
        slots: CommandSlots,
    },
    /// `module` no longer takes commands from other graphs.
    RemoveOutput(ModuleInstanceId),
    /// Keep `watched` current with every watch of every running graph (what
    /// the graphs read besides their inputs, e.g. SPS parameters), from now
    /// on and after every change.
    ReportWatches(watch::Sender<Vec<Watch>>),
}

/// Every watch of every graph in `graphs`.
fn all_watches(graphs: &HashMap<GraphKey, ToyGraph>) -> Vec<Watch> {
    graphs
        .values()
        .flat_map(|g| g.graph.shape().nodes)
        .flat_map(|node| node.watches)
        .collect()
}

/// Where each registered output's commands go.
type Outputs = HashMap<ModuleInstanceId, CommandSlots>;

/// Writes `cmd` to the output that owns its target, or to `fallback`.
fn route(cmd: ClampedCmd, outputs: &Outputs, fallback: &CommandSlots) {
    outputs
        .get(&cmd.target().module)
        .unwrap_or(fallback)
        .write(cmd);
}

/// A running graph, for views of it.
#[derive(Clone, Debug)]
pub struct GraphView {
    /// Which graph.
    pub key: GraphKey,
    /// Its nodes, wiring and sinks.
    pub shape: GraphShape,
    /// Its current values.
    pub values: GraphSnapshot,
}

struct ToyGraph {
    graph: Graph,
    slots: CommandSlots,
}

/// Sends 0 to every output `toy` drives.
fn stop_outputs(toy: &ToyGraph, safety: &SafetyLayer, outputs: &Outputs) {
    let mut stopped = HashSet::new();
    for (target, duration_ms) in toy.graph.sink_targets() {
        if !stopped.insert(target) {
            continue;
        }
        if let Some(cmd) = safety.clamp(target.clone(), 0.0, duration_ms) {
            route(cmd, outputs, &toy.slots);
        }
    }
}

/// Sends 0 to every output `old` drove that `new` doesn't, e.g. a feature
/// that was just disabled. Nothing drives those outputs any more, so without
/// this they'd keep their last level.
fn stop_dropped_outputs(old: &ToyGraph, new: &ToyGraph, safety: &SafetyLayer, outputs: &Outputs) {
    let kept: HashSet<&PortId> = new.graph.sink_targets().map(|(t, _)| t).collect();
    let mut stopped = HashSet::new();
    for (target, duration_ms) in old.graph.sink_targets() {
        if kept.contains(target) || !stopped.insert(target) {
            continue;
        }
        if let Some(cmd) = safety.clamp(target.clone(), 0.0, duration_ms) {
            route(cmd, outputs, &new.slots);
        }
    }
}

/// Events processors emit are published on `events`. While diagnostics are
/// on, graph values are copied into `values`.
pub async fn graph_runner(
    mut control: UnboundedReceiver<GraphControl>,
    signals: SignalPlane,
    events: EventPlane,
    safety: SafetyLayer,
    values: GraphValues,
) {
    let mut toys: HashMap<GraphKey, ToyGraph> = HashMap::new();
    let mut outputs: Outputs = HashMap::new();
    // Told every graph's watches after each change (ReportWatches)
    let mut watched: Option<watch::Sender<Vec<Watch>>> = None;

    let mut ticker = interval(Duration::from_nanos(1_000_000_000 / GRAPH_TICK_HZ));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut last_tick = Instant::now();
    let mut ticks: u32 = 0;

    loop {
        tokio::select! {
            // Apply graph changes before the next tick
            biased;
            change = control.recv() => match change {
                Some(GraphControl::Set { key, graph, slots }) => {
                    let old = toys.insert(key.clone(), ToyGraph { graph, slots });
                    if let Some(old) = old {
                        stop_dropped_outputs(&old, &toys[&key], &safety, &outputs);
                    }
                    if let Some(w) = &watched {
                        w.send_replace(all_watches(&toys));
                    }
                }
                Some(GraphControl::Remove(key)) => {
                    toys.remove(&key);
                    if let Some(w) = &watched {
                        w.send_replace(all_watches(&toys));
                    }
                }
                Some(GraphControl::ReportWatches(w)) => {
                    w.send_replace(all_watches(&toys));
                    watched = Some(w);
                }
                Some(GraphControl::Describe(reply)) => {
                    let views = toys
                        .iter()
                        .map(|(key, toy)| GraphView {
                            key: key.clone(),
                            shape: toy.graph.shape(),
                            values: toy.graph.snapshot(),
                        })
                        .collect();
                    let _ = reply.send(views);
                }
                Some(GraphControl::SetOutput { module, slots }) => {
                    outputs.insert(module, slots);
                }
                Some(GraphControl::RemoveOutput(module)) => {
                    outputs.remove(&module);
                }
                Some(GraphControl::ResetAll) => {
                    for toy in toys.values_mut() {
                        toy.graph.reset();
                        stop_outputs(toy, &safety, &outputs);
                    }
                }
                None => break,
            },
            _ = ticker.tick() => {
                let now = Instant::now();
                let dt = (now - last_tick).as_secs_f32().min(MAX_DT);
                last_tick = now;
                for toy in toys.values_mut() {
                    for cmd in toy.graph.tick(&signals, dt, &safety, &events) {
                        route(cmd, &outputs, &toy.slots);
                    }
                }
                ticks = ticks.wrapping_add(1);
                if ticks % SNAPSHOT_EVERY == 0 {
                    values.update(&toys);
                }
            }
        }
    }
    info!("Graph runner stopped");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toy_handling::{
        runtime::toy_graph::{build_toy_graph, OSC_MODULE},
        toyops::{ProcessingMode, ToyParameter, VCFeatureType, VCToyFeature, VCToyFeatures},
    };
    use tokio::{sync::mpsc::unbounded_channel, time::sleep};
    use vibecheck_core::{ModuleInstanceId, Signal};

    fn graph() -> Graph {
        graph_with(true)
    }

    fn graph_with(enabled: bool) -> Graph {
        let mut feature = VCToyFeature::new(
            vec![ToyParameter {
                parameter: "/avatar/parameters/Boop".into(),
                processing_mode: ProcessingMode::Raw,
            }],
            0,
            VCFeatureType::Vibrator,
        );
        feature.feature_enabled = enabled;
        build_toy_graph(
            &ModuleInstanceId::new("toy:test:0"),
            &VCToyFeatures {
                features: vec![feature],
            },
        )
        .unwrap()
    }

    #[tokio::test]
    async fn watches_are_reported_as_graphs_come_and_go() {
        use crate::toy_handling::input_processor::penetration_systems::PenetrationSystemType;
        let (control, control_rx) = unbounded_channel();
        tokio::spawn(graph_runner(
            control_rx,
            SignalPlane::new(),
            EventPlane::new().0,
            SafetyLayer::default(),
            GraphValues::default(),
        ));
        let (watched_tx, watched) = watch::channel(Vec::new());
        control
            .send(GraphControl::ReportWatches(watched_tx))
            .unwrap();

        // A toy feature with SPS watches SPS parameters
        let mut feature = VCToyFeature::new(Vec::new(), 0, VCFeatureType::Vibrator);
        feature.penetration_system.pen_system_type = PenetrationSystemType::Sps;
        let sps = build_toy_graph(
            &ModuleInstanceId::new("toy:test:0"),
            &VCToyFeatures {
                features: vec![feature],
            },
        )
        .unwrap();
        control
            .send(GraphControl::Set {
                key: GraphKey::Toy(0),
                graph: sps,
                slots: CommandSlots::new(),
            })
            .unwrap();
        sleep(Duration::from_millis(50)).await;
        assert!(crate::osc::sps_switch::wants_sps(&watched.borrow()));

        control
            .send(GraphControl::Remove(GraphKey::Toy(0)))
            .unwrap();
        sleep(Duration::from_millis(50)).await;
        assert!(watched.borrow().is_empty());
    }

    #[tokio::test]
    async fn commands_go_to_the_output_that_owns_their_target() {
        let signals = SignalPlane::new();
        let (control, control_rx) = unbounded_channel();
        tokio::spawn(graph_runner(
            control_rx,
            signals.clone(),
            EventPlane::new().0,
            SafetyLayer::default(),
            GraphValues::default(),
        ));
        // The test graph drives toy:test:0 (see `graph`)
        let toy = CommandSlots::new();
        let own = CommandSlots::new();
        control
            .send(GraphControl::SetOutput {
                module: ModuleInstanceId::new("toy:test:0"),
                slots: toy.clone(),
            })
            .unwrap();
        control
            .send(GraphControl::Set {
                key: GraphKey::App("app".into()),
                graph: graph(),
                slots: own.clone(),
            })
            .unwrap();
        let boop = PortId::new(ModuleInstanceId::new(OSC_MODULE), "/avatar/parameters/Boop");
        signals.write(&boop, Signal::Float(0.5));
        sleep(Duration::from_millis(50)).await;
        assert_eq!(toy.take_changed().len(), 1, "routed to the toy");
        assert!(own.take_changed().is_empty());

        // Not registered any more: back to the graph's own slots
        control
            .send(GraphControl::RemoveOutput(ModuleInstanceId::new(
                "toy:test:0",
            )))
            .unwrap();
        sleep(Duration::from_millis(20)).await;
        signals.write(&boop, Signal::Float(0.9));
        sleep(Duration::from_millis(50)).await;
        assert!(toy.take_changed().is_empty());
        assert_eq!(own.take_changed().len(), 1);
    }

    #[tokio::test]
    async fn describes_every_graph_with_its_values() {
        let signals = SignalPlane::new();
        let (control, control_rx) = unbounded_channel();
        tokio::spawn(graph_runner(
            control_rx,
            signals.clone(),
            EventPlane::new().0,
            SafetyLayer::default(),
            GraphValues::default(),
        ));
        control
            .send(GraphControl::Set {
                key: GraphKey::App("app".into()),
                graph: graph(),
                slots: CommandSlots::new(),
            })
            .unwrap();
        signals.write(
            &PortId::new(ModuleInstanceId::new(OSC_MODULE), "/avatar/parameters/Boop"),
            Signal::Float(0.5),
        );
        sleep(Duration::from_millis(50)).await;

        let (reply, answer) = oneshot::channel();
        control.send(GraphControl::Describe(reply)).unwrap();
        let views = answer.await.unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].key, GraphKey::App("app".into()));
        assert!(!views[0].shape.nodes.is_empty());
        assert_eq!(views[0].values.sinks[0].last, Some(0.5));
    }

    #[tokio::test]
    async fn ticks_graphs_into_their_slots_until_removed() {
        let signals = SignalPlane::new();
        let (control, control_rx) = unbounded_channel();
        let runner = tokio::spawn(graph_runner(
            control_rx,
            signals.clone(),
            EventPlane::new().0,
            SafetyLayer::default(),
            GraphValues::default(),
        ));

        let slots = CommandSlots::new();
        control
            .send(GraphControl::Set {
                key: GraphKey::Toy(1),
                graph: graph(),
                slots: slots.clone(),
            })
            .unwrap();

        let boop = PortId::new(ModuleInstanceId::new(OSC_MODULE), "/avatar/parameters/Boop");
        signals.write(&boop, Signal::Float(0.5));
        sleep(Duration::from_millis(50)).await;
        let sent = slots.take_changed();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].level(), 0.5);

        control
            .send(GraphControl::Remove(GraphKey::Toy(1)))
            .unwrap();
        sleep(Duration::from_millis(20)).await;
        signals.write(&boop, Signal::Float(0.9));
        sleep(Duration::from_millis(50)).await;
        assert!(slots.take_changed().is_empty());

        // Dropping the control channel stops the runner
        drop(control);
        tokio::time::timeout(Duration::from_secs(1), runner)
            .await
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn disabling_a_feature_stops_its_output() {
        let signals = SignalPlane::new();
        let (control, control_rx) = unbounded_channel();
        tokio::spawn(graph_runner(
            control_rx,
            signals.clone(),
            EventPlane::new().0,
            SafetyLayer::default(),
            GraphValues::default(),
        ));
        let slots = CommandSlots::new();
        let set = |graph| GraphControl::Set {
            key: GraphKey::Toy(1),
            graph,
            slots: slots.clone(),
        };

        control.send(set(graph_with(true))).unwrap();
        let boop = PortId::new(ModuleInstanceId::new(OSC_MODULE), "/avatar/parameters/Boop");
        signals.write(&boop, Signal::Float(0.5));
        sleep(Duration::from_millis(50)).await;
        assert_eq!(slots.take_changed()[0].level(), 0.5);

        // The rebuilt graph no longer has the motor: it gets one 0, then nothing
        control.send(set(graph_with(false))).unwrap();
        sleep(Duration::from_millis(50)).await;
        let sent = slots.take_changed();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].level(), 0.0);
        signals.write(&boop, Signal::Float(0.9));
        sleep(Duration::from_millis(50)).await;
        assert!(slots.take_changed().is_empty());
    }

    #[tokio::test]
    async fn reset_all_sends_zero_to_every_output() {
        let signals = SignalPlane::new();
        let (control, control_rx) = unbounded_channel();
        tokio::spawn(graph_runner(
            control_rx,
            signals.clone(),
            EventPlane::new().0,
            SafetyLayer::default(),
            GraphValues::default(),
        ));
        let slots = CommandSlots::new();
        control
            .send(GraphControl::Set {
                key: GraphKey::Toy(1),
                graph: graph_with(true),
                slots: slots.clone(),
            })
            .unwrap();
        let boop = PortId::new(ModuleInstanceId::new(OSC_MODULE), "/avatar/parameters/Boop");
        signals.write(&boop, Signal::Float(0.5));
        sleep(Duration::from_millis(50)).await;
        assert_eq!(slots.take_changed()[0].level(), 0.5);

        // Like an avatar change: the values are cleared, then the graphs reset
        signals.clear(&boop);
        control.send(GraphControl::ResetAll).unwrap();
        sleep(Duration::from_millis(50)).await;
        let sent = slots.take_changed();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].level(), 0.0);
    }

    #[tokio::test]
    async fn copies_graph_values_only_while_diagnostics_are_on() {
        let signals = SignalPlane::new();
        let diagnostics = Diagnostics::default();
        let values = GraphValues::new(diagnostics.clone());
        let (control, control_rx) = unbounded_channel();
        tokio::spawn(graph_runner(
            control_rx,
            signals.clone(),
            EventPlane::new().0,
            SafetyLayer::default(),
            values.clone(),
        ));
        control
            .send(GraphControl::Set {
                key: GraphKey::Toy(1),
                graph: graph(),
                slots: CommandSlots::new(),
            })
            .unwrap();
        let boop = PortId::new(ModuleInstanceId::new(OSC_MODULE), "/avatar/parameters/Boop");
        signals.write(&boop, Signal::Float(0.5));

        sleep(Duration::from_millis(250)).await;
        assert!(values.latest().is_empty());

        diagnostics.set_enabled(true);
        sleep(Duration::from_millis(250)).await;
        let latest = values.latest();
        let sinks = &latest[&1].sinks;
        assert_eq!(sinks.len(), 1);
        assert_eq!(sinks[0].last, Some(0.5));
    }

    #[tokio::test]
    async fn applies_the_master_intensity_even_while_running() {
        let signals = SignalPlane::new();
        let safety = SafetyLayer::default();
        let (control, control_rx) = unbounded_channel();
        let runner = tokio::spawn(graph_runner(
            control_rx,
            signals.clone(),
            EventPlane::new().0,
            safety.clone(),
            GraphValues::default(),
        ));
        let slots = CommandSlots::new();
        control
            .send(GraphControl::Set {
                key: GraphKey::Toy(1),
                graph: graph(),
                slots: slots.clone(),
            })
            .unwrap();

        let boop = PortId::new(ModuleInstanceId::new(OSC_MODULE), "/avatar/parameters/Boop");
        safety.set_intensity(0.5);
        signals.write(&boop, Signal::Float(0.8));
        sleep(Duration::from_millis(50)).await;
        assert!((slots.take_changed()[0].level() - 0.4).abs() < 1e-6);

        // Changing the intensity from Settings affects the running session
        safety.set_intensity(1.0);
        signals.write(&boop, Signal::Float(0.6));
        sleep(Duration::from_millis(50)).await;
        assert!((slots.take_changed()[0].level() - 0.6).abs() < 1e-6);

        drop(control);
        runner.await.unwrap();
    }

    #[tokio::test]
    async fn sends_nothing_while_panicked_and_waits_for_new_input_after() {
        let signals = SignalPlane::new();
        let safety = SafetyLayer::default();
        let (control, control_rx) = unbounded_channel();
        let runner = tokio::spawn(graph_runner(
            control_rx,
            signals.clone(),
            EventPlane::new().0,
            safety.clone(),
            GraphValues::default(),
        ));
        let slots = CommandSlots::new();
        control
            .send(GraphControl::Set {
                key: GraphKey::Toy(1),
                graph: graph(),
                slots: slots.clone(),
            })
            .unwrap();
        let boop = PortId::new(ModuleInstanceId::new(OSC_MODULE), "/avatar/parameters/Boop");

        safety.panic();
        signals.write(&boop, Signal::Float(0.8));
        sleep(Duration::from_millis(50)).await;
        assert!(slots.take_changed().is_empty(), "nothing while panicked");

        safety.resume();
        sleep(Duration::from_millis(50)).await;
        assert!(
            slots.take_changed().is_empty(),
            "no replay of 0.8 after resume"
        );

        signals.write(&boop, Signal::Float(0.5));
        sleep(Duration::from_millis(50)).await;
        assert_eq!(slots.take_changed().len(), 1, "new input goes through");

        drop(control);
        runner.await.unwrap();
    }
}
