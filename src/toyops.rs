use buttplug::{core::message::{ActuatorType, ClientDeviceMessageAttributes}, client::ButtplugClientDevice};
use serde::{Serialize, Deserialize};
use core::fmt;
use std::{collections::HashMap, sync::{mpsc::Sender, Arc}};

use crate::{ui::{ToyManagementEvent, ToyUpdate}, config::save_toy_config};


#[derive(Clone, Debug)]
pub struct VCToy {
    pub toy_id: u32,
    pub toy_name: String,
    pub battery_level: f64,
    pub toy_connected: bool,
    pub toy_features: ClientDeviceMessageAttributes,
    pub osc_params_list: Vec<String>,
    pub param_feature_map: FeatureParamMap,
    pub listening: bool,
    pub device_handle: Arc<ButtplugClientDevice>,
}

impl VCToy {

    fn populate_routine(&mut self) {

        let features = self.toy_features.clone();
        println!(
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
        }

        if features.rotate_cmd().is_some() {
            let mut indexer = 0;
            features.rotate_cmd().as_ref().unwrap().iter().for_each(|_rotate_feature| {
                
                self.param_feature_map.features.push(VCToyFeature::new(format!("/avatar/parameters/{:?}_{}", VCFeatureType::Rotator, indexer), indexer, VCFeatureType::Rotator));
                indexer += 1;
            });
        }

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
        }
        // Save toy on first time add
        save_toy_config(&self.toy_name, self.param_feature_map.clone());
    }

    pub fn populate_toy_feature_param_map(&mut self, param_feature_map: Option<FeatureParamMap>) {
        // If a param feature map is passed, configure the toy
        // If None is passed, set toy to loaded map
        match param_feature_map {
            Some(map) => {
                // If feature count differs the user probably swapped between connection types
                let mut conn_toy_feature_len = 0;

                if self.toy_features.scalar_cmd().is_some() {
                    conn_toy_feature_len += self.toy_features.scalar_cmd().as_ref().unwrap().iter().len();
                }

                if self.toy_features.rotate_cmd().is_some() {
                    conn_toy_feature_len += self.toy_features.rotate_cmd().as_ref().unwrap().iter().len();
                }

                if self.toy_features.linear_cmd().is_some() {
                    conn_toy_feature_len += self.toy_features.linear_cmd().as_ref().unwrap().iter().len();
                }

                if conn_toy_feature_len != map.features.len() {
                    self.populate_routine();
                    return;
                }
                // Feature count is the same so its probably safe to assume the toy config is intact
                self.param_feature_map = map;
            },
            None => {
                self.populate_routine();
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VCToyFeature {

    pub feature_enabled: bool,

    pub feature_type: VCFeatureType,

    pub osc_parameter: String,

    pub feature_index: u32,

    pub feature_levels: LevelTweaks,

    pub smooth_enabled: bool,
    pub smooth_entries: Vec<f64>,

    pub saved: bool,
}

impl VCToyFeature {
    fn new(osc_parameter: String, feature_index: u32, feature_type: VCFeatureType) -> Self {
        VCToyFeature { feature_enabled: true, feature_type, osc_parameter, feature_index, feature_levels: LevelTweaks::default(), smooth_enabled: true, smooth_entries: Vec::new(), saved: true }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq)]
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

#[derive(Serialize, Deserialize)]
pub struct ToyConfig {
    pub toy_feature_map: HashMap<String, VCToyFeature>,
}

pub fn alter_toy(tme_send: &Sender<ToyManagementEvent>, altered_toy: VCToy) {
    let _ = tme_send.send(ToyManagementEvent::Tu(ToyUpdate::AlterToy(altered_toy)));
}

/*
    Parse configs of toy names and populate toys on ToyAdd
    If no config put toy to Auto params
*/


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub struct LevelTweaks {
    pub minimum_level: f64,
    pub maximum_level: f64,
    pub idle_level: f64,
    pub smooth_rate: f64,
}

impl Default for LevelTweaks {
    fn default() -> Self {
        LevelTweaks { minimum_level: 0., maximum_level: 100., idle_level: 0., smooth_rate: 2.0 }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}