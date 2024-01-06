use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::toy_handling::{input_processor::InputProcessor, ModeProcessorInputType};

#[derive(Default, Clone, Debug, Serialize, Deserialize, TS)]
pub struct TPSProcessor {
    pub parameter_list: Vec<String>,
}

impl InputProcessor for TPSProcessor {
    fn is_parameter(&self, param: &String) -> bool {
        param.starts_with("/avatar/parameters/TPS_Internal")
    }

    fn process(&mut self, _addr: &str, _input: ModeProcessorInputType) -> Option<f64> {
        todo!()
    }
}
