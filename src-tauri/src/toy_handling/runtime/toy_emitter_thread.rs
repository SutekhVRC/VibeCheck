use std::collections::HashMap;

use log::{debug, error as logerr};
use tokio::{
    sync::{mpsc::error::TryRecvError, mpsc::UnboundedReceiver},
    time::Instant,
};
use vibecheck_core::{
    modules::buttplug::{ActuatorSpec, ButtplugOutput},
    CommandSlots, ModuleInstanceId, Output, PortId, SafetyLayer, Signal, SignalPlane,
};

use crate::toy_handling::runtime::toy_management_handler::sleep_for_constant_rate;

pub enum ToyEmitterThreadSignal {
    StopExecution,
    UpdateRate(u64),
    /// New actuator mapping, e.g. after Flip Input changed.
    UpdateActuators(Vec<(String, ActuatorSpec)>),
}

pub struct EmitterThreadData {
    update_rate: u64,
    in_signal: UnboundedReceiver<ToyEmitterThreadSignal>,
    slots: CommandSlots,
    output: ButtplugOutput,
    safety: SafetyLayer,
    sent: SentLevels,
}

impl EmitterThreadData {
    pub fn new(
        in_signal: UnboundedReceiver<ToyEmitterThreadSignal>,
        slots: CommandSlots,
        output: ButtplugOutput,
        update_rate: u64,
        safety: SafetyLayer,
        sent: SentLevels,
    ) -> Self {
        Self {
            update_rate,
            in_signal,
            slots,
            output,
            safety,
            sent,
        }
    }
}

/// The levels a toy was last sent, on the signal plane under the toy's
/// module and each motor's port (`toy:<name>:<sub id>` / `Vibrate_0`), so
/// other graphs and the diagnostics view can see what the toy is doing.
/// They're intensities after the safety layer, as commanded (before Flip
/// Input, which the output applies). A panic stop shows as 0, and the
/// toy's values are removed when it stops.
pub struct SentLevels {
    signals: SignalPlane,
    module: ModuleInstanceId,
    last: HashMap<PortId, f64>,
}

impl SentLevels {
    pub fn new(signals: SignalPlane, module: ModuleInstanceId) -> Self {
        Self {
            signals,
            module,
            last: HashMap::new(),
        }
    }

    fn sent(&mut self, port: &PortId, level: f64) {
        if self.last.get(port) != Some(&level) {
            self.signals.write(port, Signal::Float(level));
            self.last.insert(port.clone(), level);
        }
    }

    /// Panic stops devices directly, not through the output: show them at 0.
    fn stopped(&mut self) {
        let ports: Vec<PortId> = self.last.keys().cloned().collect();
        for port in ports {
            self.sent(&port, 0.0);
        }
    }
}

impl Drop for SentLevels {
    fn drop(&mut self) {
        self.signals.clear_module(&self.module);
    }
}

/// Sends the latest command for every motor that changed since the last tick,
/// at the toy's update rate, and reports when the toy starts or stops.
pub async fn toy_emitter_thread(mut data: EmitterThreadData) {
    loop {
        let start = Instant::now();

        let mut cmds = data.slots.take_changed();
        // Drop anything a panic (or disarm) made stale since it was queued
        cmds.retain(|cmd| data.safety.admits(cmd));
        if !cmds.is_empty() {
            for cmd in &cmds {
                debug!("Sending {} to {}", cmd.level(), cmd.target());
            }
            match data.output.apply(&cmds).await {
                Ok(()) => {
                    for cmd in &cmds {
                        data.sent.sent(cmd.target(), cmd.level());
                    }
                }
                Err(e) => logerr!("Send to device error: {}", e),
            }
        }
        if data.safety.is_panicked() {
            data.sent.stopped();
        }
        // A panic stops devices directly, not through the output
        data.output
            .update_activity(std::time::Instant::now(), data.safety.is_panicked());

        // Check for incoming update messages
        match data.in_signal.try_recv() {
            Ok(signal) => match signal {
                ToyEmitterThreadSignal::StopExecution => return,
                ToyEmitterThreadSignal::UpdateRate(hz) => data.update_rate = hz,
                ToyEmitterThreadSignal::UpdateActuators(actuators) => {
                    data.output.set_actuators(actuators)
                }
            },
            Err(e) => match e {
                TryRecvError::Disconnected => return,
                TryRecvError::Empty => (),
            },
        }
        sleep_for_constant_rate(data.update_rate, start).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sent_levels_show_on_the_plane_drop_to_zero_on_panic_and_go_with_the_toy() {
        let signals = SignalPlane::new();
        let module = ModuleInstanceId::new("toy:lush:0");
        let motor = PortId::new(module.clone(), "Vibrate_0");
        let mut sent = SentLevels::new(signals.clone(), module);

        sent.sent(&motor, 0.4);
        assert_eq!(signals.read(&motor), Some(Signal::Float(0.4)));

        sent.stopped();
        assert_eq!(signals.read(&motor), Some(Signal::Float(0.0)));

        drop(sent);
        assert_eq!(signals.read(&motor), None);
    }
}
