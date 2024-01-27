use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::string::ToString;
use strum::{Display, EnumString};
use ts_rs::TS;

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
    // contact type (PenOthersNewRoot / PenOthersNewTip)
    param_contact_type: String,
    // K: Contact type | V: OSCValue
    // Values are leaf params
    related_osc_values: HashMap<String, f64>,
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
        let param_contact_type = param_split.last().unwrap().to_owned();

        Some(Self {
            original_parameter: param,
            param_type,
            param_obj_id,
            param_contact_type,
            related_osc_values: HashMap::new(),
        })
    }

    pub fn get_probable_penetrator_length(&mut self) -> f64 {
        todo!()
    }
}
