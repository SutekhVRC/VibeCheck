//! Output module for toys connected through Buttplug IO.
//!
//! Each actuator (motor) on the toy is one signal-in port. Port names are
//! stable across reconnects, e.g. `Vibrator_0` or `Linear_0`; Buttplug's
//! device index is only used internally.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use buttplug::client::{
    ButtplugClientDevice, LinearCommand::LinearMap, RotateCommand::RotateMap,
    ScalarCommand::ScalarMap,
};
use buttplug::core::message::ActuatorType;

use crate::error::{Error, Result};
use crate::module::{Output, PortSpec};
use crate::safety::ClampedCmd;

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
        }
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
        first_error.map_or(Ok(()), Err)
    }
}
