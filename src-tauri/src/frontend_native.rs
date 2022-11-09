
/*
 * Native JS Interface
 * 
 */

use std::collections::HashMap;

use crate::{vcupdate, vcore, toyops::FrontendVCToyModel};

/*
 * vibecheck_version
 * Returns the VibeCheck Version as a String
 * Args: None
 * Return: String
 */
#[tauri::command]
pub fn vibecheck_version() -> String {
    vcupdate::VERSION.to_string()
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
pub fn get_toys(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Option<HashMap<u32, FrontendVCToyModel>> {
    vcore::native_get_toys(vc_state)
}