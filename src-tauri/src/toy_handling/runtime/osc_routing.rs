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
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use vibecheck_core::{
    modules::osc::rosc::OscMessage, Diagnostics, Event, EventKind, EventReceiver, ModuleInstanceId,
    SafetyLayer, SignalPlane, Value,
};

use crate::{
    osc::{avatar_catalog::AvatarCatalog, avatar_query::AvatarQuery},
    osc_api::handle_api_message,
    toy_handling::runtime::{
        graph_runner::GraphControl, presence::VRCHAT_MODULE, toy_graph::OSC_MODULE,
    },
    vcore::{
        config::toy_location::ToySettingsLocations, ipc::call_plane::native_reload_toy_settings,
        state::VCStateMutex,
    },
};

const API_PREFIX: &str = "/avatar/parameters/vibecheck/api/";

/// The module diagnostics log entries about events are logged under.
pub const EVENTS_LOG_MODULE: &str = "events";

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

/// What the diagnostics log shows for an event, if anything. Avatar changes
/// are left out: the OSC message itself is already logged.
fn describe(event: &Event) -> Option<String> {
    let on_off = |on: &str, off: &str| match event.value {
        Value::Bool(true) => Some(on.to_owned()),
        Value::Bool(false) => Some(off.to_owned()),
        _ => None,
    };
    match (&event.kind, &event.value) {
        (EventKind::Connection, _) => on_off("connected", "disconnected"),
        (EventKind::Activity, _) => on_off("started", "stopped"),
        (EventKind::OutputError, Value::Text(error)) => Some(format!("failing: {error}")),
        (EventKind::OutputError, _) => Some("working again".to_owned()),
        _ => None,
    }
}

/// Reacts to events from the event plane. While diagnostics are on, they're
/// also logged under `events` (see `describe`).
///
/// When VRChat disconnects (crashed or closed), OSC just stops arriving, so
/// toys would keep their last level. Instead everything stops as on an
/// avatar change: the old values are cleared, toys stop, graphs start over,
/// and risky outputs are disarmed. When VRChat is back, current values are
/// read from OSCQuery again.
pub async fn core_event_handler(
    mut events: EventReceiver,
    avatar_query: AvatarQuery,
    signals: SignalPlane,
    graphs: UnboundedSender<GraphControl>,
    safety: SafetyLayer,
    diagnostics: Diagnostics,
    app_handle: AppHandle,
) {
    let log_module = ModuleInstanceId::new(EVENTS_LOG_MODULE);
    while let Some(event) = events.recv().await {
        debug!("Core event: {:?}", event);
        if let Some(text) = describe(&event) {
            diagnostics.log(&log_module, event.source.module.to_string(), || text);
        }
        match event.kind {
            EventKind::AvatarChange => {
                info!("Avatar Changed: Halting toy actions");
                // Fill in parameters VRChat won't send (e.g. SPS plug lengths)
                let query = avatar_query.clone();
                tokio::spawn(async move { query.fill_after_avatar_change().await });
                stop_all_devices(&app_handle).await;
                // Toys' settings may be kept per avatar: take the new
                // avatar's before the graphs start over, so its values never
                // run through the old avatar's settings
                if let Value::Text(id) = &event.value {
                    let new_avatar = app_handle
                        .try_state::<AvatarCatalog>()
                        .is_some_and(|catalog| catalog.note_avatar(id));
                    if new_avatar && ToySettingsLocations::given(&app_handle) {
                        native_reload_toy_settings(&app_handle);
                    }
                }
                // OscInput has already cleared the old avatar's values, in
                // message order. Start every graph over, so no processor
                // (Smooth, Rate, SPS zones and lengths) carries old state
                // across. Values the new avatar already sent are given to
                // the graphs again after the reset, so this comes after the
                // stop or the stop would silence them.
                let _ = graphs.send(GraphControl::ResetAll);
            }
            EventKind::Connection if event.source.module.as_str() == VRCHAT_MODULE => {
                if event.value == Value::Bool(false) {
                    warn!("VRChat stopped answering: stopping toys");
                    // No OSC is arriving, so nothing races this clear
                    signals.clear_module(&ModuleInstanceId::new(OSC_MODULE));
                    stop_all_devices(&app_handle).await;
                    let _ = graphs.send(GraphControl::ResetAll);
                    safety.disarm_all();
                } else {
                    info!("VRChat is running");
                    let query = avatar_query.clone();
                    tokio::spawn(async move { query.fill_now().await });
                }
            }
            // Anything else disconnecting must be re-armed on purpose once
            // it's back (no-op for toys)
            EventKind::Connection if event.value == Value::Bool(false) => {
                if safety.is_armed(&event.source.module) {
                    safety.disarm(&event.source.module);
                    warn!("Disarmed {} because it disconnected", event.source.module);
                }
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
                warn!("Failed to stop devices: {}", e);
            }
        }
        None => warn!("No Buttplug client to stop devices"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibecheck_core::{EventId, PortId};

    fn event(kind: EventKind, value: Value) -> Event {
        Event {
            id: EventId(1),
            source: PortId::new(ModuleInstanceId::new("toy:lush:0"), "activity"),
            kind,
            value,
            ts_ms: 0,
            cause: None,
            depth: 0,
        }
    }

    #[test]
    fn describes_the_events_the_diagnostics_log_shows() {
        let cases = [
            (EventKind::Connection, Value::Bool(true), Some("connected")),
            (
                EventKind::Connection,
                Value::Bool(false),
                Some("disconnected"),
            ),
            (EventKind::Activity, Value::Bool(true), Some("started")),
            (EventKind::Activity, Value::Bool(false), Some("stopped")),
            (
                EventKind::OutputError,
                Value::Text("gone".into()),
                Some("failing: gone"),
            ),
            (EventKind::OutputError, Value::None, Some("working again")),
            (EventKind::AvatarChange, Value::Text("avtr_1".into()), None),
        ];
        for (kind, value, expected) in cases {
            assert_eq!(
                describe(&event(kind.clone(), value)).as_deref(),
                expected,
                "{kind:?}"
            );
        }
    }
}
