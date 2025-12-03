use std::sync::Arc;

use buttplug::client::ButtplugClientDevice;
use log::debug;
use tokio::{sync::{mpsc::error::TryRecvError, mpsc::UnboundedReceiver, watch}, time::Instant};

use crate::toy_handling::{runtime::toy_management_handler::sleep_for_constant_rate, toy_command_processor::command_toy, toyops::{LevelTweaks, VCFeatureType}};

pub enum ToyEmitterThreadSignal {
    StopExecution,
    UpdateRate(u64),
}

#[derive(Debug, Clone)]
pub struct OscParserData {
    dev: Arc<ButtplugClientDevice>,
    feature_type: VCFeatureType,
    float_level: f64,
    feature_index: u32,
    flip_float: bool,
    feature_levels: LevelTweaks,
}

impl OscParserData {
    pub fn new(
        dev: Arc<ButtplugClientDevice>,
        feature_type: VCFeatureType,
        float_level: f64,
        feature_index: u32,
        flip_float: bool,
        feature_levels: LevelTweaks,
    ) -> Self {
        Self {
            dev,
            feature_type,
            float_level,
            feature_index,
            flip_float,
            feature_levels,
        }
    }
}

pub struct EmitterThreadData {
    update_rate: u64,
    in_signal: UnboundedReceiver<ToyEmitterThreadSignal>,
    in_osc_data: watch::Receiver<Option<OscParserData>>,
}

impl EmitterThreadData {
    pub fn new(
        in_signal: UnboundedReceiver<ToyEmitterThreadSignal>,
        in_osc_data: watch::Receiver<Option<OscParserData>>,
        update_rate: u64,
    ) -> Self {
        Self {
            update_rate,
            in_signal,
            in_osc_data,
        }
    }
}

pub async fn toy_emitter_thread(mut data: EmitterThreadData) {

    loop {
        let start = Instant::now();
        // Logic
        // Parse & send bluetooth command (watch)
        if let Ok(changed) = data.in_osc_data.has_changed() {
            if changed {
                let osc_data = {data.in_osc_data.borrow_and_update().to_owned()};
                if let Some(osc_data) = osc_data {
                    debug!("Sending {} to toy {} and feature {}", osc_data.float_level, osc_data.dev.index(), osc_data.feature_index);
                    command_toy(osc_data.dev, osc_data.feature_type, osc_data.float_level, osc_data.feature_index, osc_data.flip_float, osc_data.feature_levels).await;
                }
            }
        }

        // Check for incoming update messages
        match data.in_signal.try_recv() {
            Ok(signal) => {
                match signal {
                    ToyEmitterThreadSignal::StopExecution => return,
                    ToyEmitterThreadSignal::UpdateRate(hz) => data.update_rate = hz,
                }
            },
            Err(e) => match e {
                TryRecvError::Disconnected => return,
                TryRecvError::Empty => (),
            }
        }
        sleep_for_constant_rate(data.update_rate, start).await;
    }
}