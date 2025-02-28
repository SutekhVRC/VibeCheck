/*
    This handler will send and receive updates to toys
    - communicate ToyUpdate to and from main thread SEND/RECV (Toys will be indexed on the main thread) (Connects and disconnect toy updates are handled by client event handler)
        + Keep a thread count of connected toys. Add/remove as recvs ToyUpdates from main thread
        + Send toy updates like (battery updates)
*/
// Uses TME send and recv channel

use std::{collections::HashMap, sync::Arc, thread};

use buttplug::client::ButtplugClientDevice;
use log::{error as logerr, info, warn};
use tauri::AppHandle;
use tokio::{
    runtime::Runtime,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use crate::{
    osc::{logic::toy_input_routine, OSCNetworking},
    toy_handling::{
        osc_processor::parse_osc_message, toy_manager::ToyManager, toyops::VCToyFeatures, ToySig,
    },
    vcore::ipc::call_plane::{TmSig, ToyManagementEvent, ToyUpdate},
};

use tokio::sync::{
    self,
    broadcast::{Receiver as BReceiver, Sender as BSender},
};

#[inline(always)]
fn update_toy(toy: ToyUpdate, dev: Arc<ButtplugClientDevice>, vc_toy_features: &mut VCToyFeatures) {
    let ToyUpdate::AlterToy(new_toy) = toy else {
        return;
    };
    if new_toy.toy_id != dev.index() {
        return;
    }
    *vc_toy_features = new_toy.parsed_toy_features;
    info!("Altered toy: {}", new_toy.toy_id);
}

pub async fn toy_management_handler(
    tme_send: UnboundedSender<ToyManagementEvent>,
    mut tme_recv: UnboundedReceiver<ToyManagementEvent>,
    mut core_toy_manager: ToyManager,
    mut vc_config: OSCNetworking,
    app_handle: AppHandle,
) {
    let f = |dev: Arc<ButtplugClientDevice>,
             mut toy_bcst_rx: BReceiver<ToySig>,
             mut vc_toy_features: VCToyFeatures| {
        // Read toy config here?
        async move {
            // Put smooth_queue here
            // Put rate tracking here
            // Time tracking here?
            // Async runtime wrapped in Option for rate updating here????

            // Lock this to a user-set HZ value
            while dev.connected() {
                let Ok(ts) = toy_bcst_rx.recv().await else {
                    continue;
                };
                match ts {
                    ToySig::OSCMsg(mut msg) => {
                        parse_osc_message(&mut msg, dev.clone(), &mut vc_toy_features).await
                    }
                    ToySig::UpdateToy(toy) => update_toy(toy, dev.clone(), &mut vc_toy_features),
                }
            }
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

        // This is a nested runtime maybe remove
        // Would need to pass toy thread handles to VibeCheckState
        let toy_async_rt = Runtime::new().unwrap();
        info!("Started listening!");
        // Recv events (listening)
        // Create toy bcst channel

        // Toy threads
        let mut running_toy_ths: HashMap<u32, JoinHandle<()>> = HashMap::new();

        // Broadcast channels for toy commands
        let (toy_bcst_tx, _toy_bcst_rx): (BSender<ToySig>, BReceiver<ToySig>) =
            sync::broadcast::channel(1024);

        // Create toy threads
        for toy in &core_toy_manager.online_toys {
            let f_run = f(
                toy.1.device_handle.clone(),
                toy_bcst_tx.subscribe(),
                toy.1.parsed_toy_features.clone(),
            );
            running_toy_ths.insert(
                *toy.0,
                toy_async_rt.spawn(async move {
                    f_run.await;
                }),
            );
            info!("Toy: {} started listening..", *toy.0);
        }

        // Create OSC listener thread
        let toy_bcst_tx_osc = toy_bcst_tx.clone();
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
                            let f_run = f(
                                toy.device_handle,
                                toy_bcst_tx.subscribe(),
                                toy.parsed_toy_features.clone(),
                            );
                            running_toy_ths.insert(
                                toy.toy_id,
                                toy_async_rt.spawn(async move {
                                    f_run.await;
                                }),
                            );
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
                            match toy_bcst_tx
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
                            drop(_toy_bcst_rx); // Causes OSC listener to die
                            toy_async_rt.shutdown_background();
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
                            drop(_toy_bcst_rx); // Causes OSC listener to die
                            toy_async_rt.shutdown_background();
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
