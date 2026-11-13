use buttplug_client::ButtplugClientDevice;
use buttplug_core::message::OutputType;
use core::fmt;
use log::{debug, error as logerr, info, warn};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, sync::Arc};
use tauri::AppHandle;
use ts_rs::TS;
use vibecheck_core::modules::buttplug::LegacyLayout;

use crate::{
    config::toy::{VCToyAnatomy, VCToyConfig},
    frontend::{
        frontend_types::{
            FeLevelTweaks, FeProcessingMode, FeToyParameter, FeVCFeatureType, FeVCToyFeature,
        },
        FromFrontend, ToBackend, ToFrontend,
    },
    util::fs::{build_path_dir, build_path_file, file_exists, get_config_dir, ConfigFileType},
    vcore::errors::{
        self,
        backend::{VibeCheckFSError, VibeCheckToyConfigError},
    },
};

use crate::toy_handling::input_processor::penetration_systems::PenetrationSystem;

use super::ToyPower;

#[derive(Clone, Debug)]
pub struct VCToy {
    pub toy_id: u32,
    pub toy_name: String,
    pub toy_power: ToyPower,
    pub toy_connected: bool,
    /// The toy's actuators, numbered the way its config numbers its features.
    pub layout: LegacyLayout,
    pub parsed_toy_features: VCToyFeatures,
    pub osc_data: bool,
    pub listening: bool,
    pub bt_update_rate: u64,
    pub device_handle: Arc<ButtplugClientDevice>,
    pub config: Option<VCToyConfig>,
    pub sub_id: u8,
    pub app_handle: AppHandle,
}

impl VCToy {
    fn populate_linears(&mut self, layout: &LegacyLayout) {
        for index in 0..layout.linears.len() as u32 {
            self.parsed_toy_features.features.push(VCToyFeature::new(
                vec![ToyParameter {
                    parameter: format!("/avatar/parameters/{:?}_{}", VCFeatureType::Linear, index),
                    processing_mode: ProcessingMode::Raw,
                }],
                index,
                VCFeatureType::Linear,
            ));
        }
        info!("Populated {} linears", layout.linears.len());
    }

    fn populate_rotators(&mut self, layout: &LegacyLayout) {
        for index in 0..layout.rotators.len() as u32 {
            self.parsed_toy_features.features.push(VCToyFeature::new(
                vec![],
                index,
                VCFeatureType::Rotator,
            ));
        }
        info!("Populated {} rotators", layout.rotators.len());
    }

    fn populate_scalars(&mut self, layout: &LegacyLayout) {
        for (index, &(_, output)) in layout.scalars.iter().enumerate() {
            let feature_type = match output {
                OutputType::Vibrate => VCFeatureType::Vibrator,
                OutputType::Rotate => VCFeatureType::ScalarRotator,
                OutputType::Oscillate => VCFeatureType::Oscillate,
                OutputType::Constrict => VCFeatureType::Constrict,
                OutputType::Position => VCFeatureType::Position,
                // LegacyLayout only lists the outputs above
                _ => continue,
            };
            self.parsed_toy_features.features.push(VCToyFeature::new(
                vec![],
                index as u32,
                feature_type,
            ));
        }
        info!("Populated {} scalars", layout.scalars.len());
    }

    // Populate if no config can be read for toy
    fn populate_routine(&mut self) {
        info!("Populating toy: {}", self.toy_id,);

        let layout = self.layout.clone();

        self.populate_linears(&layout);
        self.populate_rotators(&layout);
        self.populate_scalars(&layout);

        self.config = Some(VCToyConfig {
            toy_name: self.toy_name.clone(),
            features: self.parsed_toy_features.clone(),
            osc_data: false,
            bt_update_rate: 10,
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

                // Each kind of actuator counts only if the config has the same
                // number of them as the toy
                let counts = [
                    (
                        conf.features.get_feature_scalar_count(),
                        self.layout.scalars.len(),
                    ),
                    (
                        conf.features.get_feature_rotator_count(),
                        self.layout.rotators.len(),
                    ),
                    (
                        conf.features.get_feature_linear_count(),
                        self.layout.linears.len(),
                    ),
                ];
                let conn_toy_feature_count: usize = counts
                    .iter()
                    .filter(|(in_config, on_toy)| in_config == on_toy)
                    .map(|(_, on_toy)| on_toy)
                    .sum();
                debug!("Feature counts (config, toy): {:?}", counts);

                // If Toy has a different count of features repopulate config
                if conn_toy_feature_count != conf.features.features.len() {
                    warn!("Config is likely corrupted! Repopulating features!");
                    self.populate_routine();
                    return;
                }

                // Feature count is the same so its probably safe to assume the toy config is intact
                self.parsed_toy_features = conf.features.clone();

                self.osc_data = conf.osc_data;
                self.bt_update_rate = conf.bt_update_rate;
                info!("Populated toy with loaded config from file!");
            }
            // If config is not loaded populate the toy
            None => {
                self.populate_routine();
            }
        }
    }

    pub fn load_toy_config(&mut self) -> Result<(), VibeCheckToyConfigError> {
        // Generate config path

        let config_dir = match get_config_dir(&self.app_handle) {
            Ok(d) => d,
            Err(_) => return Err(VibeCheckToyConfigError::ConfigDirFail),
        };

        let toy_config_dir = build_path_dir(&[&config_dir, "ToyConfigs"]);
        let config_path = build_path_file(&[&toy_config_dir, &format!("{}.json", self.toy_name)]);

        if !file_exists(&config_path) {
            self.config = None;
            debug!("Attempted to load toy config file: {}", config_path);
            Ok(())
        } else {
            let con = fs::read_to_string(config_path).unwrap();

            let config: VCToyConfig = match serde_json::from_str(&con) {
                Ok(vc_toy_config) => vc_toy_config,
                Err(_) => {
                    self.config = None;
                    return Err(errors::backend::VibeCheckToyConfigError::DeserializeError);
                }
            };
            debug!("Loaded & parsed toy config successfully!");
            self.config = Some(config);
            Ok(())
        }
    }

    // Save Toy config by name
    pub fn save_toy_config(&self) -> Result<(), VibeCheckToyConfigError> {
        let config_dir = match get_config_dir(&self.app_handle) {
            Ok(d) => d,
            Err(_) => return Err(VibeCheckToyConfigError::ConfigDirFail),
        };
        let toy_config_dir = build_path_dir(&[&config_dir, "ToyConfigs"]);
        let config_path = build_path_file(&[&toy_config_dir, &format!("{}.json", self.toy_name)]);
        info!("Saving toy config to: {}", config_path);

        if let Some(conf) = &self.config {
            if let Ok(json_string) = serde_json::to_string(conf) {
                match fs::write(&config_path, json_string) {
                    Ok(()) => {
                        info!("Saved toy config: {}", config_path);
                    }
                    Err(e) => {
                        logerr!("Failed to write to file: {}", e);
                        return Err(VibeCheckToyConfigError::FSFailure(
                            VibeCheckFSError::FileWriteFailure,
                        ));
                    }
                }
            } else {
                warn!("Failed to serialize config to json");
            }
        } else {
            warn!("save_toy_config() called while toy config is None");
        }
        Ok(())
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
        false
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
    pub osc_parameters: Vec<ToyParameter>,
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
    pub(crate) fn new(
        osc_parameters: Vec<ToyParameter>,
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
        self.penetration_system
            .from_frontend(frontend_type.penetration_system);
        self.feature_levels.from_fe(frontend_type.feature_levels);
        self.smooth_enabled = frontend_type.smooth_enabled;
        self.rate_enabled = frontend_type.rate_enabled;
        true
    }
}

impl FromFrontend<Vec<FeToyParameter>> for Vec<ToyParameter> {
    type OutputType = bool;

    fn from_frontend(&mut self, frontend_type: Vec<FeToyParameter>) -> Self::OutputType {
        // Remove make not shit
        self.clear();

        for toy_param in frontend_type {
            info!("FTP: {:?}", toy_param);
            self.push(ToyParameter {
                parameter: toy_param.parameter,
                processing_mode: toy_param.processing_mode.to_backend(),
            });
        }

        true
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
    /// Rate mode's wind-down speed (shown as "Wind-down"). Higher stops sooner.
    pub rate_tune: f64,
    /// Rate mode's sensitivity. Missing from toy configs saved before it
    /// existed, so it defaults to 1.
    #[serde(default = "default_rate_sensitivity")]
    pub rate_sensitivity: f64,
    pub constant_level: f64,
}

fn default_rate_sensitivity() -> f64 {
    1.0
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
            rate_sensitivity: default_rate_sensitivity(),
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
        self.rate_sensitivity = fe_lt.rate_sensitivity;
        self.constant_level = fe_lt.constant_level
    }

    pub fn to_fe(&self) -> FeLevelTweaks {
        FeLevelTweaks {
            minimum_level: self.minimum_level,
            maximum_level: self.maximum_level,
            idle_level: self.idle_level,
            smooth_rate: self.smooth_rate,
            linear_position_speed: self.linear_position_speed,
            rate_tune: self.rate_tune,
            rate_sensitivity: self.rate_sensitivity,
            constant_level: self.constant_level,
        }
    }
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

    pub fn get_feature_linear_count(&self) -> usize {
        let mut count = 0;
        for f in self.features.iter() {
            match f.feature_type {
                VCFeatureType::Linear => count += 1,
                _ => {}
            }
        }
        count
    }

    pub fn get_feature_rotator_count(&self) -> usize {
        let mut count = 0;
        for f in self.features.iter() {
            match f.feature_type {
                VCFeatureType::Rotator => count += 1,
                _ => {}
            }
        }
        count
    }

    pub fn get_feature_scalar_count(&self) -> usize {
        let mut count = 0;
        for f in self.features.iter() {
            match f.feature_type {
                VCFeatureType::Constrict => count += 1,
                VCFeatureType::Inflate => count += 1,
                VCFeatureType::Oscillate => count += 1,
                VCFeatureType::Position => count += 1,
                VCFeatureType::Vibrator => count += 1,
                VCFeatureType::ScalarRotator => count += 1,
                _ => {}
            }
        }
        count
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

    fn from_frontend(&mut self, frontend_feature: FeVCToyFeature) -> Self::OutputType {
        let mut success = false;
        self.features.iter_mut().for_each(|f| {
            info!(
                "Checking Loaded: [{}: {:?}] - Fe: [{}: {:?}]",
                f.feature_index,
                f.feature_type,
                frontend_feature.feature_index,
                frontend_feature.feature_type
            );
            // Check that the index and type are the same
            // Note that here there is an OR for when the feature type is a ScalarRotator
            // May be a good idea in the future to create Scalar types and then convert the names in the frontend.
            if f.feature_index == frontend_feature.feature_index
                && (f.feature_type == frontend_feature.feature_type
                    || f.feature_type == VCFeatureType::ScalarRotator
                        && frontend_feature.feature_type == FeVCFeatureType::Rotator)
            {
                info!(
                    "FE Object and Loaded Object are Eq: {}: {:?}",
                    f.feature_index, f.feature_type
                );
                f.from_frontend(frontend_feature.clone());
                success = true;
            }
        });
        success
    }
}
