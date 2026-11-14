/*
 * Ticks every toy's graph at a fixed rate.
 *
 * One task per listening session. Each tick it runs every toy's graph against
 * the session's signal plane and writes the resulting commands into that
 * toy's CommandSlots, which the toy's emitter sends at the toy's own update
 * rate.
 */

use std::{collections::HashMap, sync::Arc, time::Duration};

use log::info;
use parking_lot::Mutex;
use tokio::{
    sync::mpsc::UnboundedReceiver,
    time::{interval, Instant, MissedTickBehavior},
};
use vibecheck_core::{
    graph::Graph, CommandSlots, Diagnostics, EventPlane, GraphSnapshot, SafetyLayer, SignalPlane,
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

    fn update(&self, toys: &HashMap<u32, ToyGraph>) {
        let mut latest = self.latest.lock();
        latest.clear();
        if self.diagnostics.enabled() {
            latest.extend(toys.iter().map(|(id, toy)| (*id, toy.graph.snapshot())));
        }
    }
}

pub enum GraphControl {
    /// Run `graph` for this toy from now on, replacing any previous graph.
    Set {
        toy_id: u32,
        graph: Graph,
        slots: CommandSlots,
    },
    Remove(u32),
    /// Start every graph over (see `Graph::reset`), e.g. on avatar change.
    ResetAll,
}

struct ToyGraph {
    graph: Graph,
    slots: CommandSlots,
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
    let mut toys: HashMap<u32, ToyGraph> = HashMap::new();

    let mut ticker = interval(Duration::from_nanos(1_000_000_000 / GRAPH_TICK_HZ));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut last_tick = Instant::now();
    let mut ticks: u32 = 0;

    loop {
        tokio::select! {
            // Apply graph changes before the next tick
            biased;
            change = control.recv() => match change {
                Some(GraphControl::Set { toy_id, graph, slots }) => {
                    toys.insert(toy_id, ToyGraph { graph, slots });
                }
                Some(GraphControl::Remove(toy_id)) => {
                    toys.remove(&toy_id);
                }
                Some(GraphControl::ResetAll) => {
                    for toy in toys.values_mut() {
                        toy.graph.reset();
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
                        toy.slots.write(cmd);
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
    use vibecheck_core::{ModuleInstanceId, PortId, Signal};

    fn graph() -> Graph {
        let feature = VCToyFeature::new(
            vec![ToyParameter {
                parameter: "/avatar/parameters/Boop".into(),
                processing_mode: ProcessingMode::Raw,
            }],
            0,
            VCFeatureType::Vibrator,
        );
        build_toy_graph(
            &ModuleInstanceId::new("toy:test:0"),
            &VCToyFeatures {
                features: vec![feature],
            },
        )
        .unwrap()
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
                toy_id: 1,
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

        control.send(GraphControl::Remove(1)).unwrap();
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
                toy_id: 1,
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
                toy_id: 1,
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
                toy_id: 1,
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
