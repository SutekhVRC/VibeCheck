/*
 * Control: what the window shows and does, for code outside the window (an
 * app built on VibeCheck, see extensions.rs). It reads the app's state as a
 * snapshot and changes it through the same paths as the window's buttons
 * and the tray, so nothing behaves differently, and the window follows
 * every change:
 *
 * - Turning on and off goes through the frontend, exactly like the tray's
 *   menu (FeStateEvent), so scanning stops first and the window's switch
 *   stays right. Starting a scan does too; stopping one is what the scan
 *   length's timer does (vcore/scan.rs), then the window is told.
 * - Panic and resume are the same functions as the panic button, hotkey and
 *   OSC API parameter.
 * - Master intensity is saved like Settings saves it, and the window is
 *   told to reload the settings.
 *
 * Nothing here blocks for long: reading takes the state lock briefly, and
 * the changes either take it briefly or hand the work to the frontend.
 * Call it from any thread, not from inside an async task that holds the
 * state lock.
 */

use log::{info, warn};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

use crate::{
    frontend::frontend_types::{FeConfigEvent, FeCoreEvent, FeScanEvent, FeStateEvent},
    toy_handling::ToyPower,
    util::toy_service::ToyServiceStatus,
    vcore::{
        ipc::{
            call_plane::{native_reload_toy_settings, save_config},
            emit_plane::emit_core_event,
        },
        panic, scan,
        state::{RunningState, VCStateMutex},
    },
};

/// A handle on the app's state and its main actions. Cheap to clone.
#[derive(Clone)]
pub struct Control {
    app: AppHandle,
}

/// The app's state, as the window shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlState {
    /// VibeCheck is on (listening to VRChat).
    pub on: bool,
    /// Panic stop is active: every toy stopped until resumed.
    pub panicked: bool,
    /// Master intensity, 0.0 to 1.0.
    pub master_intensity: f64,
    /// Looking for toys (a scan is running).
    pub scanning: bool,
    /// Toys connected now, by name.
    pub toys: Vec<ToyState>,
    /// VRChat is running (found over OSCQuery).
    pub vrchat_found: bool,
    /// How the connection to the toy service (built in or Intiface) is doing.
    pub toy_service: ToyService,
    /// An opaque, stable key for the avatar VRChat loaded last (a hash of its
    /// id, never the id itself), once one is known; kept while VRChat is
    /// gone. The same avatar always gives the same key, so settings can be
    /// kept per avatar.
    pub avatar_key: Option<String>,
    /// That avatar's name, once VRChat has written its OSC config (it can
    /// take a moment after loading). For showing in the app; don't log it.
    pub avatar_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToyState {
    pub name: String,
    /// Battery, 0.0 to 1.0, once read (None for toys without one).
    pub battery: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToyService {
    Connecting,
    Connected,
    /// Not connected, and why; it tries again every few seconds.
    Failed(String),
}

impl Control {
    /// A handle for the app `app` runs (call once VibeCheck has started,
    /// e.g. from `Extensions::on_start`).
    pub fn new(app: &AppHandle) -> Self {
        Self { app: app.clone() }
    }

    /// The app's state now.
    pub fn state(&self) -> ControlState {
        let state = self.app.state::<VCStateMutex>();
        let vc = state.0.lock();
        // Looked up after the lock is let go: it may read a file
        let catalog = vc.avatar_catalog.clone();
        let mut toys: Vec<ToyState> = vc
            .toy_manager()
            .online_toys
            .values()
            .filter(|toy| toy.toy_connected)
            .map(|toy| ToyState {
                name: toy.toy_name.clone(),
                battery: match toy.toy_power {
                    ToyPower::Battery(level) => Some(level),
                    _ => None,
                },
            })
            .collect();
        toys.sort_by(|a, b| a.name.cmp(&b.name));
        let toy_service = match &*vc.toy_service_status.lock() {
            ToyServiceStatus::Connecting => ToyService::Connecting,
            ToyServiceStatus::Connected => ToyService::Connected,
            ToyServiceStatus::Failed(why) => ToyService::Failed(why.clone()),
        };
        let mut snapshot = ControlState {
            on: matches!(vc.running, RunningState::Running),
            panicked: vc.safety.is_panicked(),
            master_intensity: vc.config.master_intensity,
            scanning: scan::is_scanning(),
            toys,
            vrchat_found: *vc.avatar_catalog.presence().borrow(),
            toy_service,
            avatar_key: vc.avatar_catalog.last_avatar().as_deref().map(avatar_key),
            avatar_name: None,
        };
        drop(vc);
        snapshot.avatar_name = catalog.last_avatar_name();
        snapshot
    }

    /// Turns VibeCheck on, as the tray's menu does (through the window, so
    /// its switch follows; if turning on fails, the window says why).
    pub fn turn_on(&self) {
        info!("Turning on (control)");
        emit_core_event(&self.app, FeCoreEvent::State(FeStateEvent::Enable));
    }

    /// Turns VibeCheck off, as the tray's menu does (scanning stops first).
    pub fn turn_off(&self) {
        info!("Turning off (control)");
        emit_core_event(&self.app, FeCoreEvent::State(FeStateEvent::Disable));
    }

    /// Looks for toys, as the window's scan button does: while VibeCheck is
    /// on, or turning it on first if "Turn on when scanning" is set. The
    /// scan stops by itself after the scan length (or with
    /// [`stop_scan`](Self::stop_scan)).
    pub fn scan(&self) -> Result<(), String> {
        let (on, turn_on) = {
            let state = self.app.state::<VCStateMutex>();
            let vc = state.0.lock();
            (
                matches!(vc.running, RunningState::Running),
                vc.config.auto_enable_on_scan,
            )
        };
        let event = match (on, turn_on) {
            (true, _) => FeStateEvent::Scan,
            (false, true) => FeStateEvent::EnableAndScan,
            (false, false) => return Err("VibeCheck is off".into()),
        };
        info!("Scanning (control)");
        emit_core_event(&self.app, FeCoreEvent::State(event));
        Ok(())
    }

    /// Stops a scan, as the window's stop button does; the window follows.
    pub fn stop_scan(&self) {
        info!("Stopping the scan (control)");
        let state = self.app.state::<VCStateMutex>();
        let vc = state.0.lock();
        let Some(client) = vc.bp_client.as_ref() else {
            return;
        };
        let stop = client.stop_scanning();
        let app = self.app.clone();
        vc.async_rt.spawn(async move {
            match stop.await {
                Ok(()) => {
                    scan::scan_stopped();
                    emit_core_event(&app, FeCoreEvent::Scan(FeScanEvent::Stop));
                }
                Err(e) => warn!("Couldn't stop the scan: {}", e),
            }
        });
    }

    /// Panic stop: every toy stops until [`resume`](Self::resume).
    pub fn panic(&self) {
        panic::trigger_panic(&self.app);
    }

    pub fn resume(&self) {
        panic::resume_from_panic(&self.app);
    }

    /// Loads every toy's settings again from where they're kept now (see
    /// `Extensions::toy_settings_location`), e.g. after moving them. Toys
    /// that are on use them straight away and the window shows them. An
    /// avatar change does this by itself.
    pub fn reload_toy_settings(&self) {
        info!("Reloading toy settings (control)");
        native_reload_toy_settings(&self.app);
    }

    /// Sets and saves the master intensity (clamped to 0.0 to 1.0). It
    /// applies straight away, also to toys already running.
    pub fn set_master_intensity(&self, value: f64) -> Result<(), String> {
        let value = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            return Err("not a number".into());
        };
        let config = {
            let state = self.app.state::<VCStateMutex>();
            let mut vc = state.0.lock();
            vc.config.master_intensity = value;
            vc.safety.set_intensity(value);
            vc.config.clone()
        };
        let saved = save_config(config, &self.app).map_err(|e| format!("{e:?}"));
        // The window keeps its own copy of the settings
        emit_core_event(&self.app, FeCoreEvent::Config(FeConfigEvent::Changed));
        saved
    }
}

/// The opaque key for an avatar id: a SHA-256 of it (with a fixed prefix,
/// so it's VibeCheck's own), 32 hex digits. Stable across runs and
/// versions.
pub(crate) fn avatar_key(id: &str) -> String {
    let digest = Sha256::digest(format!("VibeCheck avatar key\0{id}").as_bytes());
    digest[..16].iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::avatar_key;

    #[test]
    fn avatar_keys_are_stable_opaque_and_distinct() {
        let id = "avtr_00000000-1111-2222-3333-444444444444";
        let key = avatar_key(id);
        assert_eq!(key, avatar_key(id));
        assert_eq!(key.len(), 32);
        assert!(!key.contains("avtr"));
        assert_ne!(key, avatar_key("avtr_other"));
    }
}
