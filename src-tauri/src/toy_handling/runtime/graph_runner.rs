/*
 * Ticks every toy's graph at a fixed rate.
 *
 * One task per listening session. Each tick it runs every toy's graph against
 * the session's signal plane and writes the resulting commands into that
 * toy's CommandSlots, which the toy's emitter sends at the toy's own update
 * rate.
 */

use std::{collections::HashMap, time::Duration};

use log::info;
use tokio::{
    sync::mpsc::UnboundedReceiver,
    time::{interval, Instant, MissedTickBehavior},
};
use vibecheck_core::{graph::Graph, CommandSlots, EventPlane, SafetyLayer, SignalPlane};

/// How often graphs run.
pub const GRAPH_TICK_HZ: u64 = 100;

/// Longest gap a single tick accounts for, so a stall (e.g. the machine
/// sleeping) doesn't make time-based processors jump.
const MAX_DT: f32 = 0.1;

pub enum GraphControl {
    /// Run `graph` for this toy from now on, replacing any previous graph.
    Set {
        toy_id: u32,
        graph: Graph,
        slots: CommandSlots,
    },
    Remove(u32),
}

struct ToyGraph {
    graph: Graph,
    slots: CommandSlots,
}

/// Events processors emit are published on `events`.
pub async fn graph_runner(
    mut control: UnboundedReceiver<GraphControl>,
    signals: SignalPlane,
    events: EventPlane,
    safety: SafetyLayer,
) {
    let mut toys: HashMap<u32, ToyGraph> = HashMap::new();

    let mut ticker = interval(Duration::from_nanos(1_000_000_000 / GRAPH_TICK_HZ));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut last_tick = Instant::now();

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
    async fn applies_the_master_volume_even_while_running() {
        let signals = SignalPlane::new();
        let safety = SafetyLayer::default();
        let (control, control_rx) = unbounded_channel();
        let runner = tokio::spawn(graph_runner(
            control_rx,
            signals.clone(),
            EventPlane::new().0,
            safety.clone(),
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
        safety.set_volume(0.5);
        signals.write(&boop, Signal::Float(0.8));
        sleep(Duration::from_millis(50)).await;
        assert!((slots.take_changed()[0].level() - 0.4).abs() < 1e-6);

        // Changing the volume from Settings affects the running session
        safety.set_volume(1.0);
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
