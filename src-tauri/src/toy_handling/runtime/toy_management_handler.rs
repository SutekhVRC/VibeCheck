/*
    This handler will send and receive updates to toys
    - communicate ToyUpdate to and from main thread SEND/RECV (Toys will be indexed on the main thread) (Connects and disconnect toy updates are handled by client event handler)
        + Keep a thread count of connected toys. Add/remove as recvs ToyUpdates from main thread
        + Send toy updates like (battery updates)
*/
// Uses TME send and recv channel

use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use buttplug::client::ButtplugClientDevice;
use futures_timer::Delay;
use log::{error as logerr, info, warn};
use parking_lot::{lock_api::Mutex, RawMutex};
use tauri::AppHandle;
use tokio::{
    runtime::Runtime,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        watch,
    },
    task::JoinHandle,
    time::Instant,
};

use crate::{
    osc::{logic::toy_input_routine, OSCNetworking},
    toy_handling::{
        osc_processor::parse_osc_message,
        runtime::toy_emitter_thread::{
            toy_emitter_thread, EmitterThreadData, ToyEmitterThreadSignal,
        },
        toy_manager::ToyManager,
        toyops::VCToy,
        ToySig,
    },
    vcore::ipc::call_plane::{TmSig, ToyManagementEvent, ToyUpdate},
};

use tokio::sync::{
    self,
    broadcast::{Receiver as BReceiver, Sender as BSender},
};

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
    toy: ToyUpdate,
    dev: Arc<ButtplugClientDevice>,
    vc_toy: &mut VCToy,
) {
    let ToyUpdate::AlterToy(new_toy) = toy else {
        return;
    };
    if new_toy.toy_id != dev.index() {
        return;
    }

    vc_toy.parsed_toy_features = new_toy.parsed_toy_features;
    if vc_toy.bt_update_rate != new_toy.bt_update_rate {
        vc_toy.bt_update_rate = new_toy.bt_update_rate;
        emitter_thread_tx.send(ToyEmitterThreadSignal::UpdateRate(new_toy.bt_update_rate));
    }

    info!("Altered toy: {}", new_toy.toy_id);
}

pub async fn toy_management_handler(
    tme_send: UnboundedSender<ToyManagementEvent>,
    mut tme_recv: UnboundedReceiver<ToyManagementEvent>,
    mut core_toy_manager: ToyManager,
    mut vc_config: OSCNetworking,
    app_handle: AppHandle,
) {
    let toy_thread_function = |async_rt: Arc<Mutex<RawMutex, Option<Runtime>>>,
                               dev: Arc<ButtplugClientDevice>,
                               mut toy_bcst_rx: BReceiver<ToySig>,
                               mut vc_toy: VCToy| {
        // Read toy config here?
        async move {
            // Create in_signal channel for emitter thread
            let (emitter_thread_tx, emitter_thread_rx) =
                unbounded_channel::<ToyEmitterThreadSignal>();
            let (emitter_thread_osc_tx, emitter_thread_osc_rx) = watch::channel(None);

            let tet_data = EmitterThreadData::new(
                emitter_thread_rx,
                emitter_thread_osc_rx,
                vc_toy.bt_update_rate,
            );

            async_rt
                .lock()
                .as_ref()
                .unwrap()
                .spawn(async move { toy_emitter_thread(tet_data).await });

            while dev.connected() {
                let Ok(ts) = toy_bcst_rx.recv().await else {
                    continue;
                };
                match ts {
                    ToySig::OSCMsg(mut msg) => {
                        parse_osc_message(
                            &emitter_thread_osc_tx,
                            &mut msg,
                            dev.clone(),
                            &mut vc_toy.parsed_toy_features,
                        )
                        .await
                    }
                    ToySig::UpdateToy(toy) => {
                        update_toy(&emitter_thread_tx, toy, dev.clone(), &mut vc_toy);
                    }
                }
            }
            emitter_thread_tx.send(ToyEmitterThreadSignal::StopExecution);
            info!(
                "Device {} disconnected! Leaving listening routine!",
                dev.index()
            );
        }
    }; // Toy listening routine

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

        let toy_async_rt: Arc<Mutex<RawMutex, Option<Runtime>>> =
            Arc::new(Mutex::new(Some(Runtime::new().unwrap())));
        info!("Started listening!");
        // Recv events (listening)

        // Toy threads
        let mut running_toy_ths: HashMap<u32, JoinHandle<()>> = HashMap::new();

        // Broadcast channels for toy commands
        // These will only be used for UpdateToy commands now
        let (toy_sig_bcst_tx, _toy_sig_bcst_rx): (BSender<ToySig>, BReceiver<ToySig>) =
            sync::broadcast::channel(1024);

        // Create toy threads
        for toy in &core_toy_manager.online_toys {
            let toy_thread_function_run = toy_thread_function(
                toy_async_rt.clone(),
                toy.1.device_handle.clone(),
                toy_sig_bcst_tx.subscribe(),
                toy.1.clone(),
            );
            let new_thread = {
                toy_async_rt
                    .clone()
                    .lock()
                    .as_ref()
                    .unwrap()
                    .spawn(async move {
                        toy_thread_function_run.await;
                    })
            };
            running_toy_ths.insert(*toy.0, new_thread);
            info!("Toy: {} started listening..", *toy.0);
        }

        // Create OSC listener thread
        let toy_bcst_tx_osc = toy_sig_bcst_tx.clone();
        info!("Spawning OSC listener..");
        let vc_conf_clone = vc_config.clone();
        let tme_send_clone = tme_send.clone();
        let app_handle_clone = app_handle.clone();
        thread::spawn(move || {
            toy_input_routine(
                toy_bcst_tx_osc,
                tme_send_clone,
                app_handle_clone,
                vc_conf_clone,
            )
        });

        loop {
            // Recv event (listening)
            let event = tme_recv.recv().await;
            let Some(event) = event else { continue };
            match event {
                // Handle Toy Update Signals
                ToyManagementEvent::Tu(tu) => {
                    match tu {
                        ToyUpdate::AddToy(toy) => {
                            core_toy_manager.online_toys.insert(toy.toy_id, toy.clone());
                            let toy_thread_function_run = toy_thread_function(
                                toy_async_rt.clone(),
                                toy.device_handle.clone(),
                                toy_sig_bcst_tx.subscribe(),
                                toy.clone(),
                            );
                            let new_thread =
                                {
                                    toy_async_rt.clone().lock().as_ref().unwrap().spawn(
                                        async move {
                                            toy_thread_function_run.await;
                                        },
                                    )
                                };
                            running_toy_ths.insert(toy.toy_id, new_thread);
                            info!("Toy: {} started listening..", toy.toy_id);
                        }
                        ToyUpdate::RemoveToy(id) => {
                            // OSC Listener thread will only die on StopListening event
                            if let Some(toy) = running_toy_ths.remove(&id) {
                                toy.abort();
                                match toy.await {
                                    Ok(()) => info!("Toy {} thread finished", id),
                                    Err(e) => {
                                        warn!("Toy {} thread failed to reach completion: {}", id, e)
                                    }
                                }
                                info!("[TOY ID: {}] Stopped listening. (ToyUpdate::RemoveToy)", id);
                                running_toy_ths.remove(&id);
                                core_toy_manager.online_toys.remove(&id);
                            }
                        }
                        ToyUpdate::AlterToy(toy) => {
                            match toy_sig_bcst_tx
                                .send(ToySig::UpdateToy(ToyUpdate::AlterToy(toy.clone())))
                            {
                                Ok(receivers) => {
                                    info!("Sent ToyUpdate broadcast to {} toys", receivers - 1)
                                }
                                Err(e) => {
                                    logerr!("Failed to send UpdateToy: {}", e)
                                }
                            }
                            core_toy_manager.online_toys.insert(toy.toy_id, toy);
                        }
                    }
                }
                // Handle Management Signals
                ToyManagementEvent::Sig(tm_sig) => {
                    match tm_sig {
                        TmSig::StartListening(osc_net) => {
                            vc_config = osc_net;
                            // Already listening
                        }
                        TmSig::StopListening => {
                            // Stop listening on every device and clean running thread hashmap

                            for toy in &mut running_toy_ths {
                                toy.1.abort();
                                match toy.1.await {
                                    Ok(()) => {
                                        info!("Toy {} thread finished", toy.0)
                                    }
                                    Err(e) => warn!(
                                        "Toy {} thread failed to reach completion: {}",
                                        toy.0, e
                                    ),
                                }
                                info!("[TOY ID: {}] Stopped listening. (TMSIG)", toy.0);
                            }
                            running_toy_ths.clear();
                            drop(_toy_sig_bcst_rx); // Causes OSC listener to die
                            toy_async_rt
                                .clone()
                                .lock()
                                .take()
                                .unwrap()
                                .shutdown_background();
                            listening = false;
                            info!("Toys: {}", core_toy_manager.online_toys.len());
                            break; //Stop Listening
                        }
                        TmSig::TMHReset => {
                            // Stop listening on every device and clean running thread hashmap
                            info!("TMHReset");

                            for toy in &mut running_toy_ths {
                                toy.1.abort();
                                match toy.1.await {
                                    Ok(()) => {
                                        info!("Toy {} thread finished", toy.0)
                                    }
                                    Err(e) => warn!(
                                        "Toy {} thread failed to reach completion: {}",
                                        toy.0, e
                                    ),
                                }
                                info!("[TOY ID: {}] Stopped listening. (TMSIG)", toy.0);
                            }
                            running_toy_ths.clear();
                            drop(_toy_sig_bcst_rx); // Causes OSC listener to die
                            toy_async_rt
                                .clone()
                                .lock()
                                .take()
                                .unwrap()
                                .shutdown_background();
                            listening = false;
                            info!("Toys: {}", core_toy_manager.online_toys.len());
                            break; //Stop Listening
                        }
                        _ => {}
                    }
                } // Event handled
            }
        }
    } // Management loop
}
