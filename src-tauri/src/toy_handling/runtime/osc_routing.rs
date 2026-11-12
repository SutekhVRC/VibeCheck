/*
 * Routes OSC messages from OscInput to the toy tasks that need them.
 *
 * Plain parameters don't need routing: OscInput writes them to the signal
 * plane and each toy's graph reads them from there. Toy tasks only need the
 * messages their SPS/TPS input processors handle, which are message-driven.
 * A toy gets a message if one of its features has an input processor that
 * claims the address. This is deliberately a superset: the toy task still
 * checks the feature is enabled and applies input filters.
 *
 * VibeCheck OSC API messages are handled here and never reach toys.
 */

use std::{collections::HashMap, sync::Arc};

use log::{debug, info, warn};
use parking_lot::RwLock;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use vibecheck_core::{
    modules::osc::rosc::OscMessage, EventKind, EventReceiver, ModuleInstanceId, SignalPlane, Value,
};

use crate::{
    osc_api::handle_api_message,
    toy_handling::{
        input_processor::InputProcessor, runtime::toy_graph::OSC_MODULE, toyops::VCToyFeatures,
    },
    vcore::state::VCStateMutex,
};

const API_PREFIX: &str = "/avatar/parameters/vibecheck/api/";

/// Which OSC addresses a toy's input processors (SPS/TPS) care about.
#[derive(Default)]
pub struct ToyInterest {
    processors: Vec<Box<dyn InputProcessor>>,
}

impl ToyInterest {
    /// Built from every feature, enabled or not, so that enabling a feature
    /// takes effect without waiting for the routes to change.
    pub fn from_features(features: &VCToyFeatures) -> Self {
        let mut interest = Self::default();
        for feature in &features.features {
            if let Some(processor) = &feature.penetration_system.pen_system {
                interest.processors.push(processor.clone());
            }
        }
        interest
    }

    pub fn wants(&self, addr: &str) -> bool {
        self.processors.iter().any(|p| p.is_parameter(addr))
    }
}

struct ToyRoute {
    tx: UnboundedSender<OscMessage>,
    interest: ToyInterest,
}

/// The routing table, shared between the toy management handler (which
/// edits it) and the router task (which reads it).
#[derive(Clone, Default)]
pub struct OscRoutes(Arc<RwLock<HashMap<u32, ToyRoute>>>);

impl OscRoutes {
    pub fn insert(&self, toy_id: u32, tx: UnboundedSender<OscMessage>, features: &VCToyFeatures) {
        self.0.write().insert(
            toy_id,
            ToyRoute {
                tx,
                interest: ToyInterest::from_features(features),
            },
        );
    }

    /// Call when a toy's features change.
    pub fn update(&self, toy_id: u32, features: &VCToyFeatures) {
        if let Some(route) = self.0.write().get_mut(&toy_id) {
            route.interest = ToyInterest::from_features(features);
        }
    }

    pub fn remove(&self, toy_id: u32) {
        self.0.write().remove(&toy_id);
    }

    pub fn clear(&self) {
        self.0.write().clear();
    }

    /// Send `msg` to every toy that wants it. Returns how many got it.
    pub fn route(&self, msg: &OscMessage) -> usize {
        let routes = self.0.read();
        routes
            .values()
            .filter(|route| route.interest.wants(&msg.addr))
            .filter(|route| route.tx.send(msg.clone()).is_ok())
            .count()
    }
}

/// Reads OscInput's tap until it closes: handles OSC API messages and routes
/// everything else to toys.
pub async fn osc_router(
    mut rx: UnboundedReceiver<OscMessage>,
    routes: OscRoutes,
    app_handle: AppHandle,
) {
    while let Some(msg) = rx.recv().await {
        if msg.addr.starts_with(API_PREFIX) {
            // The API handler locks the global state and may save config, so
            // keep it off the async workers while staying in message order.
            tokio::task::block_in_place(|| handle_api_message(msg, &app_handle));
        } else {
            routes.route(&msg);
        }
    }
    info!("OSC router stopped");
}

/// Reacts to events from the event plane.
pub async fn core_event_handler(
    mut events: EventReceiver,
    signals: SignalPlane,
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
                    let state = app_handle.state::<VCStateMutex>();
                    let safety = state.0.lock().safety.clone();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toy_handling::{
        input_processor::penetration_systems::{sps::SPSProcessor, PenetrationSystemType},
        toyops::{ProcessingMode, ToyParameter, VCFeatureType, VCToyFeature},
    };
    use tokio::sync::mpsc::unbounded_channel;
    use vibecheck_core::modules::osc::rosc::OscType;

    fn features(params: &[&str], sps: bool) -> VCToyFeatures {
        let mut feature = VCToyFeature::new(
            params
                .iter()
                .map(|p| ToyParameter {
                    parameter: p.to_string(),
                    processing_mode: ProcessingMode::Raw,
                })
                .collect(),
            0,
            VCFeatureType::Vibrator,
        );
        if sps {
            feature.penetration_system.pen_system_type = PenetrationSystemType::Sps;
            feature.penetration_system.pen_system = Some(Box::<SPSProcessor>::default());
        }
        VCToyFeatures {
            features: vec![feature],
        }
    }

    fn msg(addr: &str) -> OscMessage {
        OscMessage {
            addr: addr.into(),
            args: vec![OscType::Float(0.5)],
        }
    }

    #[test]
    fn routes_input_processor_messages_only_to_toys_that_use_them() {
        let routes = OscRoutes::default();
        let (boop_tx, mut boop_rx) = unbounded_channel();
        let (sps_tx, mut sps_rx) = unbounded_channel();
        routes.insert(1, boop_tx, &features(&["/avatar/parameters/Boop"], false));
        routes.insert(2, sps_tx, &features(&[], true));

        // Plain parameters reach graphs through the signal plane, not toy tasks
        assert_eq!(routes.route(&msg("/avatar/parameters/Boop")), 0);
        assert_eq!(
            routes.route(&msg("/avatar/parameters/OGB/Orf/Mouth/PenSelf")),
            1
        );
        assert_eq!(routes.route(&msg("/avatar/parameters/Unused")), 0);

        assert!(boop_rx.try_recv().is_err());
        assert!(sps_rx.try_recv().unwrap().addr.contains("/OGB/"));
        assert!(sps_rx.try_recv().is_err());
    }

    #[test]
    fn follows_feature_changes_and_removal() {
        let routes = OscRoutes::default();
        let (tx, mut rx) = unbounded_channel();
        routes.insert(1, tx, &features(&[], false));
        assert_eq!(routes.route(&msg("/avatar/parameters/OGB/x")), 0);

        routes.update(1, &features(&[], true));
        assert_eq!(routes.route(&msg("/avatar/parameters/OGB/x")), 1);
        assert_eq!(rx.try_recv().unwrap().addr, "/avatar/parameters/OGB/x");

        routes.remove(1);
        assert_eq!(routes.route(&msg("/avatar/parameters/OGB/x")), 0);
    }
}
