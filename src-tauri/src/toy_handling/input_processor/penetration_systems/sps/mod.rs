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
     * There seems to be two parameter leafs that are considered 'Legacy' (*OthersClose) & (*Others) + few more
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

    fn process(&mut self, addr: &str, input: ModeProcessorInputType) -> Option<f64> {
        // Don't support booleans
        let ModeProcessorInputType::Float(float_input) = input else {
            return None;
        };

        // Strip away VRChat avatar parameter prefix
        let sps_param = addr.strip_prefix("/avatar/parameters/")?;

        // Process SPS Param object key
        let (sps_key, sps_leaf) = SPSProcessor::get_sps_param_key_leaf(sps_param)?;

        // Process parameter and create or get mutable ref to mapping
        let mapping = self.populate_mapping(&sps_key, &sps_leaf, float_input)?;

        let others = match sps_leaf.as_str() {
            "PenOthersNewRoot" | "PenOthersNewTip" => true,
            "PenSelfNewRoot" | "PenSelfNewTip" => false,
            _ => return None,
        };

        mapping.update_mapping_length_values(others);
        mapping.update_mapping_length(others);
        mapping.update_level(others)
    }
}

impl SPSProcessor {
    fn get_sps_param_key_leaf(osc_addr: &str) -> Option<(String, String)> {
        let sps_param_split = osc_addr.split('/').collect::<Vec<&str>>();

        if sps_param_split.len() != 4 {
            return None;
        };

        // Orf/Pen/etc.
        let p_type = sps_param_split[1];
        // Unity Object name
        let p_id = sps_param_split[2];
        // SPS Key
        Some((
            format!("{}__{}", p_type, p_id),
            sps_param_split[3].to_string(),
        ))
    }

    fn populate_mapping(
        &mut self,
        sps_key: &str,
        sps_leaf: &str,
        float_value: f64,
    ) -> Option<&mut SPSMapping> {
        if !self.mappings.contains_key(sps_key) {
            let Some(new_sps_param_obj) = SPSMapping::new(sps_key.to_string()) else {
                return None;
            };

            self.mappings.insert(sps_key.to_string(), new_sps_param_obj);
        }

        let mut mapping = self.mappings.get_mut(sps_key);

        mapping
            .as_mut()
            .unwrap()
            .add_osc_value(sps_leaf.to_string(), float_value);

        mapping
    }
}
