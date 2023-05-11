/*
 * Frontend type binding generation
 */
use serde::{Serialize, Deserialize};
use ts_rs::TS;

use crate::toyops::VCFeatureType;


#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeVibeCheckConfig {
    pub networking: FeOSCNetworking,
    pub scan_on_disconnect: bool,
    pub minimize_on_exit: bool,
    pub desktop_notifications: bool,
    pub lc_override: Option<String>,
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
    Remove(u32),
    Update(FeVCToy),
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub enum FeScanEvent {
    Start,
    //Stop
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub enum FeStateEvent {
    EnableAndScan,
    Disable,
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
#[serde(tag="kind", content="data")]
pub enum FeCoreEvent {
    Scan(FeScanEvent),
    State(FeStateEvent),
}

#[derive(Deserialize, Clone, TS)]
#[ts(export)]
pub enum FeSocialLink {
    Github,
    VRChatGroup,
    Discord,
}

#[derive(Serialize, Deserialize, Clone, TS, Debug)]
#[ts(export)]
pub enum FeVCToyAnatomy {
    Anus,
    //Any, I forgot what the point of this was?
    Breasts,
    Buttocks,
    Chest,
    Clitoris,
    Face,
    Feet,
    FootL,
    FootR,
    HandLeft,
    HandRight,
    Hands,
    Labia,
    Mouth,
    NA,
    Nipples,
    Penis,
    Perineum,
    Testicles,
    Vagina,
    Vulva,
    Wrist,
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub struct FeVCToy {
    pub toy_id: u32,
    pub toy_name: String,
    pub toy_anatomy: FeVCToyAnatomy,
    pub battery_level: f64,
    pub toy_connected: bool,
    pub features: Vec<FeVCToyFeature>,
    pub listening: bool,
    pub osc_data: bool,
    pub sub_id: u8,
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
    pub flip_input_float: bool,
    pub feature_levels: FeLevelTweaks,
    pub smooth_enabled: bool,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub enum FeToyAlter {
    Feature(FeVCToyFeature),
    OSCData(bool),
    Anatomy(FeVCToyAnatomy),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, TS)]
#[ts(export)]
pub enum FeVCFeatureType {
    Vibrator = 0,
    Rotator = 1,
    Linear = 2,
    Oscillate = 3,
    Constrict = 4,
    Inflate = 5,
    Position = 6,
    // Note no ScalarRotator bc conversion is done in vcore
}

impl PartialEq<VCFeatureType> for FeVCFeatureType {
    fn eq(&self, other: &VCFeatureType) -> bool {
        *self as u32 == *other as u32
    }

    fn ne(&self, other: &VCFeatureType) -> bool {
        !self.eq(other)
    }
}