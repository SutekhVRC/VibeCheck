
/*
 * Native JS Interface
 * 
 */

use log::trace;
use crate::{vcore, frontend_types::{FeVibeCheckConfig, FeToyAlter}, vcerror::frontend};

/*
 * vibecheck_version
 * Returns the VibeCheck Version as a String
 * Args: None
 * Return: String
 */
#[tauri::command]
pub fn vibecheck_version() -> &'static str {
    "0.2.0-beta-windows"
}

/*
 * vibecheck_enable
 * Enables vibecheck handling runtime
 * Args: VibeCheck State
 * Return: Result<Ok, Err(VCFeError)>
 */
#[tauri::command]
pub fn vibecheck_enable(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Result<(), frontend::VCFeError> {
    trace!("vibecheck_enable");
    tauri::async_runtime::block_on(async move {vcore::native_vibecheck_enable(vc_state).await})
}

/*
 * vibecheck_disable
 * Disables vibecheck handling runtime
 * Args: VibeCheck State
 * Return: Result<Ok, Err(VCFeError)>
 */
#[tauri::command]
pub fn vibecheck_disable(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Result<(), frontend::VCFeError> {
    trace!("vibecheck_disable");
    tauri::async_runtime::block_on(async move {vcore::native_vibecheck_disable(vc_state).await})
}

/*
 * vibecheck_start_bt_scan
 * Starts scanning for BTLE devices
 * Args: VibeCheck State
 * Return: None
 */
#[tauri::command]
pub fn vibecheck_start_bt_scan(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Result<(), frontend::VCFeError> {
    trace!("vibecheck_start_bt_scan");
    tauri::async_runtime::block_on(async move {vcore::native_vibecheck_start_bt_scan(vc_state).await})
}

/*
 * vibecheck_stop_bt_scan
 * Stops scanning for BTLE devices
 * Args: VibeCheck State
 * Return: None
 */
#[tauri::command]
pub fn vibecheck_stop_bt_scan(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Result<(), frontend::VCFeError> {
    trace!("vibecheck_stop_bt_scan");
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
pub fn get_vibecheck_config(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> FeVibeCheckConfig {
    trace!("get_vibecheck_config");
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
pub fn set_vibecheck_config(vc_state: tauri::State<'_, vcore::VCStateMutex>, fe_vc_config: FeVibeCheckConfig) -> Result<(), frontend::VCFeError>{
    trace!("set_vibecheck_config({:?})", fe_vc_config);
    vcore::native_set_vibecheck_config(vc_state, fe_vc_config)
}

/*
 * alter_toy
 * Alters a toy state
 * Args: toy_id, FeToyAlter
 * Javascript input example
 * let altered = {
 *  feature_enabled: true,
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
pub fn alter_toy(vc_state: tauri::State<'_, vcore::VCStateMutex>, app_handle: tauri::AppHandle, toy_id: u32, mutate: FeToyAlter) -> Result<(), frontend::VCFeError> {
    trace!("alter_toy({}, {:?})", toy_id, mutate);
    vcore::native_alter_toy(vc_state, app_handle, toy_id, mutate)
}
/*
#[tauri::command(async)]
pub fn get_connection_modes(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> ConnectionModes {
    let vc_lock = vc_state.0.lock();
    vc_lock.connection_modes.clone()
}*/

/*
#[tauri::command(async)]
pub fn set_lc_override(vc_state: tauri::State<'_, vcore::VCStateMutex>, host: FeLCOverride) -> Result<(), frontend::VCFeError> {
    trace!("set_lc_override({:?})", host);
    vcore::native_set_lc_override(vc_state, host)
}

#[tauri::command(async)]
pub fn get_lc_override(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Result<String, frontend::VCFeError> {
    trace!("get_lc_override()");
    vcore::native_get_lc_override(vc_state)
}*/