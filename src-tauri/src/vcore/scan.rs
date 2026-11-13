/*
 * Bluetooth scans stop themselves after the configured scan length
 * (`VibeCheckConfig::scan_length`, 0 for never).
 *
 * The backend owns the timer, so every scan is covered: the scan button and
 * scans started by "Scan on disconnect". When the timer stops a scan, the
 * frontend is told with a Scan(Stop) event.
 *
 * Every scan start and stop bumps a generation number. A timer only stops
 * the scan it was started for, so stopping and starting again doesn't get
 * the new scan cut short by the old timer.
 */

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use log::{info, warn};
use parking_lot::Mutex;

use crate::frontend::frontend_types::{FeCoreEvent, FeScanEvent};
use crate::vcore::ipc::emit_plane::emit_core_event;
use crate::vcore::state::VibeCheckState;

static SCAN_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Call when a scan stops for any other reason, so its timer does nothing.
pub fn scan_stopped() {
    SCAN_GENERATION.fetch_add(1, Ordering::SeqCst);
}

/// Call after a scan starts: stops it once the scan length has passed,
/// unless it was stopped or another scan started first. Uses the scan length
/// configured now.
pub fn stop_after_scan_length(vc_lock: &VibeCheckState, state: Arc<Mutex<VibeCheckState>>) {
    let generation = SCAN_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let seconds = vc_lock.config.scan_length;
    if seconds == 0 {
        return; // Scans until stopped
    }
    vc_lock.async_rt.spawn(async move {
        tokio::time::sleep(Duration::from_secs(seconds.into())).await;
        if SCAN_GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }
        let (stop, app_handle) = {
            let vc_lock = state.lock();
            (
                vc_lock.bp_client.as_ref().map(|c| c.stop_scanning()),
                vc_lock.app_handle.clone(),
            )
        };
        if let Some(stop) = stop {
            match stop.await {
                Ok(()) => info!("Scan stopped after {} s", seconds),
                Err(e) => warn!("Couldn't stop the scan after {} s: {}", seconds, e),
            }
        }
        if let Some(app_handle) = app_handle {
            emit_core_event(&app_handle, FeCoreEvent::Scan(FeScanEvent::Stop));
        }
    });
}
