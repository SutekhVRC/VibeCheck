use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{cmp::Ordering, collections::HashMap};
use strum::{Display, EnumString};
use ts_rs::TS;

use crate::toy_handling::ModeProcessorInputType;

use super::SPSWho;

const SAVED_LENGTH_VALUES_MAX: usize = 8;
const SAVED_LENGTH_VALUES_MIN: usize = 4;

// Orifice's (TYPE,LEAF)
const ORF_TOUCHOTHERSCLOSE: (SPSParameterType, &str) = (SPSParameterType::Orf, "TouchOthersClose");
const ORF_TOUCHSELFCLOSE: (SPSParameterType, &str) = (SPSParameterType::Orf, "TouchSelfClose");
const ORF_TOUCHSELF: (SPSParameterType, &str) = (SPSParameterType::Orf, "TouchSelf");
const ORF_TOUCHOTHERS: (SPSParameterType, &str) = (SPSParameterType::Orf, "TouchOthers");
const ORF_PENSELF: (SPSParameterType, &str) = (SPSParameterType::Orf, "PenSelf");
const ORF_PENOTHERSCLOSE: (SPSParameterType, &str) = (SPSParameterType::Orf, "PenOthersClose");
const ORF_PENOTHERS: (SPSParameterType, &str) = (SPSParameterType::Orf, "PenOthers");
const ORF_FROTOTHERS: (SPSParameterType, &str) = (SPSParameterType::Orf, "FrotOthers");
const ORF_PENSELFNEWROOT: (SPSParameterType, &str) = (SPSParameterType::Orf, "PenSelfNewRoot");
const ORF_PENOTHERSNEWROOT: (SPSParameterType, &str) = (SPSParameterType::Orf, "PenOthersNewRoot");
const ORF_PENSELFNEWTIP: (SPSParameterType, &str) = (SPSParameterType::Orf, "PenSelfNewTip");
const ORF_PENOTHERSNEWTIP: (SPSParameterType, &str) = (SPSParameterType::Orf, "PenOthersNewTip");

// Penetrator's (TYPE,LEAF)
const PEN_TOUCHSELFCLOSE: (SPSParameterType, &str) = (SPSParameterType::Pen, "TouchSelfClose");
const PEN_TOUCHOTHERSCLOSE: (SPSParameterType, &str) = (SPSParameterType::Pen, "TouchOthersClose");
const PEN_TOUCHSELF: (SPSParameterType, &str) = (SPSParameterType::Pen, "TouchSelf");
const PEN_TOUCHOTHERS: (SPSParameterType, &str) = (SPSParameterType::Pen, "TouchOthers");
const PEN_PENSELF: (SPSParameterType, &str) = (SPSParameterType::Pen, "PenSelf");
const PEN_PENOTHERS: (SPSParameterType, &str) = (SPSParameterType::Pen, "PenOthers");
const PEN_FROTOTHERSCLOSE: (SPSParameterType, &str) = (SPSParameterType::Pen, "FrotOthersClose");
const PEN_FROTOTHERS: (SPSParameterType, &str) = (SPSParameterType::Pen, "FrotOthers");

// Touch's (TYPE,LEAF)
const TOUCH_SELF: (SPSParameterType, &str) = (SPSParameterType::Touch, "Self");
const TOUCH_OTHERS: (SPSParameterType, &str) = (SPSParameterType::Touch, "Others");

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS, Display, EnumString, PartialEq, Eq)]
pub enum SPSParameterType {
    Orf,
    Pen,
    Touch,
}

impl SPSParameterType {
    pub fn is_orf(&self) -> bool {
        match self {
            Self::Orf => true,
            _ => false,
        }
    }
    pub fn is_pen(&self) -> bool {
        match self {
            Self::Pen => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct SPSMapping {
    // Orf || Pen || Touch
    pub param_type: SPSParameterType,
    // The mesh name / identifier for the orifice or penetrator (blowjob/anal/etc.)
    param_obj_id: String,

    length_values_others: Vec<f64>,
    others_stored_length: f64,
    length_values_self: Vec<f64>,
    self_stored_length: f64,

    pub others_touch_enabled: bool,
    pub self_touch_enabled: bool,
    pub legacy_orf_enabled: bool,
    pub pen_frot_others: bool,
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
            legacy_orf_enabled: false,
            pen_frot_others: false,
            osc_values: HashMap::new(),
        })
    }

    pub fn parse_features_get_who(&mut self, leaf: &str, input: ModeProcessorInputType) -> SPSWho {
        let support_map = (self.param_type, leaf);

        let others: SPSWho = match support_map {
            /*
             * -= Unsupported =-
             *
             * Self
             * Others
             */
            // -= "New SPS tip/root" =-
            ORF_PENOTHERSNEWROOT | ORF_PENOTHERSNEWTIP => SPSWho::Others,
            ORF_PENSELFNEWROOT | ORF_PENSELFNEWTIP => SPSWho::_Self,
            // -= SPS Frot =-
            PEN_FROTOTHERSCLOSE => {
                let Some(b) = input.try_bool() else {
                    return SPSWho::Pass;
                };

                if b {
                    self.pen_frot_others = true;
                } else {
                    self.pen_frot_others = false;
                    return SPSWho::Stop;
                }

                SPSWho::Others
            }
            ORF_FROTOTHERS => SPSWho::Bypass(input.try_float()),
            PEN_FROTOTHERS => {
                if self.pen_frot_others {
                    // Is frotting others
                    SPSWho::Others
                } else {
                    // Is not frotting others
                    SPSWho::Pass
                }
            }
            // -= SPS Legacy =-
            ORF_PENOTHERSCLOSE => {
                // Only applies to orifices.. Pen type only uses PenOthers/PenSelf and not length calcs

                let Some(b) = input.try_bool() else {
                    return SPSWho::Pass;
                };

                if b {
                    self.legacy_orf_enabled = true;
                } else {
                    self.legacy_orf_enabled = false;
                    return SPSWho::Stop;
                }
                SPSWho::Others
            }
            ORF_PENSELF | PEN_PENSELF => SPSWho::_Self,
            PEN_PENOTHERS | ORF_PENOTHERS => SPSWho::Others,
            // -= SPS Touch =-
            ORF_TOUCHOTHERSCLOSE | PEN_TOUCHOTHERSCLOSE => {
                // If this parameter is not a bool then skip and return None
                let Some(b) = input.try_bool() else {
                    return SPSWho::Pass;
                };

                if b {
                    self.others_touch_enabled = true;
                } else {
                    self.others_touch_enabled = false;
                    return SPSWho::Stop;
                }

                SPSWho::Others
            }
            ORF_TOUCHSELFCLOSE | PEN_TOUCHSELFCLOSE => {
                // If this parameter is not a bool then skip and return None
                let Some(b) = input.try_bool() else {
                    // Parameter was wrong type so pass calculation
                    return SPSWho::Pass;
                };

                if b {
                    self.self_touch_enabled = true;
                } else {
                    self.self_touch_enabled = false;
                    return SPSWho::Stop;
                }

                SPSWho::_Self
            }
            ORF_TOUCHOTHERS | PEN_TOUCHOTHERS => SPSWho::Others,
            ORF_TOUCHSELF | PEN_TOUCHSELF => SPSWho::_Self,
            TOUCH_SELF => SPSWho::_Self,
            TOUCH_OTHERS => SPSWho::Others,
            _ => {
                warn!(
                    "No Leaf Match for Other | Self - Unhandled OGB parameter?: {}",
                    leaf
                );
                SPSWho::Pass
            }
        };

        debug!(
            "WHO: {:?} | TOUCH: O{}/S{}",
            others, self.others_touch_enabled, self.self_touch_enabled
        );
        others
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

    pub fn is_frot(&self) -> bool {
        self.pen_frot_others
    }

    pub fn is_legacy_orf(&self) -> bool {
        if self.legacy_orf_enabled {
            if let SPSParameterType::Orf = self.param_type {
                return true;
            }
        }
        false
    }

    pub fn get_frot_value(&self) -> Option<f64> {
        /*
        if !self.is_frot() {
            return None;
        }*/

        // There is no FrotSelf
        self.osc_values.get("FrotOthers")?.try_float()
    }

    pub fn get_touch_value(&self, others: SPSWho) -> Option<f64> {
        /*
        if !self.is_touch() {
            return None;
        }*/

        if let SPSWho::Others = others {
            self.osc_values.get("TouchOthers")?.try_float()
        } else {
            self.osc_values.get("TouchSelf")?.try_float()
        }
    }

    pub fn get_legacy_value(&self, others: SPSWho) -> Option<f64> {
        if let SPSWho::Others = others {
            self.osc_values.get("PenOthers")?.try_float()
        } else {
            self.osc_values.get("PenSelf")?.try_float()
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

            self.others_stored_length = stacks[iterator - 1][0];
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
        // If mapping is in touch mode
        if self.is_touch() {
            let new_touch_value = self.get_touch_value(others)?;
            debug!("TOUCH VALUE: {}", new_touch_value);
            return Some(new_touch_value);
        }

        // If mapping is in frot mode (NO OTHER)
        if self.is_frot() {
            let new_frot_value = self.get_frot_value()?;
            debug!("FROT VALUE: {}", new_frot_value);
            return Some(new_frot_value);
        }

        // Mapping is legacy orifice or mapping is a Pen type
        // I will assume here that legacy orf mode applies to PenOthers & PenSelf
        // Assuming that when someone is PenSelf they are using the Self Tip/Root method
        if self.is_legacy_orf() | self.param_type.is_pen() {
            let new_legacy_value = self.get_legacy_value(others)?;
            debug!("LEGACY VALUE: {}", new_legacy_value);
            return Some(new_legacy_value);
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
