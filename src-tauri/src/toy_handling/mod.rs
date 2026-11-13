use buttplug_client::ButtplugClientDevice;
use buttplug_core::message::InputType;
use log::warn;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod errors;
pub mod input_processor;
pub mod runtime;
pub mod toy_command_processor;
pub mod toy_manager;
pub mod toyops;

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum ToyPower {
    Pending,
    Battery(f64),
    NoBattery,
    Offline,
}

impl ToyPower {
    /// What to show before the battery has been read: `Pending` if the
    /// device has a battery, `NoBattery` if it doesn't.
    pub fn unread(device: &ButtplugClientDevice) -> Self {
        if device.input_available(InputType::Battery) {
            Self::Pending
        } else {
            Self::NoBattery
        }
    }

    /// Reads a device's battery. Buttplug reports it as a percentage;
    /// VibeCheck keeps it as 0.0 to 1.0.
    pub async fn read(device: &ButtplugClientDevice) -> Self {
        if !device.input_available(InputType::Battery) {
            return Self::NoBattery;
        }
        match device.battery().await {
            Ok(percent) => Self::Battery(f64::from(percent.min(100)) / 100.0),
            Err(e) => {
                warn!("Couldn't read the battery of {}: {:?}", device.name(), e);
                Self::Pending
            }
        }
    }

    pub fn to_float(&self) -> f64 {
        match self {
            Self::Battery(level) => *level,
            _ => 0.0,
        }
    }
}

impl ToString for ToyPower {
    fn to_string(&self) -> String {
        match self {
            Self::Pending => "Pending".to_owned(),
            Self::Battery(level) => {
                let m = 100.0 * level;
                format!("{}%", m)
            }
            Self::NoBattery => "Powered".to_owned(),
            Self::Offline => "Offline".to_owned(),
        }
    }
}
