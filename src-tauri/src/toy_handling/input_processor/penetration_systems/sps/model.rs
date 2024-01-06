use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub enum SPSParameterType {
    Orf,
    Pen,
    Touch,
}

impl ToString for SPSParameterType {
    fn to_string(&self) -> String {
        match self {
            Self::Orf => "Orf".to_string(),
            Self::Pen => "Pen".to_string(),
            Self::Touch => "Touch".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct SPSParameter {
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

impl SPSParameter {
    pub fn new(param: String) -> Option<Self> {
        // Parse out the parts of SPS parameter
        let param_split = param
            .split("/")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // Potato check
        if param_split.len() != 4 {
            return None;
        }

        let param_type = param_split[1].to_owned();
        let param_obj_id = param_split[2].to_owned();
        let param_contact_type = param_split.last().unwrap().to_owned();

        Some(Self {
            param_type,
            param_obj_id,
            param_contact_type,
            related_osc_values: HashMap::new(),
        })
    }
}
