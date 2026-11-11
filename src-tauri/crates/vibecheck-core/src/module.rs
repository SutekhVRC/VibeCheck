//! The three kinds of module.
//!
//! - [`Input`]s produce signals and events (OSC, audio, ...).
//! - [`Processor`]s transform signals each tick and may emit events
//!   (smoothing, curves, thresholds, ...).
//! - [`Output`]s act on devices or services (toys, other devices, ...). They only
//!   ever receive [`ClampedCmd`]s, which have passed the safety layer.
//!
//! Modules are ordinary Rust types compiled into the app; there is no plugin
//! ABI.

use async_trait::async_trait;

use crate::address::{ModuleInstanceId, PortId};
use crate::error::Result;
use crate::event::{Event, EventKind, EventPlane, Value};
use crate::safety::ClampedCmd;
use crate::signal::{Signal, SignalPlane};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDirection {
    In,
    Out,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortKind {
    /// A continuous value on the signal plane.
    Signal,
    /// Discrete happenings on the event plane.
    Event,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortSpec {
    /// Port name, unique within its module instance.
    pub name: String,
    pub direction: PortDirection,
    pub kind: PortKind,
}

impl PortSpec {
    pub fn signal_in(name: impl Into<String>) -> Self {
        Self::new(name, PortDirection::In, PortKind::Signal)
    }

    pub fn signal_out(name: impl Into<String>) -> Self {
        Self::new(name, PortDirection::Out, PortKind::Signal)
    }

    pub fn event_out(name: impl Into<String>) -> Self {
        Self::new(name, PortDirection::Out, PortKind::Event)
    }

    pub fn new(name: impl Into<String>, direction: PortDirection, kind: PortKind) -> Self {
        Self {
            name: name.into(),
            direction,
            kind,
        }
    }
}

/// What an [`Input`] gets to talk to the rest of the runtime.
#[derive(Clone)]
pub struct InputCtx {
    module: ModuleInstanceId,
    signals: SignalPlane,
    events: EventPlane,
}

impl InputCtx {
    pub fn new(module: ModuleInstanceId, signals: SignalPlane, events: EventPlane) -> Self {
        Self {
            module,
            signals,
            events,
        }
    }

    pub fn module(&self) -> &ModuleInstanceId {
        &self.module
    }

    /// The address of one of this module's own ports.
    pub fn port(&self, name: &str) -> PortId {
        PortId::new(self.module.clone(), name)
    }

    /// Overwrite the latest value on one of this module's ports.
    pub fn write_signal(&self, port: &str, signal: Signal) {
        self.signals.write(&self.port(port), signal);
    }

    /// Publish a root event from one of this module's ports.
    pub fn publish(&self, port: &str, kind: EventKind, value: Value) {
        self.events.publish(self.port(port), kind, value);
    }
}

#[async_trait]
pub trait Input: Send {
    fn ports(&self) -> Vec<PortSpec>;

    /// Runs for the life of the module, writing signals and publishing events
    /// through `ctx`. Returning ends the module.
    async fn run(&mut self, ctx: InputCtx) -> Result<()>;
}

pub trait Processor: Send {
    fn ports(&self) -> Vec<PortSpec>;

    /// Called once per graph tick, in topological order.
    ///
    /// `ins` and `out` are in the order of this module's signal in/out ports.
    /// Each `out` slot starts as `None`; set it to `Some` to write a new value
    /// to that port, or leave it `None` to leave the port unchanged (e.g. a
    /// smoother still collecting values). `dt` is the time since the last
    /// tick, in seconds. Events pushed onto `events` are published on the
    /// event plane after the tick.
    fn tick(
        &mut self,
        ins: &[Signal],
        dt: f32,
        out: &mut [Option<Signal>],
        events: &mut Vec<Event>,
    );
}

/// Runs a single-input, single-output processor once and returns its output.
///
/// For callers that drive a processor per incoming value rather than from a
/// graph tick. Events the processor emits are discarded.
pub fn process_one(processor: &mut dyn Processor, input: Signal, dt: f32) -> Option<Signal> {
    let mut out = [None];
    processor.tick(&[input], dt, &mut out, &mut Vec::new());
    out[0]
}

#[async_trait]
pub trait Output: Send {
    fn ports(&self) -> Vec<PortSpec>;

    /// Act on the device or service. Commands have already been clamped and
    /// checked by the safety layer.
    async fn apply(&mut self, cmds: &[ClampedCmd]) -> Result<()>;
}
