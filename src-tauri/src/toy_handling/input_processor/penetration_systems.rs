use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::toy_handling::ModeProcessorInputType;

use super::InputProcessor;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PenetrationSystemType {
    NONE,
    TPS,
    SPS,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct TPSProcessor {
    pub parameter_list: Vec<String>,
}

impl InputProcessor for TPSProcessor {
    fn is_parameter(&self, param: &String) -> bool {
        self.parameter_list.contains(param)
    }

    fn process(&self, _input: ModeProcessorInputType) -> Option<f64> {
        todo!()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct SPSProcessor {
    pub parameter_list: Vec<String>,
}

impl InputProcessor for SPSProcessor {
    fn is_parameter(&self, param: &String) -> bool {
        self.parameter_list.contains(param)
    }

    fn process(&self, _input: ModeProcessorInputType) -> Option<f64> {
        todo!()
    }
}
