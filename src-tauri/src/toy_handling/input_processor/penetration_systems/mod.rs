pub mod sps;
pub mod tps;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    frontend::{frontend_types::FePenetrationSystem, ToFrontend},
    toy_handling::toyops::{ProcessingMode, ProcessingModeValues},
};

use super::InputProcessor;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PenetrationSystemType {
    NONE, // No Input Processor
    TPS,  // TPS Input Processor
    SPS,  // SPS Input Processor
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct PenetrationSystem {
    #[serde(skip)]
    pub pen_system: Option<Box<dyn InputProcessor>>,
    pub pen_system_type: PenetrationSystemType,
    pub pen_system_processing_mode: ProcessingMode,
    #[serde(skip)]
    pub pen_system_processing_mode_values: ProcessingModeValues,
}

impl Default for PenetrationSystem {
    fn default() -> Self {
        Self {
            pen_system: None,
            pen_system_type: PenetrationSystemType::NONE,
            pen_system_processing_mode: ProcessingMode::Raw,
            pen_system_processing_mode_values: ProcessingModeValues::Raw,
        }
    }
}

impl ToFrontend<FePenetrationSystem> for PenetrationSystem {
    type OutputType = FePenetrationSystem;
    fn to_frontend(&self) -> Self::OutputType {
        FePenetrationSystem {
            pen_system_type: self.pen_system_type.clone(),
            pen_system_processing_mode: self.pen_system_processing_mode.to_frontend(),
        }
    }
}
