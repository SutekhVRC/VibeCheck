use std::collections::HashMap;

use crate::frontend::ToFrontend;
use crate::{
    config::toy::VCToyConfig,
    frontend::frontend_types::FeVCToy,
    toy_handling::toyops::VCToy,
    util::fs::{file_exists, get_config_dir},
};
use log::{debug, info, trace};
use std::fs;
use std::io::Read;
use tauri::AppHandle;

#[derive(Clone)]
pub struct ToyManager {
    pub configs: HashMap<String, VCToyConfig>,
    pub online_toys: HashMap<u32, VCToy>,
    _app_handle: AppHandle,
}

impl ToyManager {
    pub fn new(app_handle: AppHandle) -> Self {
        /*
         * Read all toy configs
         * Send update to frontend
         *
         * Create methods:
         * Method for a toy disconnecting
         * Method for a toy connecting
         *
         * This struct is the new toys handler / object
         */

        let mut ot = Self {
            configs: HashMap::new(),
            online_toys: HashMap::new(),
            _app_handle: app_handle,
        };

        ot.populate_configs();
        trace!("ToyManager config population complete!");

        /*
                ot.sync_frontend();
                trace!("ToyManager initialization sent frontend sync");
        */
        ot
    }

    pub fn populate_configs(&mut self) {
        let toy_config_dir = match fs::read_dir(format!("{}\\ToyConfigs", get_config_dir())) {
            Ok(config_paths) => config_paths,
            // Doesn't populate
            Err(_e) => return,
        };

        for f in toy_config_dir {
            let path = f.unwrap().path();
            if !path.is_file() {
                continue;
            }
            let mut file = fs::File::open(path).unwrap();
            let mut data = String::new();
            file.read_to_string(&mut data);
            let config: VCToyConfig = match serde_json::from_str(&data) {
                Ok(vc_toy_config) => vc_toy_config,
                Err(_) => {
                    continue;
                }
            };

            trace!(
                "Loaded & parsed toy config [{}] successfully!",
                config.toy_name
            );
            let toy_name = config.toy_name.clone();
            self.configs.insert(toy_name, config);
        }

        debug!("Loaded {} Offline toy configs!", self.configs.len());
    }

    pub fn sync_frontend(&mut self, refresh_toys: bool) -> Vec<FeVCToy> {
        if refresh_toys {
            info!("Clearing toy manager configs map and repopulating from disk..");
            self.configs.clear();
            self.populate_configs();
        }

        trace!("Generating offline toy sync..");
        self.fetoy_vec_from_offline_toys()
    }

    fn check_toy_online(&self, config_toy_name: &String) -> bool {
        for online_toy in self.online_toys.iter() {
            if *config_toy_name == online_toy.1.toy_name.replace("Lovense Connect", "Lovense") {
                return true;
            }
        }
        false
    }

    fn fetoy_vec_from_offline_toys(&self) -> Vec<FeVCToy> {
        let mut offline_toy_vec = Vec::new();

        for (_toy_key, config) in self.configs.iter() {
            if self.check_toy_online(&config.toy_name) {
                continue;
            }

            offline_toy_vec.push(FeVCToy {
                toy_id: None,
                toy_name: config.toy_name.clone(),
                toy_anatomy: config.anatomy.to_fe(),
                toy_power: super::ToyPower::Offline,
                toy_connected: false,
                features: config.features.features.to_frontend(),
                listening: false,
                osc_data: config.osc_data,
                sub_id: 255,
            });
        }

        offline_toy_vec
    }
}
