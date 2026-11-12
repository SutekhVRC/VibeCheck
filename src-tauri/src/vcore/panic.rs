/*
 * Panic stop: one place for the Settings/sidebar button, the global hotkey
 * and the OSC API parameter, so they all behave the same.
 *
 * Panicking tells the app-wide safety layer to refuse every command and drop
 * any not yet sent (see vibecheck-core's SafetyLayer::panic), stops every
 * device, and tells the frontend. Toys stay stopped until resume, and after
 * resume each toy waits for its next new input rather than jumping back to
 * where it was.
 */

use log::{info, warn};
use tauri::{AppHandle, Manager};

use crate::{
    frontend::frontend_types::{FeCoreEvent, FeSafetyEvent},
    vcore::{ipc::emit_plane::emit_core_event, state::VCStateMutex},
};

/// The global hotkey that triggers a panic stop.
pub const PANIC_HOTKEY: &str = "Ctrl+Alt+P";

pub fn trigger_panic(app_handle: &AppHandle) {
    let (stop, runtime) = {
        let state = app_handle.state::<VCStateMutex>();
        let vc_lock = state.0.lock();
        vc_lock.safety.panic();
        (
            vc_lock.bp_client.as_ref().map(|c| c.stop_all_devices()),
            vc_lock.async_rt.handle().clone(),
        )
    };
    warn!("PANIC STOP: all toys stopped until resumed");

    // Wait for Buttplug without holding the state lock
    if let Some(stop) = stop {
        runtime.spawn(async move {
            if let Err(e) = stop.await {
                warn!("Panic stop: failed to stop devices: {}", e);
            }
        });
    }

    emit_core_event(app_handle, FeCoreEvent::Safety(FeSafetyEvent::Panicked));
}

pub fn resume_from_panic(app_handle: &AppHandle) {
    app_handle.state::<VCStateMutex>().0.lock().safety.resume();
    info!("Resumed after panic stop");
    emit_core_event(app_handle, FeCoreEvent::Safety(FeSafetyEvent::Resumed));
}

pub fn is_panicked(app_handle: &AppHandle) -> bool {
    app_handle
        .state::<VCStateMutex>()
        .0
        .lock()
        .safety
        .is_panicked()
}
