pub mod model;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::toy_handling::{input_processor::InputProcessor, ModeProcessorInputType};

use self::model::SPSMapping;

/*
 * Identifier for each type/id combination - This should be HashMap key
 * Try
 */

#[derive(Default, Clone, Debug, Serialize, Deserialize, TS)]
pub struct SPSProcessor {
    mappings: HashMap<String, SPSMapping>,
}

impl InputProcessor for SPSProcessor {
    fn is_parameter(&self, param: &String) -> bool {
        param.starts_with("/avatar/parameters/OGB/")
    }

    /**
     * Inner workings of SPS according to SPS creator's app OGB (https://github.com/OscToys/OscGoesBrrr)
     *
     * addKey(OSC K,V) -> Overwrites values via param leaf key
     * onKeyChange() -> Tests for Self|Other && NewRoot|NewTip
     * Length Detectors -> update() method(NewRoot.value, NewTip.value)
     * within update() -> test for bad samples (set bad sample) -> get length (len = NewTip.value - NewRoot.value) -> save if tip <= 0.99
     * calculate length from samples -> if less than 4 samples return a bad sample
     * if 4 or more samples do length decision algo
     * motor level is calculated by
     * ~
     * activeDistance = 1 - NewRoot.value
     * activeRatio = activeDistance / saved_mapping_length
     * level = 1 - activeRatio
     * ~
     *
     */

    fn process(&mut self, addr: &str, _input: ModeProcessorInputType) -> Option<f64> {
        // Strip away VRChat avatar parameter prefix
        let sps_param = addr.strip_prefix("/avatar/parameters/")?;

        // Process SPS Param object key
        let sps_key = SPSProcessor::get_sps_param_key(sps_param)?;

        // Process parameter and create or get mutable ref to mapping
        let mapping = self.populate_mapping(&sps_key)?;

        // Update SPS parameter objects internal length

        // Need routine for getting penetrator length
        // Need routine for detecting change of each stored param
        // If changed save it and returned changed value

        None
    }
}

impl SPSProcessor {
    fn get_sps_param_key(osc_addr: &str) -> Option<String> {
        let sps_param_split = osc_addr.split('/').collect::<Vec<&str>>();

        if sps_param_split.len() != 4 {
            return None;
        };

        // Orf/Pen/etc.
        let p_type = sps_param_split[1];
        // Unity Object name
        let p_id = sps_param_split[2];
        // SPS Key
        Some(format!("{}__{}", p_type, p_id))
    }

    fn populate_mapping(&mut self, sps_key: &str) -> Option<&mut SPSMapping> {
        if !self.mappings.contains_key(sps_key) {
            let Some(new_sps_param_obj) = SPSMapping::new(sps_key.to_string()) else {
                return None;
            };

            self.mappings.insert(sps_key.to_string(), new_sps_param_obj);
        }

        self.mappings.get_mut(sps_key)
    }
}
