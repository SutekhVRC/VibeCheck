//! Output module for toys connected through Buttplug IO.
//!
//! Each actuator (motor) on the toy is one signal-in port. Port names are
//! stable across reconnects, e.g. `Vibrator_0` or `Linear_0`; Buttplug's
//! device index is only used internally.
//!
//! With [`with_events`](ButtplugOutput::with_events), the output publishes an
//! [`EventKind::OutputError`] event on its `error` port when sending starts
//! failing (value: the error as text) and again when it works again (value:
//! `None`). It only fires when that state changes, not on every failed send.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use buttplug::client::{
    ButtplugClientDevice, LinearCommand::LinearMap, RotateCommand::RotateMap,
    ScalarCommand::ScalarMap,
};
use buttplug::core::message::ActuatorType;

use crate::address::{ModuleInstanceId, PortId};
use crate::error::{Error, Result};
use crate::event::{EventKind, EventPlane, Value};
use crate::module::{Output, PortSpec};
use crate::safety::ClampedCmd;

/// Event port for send failures and recoveries.
pub const ERROR_PORT: &str = "error";

/// Tracks whether an output is failing, so error events fire only when that
/// changes.
#[derive(Default)]
struct ErrorState {
    failing: bool,
}

impl ErrorState {
    /// The event value to publish for this result, if the state changed.
    fn update(&mut self, result: &Result<()>) -> Option<Value> {
        match (result, self.failing) {
            (Err(e), false) => {
                self.failing = true;
                Some(Value::Text(e.to_string().into()))
            }
            (Ok(()), true) => {
                self.failing = false;
                Some(Value::None)
            }
            _ => None,
        }
    }
}

/// How to drive one actuator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Actuator {
    /// Index into the device's scalar features (vibrate, constrict, oscillate, ...).
    Scalar { index: u32, actuator: ActuatorType },
    /// Index into the device's rotate features. Always rotates clockwise.
    Rotate { index: u32 },
    /// Index into the device's linear features. The command's duration is the
    /// requested movement time in milliseconds.
    Linear { index: u32 },
}

pub struct ButtplugOutput {
    device: Arc<ButtplugClientDevice>,
    actuators: HashMap<Arc<str>, Actuator>,
    events: Option<(PortId, EventPlane)>,
    error_state: ErrorState,
}

impl ButtplugOutput {
    /// `actuators` maps each port name to the actuator it drives.
    pub fn new(
        device: Arc<ButtplugClientDevice>,
        actuators: impl IntoIterator<Item = (String, Actuator)>,
    ) -> Self {
        Self {
            device,
            actuators: actuators
                .into_iter()
                .map(|(port, actuator)| (Arc::from(port), actuator))
                .collect(),
            events: None,
            error_state: ErrorState::default(),
        }
    }

    /// Publish error and recovery events as `module`, on `events`.
    pub fn with_events(mut self, module: ModuleInstanceId, events: EventPlane) -> Self {
        self.events = Some((PortId::new(module, ERROR_PORT), events));
        self
    }

    async fn send(&self, actuator: Actuator, cmd: &ClampedCmd) -> Result<()> {
        let level = cmd.level();
        let sent = match actuator {
            Actuator::Scalar { index, actuator } => {
                self.device
                    .scalar(&ScalarMap(HashMap::from([(index, (level, actuator))])))
                    .await
            }
            Actuator::Rotate { index } => {
                self.device
                    .rotate(&RotateMap(HashMap::from([(index, (level, true))])))
                    .await
            }
            Actuator::Linear { index } => {
                let duration = cmd.duration_ms().unwrap_or(0);
                self.device
                    .linear(&LinearMap(HashMap::from([(index, (duration, level))])))
                    .await
            }
        };
        sent.map_err(|e| Error::OutputUnavailable(e.to_string()))
    }
}

#[async_trait]
impl Output for ButtplugOutput {
    fn ports(&self) -> Vec<PortSpec> {
        let mut ports: Vec<PortSpec> = self
            .actuators
            .keys()
            .map(|port| PortSpec::signal_in(port.as_ref()))
            .collect();
        ports.sort_by(|a, b| a.name.cmp(&b.name));
        ports.push(PortSpec::event_out(ERROR_PORT));
        ports
    }

    /// Sends every command, even if an earlier one fails, and returns the
    /// first error.
    async fn apply(&mut self, cmds: &[ClampedCmd]) -> Result<()> {
        let mut first_error = None;
        for cmd in cmds {
            let result = match self.actuators.get(&cmd.target().port) {
                Some(&actuator) => self.send(actuator, cmd).await,
                None => Err(Error::UnknownPort(cmd.target().clone())),
            };
            if let Err(e) = result {
                first_error.get_or_insert(e);
            }
        }
        let result = first_error.map_or(Ok(()), Err);
        if let Some((port, events)) = &self.events {
            if let Some(value) = self.error_state.update(&result) {
                events.publish(port.clone(), EventKind::OutputError, value);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_events_fire_only_when_the_state_changes() {
        let mut state = ErrorState::default();
        let failed = || Err(Error::OutputUnavailable("disconnected".into()));

        assert_eq!(state.update(&Ok(())), None);
        assert_eq!(
            state.update(&failed()),
            Some(Value::Text("output unavailable: disconnected".into()))
        );
        assert_eq!(state.update(&failed()), None, "still failing");
        assert_eq!(state.update(&Ok(())), Some(Value::None), "recovered");
        assert_eq!(state.update(&Ok(())), None);
    }
}
