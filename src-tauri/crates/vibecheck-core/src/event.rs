//! The event plane: discrete happenings, queued in order, with causality.
//!
//! Every event records the event that caused it (`cause`) and how long the
//! chain is so far (`depth`). The safety layer uses this to refuse feedback
//! loops, such as a rule reacting to its own output.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc;

use crate::address::PortId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    AvatarChange,
    Connection,
    Pattern,
    Threshold,
    RuleFired,
    Arming,
    Panic,
    /// Module-specific events that don't fit the kinds above.
    Custom(Arc<str>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    Float(f64),
    Bool(bool),
    Text(Arc<str>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub id: EventId,
    pub source: PortId,
    pub kind: EventKind,
    pub value: Value,
    pub ts_ms: u64,
    /// The event that led to this one, if any.
    pub cause: Option<EventId>,
    /// How many events came before this one in its chain. Root events are 0.
    pub depth: u8,
}

/// Sends events onto the plane. Cheap to clone; every module gets one.
#[derive(Clone)]
pub struct EventPlane {
    tx: mpsc::UnboundedSender<Event>,
    next_id: Arc<AtomicU64>,
}

/// The single consumer of the event plane (the rule engine / safety layer host).
pub struct EventReceiver {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventPlane {
    pub fn new() -> (Self, EventReceiver) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                tx,
                next_id: Arc::new(AtomicU64::new(1)),
            },
            EventReceiver { rx },
        )
    }

    /// Publish a root event, one not caused by any other event.
    pub fn publish(&self, source: PortId, kind: EventKind, value: Value) -> EventId {
        self.send(source, kind, value, None, 0)
    }

    /// Publish an event caused by `cause`. Its depth is one more than the cause's.
    pub fn publish_caused_by(
        &self,
        cause: &Event,
        source: PortId,
        kind: EventKind,
        value: Value,
    ) -> EventId {
        let depth = cause.depth.saturating_add(1);
        self.send(source, kind, value, Some(cause.id), depth)
    }

    fn send(
        &self,
        source: PortId,
        kind: EventKind,
        value: Value,
        cause: Option<EventId>,
        depth: u8,
    ) -> EventId {
        let id = EventId(self.next_id.fetch_add(1, Ordering::Relaxed));
        // A send only fails once the receiver is gone, i.e. during shutdown,
        // when there is nobody left to act on the event anyway.
        let _ = self.tx.send(Event {
            id,
            source,
            kind,
            value,
            ts_ms: now_ms(),
            cause,
            depth,
        });
        id
    }
}

impl EventReceiver {
    /// The next event, in the order events were published. `None` once every
    /// [`EventPlane`] handle has been dropped.
    pub async fn recv(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    /// The next event if one is already waiting.
    pub fn try_recv(&mut self) -> Option<Event> {
        self.rx.try_recv().ok()
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::ModuleInstanceId;

    fn port(module: &str, name: &str) -> PortId {
        PortId::new(ModuleInstanceId::new(module), name)
    }

    #[tokio::test]
    async fn events_arrive_in_order_with_causality() {
        let (plane, mut rx) = EventPlane::new();

        plane.publish(
            port("toy:lovense_lush:0", "pattern"),
            EventKind::Pattern,
            Value::Text("ramp_up".into()),
        );
        let root = rx.recv().await.unwrap();
        assert_eq!(root.cause, None);
        assert_eq!(root.depth, 0);

        plane.publish_caused_by(
            &root,
            port("output:0", "acted"),
            EventKind::Custom("output_acted".into()),
            Value::None,
        );
        let child = rx.recv().await.unwrap();
        assert_eq!(child.cause, Some(root.id));
        assert_eq!(child.depth, 1);
        assert!(child.id > root.id);

        assert!(rx.try_recv().is_none());
    }
}
