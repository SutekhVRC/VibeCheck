use crate::{
    frontend::frontend_types::FeVCToyAnatomy,
    toy_handling::toyops::VCToyFeatures,
    util::fs::{file_exists, get_config_dir},
    vcore::errors,
};
use log::{debug, error as logerr, info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub enum VCToyAnatomy {
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
    #[default]
    NA,
    Nipples,
    Penis,
    Perineum,
    Testicles,
    Thighs,
    Vagina,
    Vulva,
    Wrist,
}

impl VCToyAnatomy {
    pub fn get_anatomy(token: &String) -> Self {
        match token.to_lowercase().as_str() {
            "anus" => Self::Anus,
            //"any" => Self::Any,
            "breasts" => Self::Breasts,
            "butt" | "buttocks" => Self::Buttocks,
            "chest" => Self::Chest,
            "clitoris" => Self::Clitoris,
            "face" => Self::Face,
            "feet" => Self::Feet,
            "footl" => Self::FootL,
            "footr" => Self::FootR,
            "hands" => Self::Hands,
            "handl" => Self::HandLeft,
            "handr" => Self::HandRight,
            "labia" => Self::Labia,
            "mouth" => Self::Mouth,
            "na" => Self::NA,
            "nipples" => Self::Nipples,
            "penis" => Self::Penis,
            "perineum" => Self::Perineum,
            "testicles" => Self::Testicles,
            "thighs" => Self::Thighs,
            "vagina" => Self::Vagina,
            "vulva" => Self::Vulva,
            "wrist" => Self::Wrist,
            _ => {
                warn!(
                    "Got \"{}\" for anatomy token.. Defaulting to Self::NA",
                    token
                );
                Self::NA
            }
        }
    }

    pub fn to_fe(&self) -> FeVCToyAnatomy {
        match self {
            Self::Anus => FeVCToyAnatomy::Anus,
            Self::Breasts => FeVCToyAnatomy::Breasts,
            Self::Buttocks => FeVCToyAnatomy::Buttocks,
            Self::Chest => FeVCToyAnatomy::Chest,
            Self::Clitoris => FeVCToyAnatomy::Clitoris,
            Self::Face => FeVCToyAnatomy::Face,
            Self::Feet => FeVCToyAnatomy::Feet,
            Self::FootL => FeVCToyAnatomy::FootL,
            Self::FootR => FeVCToyAnatomy::FootR,
            Self::HandLeft => FeVCToyAnatomy::HandLeft,
            Self::HandRight => FeVCToyAnatomy::HandRight,
            Self::Hands => FeVCToyAnatomy::Hands,
            Self::Labia => FeVCToyAnatomy::Labia,
            Self::Mouth => FeVCToyAnatomy::Mouth,
            Self::NA => FeVCToyAnatomy::NA,
            Self::Nipples => FeVCToyAnatomy::Nipples,
            Self::Penis => FeVCToyAnatomy::Penis,
            Self::Perineum => FeVCToyAnatomy::Perineum,
            Self::Testicles => FeVCToyAnatomy::Testicles,
            Self::Thighs => FeVCToyAnatomy::Thighs,
            Self::Vagina => FeVCToyAnatomy::Vagina,
            Self::Vulva => FeVCToyAnatomy::Vulva,
            Self::Wrist => FeVCToyAnatomy::Wrist,
        }
    }

    pub fn from_fe(&mut self, anatomy_type: FeVCToyAnatomy) {
        match anatomy_type {
            FeVCToyAnatomy::Anus => *self = Self::Anus,
            FeVCToyAnatomy::Breasts => *self = Self::Breasts,
            FeVCToyAnatomy::Buttocks => *self = Self::Buttocks,
            FeVCToyAnatomy::Chest => *self = Self::Chest,
            FeVCToyAnatomy::Clitoris => *self = Self::Clitoris,
            FeVCToyAnatomy::Face => *self = Self::Face,
            FeVCToyAnatomy::Feet => *self = Self::Feet,
            FeVCToyAnatomy::FootL => *self = Self::FootL,
            FeVCToyAnatomy::FootR => *self = Self::FootR,
            FeVCToyAnatomy::HandLeft => *self = Self::HandLeft,
            FeVCToyAnatomy::HandRight => *self = Self::HandRight,
            FeVCToyAnatomy::Hands => *self = Self::Hands,
            FeVCToyAnatomy::Labia => *self = Self::Labia,
            FeVCToyAnatomy::Mouth => *self = Self::Mouth,
            FeVCToyAnatomy::NA => *self = Self::NA,
            FeVCToyAnatomy::Nipples => *self = Self::Nipples,
            FeVCToyAnatomy::Penis => *self = Self::Penis,
            FeVCToyAnatomy::Perineum => *self = Self::Perineum,
            FeVCToyAnatomy::Testicles => *self = Self::Testicles,
            FeVCToyAnatomy::Thighs => *self = Self::Thighs,
            FeVCToyAnatomy::Vagina => *self = Self::Vagina,
            FeVCToyAnatomy::Vulva => *self = Self::Vulva,
            FeVCToyAnatomy::Wrist => *self = Self::Wrist,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VCToyConfig {
    pub toy_name: String,
    pub features: VCToyFeatures,
    pub osc_data: bool,
    pub anatomy: VCToyAnatomy,
}

impl VCToyConfig {
    pub fn load_offline_toy_config(
        toy_name: String,
    ) -> Result<VCToyConfig, errors::backend::VibeCheckToyConfigError> {
        // Generate config path

        let config_path = format!("{}\\ToyConfigs\\{}.json", get_config_dir(), toy_name,);

        if !file_exists(&config_path) {
            Err(errors::backend::VibeCheckToyConfigError::OfflineToyConfigNotFound)
        } else {
            let con = std::fs::read_to_string(config_path).unwrap();

            let config: VCToyConfig = match serde_json::from_str(&con) {
                Ok(vc_toy_config) => vc_toy_config,
                Err(_) => {
                    return Err(errors::backend::VibeCheckToyConfigError::DeserializeError);
                }
            };
            debug!("Loaded & parsed toy config successfully!");
            Ok(config)
        }
    }

    pub fn save_offline_toy_config(&self) {
        let config_path = format!("{}\\ToyConfigs\\{}.json", get_config_dir(), self.toy_name,);

        info!("Saving toy config to: {}", config_path);

        if let Ok(json_string) = serde_json::to_string(self) {
            match std::fs::write(&config_path, json_string) {
                Ok(()) => {
                    info!("Saved toy config: {}", self.toy_name);
                }
                Err(e) => {
                    logerr!("Failed to write to file: {}", e);
                }
            }
        } else {
            warn!("Failed to serialize config to json");
        }
    }
}
