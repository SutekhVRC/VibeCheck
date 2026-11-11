//! Module runtime for VibeCheck.
//!
//! VibeCheck is built as a pipeline of modules:
//!
//! ```text
//! Inputs  ->  Processors  ->  Safety layer  ->  Outputs
//! ```
//!
//! Data moves on two planes:
//!
//! - The [signal plane](signal) carries continuous values (levels, intensities).
//!   Each port keeps only its latest value; nothing queues.
//! - The [event plane](event) carries discrete happenings (avatar change, toy
//!   connect, panic). Events are queued, ordered, and record what caused them so
//!   the safety layer can stop feedback loops.
//!
//! Modules implement the traits in [`module`]. Outputs only ever receive
//! [`ClampedCmd`](safety::ClampedCmd)s, which only the [`SafetyLayer`](safety::SafetyLayer)
//! can create.

pub mod address;
pub mod error;
pub mod event;
pub mod module;
pub mod modules;
pub mod safety;
pub mod signal;
pub mod slots;

pub use address::{ModuleInstanceId, PortId};
pub use error::{Error, Result};
pub use event::{Event, EventId, EventKind, EventPlane, EventReceiver, Value};
pub use module::{Input, InputCtx, Output, PortDirection, PortKind, PortSpec, Processor};
pub use safety::{ClampedCmd, SafetyCaps, SafetyLayer};
pub use signal::{Signal, SignalPlane};
pub use slots::CommandSlots;
