use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::toy_handling::{input_processor::InputProcessor, ModeProcessorInputType};

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
