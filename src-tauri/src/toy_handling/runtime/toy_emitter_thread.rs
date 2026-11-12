use log::{debug, error as logerr};
use tokio::{
    sync::{mpsc::error::TryRecvError, mpsc::UnboundedReceiver},
    time::Instant,
};
use vibecheck_core::{
    modules::buttplug::{ActuatorSpec, ButtplugOutput},
    CommandSlots, Output, SafetyLayer,
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
}

impl EmitterThreadData {
    pub fn new(
        in_signal: UnboundedReceiver<ToyEmitterThreadSignal>,
        slots: CommandSlots,
        output: ButtplugOutput,
        update_rate: u64,
        safety: SafetyLayer,
    ) -> Self {
        Self {
            update_rate,
            in_signal,
            slots,
            output,
            safety,
        }
    }
}

/// Sends the latest command for every motor that changed since the last tick,
/// at the toy's update rate.
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
            if let Err(e) = data.output.apply(&cmds).await {
                logerr!("Send to device error: {}", e);
            }
        }

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
