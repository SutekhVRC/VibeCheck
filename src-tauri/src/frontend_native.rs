
/*
 * Native JS Interface
 * 
 */

use std::collections::HashMap;

use crate::{vcupdate, vcore};

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
#[tauri::command(async)]
pub fn vibecheck_disable(vc_state: tauri::State<'_, vcore::VCStateMutex>) -> Result<(), &'static str> {
    let vc_lock = vc_state.0.lock();
    vcore::native_vibecheck_disable(vc_lock)
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