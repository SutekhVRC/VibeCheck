use buttplug::client::ButtplugClient;
use log::{error as logerr, info, warn};
use parking_lot::Mutex;
use std::net::SocketAddrV4;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc::unbounded_channel, mpsc::UnboundedReceiver, mpsc::UnboundedSender};
use tokio::task::JoinHandle;
use vrcoscquery::OSCQuery;

use crate::osc::logic::{toy_refresh, vc_disabled_osc_command_listen};
use crate::toy_handling::runtime::client_event_handler::client_event_handler;
use crate::toy_handling::runtime::toy_management_handler::toy_management_handler;
use crate::toy_handling::toy_manager::ToyManager;
use crate::util::bluetooth;
use crate::util::net::{find_available_tcp_port, find_available_udp_port};

use super::config::app::VibeCheckConfig;
use super::ipc::call_plane::ToyManagementEvent;
use super::vcerror::VCError;

pub struct VCStateMutex(pub Arc<Mutex<VibeCheckState>>);

pub enum RunningState {
    Running,
    Stopped,
}

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
