use buttplug::client::ButtplugClient;
use log::{error as logerr, info, warn};
use parking_lot::Mutex;
use std::net::SocketAddrV4;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc::unbounded_channel, mpsc::UnboundedReceiver, mpsc::UnboundedSender};
use tokio::task::JoinHandle;
use vrcoscquery::OSCQuery;

use crate::error_signal_handler::state_comm::error_message_handler;
use crate::error_signal_handler::{ErrorSource, VibeCheckError};
use crate::osc::logic::{toy_refresh, vc_disabled_osc_command_listen};
use crate::toy_handling::runtime::client_event_handler::client_event_handler;
use crate::toy_handling::runtime::toy_management_handler::toy_management_handler;
use crate::toy_handling::toy_manager::ToyManager;
use crate::util::bluetooth;
use crate::util::net::{find_available_tcp_port, find_available_udp_port};
use crate::vcore::errors::VcoreError;

use super::config::app::VibeCheckConfig;
use super::errors::VCError;
use super::ipc::call_plane::ToyManagementEvent;

pub struct VCStateMutex(pub Arc<Mutex<VibeCheckState>>);

pub enum RunningState {
    Running,
    Stopped,
}

pub struct VibeCheckState {
    pub app_handle: Option<AppHandle>,
    pub identifier: String,
    pub mock_toys: bool,
    pub mock_toy_events_emitted: bool,
    pub mock_toys_data: Vec<crate::frontend::frontend_types::FeVCToy>,

    pub config: VibeCheckConfig,
    pub osc_query_handler: Option<OSCQuery>,
    //pub connection_modes: ConnectionModes,
    pub bp_client: Option<ButtplugClient>,

    pub running: RunningState,
    pub core_toy_manager: Option<ToyManager>,

    pub error_comm_tx: Option<UnboundedSender<VCError>>,

    // Message handler (Handles Messages from other threads. For now just error aggregator)
    pub global_message_handler_thread: Option<JoinHandle<()>>,
    // Client Event Handler
    pub client_eh_thread: Option<JoinHandle<()>>,
    //Toy update handler
    pub toy_update_h_thread: Option<JoinHandle<()>>,
    // Toy Management Handler
    pub toy_management_h_thread: Option<JoinHandle<()>>,
    // Disabled listener thread handle
    pub disabled_osc_listener_h_thread: Option<JoinHandle<()>>,

    // These stay in VibeCheckState
    pub tme_recv_rx: UnboundedReceiver<ToyManagementEvent>,
    pub tme_send_tx: UnboundedSender<ToyManagementEvent>,
    // These go in TMH. Wrapped in Option so they can be moved into TMH.
    pub tme_recv_tx: Option<UnboundedSender<ToyManagementEvent>>,
    pub tme_send_rx: Option<UnboundedReceiver<ToyManagementEvent>>,
    //================================================
    pub vibecheck_state_pointer: Option<Arc<Mutex<VibeCheckState>>>,
    //================================================
    //pub toy_input_h_thread: Option<JoinHandle<()>>,
    //================================================
    // Async Runtime for toy event handlers
    pub async_rt: Runtime,
    //================================================
}

impl VibeCheckState {
    pub fn new(config: VibeCheckConfig, mock_toys: bool) -> Self {
        // Toys hashmap
        //let core_toy_manager = ToyHandler::new();

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

        let state = Self {
            app_handle: None,
            identifier: String::new(),
            mock_toys,
            mock_toy_events_emitted: false,
            config,
            osc_query_handler: None,
            //connection_modes,
            bp_client: None,
            running: RunningState::Stopped,
            core_toy_manager: None,
            //======================================
            // Error channels
            error_comm_tx: None,

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
            // Message handler (Handles Messages from other threads. For now just error aggregator)
            global_message_handler_thread: None,
            vibecheck_state_pointer: None,

            //======================================
            // Async runtime
            async_rt,
            mock_toys_data: Vec::new(),
        };

        state
    }

    pub fn global_msg_handler_start(&mut self) -> Result<(), VibeCheckError> {
        if self.app_handle.is_none() {
            logerr!("global_msg_handler_start() called but no app_handle was set");
            return Err(VibeCheckError::new(
                ErrorSource::Vcore(VcoreError::NoAppHandle),
                None,
            ));
        }

        // Create error handling/passig channels
        let (error_comm_tx, error_comm_rx): (UnboundedSender<VCError>, UnboundedReceiver<VCError>) =
            unbounded_channel();
        self.error_comm_tx = Some(error_comm_tx);
        self.global_message_handler_thread = Some(self.async_rt.spawn(error_message_handler(
            self.app_handle.as_ref().unwrap().clone(),
            error_comm_rx,
        )));
        Ok(())
    }

    pub fn start_tmh(&mut self) -> Result<(), VibeCheckError> {
        if self.app_handle.is_none() {
            logerr!("start_tmh() called but no app_handle was set");
            return Err(VibeCheckError::new(
                ErrorSource::Vcore(VcoreError::NoAppHandle),
                None,
            ));
        }

        if self.core_toy_manager.is_none() {
            logerr!("start_tmh() called but no core_toy_manager was set");
            return Err(VibeCheckError::new(
                ErrorSource::Vcore(VcoreError::NoToyManager),
                None,
            ));
        }

        self.toy_management_h_thread = Some(self.async_rt.spawn(toy_management_handler(
            self.tme_recv_tx.take().unwrap(),
            self.tme_send_rx.take().unwrap(),
            self.core_toy_manager.as_ref().unwrap().clone(),
            self.config.networking.clone(),
            self.app_handle.as_ref().unwrap().clone(),
        )));
        info!("TMH started");
        Ok(())
    }

    pub fn start_disabled_listener(&mut self) -> Result<(), VibeCheckError> {
        if self.disabled_osc_listener_h_thread.is_some() {
            return Err(VibeCheckError::new(
                ErrorSource::Vcore(VcoreError::DisabledOscListenerThreadRunning),
                None,
            ));
        }

        self.disabled_osc_listener_h_thread =
            Some(self.async_rt.spawn(vc_disabled_osc_command_listen(
                self.app_handle.as_ref().unwrap().clone(),
                self.config.networking.clone(),
            )));
        Ok(())
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

    pub fn init_toy_manager(&mut self) -> Result<(), VibeCheckError> {
        let toy_manager = ToyManager::new(
            self.app_handle
                .as_ref()
                .expect("Failed to get app handle")
                .clone(),
        );

        match toy_manager {
            Ok(tm) => self.core_toy_manager = Some(tm),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn init_ceh(&mut self) -> Result<(), VibeCheckError> {
        // Is there a supplied state pointer?
        if self.vibecheck_state_pointer.is_none() {
            return Err(VibeCheckError::new(
                ErrorSource::Vcore(VcoreError::NoStatePointer),
                None,
            ));
        }

        // Is CEH already running?

        if self.client_eh_thread.is_some() {
            return Err(VibeCheckError::new(
                ErrorSource::Vcore(VcoreError::CehAlreadyInitialized),
                None,
            ));
        }

        // Create connection mode defaults
        //let mut connection_modes = ConnectionModes { btle_enabled: true, lc_enabled: true };

        // Get ButtPlugClient with modified connection modes

        let bp_client_future = bluetooth::vc_toy_client_server_init("VibeCheck", false);

        self.bp_client = match self.async_rt.block_on(bp_client_future) {
            Ok(bpc) => Some(bpc),
            Err(e) => {
                logerr!("Failed to initialize bpio..");
                return Err(VibeCheckError::new(
                    ErrorSource::Util(e),
                    Some("Failed to initialize bpio.."),
                ));
            }
        };
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
            self.error_comm_tx.as_ref().unwrap().clone(),
        )));
        Ok(())
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
