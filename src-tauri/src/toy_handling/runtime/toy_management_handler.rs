/*
    This handler will send and receive updates to toys
    - communicate ToyUpdate to and from main thread SEND/RECV (Toys will be indexed on the main thread) (Connects and disconnect toy updates are handled by client event handler)
        + Keep a thread count of connected toys. Add/remove as recvs ToyUpdates from main thread
        + Send toy updates like (battery updates)

    While listening, each session runs on its own tokio runtime:
    - OscInput (vibecheck-core) receives OSC, writes parameter values to the
      session's signal plane, turns avatar changes into events, and taps every
      other message to the OSC router.
    - The OSC router handles OSC API messages.
    - The core event handler reacts to events on the session's event plane
      (avatar change and VRChat disconnecting stop all toys; output errors
      are logged).
    - The presence task publishes VRChat connecting and disconnecting.
    - The graph runner ticks every toy's graph (built from its config, and
      including its SPS/TPS input processors) at a fixed rate and fills each
      toy's command slots.
    - The diagnostics task sends live values to the diagnostics view while
      it's open.
    - One task per toy applies config changes to its output; its emitter task
      sends the toy's commands.
*/
// Uses TME send and recv channel

use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use futures_timer::Delay;
use log::{debug, error as logerr, info, warn};
use parking_lot::{lock_api::Mutex, RawMutex};
use tauri::AppHandle;
use tokio::{
    runtime::Runtime,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time::Instant,
};
use vibecheck_core::{
    modules::osc::OscInput, CommandSlots, Diagnostics, EventPlane, Input, InputCtx,
    ModuleInstanceId, SafetyLayer, SignalPlane,
};

use crate::{
    extensions::{SessionContext, SessionExtension},
    osc::{avatar_catalog::AvatarCatalog, avatar_query::AvatarQuery, OSCNetworking},
    toy_handling::{
        runtime::{
            diagnostics::diagnostics_task,
            graph_runner::{graph_runner, GraphControl, GraphKey, GraphValues},
            osc_routing::{core_event_handler, osc_router},
            presence::{publish_connection, vrchat_presence},
            toy_emitter_thread::{
                toy_emitter_thread, EmitterThreadData, SentLevels, ToyEmitterThreadSignal,
            },
            toy_graph::{build_toy_graph, OSC_MODULE},
            toy_output::{buttplug_output, toy_actuators, toy_module_id},
        },
        toy_manager::ToyManager,
        toyops::VCToy,
    },
    vcore::ipc::call_plane::{TmSig, ToyManagementEvent, ToyUpdate},
};

/// The session runtime is only taken out by `stop_listening`, which ends
/// the session.
const SESSION_RUNNING: &str = "the session runtime runs until stop_listening";

type ToyRuntime = Arc<Mutex<RawMutex, Option<Runtime>>>;

/* https://github.com/snd/hertz/issues/2#issuecomment-850859904
 * The hertz crate's sleep_for_constant_rate() function is broken,
 * therefore isolating just the fixed function here.
*/
pub async fn sleep_for_constant_rate(rate: u64, start: Instant) {
    let ns_per_frame = (Duration::from_secs(1).as_nanos() as f64 / rate as f64).round() as u64;
    let duration = Duration::from_nanos(ns_per_frame);
    let elapsed = start.elapsed();
    if elapsed < duration {
        Delay::new(duration - elapsed).await;
    }
}

/// Applies a config change. Returns `false` if it was for a different toy.
#[inline(always)]
fn update_toy(
    emitter_thread_tx: &UnboundedSender<ToyEmitterThreadSignal>,
    new_toy: VCToy,
    vc_toy: &mut VCToy,
) -> bool {
    if new_toy.toy_id != vc_toy.toy_id {
        return false;
    }

    vc_toy.parsed_toy_features = new_toy.parsed_toy_features;
    // Flip Input lives on the output now
    let _ = emitter_thread_tx.send(ToyEmitterThreadSignal::UpdateActuators(toy_actuators(
        vc_toy,
    )));
    if vc_toy.bt_update_rate != new_toy.bt_update_rate {
        vc_toy.bt_update_rate = new_toy.bt_update_rate;
        let _ = emitter_thread_tx.send(ToyEmitterThreadSignal::UpdateRate(new_toy.bt_update_rate));
    }

    info!("Altered toy: {}", new_toy.toy_id);
    true
}

/// Toy listening routine: starts the toy's emitter and applies config changes
/// to it, until the device disconnects or the session ends.
async fn toy_task(
    async_rt: ToyRuntime,
    mut control_rx: UnboundedReceiver<VCToy>,
    mut vc_toy: VCToy,
    slots: CommandSlots,
    signals: SignalPlane,
    events: EventPlane,
    safety: SafetyLayer,
) {
    let dev = vc_toy.device_handle.clone();
    let module = toy_module_id(&vc_toy);

    // Create in_signal channel for emitter thread
    let (emitter_thread_tx, emitter_thread_rx) = unbounded_channel::<ToyEmitterThreadSignal>();
    let tet_data = EmitterThreadData::new(
        emitter_thread_rx,
        slots,
        buttplug_output(&vc_toy).with_events(module.clone(), events),
        vc_toy.bt_update_rate,
        safety,
        SentLevels::new(signals),
    );

    async_rt
        .lock()
        .as_ref()
        .expect(SESSION_RUNNING)
        .spawn(async move { toy_emitter_thread(tet_data).await });

    while dev.connected() {
        match control_rx.recv().await {
            Some(new_toy) => {
                update_toy(&emitter_thread_tx, new_toy, &mut vc_toy);
            }
            None => break,
        }
    }
    let _ = emitter_thread_tx.send(ToyEmitterThreadSignal::StopExecution);
    info!(
        "Device {} disconnected! Leaving listening routine!",
        dev.index()
    );
}

/// Everything one listening session shares.
struct Session {
    async_rt: ToyRuntime,
    signals: SignalPlane,
    avatar_query: AvatarQuery,
    events: EventPlane,
    safety: SafetyLayer,
    diagnostics: Diagnostics,
    graphs: UnboundedSender<GraphControl>,
}

impl Session {
    /// Creates the session runtime, and starts the core event handler and the
    /// graph runner on it. `safety` is the app-wide safety layer, so the
    /// master intensity and panic stop apply here too. `diagnostics` is the
    /// app-wide diagnostics switch and log.
    fn new(
        app_handle: &AppHandle,
        safety: SafetyLayer,
        diagnostics: Diagnostics,
        avatar_catalog: AvatarCatalog,
    ) -> Self {
        let async_rt: ToyRuntime = Arc::new(Mutex::new(Some(
            Runtime::new().expect("couldn't start the toy session's async runtime"),
        )));
        let signals = SignalPlane::new();
        let (events, event_rx) = EventPlane::new();
        let (graphs, graphs_rx) = unbounded_channel();
        let presence = avatar_catalog.presence();
        let avatar_query = AvatarQuery::new(
            signals.clone(),
            ModuleInstanceId::new(OSC_MODULE),
            avatar_catalog,
            diagnostics.clone(),
        );
        // Don't lock the global state here: native_vibecheck_enable holds it
        // while waiting for this session to start listening
        {
            let rt_lock = async_rt.lock();
            let rt = rt_lock.as_ref().expect(SESSION_RUNNING);
            rt.spawn(core_event_handler(
                event_rx,
                avatar_query.clone(),
                signals.clone(),
                graphs.clone(),
                safety.clone(),
                diagnostics.clone(),
                app_handle.clone(),
            ));
            let graph_values = GraphValues::new(diagnostics.clone());
            rt.spawn(graph_runner(
                graphs_rx,
                signals.clone(),
                events.clone(),
                safety.clone(),
                graph_values.clone(),
            ));
            rt.spawn(vrchat_presence(presence, events.clone()));
            rt.spawn(diagnostics_task(
                diagnostics.clone(),
                signals.clone(),
                graph_values,
                avatar_query.clone(),
                app_handle.clone(),
            ));
        }
        Self {
            async_rt,
            signals,
            avatar_query,
            events,
            safety,
            diagnostics,
            graphs,
        }
    }

    /// What an app's session extensions get (see extensions.rs).
    fn context(&self, app_handle: &AppHandle) -> SessionContext {
        SessionContext {
            app: app_handle.clone(),
            runtime: self
                .async_rt
                .lock()
                .as_ref()
                .expect(SESSION_RUNNING)
                .handle()
                .clone(),
            signals: self.signals.clone(),
            events: self.events.clone(),
            safety: self.safety.clone(),
            diagnostics: self.diagnostics.clone(),
            graphs: self.graphs.clone(),
        }
    }

    /// Builds the toy's graph from its config and hands it to the runner,
    /// and registers the toy as an output other graphs can drive.
    fn set_graph(&self, toy: &VCToy, slots: &CommandSlots) {
        let _ = self.graphs.send(GraphControl::SetOutput {
            module: toy_module_id(toy),
            slots: slots.clone(),
        });
        match build_toy_graph(&toy_module_id(toy), &toy.parsed_toy_features) {
            Ok(graph) => {
                let _ = self.graphs.send(GraphControl::Set {
                    key: GraphKey::Toy(toy.toy_id),
                    graph,
                    slots: slots.clone(),
                });
            }
            Err(e) => logerr!("Couldn't build the graph for toy {}: {}", toy.toy_id, e),
        }
    }
}

struct RunningToy {
    handle: JoinHandle<()>,
    control: UnboundedSender<VCToy>,
    slots: CommandSlots,
    module: ModuleInstanceId,
}

/// Starts the toy's graph and its task, and publishes that it connected
/// (also for toys already connected when listening starts).
fn start_toy(session: &Session, toy: VCToy) -> RunningToy {
    let module = toy_module_id(&toy);
    publish_connection(&session.events, module.clone(), true);
    let (control_tx, control_rx) = unbounded_channel();
    // One latest-command slot per motor: filled by the graph, drained by the emitter
    let slots = CommandSlots::new();
    let toy_id = toy.toy_id;

    session.set_graph(&toy, &slots);

    let handle = session
        .async_rt
        .lock()
        .as_ref()
        .expect(SESSION_RUNNING)
        .spawn(toy_task(
            session.async_rt.clone(),
            control_rx,
            toy,
            slots.clone(),
            session.signals.clone(),
            session.events.clone(),
            session.safety.clone(),
        ));
    info!("Toy: {} started listening..", toy_id);

    RunningToy {
        handle,
        control: control_tx,
        slots,
        module,
    }
}

/// Starts OscInput and the OSC router for a session.
fn start_osc(
    session: &Session,
    vc_config: &OSCNetworking,
    tme_send: &UnboundedSender<ToyManagementEvent>,
    app_handle: &AppHandle,
) {
    let (tap_tx, tap_rx) = unbounded_channel();
    let ctx = InputCtx::new(
        ModuleInstanceId::new(OSC_MODULE),
        session.signals.clone(),
        session.events.clone(),
    )
    .with_diagnostics(session.diagnostics.clone());
    let bind = SocketAddr::V4(vc_config.listen_addr());
    let tme_send = tme_send.clone();
    let avatar_query = session.avatar_query.clone();

    let rt_lock = session.async_rt.lock();
    let rt = rt_lock.as_ref().expect(SESSION_RUNNING);

    info!("Spawning OSC listener..");
    rt.spawn(async move {
        match OscInput::bind(bind).await {
            Ok(input) => {
                let _ = tme_send.send(ToyManagementEvent::Sig(TmSig::Listening));
                info!("Listen sock is bound {}", bind);
                // The avatar may have loaded before VibeCheck was listening:
                // fill in parameters VRChat won't send again
                tokio::spawn(async move { avatar_query.fill_now().await });
                let mut input = input.with_tap(tap_tx);
                if let Err(e) = input.run(ctx).await {
                    logerr!("OSC input stopped with an error: {}", e);
                }
                info!("OSC input stopped");
            }
            Err(e) => {
                logerr!("Failed to bind OSC listen socket {}: {}", bind, e);
                let _ = tme_send.send(ToyManagementEvent::Sig(TmSig::BindError));
            }
        }
    });
    rt.spawn(osc_router(tap_rx, app_handle.clone()));
}

/// Stops the apps' session extensions, every toy task and the session's
/// runtime (which ends OscInput, the router, the event handler, the graph
/// runner and any task an app spawned, and frees the OSC port).
async fn stop_listening(
    running_toys: &mut HashMap<u32, RunningToy>,
    session: &Session,
    extensions: &mut [Box<dyn SessionExtension>],
) {
    // First, while the session's tasks still run
    for extension in extensions.iter_mut() {
        extension.stop();
    }
    for (id, toy) in running_toys.drain() {
        toy.handle.abort();
        match toy.handle.await {
            Ok(()) => info!("Toy {} thread finished", id),
            Err(e) => warn!("Toy {} thread failed to reach completion: {}", id, e),
        }
        info!("[TOY ID: {}] Stopped listening. (TMSIG)", id);
    }
    session
        .async_rt
        .lock()
        .take()
        .expect(SESSION_RUNNING)
        .shutdown_background();
}

pub async fn toy_management_handler(
    tme_send: UnboundedSender<ToyManagementEvent>,
    mut tme_recv: UnboundedReceiver<ToyManagementEvent>,
    mut core_toy_manager: ToyManager,
    mut vc_config: OSCNetworking,
    safety: SafetyLayer,
    diagnostics: Diagnostics,
    avatar_catalog: AvatarCatalog,
    mut session_extensions: Vec<Box<dyn SessionExtension>>,
    app_handle: AppHandle,
) {
    let mut listening = false;

    // Management loop
    loop {
        // Recv event (not listening)
        if let Some(event) = tme_recv.recv().await {
            match event {
                // Handle Toy Update Signals
                ToyManagementEvent::Tu(tu) => match tu {
                    ToyUpdate::AddToy(toy) => {
                        core_toy_manager.online_toys.insert(toy.toy_id, toy);
                    }
                    ToyUpdate::RemoveToy(id) => {
                        core_toy_manager.online_toys.remove(&id);
                    }
                    ToyUpdate::AlterToy(toy) => {
                        core_toy_manager.online_toys.insert(toy.toy_id, toy);
                    }
                },
                // Handle Management Signals
                ToyManagementEvent::Sig(tm_sig) => {
                    match tm_sig {
                        TmSig::StartListening(osc_net) => {
                            vc_config = osc_net;
                            listening = true;
                        }
                        TmSig::StopListening => {
                            // Already not listening
                            info!("StopListening but not listening");
                        }
                        TmSig::TMHReset => {
                            info!("TMHReset but not listening");
                        }
                        _ => {}
                    }
                }
            } // Event handled
        }

        if !listening {
            continue;
        }

        let session = Session::new(
            &app_handle,
            safety.clone(),
            diagnostics.clone(),
            avatar_catalog.clone(),
        );
        info!("Started listening!");
        // Recv events (listening)

        let mut running_toys: HashMap<u32, RunningToy> = HashMap::new();

        // Create toy tasks
        for toy in core_toy_manager.online_toys.values() {
            running_toys.insert(toy.toy_id, start_toy(&session, toy.clone()));
        }

        // Create OSC listener
        start_osc(&session, &vc_config, &tme_send, &app_handle);

        // Apps' own graphs and outputs (see extensions.rs)
        let context = session.context(&app_handle);
        for extension in session_extensions.iter_mut() {
            extension.start(&context);
        }

        loop {
            // Recv event (listening)
            let event = tme_recv.recv().await;
            let Some(event) = event else { continue };
            match event {
                // Handle Toy Update Signals
                ToyManagementEvent::Tu(tu) => match tu {
                    ToyUpdate::AddToy(toy) => {
                        core_toy_manager.online_toys.insert(toy.toy_id, toy.clone());
                        running_toys.insert(toy.toy_id, start_toy(&session, toy));
                    }
                    ToyUpdate::RemoveToy(id) => {
                        if let Some(toy) = running_toys.remove(&id) {
                            // Its activity ends here too: no "stopped" follows
                            publish_connection(&session.events, toy.module.clone(), false);
                            let _ = session.graphs.send(GraphControl::Remove(GraphKey::Toy(id)));
                            let _ = session
                                .graphs
                                .send(GraphControl::RemoveOutput(toy.module.clone()));
                            toy.handle.abort();
                            match toy.handle.await {
                                Ok(()) => info!("Toy {} thread finished", id),
                                Err(e) => {
                                    warn!("Toy {} thread failed to reach completion: {}", id, e)
                                }
                            }
                            info!("[TOY ID: {}] Stopped listening. (ToyUpdate::RemoveToy)", id);
                            core_toy_manager.online_toys.remove(&id);
                        }
                    }
                    ToyUpdate::AlterToy(toy) => {
                        debug!("Got ToyUpdate::AlterToy signal!");
                        match running_toys.get(&toy.toy_id) {
                            Some(running) => {
                                // Fresh graph with the new settings (resets processor state)
                                session.set_graph(&toy, &running.slots);
                                if running.control.send(toy.clone()).is_err() {
                                    logerr!(
                                        "Failed to send UpdateToy: toy {} task has stopped",
                                        toy.toy_id
                                    )
                                }
                            }
                            None => debug!("AlterToy for toy {} which isn't running", toy.toy_id),
                        }
                        core_toy_manager.online_toys.insert(toy.toy_id, toy);
                    }
                },
                // Handle Management Signals
                ToyManagementEvent::Sig(tm_sig) => match tm_sig {
                    TmSig::StartListening(osc_net) => {
                        vc_config = osc_net;
                        // Already listening
                    }
                    TmSig::StopListening => {
                        stop_listening(&mut running_toys, &session, &mut session_extensions).await;
                        listening = false;
                        info!("Toys: {}", core_toy_manager.online_toys.len());
                        break; //Stop Listening
                    }
                    TmSig::TMHReset => {
                        info!("TMHReset");
                        stop_listening(&mut running_toys, &session, &mut session_extensions).await;
                        listening = false;
                        info!("Toys: {}", core_toy_manager.online_toys.len());
                        break; //Stop Listening
                    }
                    _ => {}
                },
            }
        }
    } // Management loop
}
