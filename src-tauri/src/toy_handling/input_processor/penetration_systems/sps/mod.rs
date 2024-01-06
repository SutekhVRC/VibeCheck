pub mod model;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::toy_handling::{input_processor::InputProcessor, ModeProcessorInputType};

use self::model::SPSParameter;

/*
 * Identifier for each type/id combination - This should be HashMap key
 * Try
 */

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct SPSProcessor {
    mappings: HashMap<String, SPSParameter>,
}

impl Default for SPSProcessor {
    fn default() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }
}

impl InputProcessor for SPSProcessor {
    fn is_parameter(&self, param: &String) -> bool {
        param.starts_with("/avatar/parameters/OGB/")
    }

    fn process(&mut self, _addr: &str, _input: ModeProcessorInputType) -> Option<f64> {
        // Strip away VRChat avatar parameter prefix
        let sps_param = if let Some(s) = _addr.strip_prefix("/avatar/parameters/") {
            s
        } else {
            return None;
        };

        let sps_param_split = sps_param.split("/").collect::<Vec<&str>>();

        // Orf/Pen/etc.
        let p_type = sps_param_split[1];
        //
        let p_id = sps_param_split[2];

        let sps_key = format!("{}__{}", p_type, p_id);

        // If mapping exists use it
        // If mapping does not exist, make a new one and use it
        if let Some(_sps_mapping) = self.mappings.get_mut(&sps_key) {
            todo!();
            // return sps_mapping.internal_process;
        } else {
            let new_sps_param_obj = if let Some(sp) = SPSParameter::new(sps_param.to_string()) {
                sp
            } else {
                return None;
            };

            self.mappings.insert(sps_key, new_sps_param_obj);
        }

        None
    }
}

impl SPSProcessor {
    //fn process_length(&mut self, )
}
