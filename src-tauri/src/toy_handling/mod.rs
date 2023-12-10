use serde::{Deserialize, Serialize};
use ts_rs::TS;

use self::toyops::{ProcessingModeValues, ToyParameter};

pub mod errors;
pub mod handling;
pub mod input_processor;
pub mod toy_manager;
pub mod toyops;

pub enum SmoothParser {
    Smoothed(f64),
    SkipZero(f64),
    Smoothing,
}

pub enum RateParser {
    RateCalculated(f64, bool),
    SkipZero,
}

#[derive(Clone, Debug)]
pub enum ToySig {
    //ToyCommand(ToyFeature),
    UpdateToy(crate::vcore::core::ToyUpdate),
    OSCMsg(rosc::OscMessage),
}

pub enum ModeProcessorInput<'processor> {
    InputProcessor((ModeProcessorInputType, &'processor mut ProcessingModeValues)),
    RawInput(ModeProcessorInputType, &'processor mut ToyParameter),
}

pub enum ModeProcessorInputType {
    Float(f64),
    Boolean(bool),
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum ToyPower {
    Pending,
    Battery(f64),
    NoBattery,
    Offline,
}

impl ToyPower {
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
                format!("{}%", m.to_string())
            }
            Self::NoBattery => "Powered".to_owned(),
            Self::Offline => "Offline".to_owned(),
        }
    }
}
