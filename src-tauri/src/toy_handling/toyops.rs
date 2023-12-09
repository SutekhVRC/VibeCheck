use buttplug::{
    client::ButtplugClientDevice,
    core::message::{ActuatorType, ClientDeviceMessageAttributes},
};
use core::fmt;
use log::{debug, error as logerr, info, warn};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, sync::Arc, time::Instant};
use ts_rs::TS;

use crate::{
    config::toy::{VCToyAnatomy, VCToyConfig},
    frontend::{
        frontend_types::{
            FeLevelTweaks, FeProcessingMode, FeToyParameter, FeVCFeatureType, FeVCToyFeature,
        },
        FromFrontend, ToBackend, ToFrontend,
    },
    util::fs::{file_exists, get_config_dir},
    vcore::vcerror,
};

use crate::toy_handling::input_processor::penetration_systems::PenetrationSystem;

#[derive(Clone, Debug)]
pub struct VCToy {
    pub toy_id: u32,
    pub toy_name: String,
    pub battery_level: Option<f64>,
    pub toy_connected: bool,
    pub toy_features: ClientDeviceMessageAttributes,
    pub parsed_toy_features: VCToyFeatures,
    pub osc_data: bool,
    pub listening: bool,
    pub device_handle: Arc<ButtplugClientDevice>,
    pub config: Option<VCToyConfig>,
    pub sub_id: u8,
}

impl VCToy {
    fn populate_linears(&mut self, features: &ClientDeviceMessageAttributes) {
        // Populate Linears
        if features.linear_cmd().is_some() {
            let mut indexer = 0;
            features
                .linear_cmd()
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|_linear_feature| {
                    let mut hmap = HashMap::new();

                    // This should ALWAYS be true
                    if let None = hmap.insert(
                        format!("/avatar/parameters/{:?}_{}", VCFeatureType::Linear, indexer),
                        ToyParameter {
                            parameter: format!(
                                "/avatar/parameters/{:?}_{}",
                                VCFeatureType::Linear,
                                indexer
                            ),
                            processing_mode: ProcessingMode::Raw,
                            processing_mode_values: ProcessingModeValues::default(),
                        },
                    ) {
                        self.parsed_toy_features.features.push(VCToyFeature::new(
                            hmap,
                            indexer,
                            VCFeatureType::Linear,
                        ));
                        indexer += 1;
                    }
                });
            info!("Populated {} linears", indexer);
        }
    }

    fn populate_rotators(&mut self, features: &ClientDeviceMessageAttributes) {
        // Populate rotators
        if features.rotate_cmd().is_some() {
            let mut indexer = 0;
            features
                .rotate_cmd()
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|_rotate_feature| {
                    let mut hmap = HashMap::new();

                    // This should ALWAYS be true
                    if let None = hmap.insert(
                        format!(
                            "/avatar/parameters/{:?}_{}",
                            VCFeatureType::Rotator,
                            indexer
                        ),
                        ToyParameter {
                            parameter: format!(
                                "/avatar/parameters/{:?}_{}",
                                VCFeatureType::Rotator,
                                indexer
                            ),
                            processing_mode: ProcessingMode::Raw,
                            processing_mode_values: ProcessingModeValues::default(),
                        },
                    ) {
                        self.parsed_toy_features.features.push(VCToyFeature::new(
                            hmap,
                            indexer,
                            VCFeatureType::Rotator,
                        ));
                        indexer += 1;
                    }
                });
            info!("Populated {} rotators", indexer);
        }
    }

    fn populate_scalars(&mut self, features: &ClientDeviceMessageAttributes) {
        // Populate scalars
        if features.scalar_cmd().is_some() {
            let mut scalar_rotate_indexer = 0;
            let mut scalar_vibrate_indexer = 0;
            let mut scalar_constrict_indexer = 0;
            let mut scalar_inflate_indexer = 0;
            let mut scalar_oscillate_indexer = 0;
            let mut scalar_position_indexer = 0;

            features
                .scalar_cmd()
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|scalar_feature| {
                    // Filter out Rotators
                    match scalar_feature.actuator_type() {
                        &ActuatorType::Rotate => {
                            let mut hmap = HashMap::new();

                            // This should ALWAYS be true
                            if let None = hmap.insert(
                                format!(
                                    "/avatar/parameters/{:?}_{}",
                                    VCFeatureType::ScalarRotator,
                                    scalar_rotate_indexer
                                ),
                                ToyParameter {
                                    parameter: format!(
                                        "/avatar/parameters/{:?}_{}",
                                        VCFeatureType::ScalarRotator,
                                        scalar_rotate_indexer
                                    ),
                                    processing_mode: ProcessingMode::Raw,
                                    processing_mode_values: ProcessingModeValues::default(),
                                },
                            ) {
                                self.parsed_toy_features.features.push(VCToyFeature::new(
                                    hmap,
                                    scalar_rotate_indexer,
                                    VCFeatureType::ScalarRotator,
                                ));
                                scalar_rotate_indexer += 1;
                            }
                        }
                        &ActuatorType::Vibrate => {
                            let mut hmap = HashMap::new();

                            // This should ALWAYS be true
                            if let None = hmap.insert(
                                format!(
                                    "/avatar/parameters/{:?}_{}",
                                    VCFeatureType::Vibrator,
                                    scalar_vibrate_indexer
                                ),
                                ToyParameter {
                                    parameter: format!(
                                        "/avatar/parameters/{:?}_{}",
                                        VCFeatureType::Vibrator,
                                        scalar_rotate_indexer
                                    ),
                                    processing_mode: ProcessingMode::Raw,
                                    processing_mode_values: ProcessingModeValues::default(),
                                },
                            ) {
                                self.parsed_toy_features.features.push(VCToyFeature::new(
                                    hmap,
                                    scalar_vibrate_indexer,
                                    VCFeatureType::Vibrator,
                                ));
                                scalar_vibrate_indexer += 1;
                            }
                        }
                        &ActuatorType::Constrict => {
                            let mut hmap = HashMap::new();

                            // This should ALWAYS be true
                            if let None = hmap.insert(
                                format!(
                                    "/avatar/parameters/{:?}_{}",
                                    VCFeatureType::Constrict,
                                    scalar_constrict_indexer
                                ),
                                ToyParameter {
                                    parameter: format!(
                                        "/avatar/parameters/{:?}_{}",
                                        VCFeatureType::Constrict,
                                        scalar_constrict_indexer
                                    ),
                                    processing_mode: ProcessingMode::Raw,
                                    processing_mode_values: ProcessingModeValues::default(),
                                },
                            ) {
                                self.parsed_toy_features.features.push(VCToyFeature::new(
                                    hmap,
                                    scalar_constrict_indexer,
                                    VCFeatureType::Constrict,
                                ));
                                scalar_constrict_indexer += 1;
                            }
                        }
                        &ActuatorType::Inflate => {
                            let mut hmap = HashMap::new();

                            // This should ALWAYS be true
                            if let None = hmap.insert(
                                format!(
                                    "/avatar/parameters/{:?}_{}",
                                    VCFeatureType::Inflate,
                                    scalar_inflate_indexer
                                ),
                                ToyParameter {
                                    parameter: format!(
                                        "/avatar/parameters/{:?}_{}",
                                        VCFeatureType::Inflate,
                                        scalar_inflate_indexer
                                    ),
                                    processing_mode: ProcessingMode::Raw,
                                    processing_mode_values: ProcessingModeValues::default(),
                                },
                            ) {
                                self.parsed_toy_features.features.push(VCToyFeature::new(
                                    hmap,
                                    scalar_inflate_indexer,
                                    VCFeatureType::Inflate,
                                ));
                                scalar_inflate_indexer += 1;
                            }
                        }
                        &ActuatorType::Oscillate => {
                            let mut hmap = HashMap::new();

                            // This should ALWAYS be true
                            if let None = hmap.insert(
                                format!(
                                    "/avatar/parameters/{:?}_{}",
                                    VCFeatureType::Oscillate,
                                    scalar_oscillate_indexer
                                ),
                                ToyParameter {
                                    parameter: format!(
                                        "/avatar/parameters/{:?}_{}",
                                        VCFeatureType::Oscillate,
                                        scalar_oscillate_indexer
                                    ),
                                    processing_mode: ProcessingMode::Raw,
                                    processing_mode_values: ProcessingModeValues::default(),
                                },
                            ) {
                                self.parsed_toy_features.features.push(VCToyFeature::new(
                                    hmap,
                                    scalar_oscillate_indexer,
                                    VCFeatureType::Oscillate,
                                ));
                                scalar_oscillate_indexer += 1;
                            }
                        }
                        &ActuatorType::Position => {
                            let mut hmap = HashMap::new();

                            // This should ALWAYS be true
                            if let None = hmap.insert(
                                format!(
                                    "/avatar/parameters/{:?}_{}",
                                    VCFeatureType::Position,
                                    scalar_position_indexer
                                ),
                                ToyParameter {
                                    parameter: format!(
                                        "/avatar/parameters/{:?}_{}",
                                        VCFeatureType::Position,
                                        scalar_position_indexer
                                    ),
                                    processing_mode: ProcessingMode::Raw,
                                    processing_mode_values: ProcessingModeValues::default(),
                                },
                            ) {
                                self.parsed_toy_features.features.push(VCToyFeature::new(
                                    hmap,
                                    scalar_position_indexer,
                                    VCFeatureType::Position,
                                ));
                                scalar_position_indexer += 1;
                            }
                        }
                        &ActuatorType::Unknown => {}
                    }
                });
            info!(
                "Populated {} scalars",
                scalar_constrict_indexer
                    + scalar_inflate_indexer
                    + scalar_oscillate_indexer
                    + scalar_position_indexer
                    + scalar_rotate_indexer
                    + scalar_vibrate_indexer
            );
        }
    }

    // Populate if no config can be read for toy
    fn populate_routine(&mut self) {
        info!("Populating toy: {}", self.toy_id,);

        let features = self.toy_features.clone();

        self.populate_linears(&features);
        self.populate_rotators(&features);
        self.populate_scalars(&features);

        self.config = Some(VCToyConfig {
            toy_name: self.toy_name.clone(),
            features: self.parsed_toy_features.clone(),
            osc_data: false,
            anatomy: VCToyAnatomy::default(),
        });
        info!("Set toy config populate defaults");
        // Save toy on first time add
        self.save_toy_config();
    }

    pub fn populate_toy_config(&mut self) {
        match self.config {
            // If config is loaded check that its feature count matches the toy that loaded it. Then set the feature map to the one from the config.
            Some(ref conf) => {
                // If feature count differs the user probably swapped between connection types (This used to be a bug when LC impl in bp-rs wasnt done for the Max2. This was fixed but I am keeping the feature count check in case it happens again)

                let mut conn_toy_feature_count = 0;

                if self.toy_features.scalar_cmd().is_some() {
                    conn_toy_feature_count += self
                        .toy_features
                        .scalar_cmd()
                        .as_ref()
                        .unwrap()
                        .iter()
                        .len();
                }

                if self.toy_features.rotate_cmd().is_some() {
                    conn_toy_feature_count += self
                        .toy_features
                        .rotate_cmd()
                        .as_ref()
                        .unwrap()
                        .iter()
                        .len();
                }

                if self.toy_features.linear_cmd().is_some() {
                    conn_toy_feature_count += self
                        .toy_features
                        .linear_cmd()
                        .as_ref()
                        .unwrap()
                        .iter()
                        .len();
                }

                if conn_toy_feature_count != conf.features.features.len() {
                    self.populate_routine();
                    return;
                }

                // Feature count is the same so its probably safe to assume the toy config is intact
                self.parsed_toy_features = conf.features.clone();
                self.osc_data = conf.osc_data;
                info!("Populated toy with loaded config from file!");
            }
            // If config is not loaded populate the toy
            None => {
                self.populate_routine();
            }
        }
    }

    pub fn load_toy_config(&mut self) -> Result<(), vcerror::backend::VibeCheckToyConfigError> {
        // Generate config path

        let config_path = format!(
            "{}\\ToyConfigs\\{}.json",
            get_config_dir(),
            // - Transform Lovense Connect toys to load lovense configs
            self.toy_name.replace("Lovense Connect ", "Lovense "),
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
            debug!("Loaded & parsed toy config successfully!");
            self.config = Some(config);
            return Ok(());
        }
    }

    // Save Toy config by name
    pub fn save_toy_config(&self) {
        let config_path = format!(
            "{}\\ToyConfigs\\{}.json",
            get_config_dir(),
            self.toy_name.replace("Lovense Connect ", "Lovense "),
        );
        info!("Saving toy config to: {}", config_path);

        if let Some(conf) = &self.config {
            if let Ok(json_string) = serde_json::to_string(conf) {
                match fs::write(&config_path, json_string) {
                    Ok(()) => {
                        info!("Saved toy config: {}", self.toy_name);
                        return;
                    }
                    Err(e) => {
                        logerr!("Failed to write to file: {}", e);
                        return;
                    }
                }
            } else {
                warn!("Failed to serialize config to json");
            }
        } else {
            warn!("save_toy_config() called while toy config is None");
        }
    }

    pub fn mutate_state_by_anatomy(&mut self, anatomy_type: &VCToyAnatomy, value: bool) -> bool {
        if self.config.as_ref().unwrap().anatomy == *anatomy_type {
            self.parsed_toy_features
                .features
                .iter_mut()
                .for_each(|feature| {
                    feature.feature_enabled = value;
                });
            return true;
        }
        return false;
    }
}

#[derive(Clone, Default, Debug)]
pub struct SmoothProcessingValues {
    pub smooth_queue: Vec<f64>,
}

#[derive(Clone, Default, Debug)]
pub struct RateProcessingValues {
    pub rate_saved_level: f64,
    pub rate_saved_osc_input: f64,
    pub rate_timestamp: Option<Instant>,
}

#[derive(Clone, Default, Debug)]
pub enum ProcessingModeValues {
    #[default]
    Raw,
    Smooth(SmoothProcessingValues),
    Rate(RateProcessingValues),
    Constant,
}

impl ProcessingModeValues {
    pub fn new_from(processing_mode: ProcessingMode) -> Self {
        match processing_mode {
            ProcessingMode::Raw => Self::Raw,
            ProcessingMode::Constant => Self::Constant,
            ProcessingMode::Smooth => Self::Smooth(SmoothProcessingValues {
                smooth_queue: Vec::new(),
            }),
            ProcessingMode::Rate => Self::Rate(RateProcessingValues {
                rate_saved_level: 0.0,
                rate_saved_osc_input: 0.0,
                rate_timestamp: None,
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub enum ProcessingMode {
    Raw,
    Smooth,
    Rate,
    Constant,
}

impl ToFrontend<FeProcessingMode> for ProcessingMode {
    type OutputType = FeProcessingMode;

    fn to_frontend(&self) -> Self::OutputType {
        match self {
            Self::Raw => FeProcessingMode::Raw,
            Self::Smooth => FeProcessingMode::Smooth,
            Self::Rate => FeProcessingMode::Rate,
            Self::Constant => FeProcessingMode::Constant,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct ToyParameter {
    pub parameter: String,
    pub processing_mode: ProcessingMode,
    // Temporary values for calculations for Processing Modes
    #[serde(skip)]
    pub processing_mode_values: ProcessingModeValues,
}

impl ToyParameter {
    fn is_assigned_param(&self, param: &String) -> bool {
        if self.parameter == *param {
            true
        } else {
            false
        }
    }
}

impl ToFrontend<Vec<FeToyParameter>> for Vec<ToyParameter> {
    type OutputType = Vec<FeToyParameter>;

    fn to_frontend(&self) -> Self::OutputType {
        let mut out = Vec::new();

        for tp in self {
            out.push(FeToyParameter {
                parameter: tp.parameter.clone(),
                processing_mode: tp.processing_mode.to_frontend(),
            });
        }

        out
    }
}

/*
#[derive(Clone, Debug, Default)]
pub struct FeatureProcessor {


    pub rate_saved_level: f64,
    pub rate_saved_osc_input: f64,
    pub rate_timestamp: Option<Instant>,
}
*/
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct VCToyFeature {
    // The feature index of the toy
    pub feature_index: u32,
    // Is this feature enabled
    pub feature_enabled: bool,
    // The type of feature
    pub feature_type: VCFeatureType,
    // Assigned OSC parameters with their respective processing modes
    pub osc_parameters: HashMap<String, ToyParameter>,
    // The assigned penetration system to the feature
    pub penetration_system: PenetrationSystem,
    // Should toy input be flipped
    pub flip_input_float: bool,
    // Various user defined parameters
    pub feature_levels: LevelTweaks,
    // Smooth mode enabled (This will be removed with multi-param update)
    pub smooth_enabled: bool,
    // Rate mode enabled (This will be removed with multi-param update)
    pub rate_enabled: bool,
}

impl VCToyFeature {
    fn new(
        osc_parameters: HashMap<String, ToyParameter>,
        feature_index: u32,
        feature_type: VCFeatureType,
    ) -> Self {
        VCToyFeature {
            feature_enabled: true,
            feature_type,
            osc_parameters,
            penetration_system: PenetrationSystem::default(),
            feature_index,
            flip_input_float: false,
            feature_levels: LevelTweaks::default(),
            smooth_enabled: true,
            rate_enabled: false,
            //processor: FeatureProcessor::default(),
        }
    }

    pub fn get_enabled_feature_from_param(&mut self, param: &String) -> Option<&mut Self> {
        // Implement parameter priority?
        if self.feature_enabled {
            for osc_param in &mut self.osc_parameters {
                if osc_param.1.is_assigned_param(param) {
                    return Some(self);
                }
            }
        }
        return None;
    }

    /*
    pub fn from_fe(&mut self, fe_feature: FeVCToyFeature) {
        self.feature_enabled = fe_feature.feature_enabled;
        // Not including feature type because the feature type is decided by the Server Core not the frontend user
        // we don't want to allow users to mutate feature types as it could break / make the feature unuseable until restart
        //self.feature_type.from_fe(fe_feature.feature_type);
        self.flip_input_float = fe_feature.flip_input_float;
        self.osc_parameters.from_frontend(fe_feature.osc_parameters);
        self.feature_levels.from_fe(fe_feature.feature_levels);
        self.smooth_enabled = fe_feature.smooth_enabled;
        self.rate_enabled = fe_feature.rate_enabled;
    }*/
}

impl FromFrontend<FeVCToyFeature> for VCToyFeature {
    type OutputType = bool;

    fn from_frontend(&mut self, frontend_type: FeVCToyFeature) -> Self::OutputType {
        self.feature_enabled = frontend_type.feature_enabled;
        // Not including feature type because the feature type is decided by the Server Core not the frontend user
        // we don't want to allow users to mutate feature types as it could break / make the feature unuseable until restart
        //self.feature_type.from_fe(fe_feature.feature_type);
        self.flip_input_float = frontend_type.flip_input_float;
        self.osc_parameters
            .from_frontend(frontend_type.osc_parameters);
        self.feature_levels.from_fe(frontend_type.feature_levels);
        self.smooth_enabled = frontend_type.smooth_enabled;
        self.rate_enabled = frontend_type.rate_enabled;
        true
    }
}

impl FromFrontend<HashMap<String, FeToyParameter>> for HashMap<String, ToyParameter> {
    type OutputType = HashMap<String, ToyParameter>;

    fn from_frontend(
        &mut self,
        frontend_type: HashMap<String, FeToyParameter>,
    ) -> Self::OutputType {
        let mut backend_parameter_map = HashMap::new();
        for (fep, ftp) in frontend_type {
            backend_parameter_map.insert(
                fep,
                ToyParameter {
                    parameter: ftp.parameter,
                    processing_mode: ftp.processing_mode.to_backend(),
                    processing_mode_values: ProcessingModeValues::new_from(
                        ftp.processing_mode.to_backend(),
                    ),
                },
            );
        }

        backend_parameter_map
    }
}

impl ToFrontend<HashMap<String, FeToyParameter>> for HashMap<String, ToyParameter> {
    type OutputType = HashMap<String, FeToyParameter>;

    fn to_frontend(&self) -> Self::OutputType {
        let mut frontend_parameter_map = HashMap::new();

        for (p, tp) in self {
            frontend_parameter_map.insert(
                p.to_owned(),
                FeToyParameter {
                    parameter: p.clone(),
                    processing_mode: tp.processing_mode.to_frontend(),
                },
            );
        }

        frontend_parameter_map
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, TS)]
pub enum VCFeatureType {
    Vibrator = 0,
    Rotator = 1,
    Linear = 2,
    Oscillate = 3,
    Constrict = 4,
    Inflate = 5,
    Position = 6,
    ScalarRotator = 7,
    // Note: no ScalarRotator in FeVCFeatureType bc conversion is done in vcore
    // Fe and Core feature types have different number of values
}
impl Eq for VCFeatureType {}

impl PartialEq<FeVCFeatureType> for VCFeatureType {
    fn eq(&self, other: &FeVCFeatureType) -> bool {
        *self as u32 == *other as u32
    }

    fn ne(&self, other: &FeVCFeatureType) -> bool {
        !self.eq(other)
    }
}

impl VCFeatureType {
    #[allow(unused)] // Until need to mutate feature type which will probably never happen
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
            VCFeatureType::ScalarRotator => FeVCFeatureType::Rotator,
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
    pub linear_position_speed: u32,
    pub rate_tune: f64,
    pub constant_level: f64,
}

impl Default for LevelTweaks {
    fn default() -> Self {
        LevelTweaks {
            minimum_level: 0.,
            maximum_level: 1.,
            idle_level: 0.,
            smooth_rate: 2.,
            linear_position_speed: 100,
            rate_tune: 0.4,
            constant_level: 0.5,
        }
    }
}

impl LevelTweaks {
    pub fn from_fe(&mut self, fe_lt: FeLevelTweaks) {
        self.idle_level = fe_lt.idle_level;
        self.maximum_level = fe_lt.maximum_level;
        self.minimum_level = fe_lt.minimum_level;
        self.smooth_rate = fe_lt.smooth_rate;
        self.linear_position_speed = fe_lt.linear_position_speed;
        self.rate_tune = fe_lt.rate_tune;
    }

    pub fn to_fe(&self) -> FeLevelTweaks {
        FeLevelTweaks {
            minimum_level: self.minimum_level,
            maximum_level: self.maximum_level,
            idle_level: self.idle_level,
            smooth_rate: self.smooth_rate,
            linear_position_speed: self.linear_position_speed,
            rate_tune: self.rate_tune,
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
pub struct VCToyFeatures {
    pub features: Vec<VCToyFeature>,
}

impl fmt::Display for VCToyFeatures {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

impl VCToyFeatures {
    pub fn new() -> Self {
        VCToyFeatures {
            features: Vec::new(),
        }
    }

    pub fn get_features_from_param(&mut self, param: &String) -> Option<Vec<&mut VCToyFeature>> {
        let mut out = Vec::new();

        for f in &mut self.features {
            if let Some(feature) = f.get_enabled_feature_from_param(param) {
                out.push(feature);
            }
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    pub fn get_features_with_penetration_systems(
        &mut self,
        param: &String,
    ) -> Option<Vec<&mut VCToyFeature>> {
        let mut out = Vec::new();

        for f in &mut self.features {
            // If penetration system not set for feature move to next feature
            if let None = f.penetration_system.pen_system {
                continue;
            }

            // Is parameter part of the implemented penetration system?
            if f.penetration_system
                .pen_system
                .as_ref()
                .unwrap()
                .is_parameter(param)
            {
                // Add to features vector for features with a penetration system related to the input parameter
                out.push(f);
            }
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    /*
    pub fn from_fe(&mut self, fe_feature: FeVCToyFeature) -> bool {
        let mut success = false;
        self.features.iter_mut().for_each(|f| {
            info!(
                "Checking Loaded: [{}: {:?}] - Fe: [{}: {:?}]",
                f.feature_index, f.feature_type, fe_feature.feature_index, fe_feature.feature_type
            );
            // Check that the index and type are the same
            // Note that here there is an OR for when the feature type is a ScalarRotator
            // May be a good idea in the future to create Scalar types and then convert the names in the frontend.
            if f.feature_index == fe_feature.feature_index
                && (f.feature_type == fe_feature.feature_type
                    || f.feature_type == VCFeatureType::ScalarRotator
                        && fe_feature.feature_type == FeVCFeatureType::Rotator)
            {
                info!(
                    "FE Object and Loaded Object are Eq: {}: {:?}",
                    f.feature_index, f.feature_type
                );
                f.from_fe(fe_feature.clone());
                success = true;
            }
        });
        success
    }*/

    /*
    pub fn to_fe(&self) -> Vec<FeVCToyFeature> {
        let mut fe_features = Vec::new();

        self.features.iter().for_each(|f| {
            fe_features.push(FeVCToyFeature {
                feature_enabled: f.feature_enabled,
                feature_type: f.feature_type.to_fe(),
                osc_parameters: f.osc_parameters.to_frontend(),
                penetration_system: f.penetration_system,
                feature_index: f.feature_index,
                flip_input_float: f.flip_input_float,
                feature_levels: f.feature_levels.to_fe(),
                smooth_enabled: f.smooth_enabled,
                rate_enabled: f.rate_enabled,
            });
        });

        fe_features
    }*/
}

impl ToFrontend<Vec<FeVCToyFeature>> for Vec<VCToyFeature> {
    type OutputType = Vec<FeVCToyFeature>;
    fn to_frontend(&self) -> Self::OutputType {
        let mut fe_features = Vec::new();

        self.iter().for_each(|f| {
            fe_features.push(FeVCToyFeature {
                feature_enabled: f.feature_enabled,
                feature_type: f.feature_type.to_fe(),
                osc_parameters: f.osc_parameters.to_frontend(),
                penetration_system: f.penetration_system.to_frontend(),
                feature_index: f.feature_index,
                flip_input_float: f.flip_input_float,
                feature_levels: f.feature_levels.to_fe(),
                smooth_enabled: f.smooth_enabled,
                rate_enabled: f.rate_enabled,
            });
        });
        fe_features
    }
}

impl FromFrontend<FeVCToyFeature> for VCToyFeatures {
    type OutputType = bool;

    fn from_frontend(&mut self, frontend_type: FeVCToyFeature) -> Self::OutputType {
        let mut success = false;
        self.features.iter_mut().for_each(|f| {
            info!(
                "Checking Loaded: [{}: {:?}] - Fe: [{}: {:?}]",
                f.feature_index,
                f.feature_type,
                frontend_type.feature_index,
                frontend_type.feature_type
            );
            // Check that the index and type are the same
            // Note that here there is an OR for when the feature type is a ScalarRotator
            // May be a good idea in the future to create Scalar types and then convert the names in the frontend.
            if f.feature_index == frontend_type.feature_index
                && (f.feature_type == frontend_type.feature_type
                    || f.feature_type == VCFeatureType::ScalarRotator
                        && frontend_type.feature_type == FeVCFeatureType::Rotator)
            {
                info!(
                    "FE Object and Loaded Object are Eq: {}: {:?}",
                    f.feature_index, f.feature_type
                );
                f.from_frontend(frontend_type.clone());
                success = true;
            }
        });
        success
    }
}
