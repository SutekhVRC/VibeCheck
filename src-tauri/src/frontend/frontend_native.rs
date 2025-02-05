/*
 * Native JS Interface
 *
 */

use crate::{
    config::toy::VCToyConfig,
    frontend::{
        frontend_types::{
            FeBrowserLink, FeToyAlter, FeToyEvent, FeVCFeatureType, FeVCToy, FeVibeCheckConfig,
        },
        FromFrontend, ToFrontend,
    },
    vcore::core::{
        self, native_osc_query_attempt_force, native_osc_query_start, native_osc_query_stop,
    },
    vcore::vcerror::{backend, frontend},
};
use log::{error as logerr, trace};
use tauri::{Emitter, Manager};

/*
 * vibecheck_version
 * Returns the VibeCheck Version as a String
 * Args: None
 * Return: String
 */
#[tauri::command]
pub fn vibecheck_version(app_handle: tauri::AppHandle) -> String {
    format!("{} windows", app_handle.package_info().version)
}

/*
 * vibecheck_enable
 * Enables vibecheck handling runtime
 * Args: VibeCheck State
 * Return: Result<Ok, Err(VCFeError)>
 */
#[tauri::command]
pub fn vibecheck_enable(
    vc_state: tauri::State<'_, core::VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    trace!("vibecheck_enable");
    tauri::async_runtime::block_on(async move { core::native_vibecheck_enable(vc_state).await })
}

/*
 * vibecheck_disable
 * Disables vibecheck handling runtime
 * Args: VibeCheck State
 * Return: Result<Ok, Err(VCFeError)>
 */
#[tauri::command]
pub fn vibecheck_disable(
    vc_state: tauri::State<'_, core::VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    trace!("vibecheck_disable");
    tauri::async_runtime::block_on(async move { core::native_vibecheck_disable(vc_state).await })
}

/*
 * vibecheck_start_bt_scan
 * Starts scanning for BTLE devices
 * Args: VibeCheck State
 * Return: None
 */
#[tauri::command]
pub fn vibecheck_start_bt_scan(
    vc_state: tauri::State<'_, core::VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    trace!("vibecheck_start_bt_scan");
    tauri::async_runtime::block_on(
        async move { core::native_vibecheck_start_bt_scan(vc_state).await },
    )
}

/*
 * vibecheck_stop_bt_scan
 * Stops scanning for BTLE devices
 * Args: VibeCheck State
 * Return: None
 */
#[tauri::command]
pub fn vibecheck_stop_bt_scan(
    vc_state: tauri::State<'_, core::VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    trace!("vibecheck_stop_bt_scan");
    tauri::async_runtime::block_on(
        async move { core::native_vibecheck_stop_bt_scan(vc_state).await },
    )
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
pub fn get_vibecheck_config(vc_state: tauri::State<'_, core::VCStateMutex>) -> FeVibeCheckConfig {
    trace!("get_vibecheck_config");
    core::native_get_vibecheck_config(vc_state)
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
pub fn set_vibecheck_config(
    vc_state: tauri::State<'_, core::VCStateMutex>,
    fe_vc_config: FeVibeCheckConfig,
) -> Result<(), frontend::VCFeError> {
    trace!("set_vibecheck_config({:?})", fe_vc_config);
    core::native_set_vibecheck_config(vc_state, fe_vc_config)
}

/*
 * alter_toy
 * Alters a toy state
 * Args: toy_id, FeToyAlter
 * Return: Result<Ok(()), Err(ToyAlterError)>
 */
#[tauri::command(async)]
pub fn alter_toy(
    vc_state: tauri::State<'_, core::VCStateMutex>,
    app_handle: tauri::AppHandle,
    mutate: FeToyAlter,
) -> Result<(), frontend::VCFeError> {
    trace!("alter_toy({:#?})", mutate);

    match mutate {
        // Online toy alter
        FeToyAlter::Connected(fe_toy) => {
            if fe_toy.toy_connected {
                trace!(
                    "FeToyAlter::Connected: Altering online toy: {}",
                    fe_toy.toy_name
                );
                let altered = {
                    let mut vc_lock = vc_state.0.lock();
                    if let Some(toy) = vc_lock
                        .core_toy_manager
                        .as_mut()
                        .unwrap()
                        .online_toys
                        .get_mut(&fe_toy.toy_id.unwrap())
                    {
                        toy.osc_data = fe_toy.osc_data;
                        toy.config.as_mut().unwrap().osc_data = fe_toy.osc_data;
                        toy.config
                            .as_mut()
                            .unwrap()
                            .anatomy
                            .from_fe(fe_toy.toy_anatomy);

                        // Overwrite all features in the state handled toy.
                        for fe_feature in fe_toy.features {
                            if !toy.parsed_toy_features.from_frontend(fe_feature.clone()) {
                                logerr!("Failed to convert FeVCToyFeature to VCToyFeature");
                                return Err(frontend::VCFeError::AlterToyFailure(
                                    frontend::ToyAlterError::NoFeatureIndex,
                                ));
                            } else {
                                // If altering feature map succeeds write the data to the config
                                toy.config.as_mut().unwrap().features =
                                    toy.parsed_toy_features.clone();
                            }
                        }

                        toy.clone()
                    } else {
                        return Err(frontend::VCFeError::AlterToyFailure(
                            frontend::ToyAlterError::NoToyIndex,
                        ));
                    }
                };

                if core::native_alter_toy(vc_state, app_handle, altered).is_err() {
                    return Err(frontend::VCFeError::AlterToyFailure(
                        frontend::ToyAlterError::TMESendFailure,
                    ));
                }
            } else {
                return Err(frontend::VCFeError::AlterToyFailure(
                    frontend::ToyAlterError::ToyDisconnected,
                ));
            }

            Ok(())
        }
        // Offline toy alter
        FeToyAlter::Disconnected(mut fe_toy) => {
            if !fe_toy.toy_connected {
                trace!(
                    "FeToyAlter::Disconnected: Altering offline toy: {}",
                    fe_toy.toy_name
                );
                let mut offline_toy_config =
                    match VCToyConfig::load_offline_toy_config(fe_toy.toy_name.clone()) {
                        Ok(toy_config) => toy_config,
                        Err(_e) => {
                            return Err(frontend::VCFeError::AlterToyFailure(
                                frontend::ToyAlterError::OfflineToyNotExist,
                            ))
                        }
                    };

                offline_toy_config.osc_data = fe_toy.osc_data;
                offline_toy_config.anatomy.from_fe(fe_toy.toy_anatomy);

                for f in fe_toy.features {
                    if !offline_toy_config.features.from_frontend(f) {
                        return Err(frontend::VCFeError::AlterToyFailure(
                            frontend::ToyAlterError::OfflineToyNoFeatureIndex,
                        ));
                    }
                }

                fe_toy.features = offline_toy_config.features.features.to_frontend();
                fe_toy.osc_data = offline_toy_config.osc_data;
                fe_toy.toy_anatomy = offline_toy_config.anatomy.to_fe();

                offline_toy_config.save_offline_toy_config();

                let _ = app_handle.emit("fe_toy_event", FeToyEvent::Update(fe_toy));
            } else {
                return Err(frontend::VCFeError::AlterToyFailure(
                    frontend::ToyAlterError::ToyConnected,
                ));
            }

            Ok(())
        }
    }
}

/*
 * Opens the social link specified
 */
#[tauri::command(async)]
pub fn open_default_browser(link: FeBrowserLink) {
    match link {
        FeBrowserLink::Discord => {
            let _ = open::that("https://discord.gg/g6kUFtMtpw");
        }
        FeBrowserLink::Github => {
            let _ = open::that("https://github.com/SutekhVRC/VibeCheck");
        }
        FeBrowserLink::VRChatGroup => {
            let _ = open::that("https://vrc.group/VIBE.9503");
        }
        FeBrowserLink::ToyOptions => {
            let _ = open::that(
                "https://github.com/SutekhVRC/VibeCheck/wiki/Toy-Options-List#toy-options",
            );
        }
        FeBrowserLink::FeatureOptions => {
            let _ = open::that(
                "https://github.com/SutekhVRC/VibeCheck/wiki/Toy-Options-List#toy-feature-options",
            );
        }
    };
}

/*
 * Clears VRChat OSC avatar configs
 * Clears all folders in the OSC folder that start with 'usr_'
 */
#[tauri::command(async)]
pub fn clear_osc_config() -> Result<(), backend::VibeCheckFSError> {
    trace!("clear_osc_config");
    core::native_clear_osc_config()
}

/*
 * Injects motor test values into a device feature directly.
 * Args: toy_id: u32, toy_sub_id: u8, feature_index: u32, float_level: f64, stop: bool
 */
#[tauri::command(async)]
pub fn simulate_device_feature(
    vc_state: tauri::State<'_, core::VCStateMutex>,
    toy_id: u32,
    feature_index: u32,
    feature_type: FeVCFeatureType,
    float_level: f64,
    stop: bool,
) {
    trace!("simulate_device_feature");
    core::native_simulate_device_feature(
        vc_state,
        toy_id,
        feature_index,
        feature_type,
        float_level,
        stop,
    )
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

/*
 *
 */
#[tauri::command(async)]
pub fn sync_offline_toys(
    vc_state: tauri::State<'_, core::VCStateMutex>,
    refresh_toys: bool,
) -> Result<Vec<FeVCToy>, frontend::VCFeError> {
    trace!("sync_offline_toys");
    if let Some(toy_manager) = vc_state.0.lock().core_toy_manager.as_mut() {
        Ok(toy_manager.sync_frontend(refresh_toys))
    } else {
        Err(frontend::VCFeError::ToyManagerNotReady)
    }
}

#[tauri::command(async)]
pub fn osc_query_start(
    vc_state: tauri::State<'_, core::VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    trace!("osc_query_start");
    native_osc_query_start(vc_state)
}

#[tauri::command(async)]
pub fn osc_query_stop(
    vc_state: tauri::State<'_, core::VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    trace!("osc_query_stop");
    native_osc_query_stop(vc_state)
}

#[tauri::command(async)]
pub fn osc_query_attempt_force_connect(
    vc_state: tauri::State<'_, core::VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    trace!("osc_query_attempt_force_connect");
    native_osc_query_attempt_force(vc_state)
}
