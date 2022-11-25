
use futures::stream::Scan;
/*
 * Frontend type binding generation
 */
use serde::{Serialize, Deserialize};
use ts_rs::TS;


#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeVibeCheckConfig {
    pub networking: FeOSCNetworking,
    pub scan_on_disconnect: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeOSCNetworking {
    pub bind: String,
    pub remote: String,
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
#[serde(tag="kind", content="data")]
pub enum FeToyEvent {
    Add(FeVCToy),
    Remove(u32)
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub enum FeScanEvent {
    Start,
    Stop
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
#[serde(tag="kind", content="data")]
pub enum FeCoreEvent {
    Scan(FeScanEvent)
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub struct FeVCToy {
    pub toy_id: u32,
    pub toy_name: String,
    pub battery_level: f64,
    pub toy_connected: bool,
    pub features: Vec<FeVCToyFeature>,
    pub listening: bool,
    pub osc_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, TS)]
#[ts(export)]
pub struct FeLevelTweaks {
    pub minimum_level: f64,
    pub maximum_level: f64,
    pub idle_level: f64,
    pub smooth_rate: f64,
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

#[derive(Debug, Deserialize)]
pub enum FeToyAlter {
    Feature(FeVCToyFeature),
    OSCData(bool),
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