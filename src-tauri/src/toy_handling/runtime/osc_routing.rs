/*
 * Handles the OSC messages that aren't avatar parameter values, and events
 * from the event plane.
 *
 * Parameter values don't need routing: OscInput writes them to the signal
 * plane, and each toy's graph reads the ones it uses from there (including
 * input processors, which watch their address prefixes). The router only
 * handles VibeCheck OSC API messages.
 */

use log::{debug, info, warn};
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::UnboundedReceiver;
use vibecheck_core::{
    modules::osc::rosc::OscMessage, EventKind, EventReceiver, ModuleInstanceId, SafetyLayer,
    SignalPlane, Value,
};

use crate::{
    osc_api::handle_api_message, toy_handling::runtime::toy_graph::OSC_MODULE,
    vcore::state::VCStateMutex,
};

const API_PREFIX: &str = "/avatar/parameters/vibecheck/api/";

/// Reads OscInput's tap until it closes and handles OSC API messages, in
/// the order they arrived.
pub async fn osc_router(mut rx: UnboundedReceiver<OscMessage>, app_handle: AppHandle) {
    while let Some(msg) = rx.recv().await {
        if msg.addr.starts_with(API_PREFIX) {
            // The API handler locks the global state and may save config, so
            // keep it off the async workers while staying in message order.
            tokio::task::block_in_place(|| handle_api_message(msg, &app_handle));
        }
    }
    info!("OSC router stopped");
}

/// Reacts to events from the event plane.
pub async fn core_event_handler(
    mut events: EventReceiver,
    signals: SignalPlane,
    safety: SafetyLayer,
    app_handle: AppHandle,
) {
    while let Some(event) = events.recv().await {
        debug!("Core event: {:?}", event);
        match event.kind {
            EventKind::AvatarChange => {
                info!("Avatar Changed: Halting toy actions");
                // Forget the old avatar's parameter values, so a graph rebuilt
                // later doesn't replay them
                signals.clear_module(&ModuleInstanceId::new(OSC_MODULE));
                stop_all_devices(&app_handle).await;
            }
            EventKind::OutputError => match &event.value {
                Value::Text(error) => {
                    warn!("{} is failing: {}", event.source.module, error);
                    // A risky output that stops responding must be re-armed
                    // on purpose once it's back (no-op for toys)
                    if safety.is_armed(&event.source.module) {
                        safety.disarm(&event.source.module);
                        warn!(
                            "Disarmed {} because it stopped responding",
                            event.source.module
                        );
                    }
                }
                _ => info!("{} is working again", event.source.module),
            },
            _ => {}
        }
    }
}

/// Stops every device without holding the global state lock while waiting
/// for Buttplug.
async fn stop_all_devices(app_handle: &AppHandle) {
    let stop = {
        let state = app_handle.state::<VCStateMutex>();
        let vc_lock = state.0.lock();
        vc_lock.bp_client.as_ref().map(|c| c.stop_all_devices())
    };
    match stop {
        Some(stop) => {
            if let Err(e) = stop.await {
                warn!("Failed to stop devices on avatar change: {}", e);
            }
        }
        None => warn!("Avatar changed but there is no Buttplug client to stop devices"),
    }
}
