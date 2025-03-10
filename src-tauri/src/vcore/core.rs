use buttplug::client::ButtplugClient;
use log::{debug, error as logerr, info, trace, warn};
use std::fs;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

use crate::frontend::frontend_types::{FeToyEvent, FeVCFeatureType, FeVCToy, FeVibeCheckConfig};
use crate::frontend::ToFrontend;
use crate::osc::logic::{toy_refresh, vc_disabled_osc_command_listen};
use crate::toy_handling::errors::HandlerErr;
use crate::toy_handling::runtime::client_event_handler::client_event_handler;
use crate::toy_handling::toy_command_processor::command_toy;
use crate::toy_handling::toy_manager::ToyManager;
use crate::toy_handling::toyops::VCFeatureType;
use crate::util::bluetooth;
use crate::util::fs::{get_config_dir, get_user_home_dir};
use crate::util::net::{find_available_tcp_port, find_available_udp_port};
use crate::vcore::vcerror::{backend, frontend};
use crate::{
    config::{OSCNetworking, VibeCheckConfig},
    toy_handling::runtime::toy_management_handler::toy_management_handler,
    toy_handling::toyops::VCToy,
};
use parking_lot::Mutex;
use tauri::{AppHandle, Manager};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use vrcoscquery::OSCQuery;

use tokio::sync::{mpsc::unbounded_channel, mpsc::UnboundedReceiver, mpsc::UnboundedSender};

pub struct VCStateMutex(pub Arc<Mutex<VibeCheckState>>);

pub struct VibeCheckState {
    pub app_handle: Option<AppHandle>,
    pub identifier: String,

    pub config: VibeCheckConfig,
    pub osc_query_handler: Option<OSCQuery>,
    //pub connection_modes: ConnectionModes,
    pub bp_client: Option<ButtplugClient>,

    pub running: RunningState,
    pub core_toy_manager: Option<ToyManager>,
    //pub offline_toys: OfflineToys,
    //================================================
    // Handlers error recvr
    //inner_channels: Arc<RwLock<innerChannels>>,
    pub error_rx: Receiver<VCError>,
    pub error_tx: Sender<VCError>,
    //================================================
    // Disabled listener thread handle
    pub disabled_osc_listener_h_thread: Option<JoinHandle<()>>,
    //================================================
    // Client Event Handler
    pub client_eh_thread: Option<JoinHandle<()>>,
    //pub client_eh_event_rx: Arc<Mutex<UnboundedReceiver<EventSig>>>,
    //pub client_eh_event_tx: UnboundedSender<EventSig>,
    //================================================
    //Toy update handler
    pub toy_update_h_thread: Option<JoinHandle<()>>,
    // Toy Management Handler
    pub toy_management_h_thread: Option<JoinHandle<()>>,
    // These stay in VibeCheckState
    pub tme_recv_rx: UnboundedReceiver<ToyManagementEvent>,
    pub tme_send_tx: UnboundedSender<ToyManagementEvent>,
    // These go in TMH. Wrapped in Option so they can be moved into TMH.
    pub tme_recv_tx: Option<UnboundedSender<ToyManagementEvent>>,
    pub tme_send_rx: Option<UnboundedReceiver<ToyManagementEvent>>,
    //================================================
    // Message handler
    pub message_handler_thread: Option<JoinHandle<()>>,
    pub vibecheck_state_pointer: Option<Arc<Mutex<VibeCheckState>>>,
    //================================================
    //pub toy_input_h_thread: Option<JoinHandle<()>>,
    //================================================
    // Async Runtime for toy event handlers
    pub async_rt: Runtime,
    //================================================
}

impl VibeCheckState {
    pub fn new(config: VibeCheckConfig) -> Self {
        // Toys hashmap
        //let core_toy_manager = ToyHandler::new();

        // Create error handling/passig channels
        let (error_tx, error_rx): (Sender<VCError>, Receiver<VCError>) = mpsc::channel();

        // Create async runtime for toy handling routines
        let async_rt = Runtime::new().unwrap();

        // Setup channels
        let (tme_recv_tx, tme_recv_rx): (
            UnboundedSender<ToyManagementEvent>,
            UnboundedReceiver<ToyManagementEvent>,
        ) = unbounded_channel();
        let (tme_send_tx, tme_send_rx): (
            UnboundedSender<ToyManagementEvent>,
            UnboundedReceiver<ToyManagementEvent>,
        ) = unbounded_channel();

        Self {
            app_handle: None,
            identifier: String::new(),
            config,
            osc_query_handler: None,
            //connection_modes,
            bp_client: None,
            running: RunningState::Stopped,
            core_toy_manager: None,
            //======================================
            // Error channels
            error_rx,
            error_tx,

            //======================================
            // Disabled listener thread
            disabled_osc_listener_h_thread: None,
            //======================================
            // Client Event Handler
            client_eh_thread: None,
            //client_eh_event_rx,
            //client_eh_event_tx,

            //======================================
            // Toy update handler
            toy_update_h_thread: None,

            //======================================
            // Toy Management Handler
            toy_management_h_thread: None,
            tme_recv_rx,
            tme_send_tx,

            tme_recv_tx: Some(tme_recv_tx),
            tme_send_rx: Some(tme_send_rx),

            //================================================
            // Message handler
            message_handler_thread: None,
            vibecheck_state_pointer: None,

            //======================================
            // Async runtime
            async_rt,
        }
    }

    pub fn start_tmh(&mut self) {
        if self.app_handle.is_none() {
            logerr!("start_tmh() called but no app_handle was set");
            return;
        }

        if self.core_toy_manager.is_none() {
            logerr!("start_tmh() called but no core_toy_manager was set");
            return;
        }

        self.toy_management_h_thread = Some(self.async_rt.spawn(toy_management_handler(
            self.tme_recv_tx.take().unwrap(),
            self.tme_send_rx.take().unwrap(),
            self.core_toy_manager.as_ref().unwrap().clone(),
            self.config.networking.clone(),
            self.app_handle.as_ref().unwrap().clone(),
        )));
        info!("TMH started");
    }

    pub fn start_disabled_listener(&mut self) {
        if self.disabled_osc_listener_h_thread.is_some() {
            return;
        }

        self.disabled_osc_listener_h_thread =
            Some(self.async_rt.spawn(vc_disabled_osc_command_listen(
                self.app_handle.as_ref().unwrap().clone(),
                self.config.networking.clone(),
            )));
    }

    pub async fn stop_disabled_listener(&mut self) {
        if self.disabled_osc_listener_h_thread.is_none() {
            return;
        }

        let dol_thread = self.disabled_osc_listener_h_thread.take().unwrap();
        dol_thread.abort();
        match dol_thread.await {
            Ok(()) => info!("DOL thread finished"),
            Err(e) => warn!("DOL thread failed to reach completion: {}", e),
        }
    }

    pub fn set_state_pointer(&mut self, vibecheck_state_pointer: Arc<Mutex<VibeCheckState>>) {
        self.vibecheck_state_pointer = Some(vibecheck_state_pointer);
    }
    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }
    pub fn init_toy_manager(&mut self) {
        self.core_toy_manager = Some(ToyManager::new(self.app_handle.as_ref().unwrap().clone()));
    }

    pub fn init_ceh(&mut self) {
        // Is there a supplied state pointer?
        if self.vibecheck_state_pointer.is_none() {
            return;
        }

        // Is CEH already running?

        if self.client_eh_thread.is_some() {
            return;
        }

        // Create connection mode defaults
        //let mut connection_modes = ConnectionModes { btle_enabled: true, lc_enabled: true };

        // Get ButtPlugClient with modified connection modes
        self.bp_client = Some(
            self.async_rt
                .block_on(bluetooth::vc_toy_client_server_init("VibeCheck", false)),
        );
        info!("Buttplug Client Initialized.");

        // Get event stream
        let event_stream = self.bp_client.as_ref().unwrap().event_stream();

        // Start CEH
        self.client_eh_thread = Some(self.async_rt.spawn(client_event_handler(
            event_stream,
            self.vibecheck_state_pointer.as_ref().unwrap().clone(),
            self.identifier.clone(),
            self.app_handle.as_ref().unwrap().clone(),
            self.tme_send_tx.clone(),
            self.error_tx.clone(),
        )));
    }

    /*
    pub async fn destroy_ceh(&mut self) {

        if self.client_eh_thread.is_none() {
            return;
        }

        let ceh_thread = self.client_eh_thread.take().unwrap();
        ceh_thread.abort();
        match ceh_thread.await {
            Ok(()) => info!("CEH thread finished"),
            Err(e) => warn!("CEH thread failed to reach completion: {}", e),
        }
    }
    */
    pub async fn init_toy_update_handler(&mut self) {
        // Is there a supplied state pointer?
        if self.vibecheck_state_pointer.is_none() {
            return;
        }

        // Is CEH running?
        if self.client_eh_thread.is_none() {
            return;
        }

        if self.app_handle.is_none() {
            return;
        }

        self.toy_update_h_thread = Some(self.async_rt.spawn(toy_refresh(
            self.vibecheck_state_pointer.as_ref().unwrap().clone(),
            self.app_handle.as_ref().unwrap().clone(),
        )));
        info!("TUH thread started");
    }

    pub async fn destroy_toy_update_handler(&mut self) {
        if self.toy_update_h_thread.is_none() {
            return;
        }

        let tuh_thread = self.toy_update_h_thread.take().unwrap();
        tuh_thread.abort();
        match tuh_thread.await {
            Ok(()) => info!("TUH thread finished"),
            Err(e) => warn!("TUH thread failed to reach completion: {}", e),
        }
    }

    pub fn osc_query_init(&mut self) {
        let available_tcp_port =
            find_available_tcp_port(self.config.networking.bind.ip().to_string());
        let available_udp_port =
            find_available_udp_port(self.config.networking.bind.ip().to_string());

        let http_net = SocketAddrV4::new(
            *self.config.networking.bind.ip(),
            available_tcp_port.unwrap(),
        );
        let osc_net = SocketAddrV4::new(
            *self.config.networking.bind.ip(),
            available_udp_port.unwrap(),
        );

        self.osc_query_handler = Some(OSCQuery::new("VibeCheck".to_string(), http_net, osc_net));
        self.config
            .networking
            .bind
            .set_port(available_udp_port.unwrap());
    }

    pub fn osc_query_fini(&mut self) {
        if self.osc_query_handler.is_some() {
            let mut h = self.osc_query_handler.take().unwrap();
            h.stop_http_json();
            h.unregister_mdns_service();
            h.shutdown_mdns();
        }
    }

    pub fn osc_query_associate(&self) {
        if self.osc_query_handler.is_some() {
            self.osc_query_handler
                .as_ref()
                .unwrap()
                .attempt_force_vrc_response_detect(10);
        }
    }
}

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

pub enum VCError {
    HandlingErr(crate::toy_handling::errors::HandlerErr),
}

pub enum RunningState {
    Running,
    Stopped,
}

pub async fn native_vibecheck_disable(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    let mut vc_lock = vc_state.0.lock();
    trace!("Got vc_lock");
    if let RunningState::Stopped = vc_lock.running {
        return Err(frontend::VCFeError::DisableFailure);
    }

    if vc_lock.bp_client.is_none() {
        info!("ButtPlugClient is None");
        return Err(frontend::VCFeError::DisableFailure);
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
    vc_lock.start_disabled_listener();

    Ok(())
}

pub async fn native_vibecheck_enable(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    // Send Start listening signal

    let mut vc_lock = vc_state.0.lock();
    if let RunningState::Running = vc_lock.running {
        //return Err(frontend::VCFeError::EnableFailure);
        // Don't fail if already enabled
        return Ok(());
    }

    if vc_lock.bp_client.is_none() {
        return Err(frontend::VCFeError::EnableFailure);
    }

    info!("Stopping DOL");
    vc_lock.stop_disabled_listener().await;

    /* No longer disabling CEH
    vc_lock.init_ceh().await;
    info!("CEH initialized");
    */

    vc_lock
        .tme_send_tx
        .send(ToyManagementEvent::Sig(TmSig::StartListening(
            vc_lock.config.networking.clone(),
        )))
        .unwrap();

    // Check if listening succeded or not
    match vc_lock.tme_recv_rx.recv().await {
        Some(tme) => {
            match tme {
                ToyManagementEvent::Sig(sig) => {
                    match sig {
                        TmSig::Listening => {
                            vc_lock.running = RunningState::Running;

                            // Enable successful
                            // Start TUH thread
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
                            Err(frontend::VCFeError::EnableBindFailure)
                        }
                        _ => {
                            //Did not get the correct signal oops
                            warn!("Got incorrect TME signal.");
                            Err(frontend::VCFeError::EnableFailure)
                        }
                    }
                }
                _ => {
                    warn!("Got ToyUpdate in vc_enable().");
                    Err(frontend::VCFeError::EnableFailure)
                } // Got unexpected Sig
            }
        }
        None => {
            warn!("Failed to recv from TME receiver.");
            Err(frontend::VCFeError::EnableFailure)
        } // Recv failed
    } // tme recv
}

pub fn native_osc_query_start(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
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

pub fn native_osc_query_stop(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    let mut vc_lock = vc_state.0.lock();

    if vc_lock.osc_query_handler.is_none() {
        return Err(frontend::VCFeError::OSCQueryFailure(
            "OSCQuery is not initialized",
        ));
    }

    vc_lock.osc_query_fini();

    Ok(())
}

pub fn native_osc_query_attempt_force(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    let vc_lock = vc_state.0.lock();

    if vc_lock.osc_query_handler.is_none() {
        return Err(frontend::VCFeError::OSCQueryFailure(
            "OSCQuery is not initialized",
        ));
    }

    // This is only to attempt to auto-induce an mDNS response with separated answers.
    vc_lock.osc_query_associate();

    Ok(())
}

pub fn osc_query_force_populate(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    Ok(())
}

pub async fn native_vibecheck_start_bt_scan(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    let vc_lock = vc_state.0.lock();

    if vc_lock.bp_client.is_none() {
        // ButtPlugClient not created (CEH is probably not running)
        return Err(frontend::VCFeError::StartScanFailure(
            "ButtplugClient is None".to_string(),
        ));
    }

    // Start scanning for toys
    if let Err(e) = vc_lock.bp_client.as_ref().unwrap().start_scanning().await {
        let _ = vc_lock.error_tx.send(VCError::HandlingErr(HandlerErr {
            id: -2,
            msg: format!("Failed to scan for bluetooth devices. {}", e),
        }));
        logerr!("Failed to scan.");
        return Err(frontend::VCFeError::StartScanFailure(e.to_string()));
    }
    info!("Started Scanning..");
    Ok(())
}

pub async fn native_vibecheck_stop_bt_scan(
    vc_state: tauri::State<'_, VCStateMutex>,
) -> Result<(), frontend::VCFeError> {
    let vc_lock = vc_state.0.lock();

    if vc_lock.bp_client.is_none() {
        // ButtPlugClient not created (CEH is probably not running)
        return Err(frontend::VCFeError::StopScanFailure(
            "ButtPlugClient is None".to_string(),
        ));
    }

    // Stop scanning for toys
    if let Err(e) = vc_lock.bp_client.as_ref().unwrap().stop_scanning().await {
        let _ = vc_lock.error_tx.send(VCError::HandlingErr(HandlerErr {
            id: -2,
            msg: format!("Failed to stop scan for bluetooth devices. {}", e),
        }));
        logerr!("Failed to stop scan.");
        return Err(frontend::VCFeError::StopScanFailure(e.to_string()));
    }
    info!("Stopped Scanning..");
    Ok(())
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
) -> Result<(), frontend::VCFeError> {
    info!("Got fe_vc_config: {:?}", fe_vc_config);
    let bind = match SocketAddrV4::from_str(&fe_vc_config.networking.bind) {
        Ok(sa) => sa,
        Err(_e) => return Err(frontend::VCFeError::InvalidBindEndpoint),
    };

    let remote = match SocketAddrV4::from_str(&fe_vc_config.networking.remote) {
        Ok(sa) => sa,
        Err(_e) => return Err(frontend::VCFeError::InvalidRemoteEndpoint),
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
            backend::VibeCheckConfigError::SerializeError => {
                Err(frontend::VCFeError::SerializeFailure)
            }
            backend::VibeCheckConfigError::WriteFailure => Err(frontend::VCFeError::WriteFailure),
        },
    }
}

fn save_config(
    config: crate::config::VibeCheckConfig,
) -> Result<(), backend::VibeCheckConfigError> {
    let json_config_str = match serde_json::to_string(&config) {
        Ok(s) => s,
        Err(_e) => {
            logerr!("[!] Failed to serialize VibeCheckConfig into a String.");
            return Err(backend::VibeCheckConfigError::SerializeError);
        }
    };

    match fs::write(
        format!("{}\\Config.json", get_config_dir()),
        json_config_str,
    ) {
        Ok(()) => {}
        Err(_e) => {
            logerr!("[!] Failure writing VibeCheck config.");
            return Err(backend::VibeCheckConfigError::WriteFailure);
        }
    }
    Ok(())
}

pub fn native_alter_toy(
    vc_state: tauri::State<'_, VCStateMutex>,
    app_handle: tauri::AppHandle,
    altered: VCToy,
) -> Result<(), backend::ToyAlterError> {
    let alter_clone = altered.clone();
    altered.save_toy_config();
    info!("Altered toy config: {:?}", altered);

    let send_res = {
        let vc_lock = vc_state.0.lock();
        vc_lock
            .tme_send_tx
            .send(ToyManagementEvent::Tu(ToyUpdate::AlterToy(altered)))
    };

    let _ = app_handle.emit_all(
        "fe_toy_event",
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
                sub_id: alter_clone.sub_id,
            }
        }),
    );

    match send_res {
        Ok(()) => Ok(()),
        Err(_e) => Err(backend::ToyAlterError::TMESendFailure),
    }
}

pub fn native_clear_osc_config() -> Result<(), backend::VibeCheckFSError> {
    let osc_dirs = match std::fs::read_dir(format!(
        "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\",
        get_user_home_dir()
    )) {
        Ok(dirs) => dirs,
        Err(_e) => return Err(backend::VibeCheckFSError::ReadDirFailure),
    };

    //info!("osc_dirs: {}", osc_dirs.count());

    let usr_dirs = match osc_dirs
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
    {
        Ok(usr_dirs) => usr_dirs,
        Err(_) => return Err(backend::VibeCheckFSError::ReadDirPathFailure),
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
                    Err(_e) => return Err(backend::VibeCheckFSError::RemoveDirsFailure),
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
