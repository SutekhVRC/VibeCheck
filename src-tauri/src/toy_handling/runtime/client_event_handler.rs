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
        scan,
        state::VibeCheckState,
    },
};
use buttplug_client::{ButtplugClientDevice, ButtplugClientEvent};
use futures::StreamExt;
use futures_timer::Delay;
use log::{error as logerr, info, trace, warn};
use parking_lot::Mutex;
use std::sync::mpsc::Sender;
use tauri::AppHandle;
use tauri_plugin_notification::{Notification, NotificationExt};
use tokio::sync::mpsc::UnboundedSender;
use vibecheck_core::modules::buttplug::LegacyLayout;

/// How long a new toy gets to settle before its battery is read
const BATTERY_SETTLE: Duration = Duration::from_secs(3);

/// The toy as the frontend sees it
fn fe_toy(toy: &VCToy) -> FeVCToy {
    FeVCToy {
        toy_id: Some(toy.toy_id),
        toy_name: toy.toy_name.clone(),
        toy_anatomy: toy.config.as_ref().unwrap().anatomy.to_fe(),
        toy_power: toy.toy_power.clone(),
        toy_connected: toy.toy_connected,
        features: toy.parsed_toy_features.features.to_frontend(),
        listening: toy.listening,
        osc_data: toy.osc_data,
        bt_update_rate: toy.bt_update_rate,
        sub_id: toy.sub_id,
    }
}

/// Reads a new toy's battery once it has settled, then updates the toy in
/// the state and the frontend, and shows the "Toy Connected" notification.
/// Does nothing if the toy disconnected in the meantime.
async fn read_battery_later(
    state: Arc<Mutex<VibeCheckState>>,
    device: Arc<ButtplugClientDevice>,
    app_handle: AppHandle,
) {
    Delay::new(BATTERY_SETTLE).await;
    let power = ToyPower::read(&device).await;

    let (toy, notify) = {
        let mut vc_lock = state.lock();
        let notify = vc_lock.config.desktop_notifications;
        let toy = vc_lock
            .core_toy_manager
            .as_mut()
            .and_then(|manager| manager.online_toys.get_mut(&device.index()))
            // The same connection, not a toy that reconnected with this index
            .filter(|toy| Arc::ptr_eq(&toy.device_handle, &device));
        let Some(toy) = toy else {
            return;
        };
        toy.toy_power = power;
        (toy.clone(), notify)
    };

    emit_toy_event(&app_handle, FeToyEvent::Update(fe_toy(&toy)));

    if notify {
        app_handle
            .notification()
            .builder()
            .title("Toy Connected")
            .body(format!("{} ({})", toy.toy_name, toy.toy_power.to_string()).as_str())
            .show();
    }
}

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
                    // Shown straight away, even mid-scan. The battery is read
                    // in the background once the toy has settled.
                    let toy_power = ToyPower::unread(&dev);

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
                        layout: LegacyLayout::of(&dev),
                        parsed_toy_features: VCToyFeatures::new(),
                        osc_data: false,
                        bt_update_rate: 10,
                        listening: false,
                        device_handle: Arc::new(dev.clone()),
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

                    emit_toy_event(&app_handle, FeToyEvent::Add(fe_toy(&toy)));

                    let state = vibecheck_state_pointer.clone();
                    let device = toy.device_handle.clone();
                    let app = app_handle.clone();
                    tokio::spawn(async move { read_battery_later(state, device, app).await });

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
                                app_handle
                                    .notification()
                                    .builder()
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
                                scan::stop_after_scan_length(
                                    &vc_lock,
                                    vibecheck_state_pointer.clone(),
                                );
                            }

                            emit_core_event(&app_handle, FeCoreEvent::Scan(FeScanEvent::Start));
                        }
                    }
                }
                ButtplugClientEvent::ScanningFinished => info!("Scanning finished!"),
                // Devices already connected arrive as DeviceAdded events
                ButtplugClientEvent::DeviceListReceived => {}
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
