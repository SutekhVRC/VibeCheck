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

use std::sync::Arc;

use async_trait::async_trait;

use crate::address::{ModuleInstanceId, PortId};
use crate::diagnostics::Diagnostics;
use crate::error::Result;
use crate::event::{EventKind, EventPlane, Value};
use crate::safety::ClampedCmd;
use crate::signal::{Signal, SignalPlane};

/// Whether a port is read from or written to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDirection {
    /// The module reads it.
    In,
    /// The module writes or publishes on it.
    Out,
}

/// What travels on a port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortKind {
    /// A continuous value on the signal plane.
    Signal,
    /// Discrete happenings on the event plane.
    Event,
}

/// One port a module declares in its `ports()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortSpec {
    /// Port name, unique within its module instance.
    pub name: String,
    /// Read or written.
    pub direction: PortDirection,
    /// Signal or event.
    pub kind: PortKind,
}

impl PortSpec {
    /// A signal input, e.g. a processor's `in` or a motor.
    pub fn signal_in(name: impl Into<String>) -> Self {
        Self::new(name, PortDirection::In, PortKind::Signal)
    }

    /// A signal output, e.g. a processor's `out`.
    pub fn signal_out(name: impl Into<String>) -> Self {
        Self::new(name, PortDirection::Out, PortKind::Signal)
    }

    /// A port events are published from, e.g. an output's `error`.
    pub fn event_out(name: impl Into<String>) -> Self {
        Self::new(name, PortDirection::Out, PortKind::Event)
    }

    /// Any port.
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
    diagnostics: Diagnostics,
}

impl InputCtx {
    /// Diagnostics start off and unconnected; see [`with_diagnostics`](Self::with_diagnostics).
    pub fn new(module: ModuleInstanceId, signals: SignalPlane, events: EventPlane) -> Self {
        Self {
            module,
            signals,
            events,
            diagnostics: Diagnostics::default(),
        }
    }

    /// Log to `diagnostics` (shared with whatever shows it).
    pub fn with_diagnostics(mut self, diagnostics: Diagnostics) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    /// This module's id.
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

    /// Forget every value this module has written, e.g. when everything it
    /// knew is about to be replaced.
    pub fn clear_signals(&self) {
        self.signals.clear_module(&self.module);
    }

    /// Whether diagnostics are on. [`log`](Self::log) already checks; this is
    /// for skipping work needed only to log.
    pub fn diagnostics_enabled(&self) -> bool {
        self.diagnostics.enabled()
    }

    /// Add a diagnostics log entry from this module, for what never reaches
    /// the signal plane. `value` is only formatted while diagnostics are on.
    pub fn log(&self, label: impl Into<String>, value: impl FnOnce() -> String) {
        self.diagnostics.log(&self.module, label, value);
    }

    /// Publish a root event from one of this module's ports.
    pub fn publish(&self, port: &str, kind: EventKind, value: Value) {
        self.events.publish(self.port(port), kind, value);
    }
}

/// A module that brings data in from outside: it writes signals and
/// publishes events through its [`InputCtx`]. See `docs/runtime/writing-modules.md`.
///
/// # Example
///
/// An input that takes values from elsewhere in the program over a channel
/// and writes each one to a port named after it. It runs until every sender
/// is dropped.
///
/// ```
/// use async_trait::async_trait;
/// use tokio::sync::mpsc;
/// use vibecheck_core::{
///     EventKind, EventPlane, Input, InputCtx, ModuleInstanceId, PortId, PortSpec, Result,
///     Signal, SignalPlane, Value,
/// };
///
/// struct ChannelInput {
///     rx: mpsc::UnboundedReceiver<(String, f64)>,
/// }
///
/// #[async_trait]
/// impl Input for ChannelInput {
///     fn ports(&self) -> Vec<PortSpec> {
///         // Port names come from the data, so only the event port is fixed
///         vec![PortSpec::event_out("closed")]
///     }
///
///     async fn run(&mut self, ctx: InputCtx) -> Result<()> {
///         while let Some((name, value)) = self.rx.recv().await {
///             ctx.write_signal(&name, Signal::Float(value));
///             // Only formatted while diagnostics are on
///             ctx.log(name, || format!("{value}"));
///         }
///         ctx.publish("closed", EventKind::Connection, Value::Bool(false));
///         Ok(())
///     }
/// }
///
/// # tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
/// let signals = SignalPlane::new();
/// let (events, mut received) = EventPlane::new();
/// let module = ModuleInstanceId::new("channel");
/// let ctx = InputCtx::new(module.clone(), signals.clone(), events);
///
/// let (tx, rx) = mpsc::unbounded_channel();
/// tx.send(("speed".to_owned(), 0.25)).unwrap();
/// drop(tx);
/// ChannelInput { rx }.run(ctx).await.unwrap();
///
/// assert_eq!(signals.read(&PortId::new(module, "speed")), Some(Signal::Float(0.25)));
/// assert_eq!(received.try_recv().unwrap().value, Value::Bool(false));
/// # });
/// ```
#[async_trait]
pub trait Input: Send {
    /// The ports this input writes and publishes on.
    fn ports(&self) -> Vec<PortSpec>;

    /// Runs for the life of the module, writing signals and publishing events
    /// through `ctx`. Returning ends the module.
    async fn run(&mut self, ctx: InputCtx) -> Result<()>;
}

/// An event a processor wants to publish, on one of its own event ports.
/// The graph turns it into an [`Event`](crate::event::Event) with the node
/// as its source.
#[derive(Clone, Debug, PartialEq)]
pub struct EventOut {
    /// One of the processor's own event ports.
    pub port: Arc<str>,
    /// What happened.
    pub kind: EventKind,
    /// Its data.
    pub value: Value,
}

impl EventOut {
    /// An event to publish from the processor's `port`.
    pub fn new(port: impl AsRef<str>, kind: EventKind, value: Value) -> Self {
        Self {
            port: Arc::from(port.as_ref()),
            kind,
            value,
        }
    }
}

/// A set of signal-plane ports a [`Processor`] reads without wiring them one
/// by one: every port of `module` whose name starts with `prefix`.
///
/// For processors whose inputs aren't known when the graph is built, such as
/// one that reads every `/avatar/parameters/OGB/...` parameter an avatar
/// happens to send.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Watch {
    /// The module whose ports are watched, e.g. `osc`.
    pub module: ModuleInstanceId,
    /// The start of the port names to watch, e.g. `/avatar/parameters/OGB/`.
    pub prefix: String,
}

impl Watch {
    /// Watch every port of `module` whose name starts with `prefix`.
    pub fn new(module: ModuleInstanceId, prefix: impl Into<String>) -> Self {
        Self {
            module,
            prefix: prefix.into(),
        }
    }
}

/// `T`'s name without its module path or generic parameters.
fn short_type_name<T: ?Sized>() -> String {
    let full = std::any::type_name::<T>();
    let base = full.split('<').next().unwrap_or(full);
    base.rsplit("::").next().unwrap_or(base).to_owned()
}

/// A module that turns signals into signals once per graph tick, and may
/// emit events. It runs as a node in a [`Graph`](crate::graph::Graph). See
/// `docs/runtime/writing-modules.md`.
///
/// # Example
///
/// A threshold with hysteresis: `out` turns on above 0.5 and off below
/// 0.45, so a value jittering around 0.5 doesn't flip it every tick. Each
/// flip is also published as a `Threshold` event from its `crossed` port.
///
/// ```
/// use vibecheck_core::{
///     EventKind, EventOut, EventPlane, GraphBuilder, ModuleInstanceId, PortId, PortSpec,
///     Processor, SafetyLayer, Signal, SignalPlane, Value,
/// };
///
/// #[derive(Default)]
/// struct Threshold {
///     on: bool,
/// }
///
/// impl Processor for Threshold {
///     fn ports(&self) -> Vec<PortSpec> {
///         vec![
///             PortSpec::signal_in("in"),
///             PortSpec::signal_out("out"),
///             PortSpec::event_out("crossed"),
///         ]
///     }
///
///     fn tick(
///         &mut self,
///         ins: &[Signal],
///         _dt: f32,
///         out: &mut [Option<Signal>],
///         events: &mut Vec<EventOut>,
///     ) {
///         let level = ins[0].as_level();
///         let on = if self.on { level >= 0.45 } else { level > 0.5 };
///         if on != self.on {
///             self.on = on;
///             events.push(EventOut::new("crossed", EventKind::Threshold, Value::Bool(on)));
///         }
///         out[0] = Some(Signal::Bool(on));
///     }
///
///     // Called on avatar change: forget what was learned, keep settings
///     fn reset(&mut self) {
///         self.on = false;
///     }
/// }
///
/// // Wire it into a graph: an OSC parameter in, a motor out
/// let osc = ModuleInstanceId::new("osc");
/// let node = ModuleInstanceId::new("threshold");
/// let boop = PortId::new(osc, "/avatar/parameters/Boop");
/// let mut builder = GraphBuilder::new();
/// builder.add_processor(node.clone(), Box::new(Threshold::default())).unwrap();
/// builder.connect(boop.clone(), PortId::new(node.clone(), "in")).unwrap();
/// let motor = PortId::new(ModuleInstanceId::new("toy:demo:0"), "Vibrator_0");
/// builder.add_sink(PortId::new(node.clone(), "out"), motor, None).unwrap();
/// let mut graph = builder.build().unwrap();
///
/// let signals = SignalPlane::new();
/// let (events, mut received) = EventPlane::new();
/// let safety = SafetyLayer::default();
/// let mut tick = |value| {
///     signals.write(&boop, Signal::Float(value));
///     graph.tick(&signals, 0.01, &safety, &events)
/// };
///
/// // Above 0.5: the motor gets full power, and the event is published
/// assert_eq!(tick(0.6)[0].level(), 1.0);
/// let event = received.try_recv().unwrap();
/// assert_eq!(event.source, PortId::new(node, "crossed"));
/// assert_eq!(event.value, Value::Bool(true));
///
/// // Jitter just under 0.5 changes nothing: no command, no event
/// assert!(tick(0.48).is_empty());
/// assert!(received.try_recv().is_none());
///
/// // Below 0.45 it turns off
/// assert_eq!(tick(0.4)[0].level(), 0.0);
/// assert_eq!(received.try_recv().unwrap().value, Value::Bool(false));
/// ```
pub trait Processor: Send {
    /// The processor's ports. Its signal inputs and outputs, in this order,
    /// are the order of `ins` and `out` in [`tick`](Processor::tick).
    fn ports(&self) -> Vec<PortSpec>;

    /// What kind of node this is, for views of a graph, e.g. `Smooth`.
    /// Default: the type's name without its path.
    fn kind(&self) -> String {
        short_type_name::<Self>()
    }

    /// Signal-plane ports this processor reads besides its wired inputs.
    /// Asked once, when the node is added to a graph. Default: none.
    fn watches(&self) -> Vec<Watch> {
        Vec::new()
    }

    /// Called each tick, before [`tick`](Processor::tick), with the watched
    /// ports written since the last tick: the latest value of each, oldest
    /// write first (watch by watch, if there are several). Not called when
    /// nothing changed. On the first tick it gets every matching value
    /// already on the plane.
    fn watched(&mut self, _changed: &[(PortId, Signal)]) {}

    /// Forget any state built up from past input, as if the processor were
    /// new (e.g. on avatar change, so nothing from the old avatar carries
    /// over). Settings stay. Default: nothing to forget.
    fn reset(&mut self) {}

    /// Called once per graph tick, in topological order.
    ///
    /// `ins` and `out` are in the order of this module's signal in/out ports.
    /// Each `out` slot starts as `None`; set it to `Some` to write a new value
    /// to that port, or leave it `None` to leave the port unchanged (e.g. a
    /// smoother still collecting values). `dt` is the time since the last
    /// tick, in seconds. Events pushed onto `events` are published on the
    /// event plane, from this node.
    fn tick(
        &mut self,
        ins: &[Signal],
        dt: f32,
        out: &mut [Option<Signal>],
        events: &mut Vec<EventOut>,
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

/// A module that acts on a device or service. It only receives
/// [`ClampedCmd`]s, so everything it does has passed the safety layer. See
/// `docs/runtime/writing-modules.md`.
///
/// # Example
///
/// An output that records the level sent to each of its ports, as a real
/// one would send them to a device. Commands come from the safety layer,
/// usually through [`CommandSlots`](crate::slots::CommandSlots) drained at
/// the output's own rate. Whatever drains them checks
/// [`SafetyLayer::admits`](crate::safety::SafetyLayer::admits) right before
/// sending, so a command queued before a panic is dropped.
///
/// ```
/// use std::collections::BTreeMap;
/// use async_trait::async_trait;
/// use vibecheck_core::{
///     ClampedCmd, CommandSlots, Error, ModuleInstanceId, Output, PortId, PortSpec, Result,
///     SafetyCaps, SafetyLayer,
/// };
///
/// #[derive(Default)]
/// struct Recorder {
///     levels: BTreeMap<String, f64>,
/// }
///
/// #[async_trait]
/// impl Output for Recorder {
///     fn ports(&self) -> Vec<PortSpec> {
///         vec![PortSpec::signal_in("light")]
///     }
///
///     async fn apply(&mut self, cmds: &[ClampedCmd]) -> Result<()> {
///         for cmd in cmds {
///             let port = &cmd.target().port;
///             if &**port != "light" {
///                 return Err(Error::UnknownPort(cmd.target().clone()));
///             }
///             self.levels.insert(port.to_string(), cmd.level());
///         }
///         Ok(())
///     }
/// }
///
/// # tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
/// // Master intensity at 50%
/// let safety = SafetyLayer::new(SafetyCaps { intensity: 0.5, ..Default::default() });
/// let light = PortId::new(ModuleInstanceId::new("lamp"), "light");
/// let slots = CommandSlots::new();
/// let mut output = Recorder::default();
///
/// slots.write(safety.clamp(light.clone(), 0.8, None).unwrap());
/// let mut cmds = slots.take_changed();
/// cmds.retain(|cmd| safety.admits(cmd));
/// output.apply(&cmds).await.unwrap();
/// assert_eq!(output.levels["light"], 0.4);
///
/// // A command queued before a panic is never sent
/// slots.write(safety.clamp(light, 1.0, None).unwrap());
/// safety.panic();
/// let mut cmds = slots.take_changed();
/// cmds.retain(|cmd| safety.admits(cmd));
/// assert!(cmds.is_empty());
/// # });
/// ```
#[async_trait]
pub trait Output: Send {
    /// The ports commands can target (signal inputs), and any event ports
    /// it publishes on.
    fn ports(&self) -> Vec<PortSpec>;

    /// Act on the device or service. Commands have already been clamped and
    /// checked by the safety layer.
    async fn apply(&mut self, cmds: &[ClampedCmd]) -> Result<()>;
}
