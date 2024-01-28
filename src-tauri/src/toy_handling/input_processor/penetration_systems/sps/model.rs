use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::string::ToString;
use strum::{Display, EnumString};
use ts_rs::TS;

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
    original_parameter: String,
    // Orf || Pen || Touch
    param_type: SPSParameterType,
    // The mesh name / identifier for the orifice or penetrator (blowjob/anal/etc.)
    param_obj_id: String,

    length_values_others: Vec<f64>,
    others_stored_length: f64,
    length_values_self: Vec<f64>,
    self_stored_length: f64,

    // K: Contact type | V: OSCValue
    // Values are leaf param's values
    osc_values: HashMap<String, f64>,
}

impl SPSMapping {
    pub fn new(param: String) -> Option<Self> {
        // Parse out the parts of SPS parameter
        let param_split = param
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // Potato check that parameter is valid SPS parameter
        if param_split.len() != 4 {
            return None;
        }

        let param_type = SPSParameterType::from_str(param_split[1].as_str())
            .expect("parameter_type convert enum string");
        let param_obj_id = param_split[2].to_owned();

        Some(Self {
            original_parameter: param,
            param_type,
            param_obj_id,
            length_values_others: Vec::with_capacity(SAVED_LENGTH_VALUES_MAX + 1),
            others_stored_length: 0.,
            length_values_self: Vec::with_capacity(SAVED_LENGTH_VALUES_MAX + 1),
            self_stored_length: 0.,
            osc_values: HashMap::new(),
        })
    }

    fn get_root_tip_osc_values(&self, others: bool) -> Option<(f64, f64)> {
        let values: (f64, f64) = if others {
            let Some(root_value) = self.osc_values.get("PenOthersNewRoot") else {
                return None;
            };

            let Some(tip_value) = self.osc_values.get("PenOthersNewTip") else {
                return None;
            };

            (*root_value, *tip_value)
        } else {
            let Some(root_value) = self.osc_values.get("PenSelfNewRoot") else {
                return None;
            };

            let Some(tip_value) = self.osc_values.get("PenSelfNewTip") else {
                return None;
            };

            (*root_value, *tip_value)
        };

        Some(values)
    }

    pub fn add_osc_value(&mut self, leaf: String, value: f64) {
        self.osc_values.insert(leaf, value);
    }

    pub fn update_mapping_length_values(&mut self, others: bool) {
        // Logic needs to be optimized (polymorphism/whatever)

        if others {
            let Some((root_value, tip_value)) = self.get_root_tip_osc_values(others) else {
                return;
            };

            if root_value < 0.01 || tip_value < 0.01 {
                self.length_values_others.clear();
            }

            if root_value > 0.95 {
                return;
            }

            let temp_length = tip_value - root_value;

            if temp_length < 0.02 {
                return;
            }

            if tip_value > 0.99 {
                todo!()
            } else {
                self.length_values_others.insert(0, temp_length);
                self.length_values_others.truncate(SAVED_LENGTH_VALUES_MAX);
            }
        } else {
            let Some((root_value, tip_value)) = self.get_root_tip_osc_values(others) else {
                return;
            };

            if root_value < 0.01 || tip_value < 0.01 {
                self.length_values_self.clear();
            }

            if root_value > 0.95 {
                return;
            }

            let temp_length = tip_value - root_value;

            if temp_length < 0.02 {
                return;
            }

            if tip_value > 0.99 {
                todo!()
            } else {
                self.length_values_self.insert(0, temp_length);
                self.length_values_self.truncate(SAVED_LENGTH_VALUES_MAX);
            }
        }
    }

    pub fn update_mapping_length(&mut self, others: bool) {
        if others {
            // Enough stored length values?
            if self.length_values_others.len() < SAVED_LENGTH_VALUES_MIN {
                // Just don't update length?
                return;
            }
        } else {
            // Enough stored length values?
            if self.length_values_self.len() < SAVED_LENGTH_VALUES_MIN {
                // Just don't update length?
                return;
            }
        }
    }

    pub fn update_level(&mut self, others: bool) -> Option<f64> {
        let (root_value, tip_value) = self.get_root_tip_osc_values(others)?;
        if others {
            if self.others_stored_length > 0. && tip_value > 0.99 {
                let active_length = 1. - root_value;
                let active_ratio = active_length / self.others_stored_length;
                let level = 1. - active_ratio;
                return Some(level);
            }
        } else {
            if self.self_stored_length > 0. && tip_value > 0.99 {
                let active_length = 1. - root_value;
                let active_ratio = active_length / self.self_stored_length;
                let level = 1. - active_ratio;
                return Some(level);
            }
        }

        None
    }
}
