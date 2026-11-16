/*
 * Native JS Interface
 *
 */

use std::rc::Rc;

use crate::{
    config::toy::VCToyConfig,
    frontend::{
        frontend_types::{
            FeBrowserLink, FeOscQueryStatus, FeToyAlter, FeToyEvent, FeVCFeatureType, FeVCToy,
            FeVibeCheckConfig,
        },
        FromFrontend, ToFrontend,
    },
    vcore::{
        errors::{
            backend::{self, ToyAlterError},
            frontend::VCFeError,
        },
        ipc::{call_plane, emit_plane::emit_toy_event},
        state,
    },
};
use log::{error as logerr, trace};

/*
 * vibecheck_version
 * Returns the VibeCheck Version as a String
 * Args: None
 * Return: String
 */
/// Whether this build checks for updates. The updater is only registered
/// when the app's tauri.conf.json configures it (see lib.rs).
#[tauri::command]
pub fn vibecheck_updater_enabled(app_handle: tauri::AppHandle) -> bool {
    app_handle.config().plugins.0.contains_key("updater")
}

#[tauri::command]
pub fn vibecheck_version(app_handle: tauri::AppHandle) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{} windows", app_handle.package_info().version)
    }
    #[cfg(target_os = "linux")]
    {
        format!("{} linux", app_handle.package_info().version)
    }
}

/*
 * vibecheck_enable
 * Enables vibecheck handling runtime
 * Args: VibeCheck State
 * Return: Result<Ok, Err(VCFeError)>
 */
#[tauri::command]
pub fn vibecheck_enable(vc_state: tauri::State<'_, state::VCStateMutex>) -> Result<(), VCFeError> {
    trace!("vibecheck_enable");
    tauri::async_runtime::block_on(
        async move { call_plane::native_vibecheck_enable(vc_state).await },
    )
}

/*
 * vibecheck_disable
 * Disables vibecheck handling runtime
 * Args: VibeCheck State
 * Return: Result<Ok, Err(VCFeError)>
 */
#[tauri::command]
pub fn vibecheck_disable(vc_state: tauri::State<'_, state::VCStateMutex>) -> Result<(), VCFeError> {
    trace!("vibecheck_disable");
    tauri::async_runtime::block_on(
        async move { call_plane::native_vibecheck_disable(vc_state).await },
    )
}

/*
 * vibecheck_start_bt_scan
 * Starts scanning for BTLE devices
 * Args: VibeCheck State
 * Return: None
 */
#[tauri::command]
pub fn vibecheck_start_bt_scan(
    vc_state: tauri::State<'_, state::VCStateMutex>,
) -> Result<(), VCFeError> {
    trace!("vibecheck_start_bt_scan");
    tauri::async_runtime::block_on(async move {
        call_plane::native_vibecheck_start_bt_scan(vc_state).await
    })
}

/*
 * vibecheck_stop_bt_scan
 * Stops scanning for BTLE devices
 * Args: VibeCheck State
 * Return: None
 */
#[tauri::command]
pub fn vibecheck_stop_bt_scan(
    vc_state: tauri::State<'_, state::VCStateMutex>,
) -> Result<(), VCFeError> {
    trace!("vibecheck_stop_bt_scan");
    tauri::async_runtime::block_on(async move {
        call_plane::native_vibecheck_stop_bt_scan(vc_state).await
    })
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
pub fn get_vibecheck_config(vc_state: tauri::State<'_, state::VCStateMutex>) -> FeVibeCheckConfig {
    trace!("get_vibecheck_config");
    call_plane::native_get_vibecheck_config(vc_state)
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
    vc_state: tauri::State<'_, state::VCStateMutex>,
    fe_vc_config: FeVibeCheckConfig,
) -> Result<(), VCFeError> {
    trace!("set_vibecheck_config({:?})", fe_vc_config);
    call_plane::native_set_vibecheck_config(vc_state, fe_vc_config)
}

/*
 * Puts every app setting back to its default (toy settings are untouched)
 */
#[tauri::command]
pub fn reset_vibecheck_config(
    vc_state: tauri::State<'_, state::VCStateMutex>,
) -> Result<(), VCFeError> {
    trace!("reset_vibecheck_config");
    call_plane::native_reset_vibecheck_config(vc_state)
}

/*
 * alter_toy
 * Alters a toy state
 * Args: toy_id, FeToyAlter
 * Return: Result<Ok(()), Err(ToyAlterError)>
 */
#[tauri::command(async)]
pub fn alter_toy(
    vc_state: tauri::State<'_, state::VCStateMutex>,
    app_handle: tauri::AppHandle,
    mutate: FeToyAlter,
) -> Result<(), VCFeError> {
    trace!("alter_toy({:#?})", mutate);

    match mutate {
        // Online toy alter
        FeToyAlter::Connected(fe_toy) => {
            if fe_toy.toy_connected {
                trace!(
                    "FeToyAlter::Connected: Altering online toy: {}",
                    fe_toy.toy_name
                );

                let Some(toy_id) = fe_toy.toy_id else {
                    return Err(VCFeError::AlterToyFailure(ToyAlterError::NoToyIndex));
                };
                let altered = {
                    let mut vc_lock = vc_state.0.lock();
                    if let Some(toy) = vc_lock.toy_manager_mut().online_toys.get_mut(&toy_id) {
                        toy.osc_data = fe_toy.osc_data;
                        toy.bt_update_rate = fe_toy.bt_update_rate;

                        let config = toy.config_mut();
                        config.osc_data = fe_toy.osc_data;
                        config.bt_update_rate = fe_toy.bt_update_rate;
                        config.anatomy.from_fe(fe_toy.toy_anatomy);

                        // Overwrite all features in the state handled toy.
                        for fe_feature in fe_toy.features {
                            if !toy.parsed_toy_features.from_frontend(fe_feature.clone()) {
                                logerr!("Failed to convert FeVCToyFeature to VCToyFeature");
                                return Err(VCFeError::AlterToyFailure(
                                    ToyAlterError::NoFeatureIndex,
                                ));
                            } else {
                                // If altering feature map succeeds write the data to the config
                                toy.config_mut().features = toy.parsed_toy_features.clone();
                            }
                        }

                        toy.clone()
                    } else {
                        return Err(VCFeError::AlterToyFailure(ToyAlterError::NoToyIndex));
                    }
                };

                if call_plane::native_alter_toy(vc_state, app_handle, altered).is_err() {
                    return Err(VCFeError::AlterToyFailure(ToyAlterError::TMESendFailure));
                }
            } else {
                return Err(VCFeError::AlterToyFailure(ToyAlterError::ToyDisconnected));
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
                let mut offline_toy_config = match VCToyConfig::load_offline_toy_config(
                    fe_toy.toy_name.clone(),
                    &app_handle,
                ) {
                    Ok(toy_config) => toy_config,
                    Err(_e) => {
                        return Err(VCFeError::AlterToyFailure(
                            ToyAlterError::OfflineToyNotExist,
                        ))
                    }
                };

                offline_toy_config.osc_data = fe_toy.osc_data;
                offline_toy_config.bt_update_rate = fe_toy.bt_update_rate;
                offline_toy_config.anatomy.from_fe(fe_toy.toy_anatomy);

                for f in fe_toy.features {
                    if !offline_toy_config.features.from_frontend(f) {
                        return Err(VCFeError::AlterToyFailure(
                            ToyAlterError::OfflineToyNoFeatureIndex,
                        ));
                    }
                }

                fe_toy.features = offline_toy_config.features.features.to_frontend();
                fe_toy.osc_data = offline_toy_config.osc_data;
                fe_toy.bt_update_rate = offline_toy_config.bt_update_rate;
                fe_toy.toy_anatomy = offline_toy_config.anatomy.to_fe();

                if offline_toy_config
                    .save_offline_toy_config(&app_handle)
                    .is_err()
                {
                    return Err(VCFeError::SaveOfflineToyConfig);
                }

                emit_toy_event(&Rc::new(app_handle), FeToyEvent::Update(fe_toy));
            } else {
                return Err(VCFeError::AlterToyFailure(ToyAlterError::ToyConnected));
            }

            Ok(())
        }
    }
}

/*
 * Opens the social link specified
 */
#[tauri::command(async)]
pub fn open_default_browser(link: FeBrowserLink) -> Result<(), VCFeError> {
    let url = match link {
        FeBrowserLink::Discord => "https://discord.gg/C8ACSGSyfq",
        FeBrowserLink::Github => "https://github.com/SutekhVRC/VibeCheck",
        FeBrowserLink::GithubWiki => "https://github.com/SutekhVRC/VibeCheck/wiki",
        FeBrowserLink::VRChatGroup => "https://vrc.group/VIBE.9503",
        FeBrowserLink::ToyOptions => {
            "https://github.com/SutekhVRC/VibeCheck/wiki/Toy-Options-List#toy-options"
        }
        FeBrowserLink::FeatureOptions => {
            "https://github.com/SutekhVRC/VibeCheck/wiki/Toy-Options-List#toy-feature-options"
        }
    };
    open::that(url).map_err(|e| {
        logerr!("Couldn't open {} in the browser: {}", url, e);
        VCFeError::OpenBrowserFailure(url.to_owned())
    })
}

/*
 * Opens the folder with VibeCheck's log files, for bug reports
 */
#[tauri::command(async)]
pub fn open_log_folder() -> Result<(), VCFeError> {
    let dir = crate::util::log_file::log_dir().ok_or(VCFeError::NoLogFolder)?;
    // It may have been deleted while VibeCheck was running; the file manager
    // would show its own error for a missing folder
    std::fs::create_dir_all(dir).map_err(|e| {
        logerr!("Couldn't recreate the log folder: {}", e);
        VCFeError::NoLogFolder
    })?;
    open::that(dir).map_err(|e| {
        logerr!("Couldn't open the log folder: {}", e);
        VCFeError::NoLogFolder
    })
}

/*
 * Clears VRChat OSC avatar configs
 * Clears all folders in VRChat's OSC folder that start with 'usr_', on Windows
 * and on Linux (Proton). Returns the OSC folders that were cleared.
 */
#[tauri::command(async)]
pub fn clear_osc_config() -> Result<Vec<String>, backend::VibeCheckFSError> {
    trace!("clear_osc_config");
    call_plane::native_clear_osc_config()
}

/*
 * Injects motor test values into a device feature directly.
 * Args: toy_id: u32, toy_sub_id: u8, feature_index: u32, float_level: f64, stop: bool
 */
#[tauri::command(async)]
pub fn simulate_device_feature(
    vc_state: tauri::State<'_, state::VCStateMutex>,
    toy_id: u32,
    feature_index: u32,
    feature_type: FeVCFeatureType,
    float_level: f64,
    stop: bool,
) {
    trace!("simulate_device_feature");
    call_plane::native_simulate_device_feature(
        vc_state,
        toy_id,
        feature_index,
        feature_type,
        float_level,
        stop,
    )
}

/*
 * Panic stop: stops every toy until resumed
 */
#[tauri::command]
pub fn panic_stop(app_handle: tauri::AppHandle) {
    trace!("panic_stop");
    crate::vcore::panic::trigger_panic(&app_handle)
}

#[tauri::command]
pub fn panic_resume(app_handle: tauri::AppHandle) {
    trace!("panic_resume");
    crate::vcore::panic::resume_from_panic(&app_handle)
}

#[tauri::command]
pub fn get_panic_state(app_handle: tauri::AppHandle) -> bool {
    trace!("get_panic_state");
    crate::vcore::panic::is_panicked(&app_handle)
}

/*
 * Diagnostics view: live values are sent as fe_diagnostics events while on
 */
#[tauri::command]
pub fn set_diagnostics_enabled(vc_state: tauri::State<'_, state::VCStateMutex>, enabled: bool) {
    trace!("set_diagnostics_enabled {}", enabled);
    vc_state.0.lock().diagnostics.set_enabled(enabled);
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
    vc_state: tauri::State<'_, state::VCStateMutex>,
    refresh_toys: bool,
) -> Result<Vec<FeVCToy>, VCFeError> {
    trace!("sync_offline_toys");

    if let Some(toy_manager) = vc_state.0.lock().core_toy_manager.as_mut() {
        match toy_manager.sync_frontend(refresh_toys) {
            Ok(toys) => Ok(toys),
            Err(e) => {
                logerr!("Toy Manager failed to sync frontend: {}", e);
                Err(VCFeError::ToyManager(e))
            }
        }
    } else {
        Err(VCFeError::ToyManagerNotReady)
    }
}

/*
 * Forgets an offline toy: deletes its saved settings
 * Args: toy_name
 */
#[tauri::command]
pub fn remove_offline_toy(
    vc_state: tauri::State<'_, state::VCStateMutex>,
    toy_name: String,
) -> Result<(), VCFeError> {
    trace!("remove_offline_toy");

    let mut vc_lock = vc_state.0.lock();
    let Some(toy_manager) = vc_lock.core_toy_manager.as_mut() else {
        return Err(VCFeError::ToyManagerNotReady);
    };
    toy_manager
        .remove_offline_toy(&toy_name)
        .map_err(VCFeError::ToyManager)
}

/*
 * OSCQuery mode's status: the advertised port, and where VRChat listens
 */
#[tauri::command]
pub fn get_oscquery_status(vc_state: tauri::State<'_, state::VCStateMutex>) -> FeOscQueryStatus {
    call_plane::native_get_oscquery_status(vc_state)
}

/*
 * The current avatar's parameters, for the parameter picker
 */
#[tauri::command]
pub fn get_avatar_catalog(
    vc_state: tauri::State<'_, state::VCStateMutex>,
) -> crate::frontend::frontend_types::FeAvatarCatalog {
    let catalog = vc_state.0.lock().avatar_catalog.clone();
    (&catalog.state()).into()
}

/*
 * Reads the current avatar from VRChat again now
 */
#[tauri::command]
pub fn refresh_avatar_catalog(vc_state: tauri::State<'_, state::VCStateMutex>) {
    vc_state.0.lock().avatar_catalog.refresh();
}

/*
 * What a feature's settings miss on the current avatar. None while the
 * avatar isn't known (VRChat not found yet), so nothing is flagged.
 */
#[tauri::command]
pub fn check_feature(
    vc_state: tauri::State<'_, state::VCStateMutex>,
    feature: crate::frontend::frontend_types::FeVCToyFeature,
) -> Option<crate::frontend::frontend_types::FeFeatureCheck> {
    let avatar = vc_state.0.lock().avatar_catalog.avatar()?;
    let parameters: Vec<String> = feature
        .osc_parameters
        .iter()
        .map(|p| p.parameter.clone())
        .collect();
    let system = feature.penetration_system.pen_system_type;
    let tags = feature
        .penetration_system
        .pen_system_input_filter
        .unwrap_or_default();
    Some(crate::osc::avatar_catalog::check_feature(&avatar, &parameters, system, &tags).into())
}
