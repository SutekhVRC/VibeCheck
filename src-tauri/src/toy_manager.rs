use std::collections::HashMap;

use log::{debug, trace};
use tauri::{api::dir::read_dir, AppHandle, Manager};

use crate::{config::toy::VCToyConfig, util::{get_config_dir, file_exists}, toyops::VCToy, frontend_types::{FeVCToy, FeToyEvent}};

#[derive(Clone)]
pub struct ToyManager {
    pub configs: HashMap<String, VCToyConfig>,
    pub online_toys: HashMap<u32, VCToy>,
    app_handle: AppHandle,
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

        let mut ot = Self { configs: HashMap::new(), online_toys: HashMap::new(), app_handle };

        ot.populate_configs();
        trace!("ToyManager config population complete!");

        ot.sync_frontend();
        trace!("ToyManager initialization sent frontend sync");

        ot
    }

/*
    pub fn toy_disconnect(&mut self, toy_name: String) {

    }

    pub fn toy_connect(&mut self, toy_name: String) {

    }
*/
    fn populate_configs(&mut self) {
        
        let toy_config_dir = match read_dir(
            format!(
                "{}\\ToyConfigs",
                get_config_dir()
            ),
        false) {
            Ok(config_paths) => config_paths,
            // Doesn't populate
            Err(_e) => return,
        };

        for f in toy_config_dir {
            
            if !file_exists(&f.path) {
                continue;
            }

            let con = match std::fs::read_to_string(f.path) {
                Ok(contents) => contents,
                Err(_e) => continue,
            };
        
            let config: VCToyConfig = match serde_json::from_str(&con) {
                Ok(vc_toy_config) => vc_toy_config,
                Err(_) => {
                    continue;
                }
            };

            trace!("Loaded & parsed toy config [{}] successfully!", config.toy_name);
            let toy_name = config.toy_name.clone();
            self.configs.insert(toy_name, config);
        }

        debug!("Loaded {} Offline toy configs!", self.configs.len());

    }

    pub fn sync_frontend(&self) {

        let offline_fetoy_vec = self.fetoy_vec_from_offline_toys();
        let _ = self.app_handle.emit_all("fe_toy_event",
            FeToyEvent::OfflineSyncAll({
                offline_fetoy_vec
            }),
        );
        trace!("Sent FeVCToy sync");
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

            offline_toy_vec.push(
                FeVCToy {
                    toy_id: None,
                    toy_name: config.toy_name.clone(),
                    toy_anatomy: config.anatomy.to_fe(),
                    battery_level: None,
                    toy_connected: false,
                    features: config.features.to_fe(),
                    listening: false,
                    osc_data: config.osc_data,
                    sub_id: 255,
            });
        }

        offline_toy_vec
    }

    /*
    fn fetoy_vec_from_online_toys(&self) -> Vec<FeVCToy> {

        let mut online_toy_vec = Vec::new();

        self.online_toys.iter().for_each(|(_toy_id, online_toy)| {
            online_toy_vec.push(
                FeVCToy {
                    toy_id: Some(online_toy.toy_id),
                    toy_name: online_toy.toy_name.clone(),
                    toy_anatomy: online_toy.config.as_ref().unwrap().anatomy.to_fe(),
                    battery_level: online_toy.battery_level,
                    toy_connected: online_toy.toy_connected,
                    features: online_toy.param_feature_map.to_fe(),
                    listening: online_toy.listening,
                    osc_data: online_toy.osc_data,
                    sub_id: online_toy.sub_id,
                });
        });

        online_toy_vec
    }
    */
}