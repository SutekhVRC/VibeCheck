use std::{sync::Arc, time::Duration};

use crate::{
    frontend::{
        frontend_types::{FeCoreEvent, FeScanEvent, FeToyEvent, FeVCToy},
        ToFrontend,
    },
    toy_handling::{
        toyops::{VCToy, VCToyFeatures},
        ToyPower,
    },
    vcore::{
        errors::VCError,
        ipc::{
            call_plane::{ToyManagementEvent, ToyUpdate},
            emit_plane::{emit_core_event, emit_toy_event},
        },
        state::VibeCheckState,
    },
};
use buttplug::client::ButtplugClientEvent;
use futures::StreamExt;
use futures_timer::Delay;
use log::{error as logerr, info, trace, warn};
use parking_lot::Mutex;
use tauri_plugin_notification::{Notification, NotificationExt};
use std::sync::mpsc::Sender;
use tauri::{AppHandle};
use tokio::sync::mpsc::UnboundedSender;

/*
    This handler will handle the adding and removal of toys
    Needs Signals in and out to communicate with main thread
    - communicate errors and handler state (Errors to tell main thread its shutting down && State to receive shutdown from main thread) RECV/SEND
    - communicate toy events (add/remove) ONLY SEND?
*/
// Uses CEH send channel
pub async fn client_event_handler(
    mut event_stream: impl futures::Stream<Item = ButtplugClientEvent> + std::marker::Unpin,
    vibecheck_state_pointer: Arc<Mutex<VibeCheckState>>,
    identifier: String,
    app_handle: AppHandle,
    tme_send: UnboundedSender<ToyManagementEvent>,
    _error_tx: UnboundedSender<VCError>,
) {
    // Listen for toys and add them if it connects send add update
    // If a toy disconnects send remove update

    trace!("BP Client Event Handler Handling Events..");
    loop {
        if let Some(event) = event_stream.next().await {
            match event {
                ButtplugClientEvent::DeviceAdded(dev) => {
                    info!("DeviceAdded");
                    Delay::new(Duration::from_secs(3)).await;

                    // Can use this to differ between toys with batteries and toys without!
                    let toy_power = if dev.has_battery_level() {
                        match dev.battery_level().await {
                            Ok(battery_lvl) => ToyPower::Battery(battery_lvl),
                            Err(_e) => {
                                warn!("Device battery_level() error: {:?}", _e);
                                ToyPower::Pending
                            }
                        }
                    } else {
                        ToyPower::NoBattery
                    };

                    let sub_id = {
                        let vc_lock = vibecheck_state_pointer.lock();
                        let mut toy_dup_count = 0;
                        vc_lock
                            .core_toy_manager
                            .as_ref()
                            .unwrap()
                            .online_toys
                            .iter()
                            .for_each(|toy| {
                                if &toy.1.toy_name == dev.name() {
                                    toy_dup_count += 1;
                                }
                            });
                        toy_dup_count
                    };

                    // Load toy config for name of toy if it exists otherwise create the config for the toy name
                    let mut toy = VCToy {
                        toy_id: dev.index(),
                        toy_name: dev.name().clone(),
                        toy_power: toy_power.clone(),
                        toy_connected: dev.connected(),
                        toy_features: dev.message_attributes().clone(),
                        parsed_toy_features: VCToyFeatures::new(),
                        osc_data: false,
                        bt_update_rate: 20,
                        listening: false,
                        device_handle: dev.clone(),
                        config: None,
                        sub_id,
                        app_handle: app_handle.clone(),
                    };

                    // Load config with existing toy name
                    match toy.load_toy_config() {
                        Ok(()) => info!("Toy config loaded successfully."),
                        Err(e) => warn!("Toy config failed to load: {:?}", e),
                    }

                    if toy.config.is_none() {
                        // First time toy load
                        toy.populate_toy_config();
                        let mut vc_lock = vibecheck_state_pointer.lock();
                        vc_lock
                            .core_toy_manager
                            .as_mut()
                            .unwrap()
                            .populate_configs();
                    } else {
                        toy.populate_toy_config();
                    }

                    {
                        let mut vc_lock = vibecheck_state_pointer.lock();
                        vc_lock
                            .core_toy_manager
                            .as_mut()
                            .unwrap()
                            .online_toys
                            .insert(toy.toy_id, toy.clone());
                    }
                    trace!("Toy inserted into VibeCheckState toys");

                    tme_send
                        .send(ToyManagementEvent::Tu(ToyUpdate::AddToy(toy.clone())))
                        .unwrap();

                    emit_toy_event(
                        &app_handle,
                        FeToyEvent::Add({
                            FeVCToy {
                                toy_id: Some(toy.toy_id),
                                toy_name: toy.toy_name.clone(),
                                toy_anatomy: toy.config.as_ref().unwrap().anatomy.to_fe(),
                                toy_power,
                                toy_connected: toy.toy_connected,
                                features: toy.parsed_toy_features.features.to_frontend(),
                                listening: toy.listening,
                                osc_data: toy.osc_data,
                                bt_update_rate: toy.bt_update_rate,
                                sub_id: toy.sub_id,
                            }
                        }),
                    );

                    {
                        let vc_lock = vibecheck_state_pointer.lock();
                        if vc_lock.config.desktop_notifications {
                            app_handle.notification().builder()
                                .title("Toy Connected")
                                .body(
                                    format!("{} ({})", toy.toy_name, toy.toy_power.to_string())
                                        .as_str(),
                                )
                                .show();
                        }
                    }

                    info!("Toy Connected: {} | {}", toy.toy_name, toy.toy_id);
                }
                ButtplugClientEvent::DeviceRemoved(dev) => {
                    // Get scan on disconnect and toy
                    let (sod, toy) = {
                        let mut vc_lock = vibecheck_state_pointer.lock();
                        (
                            vc_lock.config.scan_on_disconnect,
                            vc_lock
                                .core_toy_manager
                                .as_mut()
                                .unwrap()
                                .online_toys
                                .remove(&dev.index()),
                        )
                    };

                    // Check if toy is valid
                    if let Some(toy) = toy {
                        trace!("Removed toy from VibeCheckState toys");
                        tme_send
                            .send(ToyManagementEvent::Tu(ToyUpdate::RemoveToy(dev.index())))
                            .unwrap();

                        emit_toy_event(&app_handle, FeToyEvent::Remove(dev.index()));

                        {
                            let vc_lock = vibecheck_state_pointer.lock();
                            if vc_lock.config.desktop_notifications {
                                app_handle.notification().builder()
                                    .title("Toy Disconnected")
                                    .body(toy.toy_name.to_string())
                                    .show();
                            }
                        }

                        if sod {
                            info!("Scan on disconnect is enabled.. Starting scan.");
                            let vc_lock = vibecheck_state_pointer.lock();
                            if vc_lock.bp_client.is_some() && vc_lock.config.scan_on_disconnect {
                                vc_lock
                                    .async_rt
                                    .spawn(vc_lock.bp_client.as_ref().unwrap().start_scanning());
                            }

                            emit_core_event(&app_handle, FeCoreEvent::Scan(FeScanEvent::Start));
                        }
                    }
                }
                ButtplugClientEvent::ScanningFinished => info!("Scanning finished!"),
                ButtplugClientEvent::ServerDisconnect => {
                    warn!("ServerDisconnect");
                    break;
                }
                ButtplugClientEvent::PingTimeout => {
                    warn!("PingTimeout");
                    break;
                }
                ButtplugClientEvent::Error(e) => {
                    logerr!("Client Event Error: {:?}", e);
                }
                ButtplugClientEvent::ServerConnect => {
                    info!("Server Connect");
                }
            }
        } else {
            warn!("GOT NONE IN EVENT HANDLER: THIS SHOULD NEVER HAPPEN LOL");
        }
    }
    info!("Event handler returning!");
}
