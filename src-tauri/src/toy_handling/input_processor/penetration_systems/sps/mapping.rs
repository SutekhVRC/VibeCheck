use log::{debug, trace};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{cmp::Ordering, collections::HashMap};
use strum::{Display, EnumString};
use ts_rs::TS;

use crate::toy_handling::ModeProcessorInputType;

use super::SPSWho;

const SAVED_LENGTH_VALUES_MAX: usize = 8;
const SAVED_LENGTH_VALUES_MIN: usize = 4;

#[derive(Clone, Debug, Serialize, Deserialize, TS, Display, EnumString)]
pub enum SPSParameterType {
    Orf,
    Pen,
    Touch,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct SPSMapping {
    // Orf || Pen || Touch
    param_type: SPSParameterType,
    // The mesh name / identifier for the orifice or penetrator (blowjob/anal/etc.)
    param_obj_id: String,

    length_values_others: Vec<f64>,
    others_stored_length: f64,
    length_values_self: Vec<f64>,
    self_stored_length: f64,

    pub others_touch_enabled: bool,
    pub self_touch_enabled: bool,
    // K: Contact type | V: OSCValue
    // Values are leaf param's values
    osc_values: HashMap<String, ModeProcessorInputType>,
}

impl SPSMapping {
    pub fn new(sps_type: String, sps_obj_id: String) -> Option<Self> {
        let param_type =
            SPSParameterType::from_str(&sps_type).expect("parameter_type convert enum string");
        let param_obj_id = sps_obj_id;

        Some(Self {
            param_type,
            param_obj_id,
            length_values_others: Vec::with_capacity(SAVED_LENGTH_VALUES_MAX + 1),
            others_stored_length: 0.,
            length_values_self: Vec::with_capacity(SAVED_LENGTH_VALUES_MAX + 1),
            self_stored_length: 0.,
            others_touch_enabled: false,
            self_touch_enabled: false,
            osc_values: HashMap::new(),
        })
    }

    fn get_root_tip_osc_values(&self, others: SPSWho) -> Option<(f64, f64)> {
        let values: (f64, f64) = if let SPSWho::Others = others {
            let root_value = self.osc_values.get("PenOthersNewRoot")?.try_float()?;
            let tip_value = self.osc_values.get("PenOthersNewTip")?.try_float()?;
            (root_value, tip_value)
        } else {
            let root_value = self.osc_values.get("PenSelfNewRoot")?.try_float()?;
            let tip_value = self.osc_values.get("PenSelfNewTip")?.try_float()?;
            (root_value, tip_value)
        };

        Some(values)
    }

    pub fn is_touch(&self) -> bool {
        if self.others_touch_enabled | self.self_touch_enabled {
            return true;
        }
        false
    }

    pub fn get_touch_value(&self, others: SPSWho) -> Option<f64> {
        if !self.is_touch() {
            return None;
        }

        if let SPSWho::Others = others {
            self.osc_values.get("TouchOthers")?.try_float()
        } else {
            self.osc_values.get("TouchSelf")?.try_float()
        }
    }

    pub fn add_osc_value(&mut self, leaf: String, value: ModeProcessorInputType) {
        self.osc_values.insert(leaf, value);
    }

    pub fn update_mapping_length_values(&mut self, others: SPSWho) {
        // Logic needs to be optimized (polymorphism/whatever)
        let Some((root_value, tip_value)) = self.get_root_tip_osc_values(others) else {
            debug!("Failed to get root & tip values!");
            if let SPSWho::Others = others {
                self.length_values_others.clear();
            } else {
                self.length_values_self.clear();
            }
            return;
        };
        trace!("Got root & tip values");

        if root_value < 0.01 || tip_value < 0.01 {
            if let SPSWho::Others = others {
                self.length_values_others.clear();
            } else {
                self.length_values_self.clear();
            }
        }

        if root_value > 0.95 {
            return;
        }

        let temp_length = tip_value - root_value;
        debug!("Length Calculation: {}", temp_length);

        if temp_length < 0.02 {
            return;
        }

        if tip_value > 0.99 {
            // Use last value
            // Should we have a bad length ?
            // No need for bad length just reuse last ??
        } else {
            if let SPSWho::Others = others {
                self.length_values_others.insert(0, temp_length);
                self.length_values_others.truncate(SAVED_LENGTH_VALUES_MAX);
                debug!("Added length value");
            } else {
                self.length_values_self.insert(0, temp_length);
                self.length_values_self.truncate(SAVED_LENGTH_VALUES_MAX);
            }
        }
    }

    pub fn update_mapping_length(&mut self, others: SPSWho) {
        let values_len = if let SPSWho::Others = others {
            self.length_values_others.len()
        } else {
            self.length_values_self.len()
        };
        // Enough stored length values?
        if values_len < SAVED_LENGTH_VALUES_MIN {
            // Just don't update length? Or give a bad length?
            return;
        }

        if let SPSWho::Others = others {
            self.length_values_others
                .sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));

            let stacks: Vec<&[f64]> = self.length_values_others.windows(2).collect();

            let mut smallest_diff = 1.;
            let mut iterator = 0;

            for stack in &stacks {
                let diff = (stack[1] - stack[0]).abs();
                if diff < smallest_diff {
                    smallest_diff = diff;
                    iterator += 1;
                }
            }

            self.others_stored_length = stacks[iterator][0];
            debug!("Length Calculated! {}", self.others_stored_length);
        } else {
            self.length_values_self
                .sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Less));

            let stacks: Vec<&[f64]> = self.length_values_self.windows(2).collect();

            let mut smallest_diff = 1.;
            let mut iterator = 0;

            for stack in &stacks {
                let diff = (stack[1] - stack[0]).abs();
                if diff < smallest_diff {
                    smallest_diff = diff;
                    iterator += 1;
                }
            }

            self.self_stored_length = stacks[iterator][0];
            debug!("Length Calculated! {}", self.self_stored_length);
        }
    }

    pub fn update_level(&mut self, others: SPSWho) -> Option<f64> {
        if self.is_touch() {
            let new_touch_value = self.get_touch_value(others)?;
            debug!("TOUCH VALUE: {}", new_touch_value);
            return Some(new_touch_value);
        }

        let (root_value, tip_value) = self.get_root_tip_osc_values(others)?;
        let stored_length = if let SPSWho::Others = others {
            self.others_stored_length
        } else {
            self.self_stored_length
        };
        debug!("Stored Length: {}", stored_length);

        if stored_length > 0. && tip_value > 0.99 {
            let active_length = 1. - root_value;
            let active_ratio = active_length / stored_length;
            let level = 1. - active_ratio;
            debug!("SPS Calculated Level: {}", level);
            return Some(level);
        }

        debug!("Did not update level!");
        None
    }
}
