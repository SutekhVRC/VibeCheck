
/*
 * Native JS Interface
 * 
 */

use std::collections::HashMap;

use crate::{vcore::{self, ToyAlterError, ConnectionModes}, frontend_types::{FeVCToy, FeVCToyFeature}};

/*
 * vibecheck_version
 * Returns the VibeCheck Version as a String
 * Args: None
 * Return: String
 */
#[tauri::command]
pub fn vibecheck_version() -> String {
    "0.2.0-beta-windows".to_string()
}

/*
 * vibecheck_enable
 * Enables vibecheck handling runtime
 * Args: VibeCheck State
 * Return: Result<Ok, Err(str)>
 */
#[tauri::command(async)]
pub fn vibecheck_enable(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Result<(), &'static str> {
    vcore::native_vibecheck_enable(vc_state)
}

/*
 * vibecheck_disable
 * Disables vibecheck handling runtime
 * Args: VibeCheck State
 * Return: Result<Ok, Err(str)>
 */
#[tauri::command]
pub fn vibecheck_disable(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Result<(), &'static str> {
    tauri::async_runtime::block_on(async move {vcore::native_vibecheck_disable(vc_state).await})
}

/*
 * vibecheck_start_bt_scan
 * Starts scanning for BTLE devices
 * Args: VibeCheck State
 * Return: None
 */
#[tauri::command]
pub fn vibecheck_start_bt_scan(vc_state: tauri::State<'_, vcore::VCStateMutex>) {
    tauri::async_runtime::block_on(async move {vcore::native_vibecheck_start_bt_scan(vc_state).await})
}

/*
 * vibecheck_stop_bt_scan
 * Stops scanning for BTLE devices
 * Args: VibeCheck State
 * Return: None
 */
#[tauri::command]
pub fn vibecheck_stop_bt_scan(vc_state: tauri::State<'_, vcore::VCStateMutex>) {
    tauri::async_runtime::block_on(async move {vcore::native_vibecheck_stop_bt_scan(vc_state).await})
}

/*
 * get_vibecheck_config
 * Retreieves the vibecheck config
 * Args: VibeCheck State
 * Return: HashMap<str, String>
 * Map Config Contents
 * host : string
 * port: string
 */
#[tauri::command(async)]
pub fn get_vibecheck_config(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> HashMap<&str, String> {
    vcore::native_get_vibecheck_config(vc_state)
}

/*
 * set_vibecheck_config
 * Sets and saves the vibecheck config
 * Args: VibeCheck State, bind map
 * Return: Result<Ok(()), Err(VibeCheckConfigError)>
 * Map Config Contents
 * bind : HashMap<host, port>
 */
#[tauri::command(async)]
pub fn set_vibecheck_config(vc_state: tauri::State<'_, vcore::VCStateMutex>, bind: HashMap<String, String>) -> Result<(), vcore::VibeCheckConfigError>{
    vcore::native_set_vibecheck_config(vc_state, bind)
}

/*
 * get_toys
 * Gets toy states
 * Args: None
 * Return: Option<Vec<FrontendVCToyModel>>
 */
#[tauri::command(async)]
pub fn get_toys(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Option<HashMap<u32, FeVCToy>> {
    vcore::native_get_toys(vc_state)
}

/*
 * alter_toy
 * Alters a toy state
 * Args: toy_id, AlterVCToyModel
 * Javascript input example
 * let altered = {
 * feature_enabled: true,
 *  osc_parameter: "/avatar/parameters/vibin",
 *  feature_index: 0,
 *  feature_levels: {
 *      minimum_level: 0.00,
 *      maximum_level: 100.00,
 *      idle_level: 0.00,
 *      smooth_rate: 2.00,
 *  },
 *      smooth_enabled: true,
 *  };
 * let alter_err = window.__TAURI_INVOKE__("alter_toy", {toyId: 0, frontendToy: altered});
 *
 * Return: Result<Ok(()), Err(ToyAlterError)>
 */
#[tauri::command(async)]
pub fn alter_toy(vc_state: tauri::State<'_, vcore::VCStateMutex>, toy_id: u32, toy_feature: FeVCToyFeature) -> Result<(), ToyAlterError> {
    vcore::native_alter_toy(vc_state, toy_id, toy_feature)
}

#[tauri::command(async)]
pub fn get_connection_modes(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> ConnectionModes {
    let vc_lock = vc_state.0.lock();
    vc_lock.connection_modes.clone()
}