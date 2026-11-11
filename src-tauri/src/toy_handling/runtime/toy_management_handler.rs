/*
    This handler will send and receive updates to toys
    - communicate ToyUpdate to and from main thread SEND/RECV (Toys will be indexed on the main thread) (Connects and disconnect toy updates are handled by client event handler)
        + Keep a thread count of connected toys. Add/remove as recvs ToyUpdates from main thread
        + Send toy updates like (battery updates)

    While listening, each session runs on its own tokio runtime:
    - OscInput (vibecheck-core) receives OSC, turns avatar changes into events
      and taps every other message to the OSC router.
    - The OSC router handles OSC API messages and sends each other message only
      to the toy tasks that use its address.
    - The core event handler reacts to events (avatar change stops all toys).
    - One task per toy processes its OSC messages and config changes, each on
      its own channel, and feeds its emitter task.
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
    modules::osc::{rosc::OscMessage, OscInput},
    CommandSlots, EventPlane, Input, InputCtx, ModuleInstanceId, SignalPlane,
};

use crate::{
    osc::OSCNetworking,
    toy_handling::{
        osc_processor::parse_osc_message,
        runtime::{
            osc_routing::{core_event_handler, osc_router, OscRoutes},
            toy_emitter_thread::{toy_emitter_thread, EmitterThreadData, ToyEmitterThreadSignal},
            toy_output::{buttplug_output, ToyCommandSink},
        },
        toy_manager::ToyManager,
        toyops::VCToy,
    },
    vcore::ipc::call_plane::{TmSig, ToyManagementEvent, ToyUpdate},
};

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

#[inline(always)]
fn update_toy(
    emitter_thread_tx: &UnboundedSender<ToyEmitterThreadSignal>,
    new_toy: VCToy,
    vc_toy: &mut VCToy,
) {
    if new_toy.toy_id != vc_toy.toy_id {
        return;
    }

    vc_toy.parsed_toy_features = new_toy.parsed_toy_features;
    if vc_toy.bt_update_rate != new_toy.bt_update_rate {
        vc_toy.bt_update_rate = new_toy.bt_update_rate;
        let _ = emitter_thread_tx.send(ToyEmitterThreadSignal::UpdateRate(new_toy.bt_update_rate));
    }

    info!("Altered toy: {}", new_toy.toy_id);
}

/// Toy listening routine: processes this toy's OSC messages and config
/// changes until the device disconnects or the session ends.
async fn toy_task(
    async_rt: ToyRuntime,
    mut osc_rx: UnboundedReceiver<OscMessage>,
    mut control_rx: UnboundedReceiver<VCToy>,
    mut vc_toy: VCToy,
) {
    let dev = vc_toy.device_handle.clone();

    // Create in_signal channel for emitter thread
    let (emitter_thread_tx, emitter_thread_rx) = unbounded_channel::<ToyEmitterThreadSignal>();
    // One latest-command slot per motor, drained by the emitter task
    let slots = CommandSlots::new();
    let command_sink = ToyCommandSink::new(&vc_toy, slots.clone());

    let tet_data = EmitterThreadData::new(
        emitter_thread_rx,
        slots,
        buttplug_output(&vc_toy),
        vc_toy.bt_update_rate,
    );

    async_rt
        .lock()
        .as_ref()
        .unwrap()
        .spawn(async move { toy_emitter_thread(tet_data).await });

    while dev.connected() {
        tokio::select! {
            // Apply config changes before any OSC still queued behind them
            biased;
            update = control_rx.recv() => match update {
                Some(new_toy) => update_toy(&emitter_thread_tx, new_toy, &mut vc_toy),
                None => break,
            },
            msg = osc_rx.recv() => match msg {
                Some(mut msg) => {
                    parse_osc_message(&command_sink, &mut msg, &mut vc_toy.parsed_toy_features)
                        .await
                }
                None => break,
            },
        }
    }
    let _ = emitter_thread_tx.send(ToyEmitterThreadSignal::StopExecution);
    info!(
        "Device {} disconnected! Leaving listening routine!",
        dev.index()
    );
}

struct RunningToy {
    handle: JoinHandle<()>,
    control: UnboundedSender<VCToy>,
}

/// Registers the toy with the router and starts its task.
fn start_toy(async_rt: &ToyRuntime, routes: &OscRoutes, toy: VCToy) -> RunningToy {
    let (osc_tx, osc_rx) = unbounded_channel();
    let (control_tx, control_rx) = unbounded_channel();
    let toy_id = toy.toy_id;
    routes.insert(toy_id, osc_tx, &toy.parsed_toy_features);

    let handle = async_rt.lock().as_ref().unwrap().spawn(toy_task(
        async_rt.clone(),
        osc_rx,
        control_rx,
        toy,
    ));
    info!("Toy: {} started listening..", toy_id);

    RunningToy {
        handle,
        control: control_tx,
    }
}

/// Starts OscInput, the OSC router and the core event handler for a session.
fn start_osc(
    async_rt: &ToyRuntime,
    routes: &OscRoutes,
    vc_config: &OSCNetworking,
    tme_send: &UnboundedSender<ToyManagementEvent>,
    app_handle: &AppHandle,
) {
    let (tap_tx, tap_rx) = unbounded_channel();
    let (events, event_rx) = EventPlane::new();
    let ctx = InputCtx::new(ModuleInstanceId::new("osc"), SignalPlane::new(), events);
    let bind = SocketAddr::V4(vc_config.bind);
    let tme_send = tme_send.clone();

    let rt_lock = async_rt.lock();
    let rt = rt_lock.as_ref().unwrap();

    info!("Spawning OSC listener..");
    rt.spawn(async move {
        match OscInput::bind(bind).await {
            Ok(input) => {
                let _ = tme_send.send(ToyManagementEvent::Sig(TmSig::Listening));
                info!("Listen sock is bound {}", bind);
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
    rt.spawn(osc_router(tap_rx, routes.clone(), app_handle.clone()));
    rt.spawn(core_event_handler(event_rx, app_handle.clone()));
}

/// Stops every toy task and the session's runtime (which ends OscInput, the
/// router and the event handler, and frees the OSC port).
async fn stop_listening(
    running_toys: &mut HashMap<u32, RunningToy>,
    routes: &OscRoutes,
    async_rt: &ToyRuntime,
) {
    for (id, toy) in running_toys.drain() {
        toy.handle.abort();
        match toy.handle.await {
            Ok(()) => info!("Toy {} thread finished", id),
            Err(e) => warn!("Toy {} thread failed to reach completion: {}", id, e),
        }
        info!("[TOY ID: {}] Stopped listening. (TMSIG)", id);
    }
    routes.clear();
    async_rt.lock().take().unwrap().shutdown_background();
}

pub async fn toy_management_handler(
    tme_send: UnboundedSender<ToyManagementEvent>,
    mut tme_recv: UnboundedReceiver<ToyManagementEvent>,
    mut core_toy_manager: ToyManager,
    mut vc_config: OSCNetworking,
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

        let toy_async_rt: ToyRuntime = Arc::new(Mutex::new(Some(Runtime::new().unwrap())));
        info!("Started listening!");
        // Recv events (listening)

        let routes = OscRoutes::default();
        let mut running_toys: HashMap<u32, RunningToy> = HashMap::new();

        // Create toy tasks
        for toy in core_toy_manager.online_toys.values() {
            running_toys.insert(toy.toy_id, start_toy(&toy_async_rt, &routes, toy.clone()));
        }

        // Create OSC listener
        start_osc(&toy_async_rt, &routes, &vc_config, &tme_send, &app_handle);

        loop {
            // Recv event (listening)
            let event = tme_recv.recv().await;
            let Some(event) = event else { continue };
            match event {
                // Handle Toy Update Signals
                ToyManagementEvent::Tu(tu) => match tu {
                    ToyUpdate::AddToy(toy) => {
                        core_toy_manager.online_toys.insert(toy.toy_id, toy.clone());
                        running_toys.insert(toy.toy_id, start_toy(&toy_async_rt, &routes, toy));
                    }
                    ToyUpdate::RemoveToy(id) => {
                        if let Some(toy) = running_toys.remove(&id) {
                            routes.remove(id);
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
                        routes.update(toy.toy_id, &toy.parsed_toy_features);
                        match running_toys.get(&toy.toy_id) {
                            Some(running) => {
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
                        stop_listening(&mut running_toys, &routes, &toy_async_rt).await;
                        listening = false;
                        info!("Toys: {}", core_toy_manager.online_toys.len());
                        break; //Stop Listening
                    }
                    TmSig::TMHReset => {
                        info!("TMHReset");
                        stop_listening(&mut running_toys, &routes, &toy_async_rt).await;
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
