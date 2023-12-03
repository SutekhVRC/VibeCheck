pub mod penetration_systems;

use std::fmt::Debug;

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::frontend::{frontend_types::FePenetrationSystem, ToFrontend};

use self::penetration_systems::PenetrationSystemType;

use super::{
    toyops::{ProcessingMode, ProcessingModeValues},
    ModeProcessorInputType,
};

/*
 * Penetration System Architecture
 */

/*
 * Trait to define easily implementable behaviour for new penetration systems
 *
 */

pub trait InputProcessor: DynClone + Debug + Send + Sync {
    fn process(&self, input: ModeProcessorInputType) -> Option<f64>;
    fn is_parameter(&self, param: &String) -> bool;
}
dyn_clone::clone_trait_object!(InputProcessor);

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
