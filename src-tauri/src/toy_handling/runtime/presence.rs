/*
 * Connection events: VRChat and toys connecting and disconnecting, published
 * on the session's event plane as `Connection` events (value: true or false).
 *
 * - VRChat: the avatar catalog tells whether VRChat is running (see
 *   osc/avatar_catalog.rs). Each session publishes "connected" when it
 *   starts if VRChat is already running, then every change. The source is
 *   VRCHAT_MODULE's `connection` port.
 *
 * The core event handler reacts to VRChat disconnecting (see osc_routing.rs).
 */

use tokio::sync::watch;
use vibecheck_core::{EventKind, EventPlane, ModuleInstanceId, PortId, Value};

/// The module VRChat's connection events come from.
pub const VRCHAT_MODULE: &str = "vrchat";

/// The port connection events are published on.
pub const CONNECTION_PORT: &str = "connection";

/// Publishes a connection event from `module`.
pub fn publish_connection(events: &EventPlane, module: ModuleInstanceId, connected: bool) {
    events.publish(
        PortId::new(module, CONNECTION_PORT),
        EventKind::Connection,
        Value::Bool(connected),
    );
}

/// Publishes VRChat's connection events for one session, until it ends.
pub async fn vrchat_presence(mut present: watch::Receiver<bool>, events: EventPlane) {
    let vrchat = ModuleInstanceId::new(VRCHAT_MODULE);
    let mut last = *present.borrow_and_update();
    if last {
        publish_connection(&events, vrchat.clone(), true);
    }
    while present.changed().await.is_ok() {
        let now = *present.borrow_and_update();
        if now != last {
            last = now;
            publish_connection(&events, vrchat.clone(), now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publishes_vrchat_connecting_and_disconnecting() {
        let (present, rx) = watch::channel(true);
        let (events, mut received) = EventPlane::new();
        let task = tokio::spawn(vrchat_presence(rx, events));

        let next = |e: vibecheck_core::Event| (e.source.to_string(), e.kind, e.value);
        let expected = |connected| {
            (
                "vrchat/connection".to_owned(),
                EventKind::Connection,
                Value::Bool(connected),
            )
        };
        // Already running when the session starts
        assert_eq!(next(received.recv().await.unwrap()), expected(true));
        present.send(false).unwrap();
        assert_eq!(next(received.recv().await.unwrap()), expected(false));
        present.send(true).unwrap();
        assert_eq!(next(received.recv().await.unwrap()), expected(true));

        drop(present);
        task.await.unwrap();
    }
}
