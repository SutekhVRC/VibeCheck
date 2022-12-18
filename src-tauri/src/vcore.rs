use std::collections::HashMap;
use std::fs;
use std::net::{SocketAddrV4, Ipv4Addr};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
//use std::time::Duration;
use buttplug::client::ButtplugClient;
//use futures_timer::Delay;
use log::{warn, error as logerr, info, trace};
use tauri::{AppHandle, Manager};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use parking_lot::Mutex;
use crate::bluetooth;
//use crate::config::save_toy_config;
use crate::handling::{HandlerErr, toy_refresh, vc_disabled_osc_command_listen};
use crate::frontend_types::{FeVCToy, FeVibeCheckConfig, FeOSCNetworking, FeToyAlter, FeToyEvent};
use crate::vcerror::{backend, frontend};
//use crate::vcupdate::{VibeCheckUpdater, VERSION};
use crate::{
    util::get_user_home_dir,
    handling::{client_event_handler, toy_management_handler},
    config::{
        VibeCheckConfig,
        OSCNetworking,
    },
    toyops::{
        VCToy,
    },
};

use tokio::sync::{
    mpsc::unbounded_channel,
    mpsc::UnboundedReceiver,
    mpsc::UnboundedSender,
};

/*
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionModes {
    btle_enabled: bool,
    lc_enabled: bool,
}*/

pub struct VCStateMutex(pub Arc<Mutex<VibeCheckState>>);

pub struct VibeCheckState {

    pub app_handle: Option<AppHandle>,
    pub identifier: String,

    pub config: VibeCheckConfig,

    //pub connection_modes: ConnectionModes,

    pub bp_client: Option<ButtplugClient>,

    pub running: RunningState,
    pub toys: HashMap<u32, VCToy>,
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
        let toys = HashMap::new();

        // Create error handling/passig channels
        let (error_tx, error_rx): (Sender<VCError>, Receiver<VCError>) = mpsc::channel();

        // Create async runtime for toy handling routines
        let async_rt = Runtime::new().unwrap();


        //let connection_modes = ConnectionModes { btle_enabled: true, lc_enabled: true };

        // Setup channels
        let (tme_recv_tx, tme_recv_rx): (UnboundedSender<ToyManagementEvent>, UnboundedReceiver<ToyManagementEvent>) = unbounded_channel();
        let (tme_send_tx, tme_send_rx): (UnboundedSender<ToyManagementEvent>, UnboundedReceiver<ToyManagementEvent>) = unbounded_channel();

        // Main thread toy management event bidirectional channels
        /*
        let tme_recv = tme_recv_rx;
        let tme_send = tme_send_tx;
        */
        // Start toy management thread
        /*
        let toy_management_h_thread = async_rt.spawn(toy_management_handler(
            tme_recv_tx,
            tme_send_rx,
            toys.clone(),
            config.networking.clone(),
            app_handle.clone(),
        ));
        */

        // Timer prob remove idrc
        //let minute_sync = Instant::now();
        

        Self {

            app_handle: None,
            identifier: String::new(),
            config,
            //connection_modes,
            bp_client: None,
            running: RunningState::Stopped,
            toys,

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
        self.toy_management_h_thread = Some(self.async_rt.spawn(toy_management_handler(
            self.tme_recv_tx.take().unwrap(),
            self.tme_send_rx.take().unwrap(),
            self.toys.clone(),
            self.config.networking.clone(),
            self.app_handle.as_ref().unwrap().clone(),
        )));
        info!("TMH started");
    }

    pub fn start_disabled_listener(&mut self) {

        if self.disabled_osc_listener_h_thread.is_some() {
            return;
        }

        self.disabled_osc_listener_h_thread = Some(self.async_rt.spawn(
            vc_disabled_osc_command_listen(            
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
        self.bp_client = Some(self.async_rt.block_on(bluetooth::vc_toy_client_server_init("VibeCheck", false)));
        info!("Buttplug Client Initialized.");

        // Get event stream
        let event_stream = self.bp_client.as_ref().unwrap().event_stream();

        // Start CEH
        self.client_eh_thread = Some(self.async_rt.spawn(
            client_event_handler(
                event_stream,
                self.vibecheck_state_pointer.as_ref().unwrap().clone(),
                self.identifier.clone(),
                self.app_handle.as_ref().unwrap().clone(),
                self.tme_send_tx.clone(),
                self.error_tx.clone()
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

        self.toy_update_h_thread = Some(self.async_rt.spawn(toy_refresh(self.vibecheck_state_pointer.as_ref().unwrap().clone(), self.app_handle.as_ref().unwrap().clone())));
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
    HandlingErr(crate::handling::HandlerErr),
}

pub enum RunningState {
    Running,
    Stopped,
}

/*
fn update_vibecheck(&mut self) {
    if let RunningState::Running = self.running {
        self.disable_vibecheck();
    }
    let blob = self.update_engine.release_blob.take().unwrap();

    thread::spawn(move || {
        VibeCheckUpdater::update_vibecheck(blob);
    });
    thread::sleep(std::time::Duration::from_secs(1));
    std::process::exit(0);
}*/

pub async fn native_vibecheck_disable(vc_state: tauri::State<'_, VCStateMutex>) -> Result<(), frontend::VCFeError> {

    let mut vc_lock = vc_state.0.lock();
    trace!("Got vc_lock");
    if let RunningState::Stopped = vc_lock.running {
        return Err(frontend::VCFeError::DisableFailure);
    }

    if vc_lock.bp_client.is_none() {
        info!("ButtPlugClient is None");
        return Err(frontend::VCFeError::DisableFailure);
    }
    

    //Delay::new(Duration::from_secs(10)).await;
    trace!("Calling destroy_toy_update_handler()");
    vc_lock.destroy_toy_update_handler().await;
    trace!("TUH destroyed");

    
    let bpc = vc_lock.bp_client.as_ref().unwrap();
    let _ = bpc.stop_scanning().await;
    let _ = bpc.stop_all_devices().await;
    //let _ = bpc.disconnect().await;
    //Delay::new(Duration::from_secs(10)).await;
    //drop(bpc);
    info!("ButtplugClient stopped operations");

    // CEH no longer gets destroyed
    //trace!("Calling destroy_ceh()");
    //vc_lock.destroy_ceh().await;
    //info!("CEH destroyed");


    //Delay::new(Duration::from_secs(10)).await;
    vc_lock.tme_send_tx
    .send(ToyManagementEvent::Sig(TmSig::TMHReset))
    .unwrap();
    info!("Sent TMHReset signal");

    // Dont clear toys anymore
    //vc_lock.toys.clear();
    //info!("Cleared toys in VibeCheckState");
    //let _ = vc_lock.bp_client.as_ref().unwrap().stop_all_devices().await;
    vc_lock.running = RunningState::Stopped;

    info!("Starting disabled state OSC cmd listener");
    vc_lock.start_disabled_listener();

    Ok(())
}

pub async fn native_vibecheck_enable(vc_state: tauri::State<'_, VCStateMutex>) -> Result<(), frontend::VCFeError> {
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
    
    vc_lock.tme_send_tx.send(ToyManagementEvent::Sig(TmSig::StartListening(vc_lock.config.networking.clone()))).unwrap();
    
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
                        },
                        TmSig::BindError => {

                            logerr!("Bind Error in TME sig: Sending shutdown signal!");

                            vc_lock.tme_send_tx.send(ToyManagementEvent::Sig(TmSig::StopListening)).unwrap();
                            vc_lock.running = RunningState::Stopped;

                            return Err(frontend::VCFeError::EnableBindFailure);
                        },
                        _ => {//Did not get the correct signal oops
                            warn!("Got incorrect TME signal.");
                            Err(frontend::VCFeError::EnableFailure)
                        }, 
                    }
                },
                _ => {
                    warn!("Got ToyUpdate in vc_enable().");
                    Err(frontend::VCFeError::EnableFailure)
                },// Got unexpected Sig
            }
        },
        None => {
            warn!("Failed to recv from TME receiver.");
            Err(frontend::VCFeError::EnableFailure)
        },// Recv failed
    }// tme recv
}

pub async fn native_vibecheck_start_bt_scan(vc_state: tauri::State<'_, VCStateMutex>) -> Result<(), frontend::VCFeError>{
    let vc_lock = vc_state.0.lock();

    if vc_lock.bp_client.is_none() {
        // ButtPlugClient not created (CEH is probably not running)
        return Err(frontend::VCFeError::StartScanFailure("ButtPlugClient is None".to_string()));
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

pub async fn native_vibecheck_stop_bt_scan(vc_state: tauri::State<'_, VCStateMutex>) -> Result<(), frontend::VCFeError> {
    let vc_lock = vc_state.0.lock();

    if vc_lock.bp_client.is_none() {
        // ButtPlugClient not created (CEH is probably not running)
        return Err(frontend::VCFeError::StopScanFailure("ButtPlugClient is None".to_string()));
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

    let lc_or = {
        if let Some(host) = config.lc_override {
            Some(host.to_string())
        } else {
            None
        }
    };

    FeVibeCheckConfig {
        networking: FeOSCNetworking {
            bind: config.networking.bind.to_string(),
            remote: config.networking.remote.to_string(),
        },
        scan_on_disconnect: config.scan_on_disconnect,
        minimize_on_exit: config.minimize_on_exit,
        desktop_notifications: config.desktop_notifications,
        lc_override: lc_or,
    }
}

pub fn native_set_vibecheck_config(vc_state: tauri::State<'_, VCStateMutex>, fe_vc_config: FeVibeCheckConfig) -> Result<(), frontend::VCFeError> {

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

        if let Some(host) = fe_vc_config.lc_override {
            // Is valid IPv4?
            match Ipv4Addr::from_str(&host) {
                Ok(sa) => {
                    // Force port because buttplug forces non http atm
                    std::env::set_var("VCLC_HOST_PORT", format!("{}:20010", sa.to_string()).as_str());
                    match std::env::var("VCLC_HOST_PORT") {
                        Ok(_) => {
                            vc_lock.config.lc_override = Some(sa);
                        },
                        Err(_) => return Err(frontend::VCFeError::SetLCOverrideFailure),
                    }
                },
                Err(_e) => return Err(frontend::VCFeError::InvalidLCHost),
            };

        } else {
            std::env::remove_var("VCLC_HOST_PORT");
            match std::env::var("VCLC_HOST_PORT") {
                Ok(_) => return Err(frontend::VCFeError::UnsetLCOverrideFailure),
                Err(e) => {
                    match e {
                        std::env::VarError::NotPresent => {
                            vc_lock.config.lc_override = None;
                        },
                        _ => {
                            logerr!("Got Non unicode var during unset routine");
                            return Err(frontend::VCFeError::UnsetLCOverrideFailure);
                        },
                    }
                }
            }
        }

        vc_lock.config.clone()
    };

    match save_config(config) {
        Ok(()) => Ok(()),
        Err(e) => {
            match e {
                backend::VibeCheckConfigError::SerializeError => Err(frontend::VCFeError::SerializeFailure),
                backend::VibeCheckConfigError::WriteFailure => Err(frontend::VCFeError::WriteFailure),
            }
        }
    }

}

fn save_config(config: crate::config::VibeCheckConfig) -> Result<(), backend::VibeCheckConfigError> {

    let json_config_str = match serde_json::to_string(&config) {
        Ok(s) => s,
        Err(_e) => {
            logerr!("[!] Failed to serialize VibeCheckConfig into a String.");
            return Err(backend::VibeCheckConfigError::SerializeError);
        }
    };

    match fs::write(
        format!(
            "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck\\Config.json",
            get_user_home_dir()
        ),
        json_config_str,
    ) {
        Ok(()) => {},
        Err(_e) => {
            logerr!("[!] Failure writing VibeCheck config.");
            return Err(backend::VibeCheckConfigError::WriteFailure);
        }
    }
    Ok(())
}

/*
pub struct FrontendVCToyModel {
    pub toy_id: u32,
    pub toy_name: String,
    pub battery_level: f64,
    pub toy_connected: bool,
    pub osc_params_list: Vec<String>,
    pub param_feature_map: FeatureParamMap,
    pub listening: bool,
}
*/

pub fn native_alter_toy(vc_state: tauri::State<'_, VCStateMutex>, app_handle: tauri::AppHandle, toy_id: u32, mutate: FeToyAlter) -> Result<(), frontend::VCFeError> {

    let altered = {
        let mut vc_lock = vc_state.0.lock();
        if let Some(toy) = vc_lock.toys.get_mut(&toy_id) {

            match mutate {
                FeToyAlter::Feature(f) => {
                    if !toy.param_feature_map.from_fe(f) {
                        logerr!("Failed to convert FeVCToyFeature to VCToyFeature");
                        return Err(frontend::VCFeError::AlterToyFailure(frontend::ToyAlterError::NoFeatureIndex));
                    } else {
                        // If altering feature map suceeds write the data to the config
                        toy.config.as_mut().unwrap().features = toy.param_feature_map.clone();
                    }
                },
                FeToyAlter::OSCData(osc_data) => {
                    toy.osc_data = osc_data;
                    // Write the data to config
                    toy.config.as_mut().unwrap().osc_data = osc_data;
                }
            }
            // Return altered toy
            toy.clone()
        } else {
            return Err(frontend::VCFeError::AlterToyFailure(frontend::ToyAlterError::NoToyIndex));
        }
    };

    //save_toy_config(&altered.toy_name, altered.param_feature_map.clone());
    let alter_clone = altered.clone();
    info!("Altered toy: {:?}", altered);
    altered.save_toy_config();

    let send_res = {
        let vc_lock = vc_state.0.lock();
        vc_lock.tme_send_tx.send(ToyManagementEvent::Tu(ToyUpdate::AlterToy(altered)))
    };

    let _ = app_handle.emit_all("fe_toy_event",
        FeToyEvent::Update ({
            FeVCToy {
                toy_id: alter_clone.toy_id,
                toy_name: alter_clone.toy_name,
                battery_level: alter_clone.battery_level,
                toy_connected: alter_clone.toy_connected,
                features: alter_clone.param_feature_map.to_fe(),
                listening: alter_clone.listening,
                osc_data: alter_clone.osc_data,
                sub_id: alter_clone.sub_id,
            }
        }),
    );

    match send_res {
        Ok(()) => Ok(()),
        Err(_e) => Err(frontend::VCFeError::AlterToyFailure(frontend::ToyAlterError::TMESendFailure)),
    }
}

/*
pub fn native_set_lc_override(vc_state: tauri::State<'_, VCStateMutex>, host: FeLCOverride) -> Result<(), frontend::VCFeError> {

    if let FeLCOverride::Host(h) = host {
        let valid = match Ipv4Addr::from_str(&h) {
            Ok(sa) => sa,
            Err(_e) => return Err(frontend::VCFeError::InvalidIpv4Host),
        };
    
        // Force port because buttplug forces non http atm
        std::env::set_var("VCLC_HOST_PORT", format!("{}:20010", valid.to_string()).as_str());
        match std::env::var("VCLC_HOST_PORT") {
            Ok(_) => {
                {
                    let mut vc_lock = vc_state.0.lock();
                    vc_lock.config.lc_override = Some(valid);
                }
                Ok(())
            },
            Err(_) => return Err(frontend::VCFeError::SetLCOverrideFailure),
        }
    } else {

        std::env::remove_var("VCLC_HOST_PORT");
        match std::env::var("VCLC_HOST_PORT") {
            Ok(_) => return Err(frontend::VCFeError::UnsetLCOverrideFailure),
            Err(e) => {
                match e {
                    std::env::VarError::NotPresent => {
                        {
                            let mut vc_lock = vc_state.0.lock();
                            vc_lock.config.lc_override = None;
                        }
                        Ok(())
                    },
                    _ => {
                        logerr!("Got Non unicode var during unset routine");
                        return Err(frontend::VCFeError::UnsetLCOverrideFailure);
                    },
                }
            }
        }
    }
}

pub fn native_get_lc_override(vc_state: tauri::State<'_, VCStateMutex>) -> Result<String, frontend::VCFeError> {

    let lc_or = {
        let vc_lock = vc_state.0.lock();
        match vc_lock.config.lc_override {
            Some(host) => {
                host.to_string()
            },
            None => "Clear".to_string()
        }
    };
    Ok(lc_or)

}
*/