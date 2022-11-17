/*
 * Frontend type binding generation
 */
use serde::{Serialize, Deserialize};
use ts_rs::TS;

#[derive(Serialize, TS)]
#[ts(export)]
pub struct FeVCToy {
    pub toy_id: u32,
    pub toy_name: String,
    pub battery_level: f64,
    pub toy_connected: bool,
    //pub osc_params_list: Vec<String>,
    pub param_feature_map: FeFeatureParamMap,
    pub listening: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, TS)]
#[ts(export)]
pub struct FeLevelTweaks {
    pub minimum_level: f64,
    pub maximum_level: f64,
    pub idle_level: f64,
    pub smooth_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeFeatureParamMap {
    pub features: Vec<FeVCToyFeature>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeVCToyFeature {

    pub feature_enabled: bool,
    pub feature_type: FeVCFeatureType,
    pub osc_parameter: String,
    pub feature_index: u32,
    pub feature_levels: FeLevelTweaks,
    pub smooth_enabled: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, TS)]
#[ts(export)]
pub enum FeVCFeatureType {
    Vibrator,
    Rotator,
    Linear,
    Oscillate,
    Constrict,
    Inflate,
    Position,
}