pub mod sps;
pub mod tps;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    frontend::{frontend_types::FePenetrationSystem, FromFrontend, ToBackend, ToFrontend},
    toy_handling::toyops::{ProcessingMode, ProcessingModeValues},
};

use self::{sps::SPSProcessor, tps::TPSProcessor};

use super::InputProcessor;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PenetrationSystemType {
    None, // No Input Processor
    Tps,  // TPS Input Processor
    Sps,  // SPS Input Processor
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct PenetrationSystem {
    #[serde(skip)]
    pub pen_system: Option<Box<dyn InputProcessor>>,
    pub pen_system_type: PenetrationSystemType,
    pub pen_system_processing_mode: ProcessingMode,
    pub pen_system_input_filter: Option<Vec<String>>,
    #[serde(skip)]
    pub pen_system_processing_mode_values: ProcessingModeValues,
}

impl Default for PenetrationSystem {
    fn default() -> Self {
        Self {
            pen_system: None,
            pen_system_type: PenetrationSystemType::None,
            pen_system_processing_mode: ProcessingMode::Raw,
            pen_system_input_filter: None,
            pen_system_processing_mode_values: ProcessingModeValues::Raw,
        }
    }
}

impl ToFrontend<FePenetrationSystem> for PenetrationSystem {
    type OutputType = FePenetrationSystem;
    fn to_frontend(&self) -> Self::OutputType {

        // Frontend expects empty Vec if no tags (In the future make tuti send null for input filter from frontend)
        let pen_system_input_filter = if self.pen_system_input_filter.as_ref().is_some() {
            self.pen_system_input_filter.clone()
        } else {
            Some(vec![])
        };

        FePenetrationSystem {
            pen_system_type: self.pen_system_type.clone(),
            pen_system_processing_mode: self.pen_system_processing_mode.to_frontend(),
            pen_system_input_filter,
        }
    }
}

impl FromFrontend<FePenetrationSystem> for PenetrationSystem {
    type OutputType = ();

    fn from_frontend(&mut self, frontend_type: FePenetrationSystem) -> Self::OutputType {
        // Allocate / Instantiate new Penetration system structure based on user's choice
        match frontend_type.pen_system_type {
            PenetrationSystemType::None => self.pen_system = None,
            PenetrationSystemType::Sps => self.pen_system = Some(Box::<SPSProcessor>::default()),
            PenetrationSystemType::Tps => self.pen_system = Some(Box::<TPSProcessor>::default()),
        }
        self.pen_system_type = frontend_type.pen_system_type;
        let backend_pspm = frontend_type.pen_system_processing_mode.to_backend();
        self.pen_system_processing_mode_values = ProcessingModeValues::new_from(&backend_pspm);
        self.pen_system_processing_mode = backend_pspm;

        // Backend expects None if no tags (In the future make tuti send null for input filter from frontend)
        if frontend_type.pen_system_input_filter.as_ref().is_some_and(|v| v.is_empty()) {
            self.pen_system_input_filter = None;
        } else {
            self.pen_system_input_filter = frontend_type.pen_system_input_filter.clone();
        }
    }
}
