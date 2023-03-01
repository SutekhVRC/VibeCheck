
/*
 * Native JS Interface
 * 
 */

use log::trace;
use crate::{vcore, frontend_types::{FeVibeCheckConfig, FeToyAlter, FeSocialLink}, vcerror::{frontend, backend}};

/*
 * vibecheck_version
 * Returns the VibeCheck Version as a String
 * Args: None
 * Return: String
 */
#[tauri::command]
pub fn vibecheck_version(app_handle: tauri::AppHandle) -> String {
    format!("{} beta windows", app_handle.package_info().version)
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
 * Return: Result<Ok(()), Err(ToyAlterError)>
 */
#[tauri::command(async)]
pub fn alter_toy(vc_state: tauri::State<'_, vcore::VCStateMutex>, app_handle: tauri::AppHandle, toy_id: u32, mutate: FeToyAlter) -> Result<(), frontend::VCFeError> {
    trace!("alter_toy({}, {:?})", toy_id, mutate);
    vcore::native_alter_toy(vc_state, app_handle, toy_id, mutate)
}

/*
 * Opens the social link specified
 */
#[tauri::command(async)]
pub fn open_default_browser(link: FeSocialLink) {
    match link {
        FeSocialLink::Discord => {let _ = open::that("https://discord.gg/g6kUFtMtpw");},
        FeSocialLink::Github => {let _ = open::that("https://github.com/SutekhVRC/VibeCheck");},
        FeSocialLink::VRChatGroup => {let _ = open::that("https://vrc.group/VIBE.9503");},
    };
}

/*
 * Clears VRChat OSC avatar configs
 * Clears all folders in the OSC folder that start with 'usr_'
 */
#[tauri::command(async)]
pub fn clear_osc_config() -> Result<(), backend::VibeCheckFSError>{
    trace!("clear_osc_config");
    vcore::native_clear_osc_config()
}

/*
 * Sends the specified OSC address / value to the app itself
 * Args: simulated_param_address, simulated_param_value
 * Removing this code for now
#[tauri::command(async)]
pub fn simulate_feature_osc_input(vc_state: tauri::State<'_, vcore::VCStateMutex>, simulated_param_address: String, simulated_param_value: f32) {
    trace!("simulate_feature_osc_input");
    vcore::native_simulate_feature_osc_input(vc_state, simulated_param_address, simulated_param_value)
}
 *
 */