//! The event plane: discrete happenings, queued in order, with causality.
//!
//! Every event records the event that caused it (`cause`) and how long the
//! chain is so far (`depth`). The plane also remembers recent events, so a
//! chain can be traced back past its direct cause ([`EventPlane::chain`]).
//! The safety layer uses both to refuse feedback loops, such as a rule
//! reacting to the result of its own action.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use tokio::sync::mpsc;

use crate::address::{ModuleInstanceId, PortId};

/// How many recent events the plane remembers for tracing chains. Chains are
/// short (the safety layer caps their depth) and close together in time, so
/// this only needs to cover a burst of activity.
const HISTORY_LEN: usize = 4096;

/// Longest chain [`EventPlane::chain`] will walk, as a backstop.
const MAX_CHAIN_WALK: usize = 64;

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
    /// A module acted in response to an event (published by the safety
    /// layer when it admits the action, from the module that asked).
    Acted,
    /// An output started failing (value: the error as text) or recovered
    /// (value: `None`).
    OutputError,
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

#[derive(Clone)]
struct Remembered {
    source: PortId,
    cause: Option<EventId>,
    depth: u8,
}

#[derive(Default)]
struct History {
    order: VecDeque<EventId>,
    events: HashMap<EventId, Remembered>,
}

impl History {
    fn remember(&mut self, id: EventId, event: Remembered) {
        self.order.push_back(id);
        self.events.insert(id, event);
        while self.order.len() > HISTORY_LEN {
            if let Some(old) = self.order.pop_front() {
                self.events.remove(&old);
            }
        }
    }
}

/// Sends events onto the plane. Cheap to clone; every module gets one, and
/// clones share the same history.
#[derive(Clone)]
pub struct EventPlane {
    tx: mpsc::UnboundedSender<Event>,
    next_id: Arc<AtomicU64>,
    history: Arc<Mutex<History>>,
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
                history: Arc::default(),
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

    /// Publish an event caused by the event with id `cause`, e.g. an output
    /// reporting the result of a command (see `ClampedCmd::cause`). If the
    /// cause is too old to be remembered, its depth is taken as 0.
    pub fn publish_caused_by_id(
        &self,
        cause: EventId,
        source: PortId,
        kind: EventKind,
        value: Value,
    ) -> EventId {
        let cause_depth = self
            .history
            .lock()
            .events
            .get(&cause)
            .map_or(0, |e| e.depth);
        self.send(
            source,
            kind,
            value,
            Some(cause),
            cause_depth.saturating_add(1),
        )
    }

    /// The sources of `event` and every event before it in its chain, newest
    /// first, as far back as the plane remembers.
    pub fn chain(&self, event: EventId) -> Vec<PortId> {
        let history = self.history.lock();
        let mut sources = Vec::new();
        let mut next = Some(event);
        while let Some(id) = next {
            let Some(remembered) = history.events.get(&id) else {
                break;
            };
            sources.push(remembered.source.clone());
            if sources.len() >= MAX_CHAIN_WALK {
                break;
            }
            next = remembered.cause;
        }
        sources
    }

    /// Whether any event in `event`'s chain (including itself) came from `module`.
    pub fn chain_includes(&self, event: EventId, module: &ModuleInstanceId) -> bool {
        self.chain(event)
            .iter()
            .any(|source| source.module == *module)
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
        self.history.lock().remember(
            id,
            Remembered {
                source: source.clone(),
                cause,
                depth,
            },
        );
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

    #[test]
    fn chains_can_be_traced_back_past_the_direct_cause() {
        let (plane, _rx) = EventPlane::new();
        let toy = port("toy:lush:0", "pattern");
        let rule = port("rule:a", "fired");
        let output = port("output:0", "acted");

        let root = plane.publish(toy.clone(), EventKind::Pattern, Value::None);
        let fired =
            plane.publish_caused_by_id(root, rule.clone(), EventKind::RuleFired, Value::None);
        let acted = plane.publish_caused_by_id(
            fired,
            output.clone(),
            EventKind::Custom("x".into()),
            Value::None,
        );

        assert_eq!(plane.chain(acted), vec![output, rule, toy]);
        assert!(plane.chain_includes(acted, &ModuleInstanceId::new("rule:a")));
        assert!(!plane.chain_includes(root, &ModuleInstanceId::new("rule:a")));
    }

    #[test]
    fn depth_follows_the_cause_by_id() {
        let (plane, mut rx) = EventPlane::new();
        let root = plane.publish(port("a", "x"), EventKind::Pattern, Value::None);
        let child =
            plane.publish_caused_by_id(root, port("b", "x"), EventKind::Pattern, Value::None);
        plane.publish_caused_by_id(child, port("c", "x"), EventKind::Pattern, Value::None);
        let depths: Vec<u8> = std::iter::from_fn(|| rx.try_recv())
            .map(|e| e.depth)
            .collect();
        assert_eq!(depths, vec![0, 1, 2]);
    }

    #[test]
    fn history_is_bounded() {
        let (plane, _rx) = EventPlane::new();
        let first = plane.publish(port("a", "x"), EventKind::Pattern, Value::None);
        for _ in 0..HISTORY_LEN {
            plane.publish(port("a", "x"), EventKind::Pattern, Value::None);
        }
        assert!(
            plane.chain(first).is_empty(),
            "the oldest event is forgotten"
        );
        assert!(plane.history.lock().events.len() <= HISTORY_LEN);
    }
}
