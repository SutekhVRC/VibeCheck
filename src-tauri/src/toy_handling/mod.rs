use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod errors;
pub mod input_processor;
pub mod mode_processor;
pub mod osc_processor;
pub mod runtime;
pub mod toy_command_processor;
pub mod toy_manager;
pub mod toyops;
<<<<<<< HEAD
<<<<<<< HEAD
=======
pub mod runtime;
pub mod osc_processor;
pub mod mode_processor;
pub mod toy_command_processor;
>>>>>>> c1ba7ed (Break up entire handling module into distinct modules)
=======
pub mod runtime;
>>>>>>> 81b9e17 (Create runtime module)


#[derive(Clone, Debug)]
pub enum ToySig {
    //ToyCommand(ToyFeature),
    UpdateToy(crate::vcore::core::ToyUpdate),
    OSCMsg(rosc::OscMessage),
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
                format!("{}%", m)
            }
            Self::NoBattery => "Powered".to_owned(),
            Self::Offline => "Offline".to_owned(),
        }
    }
}
