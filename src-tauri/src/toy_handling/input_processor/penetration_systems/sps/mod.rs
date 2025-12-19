pub mod mapping;

use std::collections::HashMap;

use log::{trace, warn};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::toy_handling::{
    input_processor::InputProcessor, mode_processor::core::ModeProcessorInputType,
};

use self::mapping::SPSMapping;

#[derive(Debug, Copy, Clone)]
pub enum SPSWho {
    Others,
    _Self,
    Pass,
    Bypass(Option<f64>),
    Stop, // For when a boolean param is zeroed we need to zero the power due to in game inaccuracies
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, TS)]
pub struct SPSProcessor {
    mappings: HashMap<String, SPSMapping>,
}

impl InputProcessor for SPSProcessor {
    fn is_parameter(&self, param: &str) -> bool {
        param.starts_with("/avatar/parameters/OGB/")
    }

    fn input_filter(&self, param: &str, filters: &Vec<String>) -> bool {
        filters.iter().any(|filter_element| {
            param.contains(filter_element)
        })
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
     * calculate length from samples -> if less than 4 samples use old / bad sample if <4
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
        //let ModeProcessorInputType::Float(float_input) = input else {
        //    return None;
        //};

        // Strip away VRChat avatar parameter prefix
        let sps_param = addr.strip_prefix("/avatar/parameters/")?;
        trace!("SPS Param: {}", sps_param);

        // Process SPS Param object key
        //let (sps_key, sps_leaf, sps_type) = SPSProcessor::get_sps_param_key_leaf(sps_param)?;
        //debug!("SPS Key: {} | SPS Leaf: {}", sps_key, sps_leaf);

        // Process parameter and create or get mutable ref to mapping
        let (mapping, _sps_type, leaf) = self.populate_mapping(sps_param, input)?;

        let others = mapping.parse_features_get_who(leaf.as_str(), input);

        match others {
            SPSWho::Pass => return None, // OSC Address does not apply to below calculations
            SPSWho::Stop => return Some(0.), // Stop feature's motor due to a bool flag being flipped
            SPSWho::Bypass(l) => return l,
            _ => {}
        }

        if !mapping.is_touch() && !mapping.is_legacy_orf() && !mapping.is_frot() {
            // Add good length calculations to mapping (self/other)
            mapping.update_mapping_length_values(others);
            // Update internal mapping length based on stored length calculations
            mapping.update_mapping_length(others);
        }
        // Get updated feature level (bzz level)
        mapping.update_level(others)
    }
}

impl SPSProcessor {
    fn get_sps_param_parsed(osc_addr: &str) -> Option<(String, String, String, String)> {
        let sps_param_split = osc_addr.split('/').collect::<Vec<&str>>();

        if sps_param_split.len() != 4 {
            return None;
        };

        // Orf/Pen/etc.
        let p_type = sps_param_split[1];

        // Unity Object name
        let p_id = sps_param_split[2];

        // Contact Leaf
        let leaf = sps_param_split[3];

        Some((
            // SPS Key
            format!("{}__{}", p_type, p_id),
            // SPS Type
            p_type.to_string(),
            // SPS Obj ID
            p_id.to_string(),
            // Contact Leaf
            leaf.to_string(),
        ))
    }

    fn populate_mapping(
        &mut self,
        sps_param: &str,
        osc_input_value: ModeProcessorInputType,
    ) -> Option<(&mut SPSMapping, String, String)> {
        let (sps_key, sps_type, _sps_obj_id, sps_leaf) =
            SPSProcessor::get_sps_param_parsed(sps_param)?;

        if !self.mappings.contains_key(&sps_key) {
            let Some(new_sps_param_obj) =
                SPSMapping::new(sps_type.to_string(), sps_type.to_string())
            else {
                warn!("Failed to create mapping!");
                return None;
            };

            self.mappings.insert(sps_key.to_string(), new_sps_param_obj);
        }

        let mut mapping = self.mappings.get_mut(&sps_key);

        mapping
            .as_mut()
            .unwrap()
            .add_osc_value(sps_leaf.to_string(), osc_input_value);

        Some((mapping.unwrap(), sps_type, sps_leaf))
    }
}
