use std::{error::Error, fs, net::SocketAddrV4, str::FromStr};

use log::{debug, error as logerr, info, trace, warn};

use crate::{
    frontend::{
        frontend_types::{FeToyEvent, FeVCFeatureType, FeVCToy, FeVibeCheckConfig},
        ToFrontend,
    },
    osc::OSCNetworking,
    toy_handling::{
        errors::HandlerErr,
        toy_command_processor::command_toy,
        toyops::{VCFeatureType, VCToy},
    },
    util::fs::{get_config_dir, get_user_home_dir},
    vcore::{
        config::app::VibeCheckConfig,
        errors::{
            backend::{ToyAlterError, VibeCheckConfigError, VibeCheckFSError},
            frontend::VCFeError,
            VCError, VcoreError,
        },
        ipc::emit_plane::emit_toy_event,
        state::{RunningState, VCStateMutex},
    },
};

use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Debug)]
pub enum ToyUpdate {
    AlterToy(VCToy),
    RemoveToy(u32),
    AddToy(VCToy),
}

#[derive(Debug)]
pub enum TmSig {
    StopListening,
    StartListening(OSCNetworking),
    TMHReset,
    /*
    Running,
    Stopped,
    */
    Listening,
    BindError,
}

#[derive(Debug)]
pub enum ToyManagementEvent {
    Tu(ToyUpdate),
    Sig(TmSig),
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

trait ToyRuntime {
    fn enable<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>>;
    fn disable<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>>;
    fn start_scan<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>>;
    fn stop_scan<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>>;
    fn alter_toy<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
        app_handle: tauri::AppHandle,
        altered: VCToy,
    ) -> BoxFuture<'a, Result<(), ToyAlterError>>;
}

struct RealToyRuntime;
struct MockToyRuntime;

fn runtime_for(vc_state: &tauri::State<'_, VCStateMutex>) -> &'static dyn ToyRuntime {
    if vc_state.0.lock().mock_toys {
        &MockToyRuntime
    } else {
        &RealToyRuntime
    }
}

impl ToyRuntime for MockToyRuntime {
    fn enable<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>> {
        Box::pin(async move {
            let mut vc_lock = vc_state.0.lock();
            vc_lock.running = RunningState::Running;
            Ok(())
        })
    }

    fn disable<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>> {
        Box::pin(async move {
            let mut vc_lock = vc_state.0.lock();
            vc_lock.running = RunningState::Stopped;
            Ok(())
        })
    }

    fn start_scan<'a>(
        &'a self,
        _vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>> {
        Box::pin(async { Ok(()) })
    }

    fn stop_scan<'a>(
        &'a self,
        _vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>> {
        Box::pin(async { Ok(()) })
    }

    fn alter_toy<'a>(
        &'a self,
        _vc_state: tauri::State<'a, VCStateMutex>,
        _app_handle: tauri::AppHandle,
        _altered: VCToy,
    ) -> BoxFuture<'a, Result<(), ToyAlterError>> {
        Box::pin(async { Ok(()) })
    }
}

impl ToyRuntime for RealToyRuntime {
    fn enable<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>> {
        Box::pin(async move {
            let mut vc_lock = vc_state.0.lock();
            if let RunningState::Running = vc_lock.running {
                return Ok(());
            }

            if vc_lock.bp_client.is_none() {
                return Err(VCFeError::EnableFailure);
            }

            info!("Stopping DOL");
            vc_lock.stop_disabled_listener().await;

            vc_lock
                .tme_send_tx
                .send(ToyManagementEvent::Sig(TmSig::StartListening(
                    vc_lock.config.networking.clone(),
                )))
                .unwrap();

            match vc_lock.tme_recv_rx.recv().await {
                Some(tme) => match tme {
                    ToyManagementEvent::Sig(sig) => match sig {
                        TmSig::Listening => {
                            vc_lock.running = RunningState::Running;
                            vc_lock.init_toy_update_handler().await;
                            Ok(())
                        }
                        TmSig::BindError => {
                            logerr!("Bind Error in TME sig: Sending shutdown signal!");

                            vc_lock
                                .tme_send_tx
                                .send(ToyManagementEvent::Sig(TmSig::StopListening))
                                .unwrap();
                            vc_lock.running = RunningState::Stopped;
                            Err(VCFeError::EnableBindFailure)
                        }
                        _ => {
                            warn!("Got incorrect TME signal.");
                            Err(VCFeError::EnableFailure)
                        }
                    },
                    _ => {
                        warn!("Got ToyUpdate in vc_enable().");
                        Err(VCFeError::EnableFailure)
                    }
                },
                None => {
                    warn!("Failed to recv from TME receiver.");
                    Err(VCFeError::EnableFailure)
                }
            }
        })
    }

    fn disable<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>> {
        Box::pin(async move {
            let mut vc_lock = vc_state.0.lock();
            trace!("Got vc_lock");
            if let RunningState::Stopped = vc_lock.running {
                return Err(VCFeError::DisableFailure);
            }

            if vc_lock.bp_client.is_none() {
                info!("ButtPlugClient is None");
                return Err(VCFeError::DisableFailure);
            }

            trace!("Calling destroy_toy_update_handler()");
            vc_lock.destroy_toy_update_handler().await;
            trace!("TUH destroyed");

            let bpc = vc_lock.bp_client.as_ref().unwrap();
            let _ = bpc.stop_scanning().await;
            let _ = bpc.stop_all_devices().await;

            info!("ButtplugClient stopped operations");

            vc_lock
                .tme_send_tx
                .send(ToyManagementEvent::Sig(TmSig::TMHReset))
                .unwrap();
            info!("Sent TMHReset signal");

            vc_lock.running = RunningState::Stopped;

            info!("Starting disabled state OSC cmd listener");
            match vc_lock.start_disabled_listener() {
                Ok(()) => (),
                Err(_e) => {
                    return Err(VCFeError::Vcore(
                        VcoreError::DisabledOscListenerThreadRunning,
                    ))
                }
            }

            Ok(())
        })
    }

    fn start_scan<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>> {
        Box::pin(async move {
            let vc_lock = vc_state.0.lock();

            if vc_lock.bp_client.is_none() {
                return Err(VCFeError::StartScanFailure(
                    "ButtplugClient is None".to_string(),
                ));
            }

            if let Err(e) =
                vc_lock
                    .tme_send_tx
                    .send(ToyManagementEvent::Sig(TmSig::StartListening(
                        vc_lock.config.networking.clone(),
                    )))
            {
                logerr!("Failed to send StartListening sig to tmh.");
                return Err(VCFeError::StartScanFailure(e.to_string()));
            }

            info!("StartListening sent to TMH. Trying to start scanning..");

            if let Err(e) = vc_lock.bp_client.as_ref().unwrap().start_scanning().await {
                let _ = vc_lock
                    .error_comm_tx
                    .as_ref()
                    .unwrap()
                    .send(VCError::HandlingErr(HandlerErr {
                        id: -2,
                        msg: format!("Failed to scan for bluetooth devices. {}", e),
                    }));
                logerr!("Failed to scan.");
                return Err(VCFeError::StartScanFailure(e.to_string()));
            }
            info!("Started Scanning..");
            Ok(())
        })
    }

    fn stop_scan<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
    ) -> BoxFuture<'a, Result<(), VCFeError>> {
        Box::pin(async move {
            let vc_lock = vc_state.0.lock();

            if vc_lock.bp_client.is_none() {
                return Err(VCFeError::StopScanFailure(
                    "ButtPlugClient is None".to_string(),
                ));
            }

            if let Err(e) = vc_lock.bp_client.as_ref().unwrap().stop_scanning().await {
                let _ = vc_lock
                    .error_comm_tx
                    .as_ref()
                    .unwrap()
                    .send(VCError::HandlingErr(HandlerErr {
                        id: -2,
                        msg: format!("Failed to stop scan for bluetooth devices. {}", e),
                    }));
                logerr!("Failed to stop scan.");
                return Err(VCFeError::StopScanFailure(e.to_string()));
            }
            info!("Stopped Scanning..");
            Ok(())
        })
    }

    fn alter_toy<'a>(
        &'a self,
        vc_state: tauri::State<'a, VCStateMutex>,
        app_handle: tauri::AppHandle,
        altered: VCToy,
    ) -> BoxFuture<'a, Result<(), ToyAlterError>> {
        Box::pin(async move {
            let alter_clone = altered.clone();
            altered.save_toy_config();
            info!("Altered toy config: {:?}", altered);

            let send_res = {
                let vc_lock = vc_state.0.lock();
                vc_lock
                    .tme_send_tx
                    .send(ToyManagementEvent::Tu(ToyUpdate::AlterToy(altered)))
            };

            emit_toy_event(
                &app_handle,
                FeToyEvent::Update({
                    FeVCToy {
                        toy_id: Some(alter_clone.toy_id),
                        toy_name: alter_clone.toy_name,
                        toy_anatomy: alter_clone.config.as_ref().unwrap().anatomy.to_fe(),
                        toy_power: alter_clone.toy_power,
                        toy_connected: alter_clone.toy_connected,
                        features: alter_clone.parsed_toy_features.features.to_frontend(),
                        listening: alter_clone.listening,
                        osc_data: alter_clone.osc_data,
                        bt_update_rate: alter_clone.bt_update_rate,
                        sub_id: alter_clone.sub_id,
                    }
                }),
            );

            match send_res {
                Ok(()) => Ok(()),
                Err(_e) => Err(ToyAlterError::TMESendFailure),
            }
        })
    }
}

pub async fn native_vibecheck_disable(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), VCFeError> {
    runtime_for(&vc_state).disable(vc_state).await
}

pub async fn native_vibecheck_enable(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), VCFeError> {
    runtime_for(&vc_state).enable(vc_state).await
}

pub fn native_osc_query_start(vc_state: tauri::State<'_, VCStateMutex>) -> Result<(), VCFeError> {
    let mut vc_lock = vc_state.0.lock();

    if vc_lock.osc_query_handler.is_none() {
        vc_lock.osc_query_init();
    }

    vc_lock
        .osc_query_handler
        .as_mut()
        .unwrap()
        .start_http_json();
    vc_lock
        .osc_query_handler
        .as_ref()
        .unwrap()
        .register_mdns_service();
    // This is only to attempt to auto-induce an mDNS response with separated answers.
    vc_lock.osc_query_associate();

    Ok(())
}

pub fn native_osc_query_stop(vc_state: tauri::State<'_, VCStateMutex>) -> Result<(), VCFeError> {
    let mut vc_lock = vc_state.0.lock();

    if vc_lock.osc_query_handler.is_none() {
        return Err(VCFeError::OSCQueryFailure("OSCQuery is not initialized"));
    }

    vc_lock.osc_query_fini();

    Ok(())
}

pub fn native_osc_query_attempt_force(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), VCFeError> {
    let vc_lock = vc_state.0.lock();

    if vc_lock.osc_query_handler.is_none() {
        return Err(VCFeError::OSCQueryFailure("OSCQuery is not initialized"));
    }

    // This is only to attempt to auto-induce an mDNS response with separated answers.
    vc_lock.osc_query_associate();

    Ok(())
}

pub fn osc_query_force_populate(vc_state: tauri::State<'_, VCStateMutex>) -> Result<(), VCFeError> {
    Ok(())
}

pub async fn native_vibecheck_start_bt_scan(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), VCFeError> {
    runtime_for(&vc_state).start_scan(vc_state).await
}

pub async fn native_vibecheck_stop_bt_scan(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), VCFeError> {
    runtime_for(&vc_state).stop_scan(vc_state).await
}

pub fn native_get_vibecheck_config(vc_state: tauri::State<'_, VCStateMutex>) -> FeVibeCheckConfig {
    let config = {
        let vc_lock = vc_state.0.lock();
        vc_lock.config.clone()
    };

    FeVibeCheckConfig {
        networking: config.networking.to_fe(),
        scan_on_disconnect: config.scan_on_disconnect,
        minimize_on_exit: config.minimize_on_exit,
        desktop_notifications: config.desktop_notifications,
        show_toy_advanced: config.show_toy_advanced,
        show_feature_advanced: config.show_feature_advanced,
    }
}

pub fn native_set_vibecheck_config(
    vc_state: tauri::State<'_, VCStateMutex>,
    fe_vc_config: FeVibeCheckConfig,
) -> Result<(), VCFeError> {
    info!("Got fe_vc_config: {:?}", fe_vc_config);
    let bind = match SocketAddrV4::from_str(&fe_vc_config.networking.bind) {
        Ok(sa) => sa,
        Err(_e) => return Err(VCFeError::InvalidBindEndpoint),
    };

    let remote = match SocketAddrV4::from_str(&fe_vc_config.networking.remote) {
        Ok(sa) => sa,
        Err(_e) => return Err(VCFeError::InvalidRemoteEndpoint),
    };

    let config = {
        let mut vc_lock = vc_state.0.lock();
        vc_lock.config.networking.bind = bind;
        vc_lock.config.networking.remote = remote;
        vc_lock.config.scan_on_disconnect = fe_vc_config.scan_on_disconnect;
        vc_lock.config.minimize_on_exit = fe_vc_config.minimize_on_exit;
        vc_lock.config.desktop_notifications = fe_vc_config.desktop_notifications;
        vc_lock.config.show_toy_advanced = fe_vc_config.show_toy_advanced;
        vc_lock.config.show_feature_advanced = fe_vc_config.show_feature_advanced;

        vc_lock.config.clone()
    };

    match save_config(config) {
        Ok(()) => Ok(()),
        Err(e) => match e {
            VibeCheckConfigError::SerializeError => Err(VCFeError::SerializeFailure),
            VibeCheckConfigError::WriteFailure => Err(VCFeError::WriteFailure),
            VibeCheckConfigError::ConfigDirFail => Err(VCFeError::ConfigDirFailure),
        },
    }
}

fn save_config(config: VibeCheckConfig) -> Result<(), VibeCheckConfigError> {
    let json_config_str = match serde_json::to_string(&config) {
        Ok(s) => s,
        Err(_e) => {
            logerr!("[!] Failed to serialize VibeCheckConfig into a String.");
            return Err(VibeCheckConfigError::SerializeError);
        }
    };

    let config_dir = match get_config_dir() {
        Ok(d) => d,
        Err(_) => return Err(VibeCheckConfigError::ConfigDirFail),
    };

    match fs::write(format!("{}\\Config.json", config_dir), json_config_str) {
        Ok(()) => {}
        Err(_e) => {
            logerr!("[!] Failure writing VibeCheck config.");
            return Err(VibeCheckConfigError::WriteFailure);
        }
    }
    Ok(())
}

pub fn native_alter_toy(
    vc_state: tauri::State<'_, VCStateMutex>,
    app_handle: tauri::AppHandle,
    altered: VCToy,
) -> Result<(), ToyAlterError> {
    tauri::async_runtime::block_on(runtime_for(&vc_state).alter_toy(vc_state, app_handle, altered))
}

pub fn native_clear_osc_config() -> Result<(), VibeCheckFSError> {
    let home_dir = match get_user_home_dir() {
        Ok(hd) => hd,
        Err(_) => return Err(VibeCheckFSError::ReadDirFailure),
    };

    let osc_dirs = match std::fs::read_dir(format!(
        "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\",
        home_dir
    )) {
        Ok(dirs) => dirs,
        Err(_e) => return Err(VibeCheckFSError::ReadDirFailure),
    };

    //info!("osc_dirs: {}", osc_dirs.count());

    let usr_dirs = match osc_dirs
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
    {
        Ok(usr_dirs) => usr_dirs,
        Err(_) => return Err(VibeCheckFSError::ReadDirPathFailure),
    };

    for dir in usr_dirs {
        if dir.is_dir() {
            let dir_path = dir.file_name().unwrap().to_str().unwrap();
            info!("Got Dir: {}", dir_path);

            if dir
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("usr_")
            {
                let delete_dir = dir.as_path().to_str().unwrap();
                info!("Clearing dir: {}", delete_dir);
                match std::fs::remove_dir_all(delete_dir) {
                    Ok(()) => {}
                    Err(_e) => return Err(VibeCheckFSError::RemoveDirsFailure),
                }
            }
        }
    }
    Ok(())
}

pub fn native_simulate_device_feature(
    vc_state: tauri::State<'_, VCStateMutex>,
    toy_id: u32,
    feature_index: u32,
    feature_type: FeVCFeatureType,
    float_level: f64,
    stop: bool,
) {
    let vc_toys = {
        let vc_lock = vc_state.0.lock();
        vc_lock
            .core_toy_manager
            .as_ref()
            .unwrap()
            .online_toys
            .clone()
    };

    let toy = match vc_toys.get(&toy_id) {
        Some(toy) => toy,
        None => return,
    }
    .clone();

    // Need to filter between ScalarCmd's and non ScalarCmd's
    for feature in toy.parsed_toy_features.features {
        // Check that feature index and feature type are the same.
        // Have to do this due to feature type separation between FE and BE. And buttplug IO mixing scalar rotator and normal rotator commands.
        // Could make this a bit simpler by creating ScalarTYPE types and converting their names in the frontend.
        if feature.feature_index == feature_index
            && (feature.feature_type == feature_type
                || feature.feature_type == VCFeatureType::ScalarRotator
                    && feature_type == FeVCFeatureType::Rotator)
        {
            let handle_clone = toy.device_handle.clone();
            {
                let vc_lock = vc_state.0.lock();
                // Add stop flag bc FE invoke simulation: diff between stop & idle.
                if stop {
                    debug!("Stopping Idle Simulate");
                    vc_lock.async_rt.spawn(handle_clone.stop());
                } else {
                    vc_lock.async_rt.spawn(command_toy(
                        handle_clone,
                        feature.feature_type,
                        float_level,
                        feature.feature_index,
                        feature.flip_input_float,
                        feature.feature_levels,
                    ));
                }
            }
            return;
        }
    }
}

/* Leaving this here in case of future use
 *
pub fn native_simulate_feature_osc_input(vc_state: tauri::State<'_, VCStateMutex>, simulated_param_address: String, simulated_param_value: f32) {

    let osc_buf = match encoder::encode(&OscPacket::Message(OscMessage {
        addr: simulated_param_address.clone(),
        args: vec![OscType::Float(simulated_param_value)],
    })) {
        Ok(buf) => buf,
        Err(_e) => return,
    };

    let simulation_sock = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)) {
        Ok(sim_sock) => sim_sock,
        Err(_e) => return,
    };

    let self_osc_bind_address = {
        let vc_config = vc_state.0.lock();
        vc_config.config.networking.bind
    };

    let _ = simulation_sock.send_to(&osc_buf, self_osc_bind_address);
    std::thread::sleep(std::time::Duration::from_secs(1));

    let osc_buf = match encoder::encode(&OscPacket::Message(OscMessage {
        addr: simulated_param_address,
        args: vec![OscType::Float(0.0)],
    })) {
        Ok(buf) => buf,
        Err(_e) => return,
    };

    let _ = simulation_sock.send_to(&osc_buf, self_osc_bind_address);
}
 *
 */
