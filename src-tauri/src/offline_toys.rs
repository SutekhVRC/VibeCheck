use crate::config::toy::VCToyConfig;

pub struct OfflineToys {
    configs: Vec<VCToyConfig>,
}

impl OfflineToys {
    pub fn new() -> Self {
        /*
         * Read all toy configs
         * Send update to frontend
         * 
         * Create methods:
         * Method for a toy disconnecting
         * Method for a toy connecting
         */
        Self { configs: vec![] }
    }
}