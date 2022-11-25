use buttplug::{core::message::{ActuatorType, ClientDeviceMessageAttributes}, client::ButtplugClientDevice};
use serde::{Serialize, Deserialize};
use tracing::info;
use ts_rs::TS;
use core::fmt;
use std::{collections::HashMap, sync::Arc, fs};

use crate::{config::toy::{VCToyConfig}, frontend_types::{FeVCToyFeature, FeVCFeatureType, FeLevelTweaks}, util::{get_user_home_dir, file_exists}, vcerror};

#[derive(Clone, Debug)]
pub struct VCToy {
    pub toy_id: u32,
    pub toy_name: String,
    pub battery_level: f64,
    pub toy_connected: bool,
    pub toy_features: ClientDeviceMessageAttributes,
    pub param_feature_map: FeatureParamMap,
    pub osc_data: bool,
    pub listening: bool,
    pub device_handle: Arc<ButtplugClientDevice>,
    pub config: Option<VCToyConfig>,
}

impl VCToy {

    // Populate if no config can be read for toy
    fn populate_routine(&mut self) {

        let features = self.toy_features.clone();
        info!(
            "Populating toy: {}",
            self.toy_id,
            //toy.toy_features.len()
        );
        // New algo: Check if exists then iterate
        /*
            - Check CMD type
            - Check Scalar actuator type
        */

        // Populate Linears
        if features.linear_cmd().is_some() {
            let mut indexer = 0;
            features.linear_cmd().as_ref().unwrap().iter().for_each(|_linear_feature| {

                self.param_feature_map.features.push(VCToyFeature::new(format!("/avatar/parameters/{:?}_{}", VCFeatureType::Linear, indexer), indexer, VCFeatureType::Linear));
                indexer += 1;
            });
            info!("Populated {} linears", indexer);
        }

        // Populate rotators
        if features.rotate_cmd().is_some() {
            let mut indexer = 0;
            features.rotate_cmd().as_ref().unwrap().iter().for_each(|_rotate_feature| {
                
                self.param_feature_map.features.push(VCToyFeature::new(format!("/avatar/parameters/{:?}_{}", VCFeatureType::Rotator, indexer), indexer, VCFeatureType::Rotator));
                indexer += 1;
            });
            info!("Populated {} rotators", indexer);
        }

        // Populate scalars
        if features.scalar_cmd().is_some() {
            let mut indexer = 0;
            
            features.scalar_cmd().as_ref().unwrap().iter().for_each(|scalar_feature| {
                
                // Filter out Rotators
                match scalar_feature.actuator_type() {
                    &ActuatorType::Rotate => {},
                    &ActuatorType::Vibrate => self.param_feature_map.features.push(VCToyFeature::new(format!("/avatar/parameters/{:?}_{}", VCFeatureType::Vibrator, indexer), indexer, VCFeatureType::Vibrator)),
                    &ActuatorType::Constrict => self.param_feature_map.features.push(VCToyFeature::new(format!("/avatar/parameters/{:?}_{}", VCFeatureType::Constrict, indexer), indexer, VCFeatureType::Constrict)),
                    &ActuatorType::Inflate => self.param_feature_map.features.push(VCToyFeature::new(format!("/avatar/parameters/{:?}_{}", VCFeatureType::Inflate, indexer), indexer, VCFeatureType::Inflate)),
                    &ActuatorType::Oscillate => self.param_feature_map.features.push(VCToyFeature::new(format!("/avatar/parameters/{:?}_{}", VCFeatureType::Oscillate, indexer), indexer, VCFeatureType::Oscillate)),
                    &ActuatorType::Position => self.param_feature_map.features.push(VCToyFeature::new(format!("/avatar/parameters/{:?}_{}", VCFeatureType::Position, indexer), indexer, VCFeatureType::Position)),
                }
                indexer += 1;
            });
            info!("Populated {} scalars", indexer);
        }
        // Save toy on first time add
        //save_toy_config(&self.toy_name, self.param_feature_map.clone());
        self.config = Some(VCToyConfig { toy_name: self.toy_name.clone(), features: self.param_feature_map.clone(), toy_data: false });
        self.save_toy_config();
    }

    pub fn populate_toy_feature_param_map(&mut self) {

        match self.config {
            // If config is loaded check that its feature count matches the toy that loaded it. Then set the feature map to the one from the config.
            Some(ref conf) => {

                // If feature count differs the user probably swapped between connection types
                let conn_toy_feature_len = self.toy_features.scalar_cmd().as_ref().unwrap().iter().len() + self.toy_features.rotate_cmd().as_ref().iter().len() + self.toy_features.linear_cmd().as_ref().iter().len();
                if conn_toy_feature_len != conf.features.features.len() {
                    self.populate_routine();
                    return;
                }
                
                // Feature count is the same so its probably safe to assume the toy config is intact
                self.param_feature_map = conf.features.clone();
            },
            // If config is not loaded populate the toy
            None => {
                self.populate_routine();
            }
        }
    }

    pub fn load_toy_config(&mut self) -> Result<(), vcerror::backend::VibeCheckToyConfigError> {
        let config_path = format!(
            "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck\\ToyConfigs\\{}.json",
            get_user_home_dir(),
            self.toy_name
        );
    
        if !file_exists(&config_path) {
            self.config = None;
            return Ok(());
        } else {
            let con = fs::read_to_string(config_path).unwrap();
    
            let config: VCToyConfig = match serde_json::from_str(&con) {
                Ok(vc_toy_config) => vc_toy_config,
                Err(_) => {
                    self.config = None;
                    return Err(vcerror::backend::VibeCheckToyConfigError::DeserializeError);
                }
            };
            self.config = Some(config);
            return Ok(());
        }
    }
    
    // Save Toy config by name
    pub fn save_toy_config(&self) {
        let config_path = format!(
            "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck\\ToyConfigs\\{}.json",
            get_user_home_dir(),
            self.toy_name
        );
    
        if let Some(conf) = &self.config {
            if let Ok(json_string) = serde_json::to_string(conf) {
                let _ = fs::write(
                    &config_path,
                    json_string,
                );
                info!("Saved toy config: {}", self.toy_name);
            }
        }
    }        
}

/*
[
    {
        "feature_enabled": true,
        "feature_index": 0,
        "feature_levels": {
            "idle_level": 0,
            "maximum_level": 100,
            "minimum_level": 0,
            "smooth_rate": 2
        },
        "feature_type": "Vibrator",
        "osc_parameter": "/avatar/parameters/vibe",
        "saved": true,
        "smooth_enabled": true,
        "smooth_entries": []
    },
    {
        "feature_enabled": true,
        "feature_index": 1,
        "feature_levels": {
            "idle_level": 0,
            "maximum_level": 100,
            "minimum_level": 0,
            "smooth_rate": 2
        },
        "feature_type": "Vibrator",
        "osc_parameter": "/avatar/parameters/vibe1",
        "saved": true,
        "smooth_enabled": true,
        "smooth_entries": []
    }
]
 */


#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct VCToyFeature {

    pub feature_enabled: bool,

    pub feature_type: VCFeatureType,

    pub osc_parameter: String,

    pub feature_index: u32,

    pub feature_levels: LevelTweaks,

    pub smooth_enabled: bool,
    pub smooth_entries: Vec<f64>,

    //pub saved: bool,
}

impl VCToyFeature {
    fn new(osc_parameter: String, feature_index: u32, feature_type: VCFeatureType) -> Self {
        VCToyFeature { feature_enabled: true, feature_type, osc_parameter, feature_index, feature_levels: LevelTweaks::default(), smooth_enabled: true, smooth_entries: Vec::new() }
    }

    pub fn from_fe(&mut self, fe_feature: FeVCToyFeature) {
        self.feature_enabled = fe_feature.feature_enabled;
        // Not including feature type because the feature type is decided by the Server Core not the frontend user
        // we don't want to allow users to mutate feature types as it could break / make the feature unuseable until restart
        //self.feature_type.from_fe(fe_feature.feature_type);
        self.osc_parameter = fe_feature.osc_parameter;
        self.feature_levels.from_fe(fe_feature.feature_levels);
        self.smooth_enabled = fe_feature.smooth_enabled;
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, TS)]
pub enum VCFeatureType {
    Vibrator,
    Rotator,
    Linear,
    Oscillate,
    Constrict,
    Inflate,
    Position,
}
impl Eq for VCFeatureType {}

impl VCFeatureType {
    #[allow(unused)]// Until need to mutate feature type which will probably never happen
    pub fn from_fe(&mut self, fe_feature_type: FeVCFeatureType) {
        match fe_feature_type {
            FeVCFeatureType::Constrict => *self = Self::Constrict,
            FeVCFeatureType::Inflate => *self = Self::Inflate,
            FeVCFeatureType::Linear => *self = Self::Linear,
            FeVCFeatureType::Oscillate => *self = Self::Oscillate,
            FeVCFeatureType::Position => *self = Self::Position,
            FeVCFeatureType::Rotator => *self = Self::Rotator,
            FeVCFeatureType::Vibrator => *self = Self::Vibrator,
        }
    }

    fn to_fe(&self) -> FeVCFeatureType {
        match self {
            VCFeatureType::Constrict => FeVCFeatureType::Constrict,
            VCFeatureType::Inflate => FeVCFeatureType::Inflate,
            VCFeatureType::Linear => FeVCFeatureType::Linear,
            VCFeatureType::Oscillate => FeVCFeatureType::Oscillate,
            VCFeatureType::Position => FeVCFeatureType::Position,
            VCFeatureType::Rotator => FeVCFeatureType::Rotator,
            VCFeatureType::Vibrator => FeVCFeatureType::Vibrator,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ToyConfig {
    pub toy_feature_map: HashMap<String, VCToyFeature>,
}

/*
    Parse configs of toy names and populate toys on ToyAdd
    If no config put toy to Auto params
*/

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, TS)]
pub struct LevelTweaks {
    pub minimum_level: f64,
    pub maximum_level: f64,
    pub idle_level: f64,
    pub smooth_rate: f64,
}

impl Default for LevelTweaks {
    fn default() -> Self {
        LevelTweaks { minimum_level: 0., maximum_level: 1., idle_level: 0., smooth_rate: 2.0 }
    }
}

impl LevelTweaks {
    pub fn from_fe(&mut self, fe_lt: FeLevelTweaks) {
        self.idle_level = fe_lt.idle_level;
        self.maximum_level = fe_lt.maximum_level;
        self.minimum_level = fe_lt.minimum_level;
        self.smooth_rate = fe_lt.smooth_rate;
    }

    pub fn to_fe(&self) -> FeLevelTweaks {
        FeLevelTweaks {
            minimum_level: self.minimum_level,
            maximum_level: self.maximum_level,
            idle_level: self.idle_level,
            smooth_rate: self.smooth_rate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scalars {
    levels: LevelTweaks,
    actuator_type: ActuatorType,
    feature_id: u32,
    osc_parameter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Rotators {
    Auto(String, LevelTweaks),
    Custom(Vec<(String, u32, LevelTweaks)>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Linears {
    Auto(String, LevelTweaks),
    Custom(Vec<(String, u32, LevelTweaks)>),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
pub struct FeatureParamMap {

    // Vec<(Feature, edit_state_bool)
    pub features: Vec<VCToyFeature>,
}

impl fmt::Display for FeatureParamMap {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {


        write!(f, "")
    }
}

impl FeatureParamMap {
    pub fn new() -> Self {
        FeatureParamMap {
            features: Vec::new()
        }
    }

    pub fn get_features_from_param(&mut self, param: &String) -> Option<Vec<(VCFeatureType, u32, LevelTweaks, bool, &mut Vec<f64>)>> {
        
        let mut parsed_features = vec![];

        // Get each feature assigned to the OSC parameter passed
        for f in &mut self.features {
            if f.feature_enabled {
                if f.osc_parameter == *param {
                    parsed_features.push((f.feature_type, f.feature_index, f.feature_levels, f.smooth_enabled, &mut f.smooth_entries));
                }
            }
        }

        if parsed_features.is_empty() {
            return None;
        } else {
            return Some(parsed_features);
        }
    }

    pub fn from_fe(&mut self, fe_feature: FeVCToyFeature) -> bool {

        let mut success = false;
        self.features.iter_mut().for_each(|f| {
            if f.feature_index == fe_feature.feature_index {
                f.from_fe(fe_feature.clone());
                success = true;
            }
        });
        success
    }

    pub fn to_fe(&self) -> Vec<FeVCToyFeature> {

        let mut fe_features = Vec::new();

        self.features.iter().for_each(|f| {
            fe_features.push(
            FeVCToyFeature {
                feature_enabled: f.feature_enabled,
                feature_type: f.feature_type.to_fe(),
                osc_parameter: f.osc_parameter.clone(),
                feature_index: f.feature_index,
                feature_levels: f.feature_levels.to_fe(),
                smooth_enabled: f.smooth_enabled,
            });
        });

        fe_features
    }
}