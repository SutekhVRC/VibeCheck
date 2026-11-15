use buttplug_client::ButtplugClient;
use log::{error as logerr, info, warn};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc::unbounded_channel, mpsc::UnboundedReceiver, mpsc::UnboundedSender};
use tokio::task::JoinHandle;

use crate::error_signal_handler::reporter::{notice_handler, ErrorReporter, Notice};
use crate::error_signal_handler::{ErrorSource, VibeCheckError};
use crate::osc::logic::{toy_refresh, vc_disabled_osc_command_listen};
use crate::osc::oscquery::OscQuery;
use crate::osc::OscMode;
use crate::toy_handling::runtime::client_event_handler::client_event_handler;
use crate::toy_handling::runtime::toy_management_handler::toy_management_handler;
use crate::toy_handling::toy_manager::ToyManager;
use crate::util::bluetooth;
use crate::vcore::errors::VcoreError;

use super::config::app::VibeCheckConfig;
use super::ipc::call_plane::ToyManagementEvent;
use vibecheck_core::{Diagnostics, SafetyLayer};

pub struct VCStateMutex(pub Arc<Mutex<VibeCheckState>>);

pub enum RunningState {
    Running,
    Stopped,
}

pub struct VibeCheckState {
    pub app_handle: Option<AppHandle>,
    pub identifier: String,

    pub config: VibeCheckConfig,
    /// Shared by every listening session and the Test button, so the master
    /// intensity applies everywhere and can change while toys are running.
    pub safety: SafetyLayer,
    /// Diagnostics switch and log, shared with every listening session.
    pub diagnostics: Diagnostics,
    /// OSCQuery mode's advertising and VRChat lookup (see osc/oscquery.rs).
    pub oscquery: OscQuery,
    //pub connection_modes: ConnectionModes,
    pub bp_client: Option<ButtplugClient>,

    pub running: RunningState,
    pub core_toy_manager: Option<ToyManager>,

    /// Reports problems from background work to the user (see
    /// error_signal_handler/reporter.rs). Usable from startup; notices wait
    /// until the handler starts.
    pub reporter: ErrorReporter,
    /// Taken by `notice_handler_start`.
    notices: Option<UnboundedReceiver<Notice>>,
    pub notice_handler_thread: Option<JoinHandle<()>>,
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
    pub fn new(config: VibeCheckConfig) -> Self {
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

        let safety = SafetyLayer::default();
        safety.set_intensity(config.master_intensity);

        let (reporter, notices) = ErrorReporter::channel();

        let state = Self {
            app_handle: None,
            identifier: String::new(),
            config,
            safety,
            diagnostics: Diagnostics::default(),
            oscquery: OscQuery::default(),
            //connection_modes,
            bp_client: None,
            running: RunningState::Stopped,
            core_toy_manager: None,
            //======================================
            // Problems reported by background work
            reporter,
            notices: Some(notices),
            notice_handler_thread: None,

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

            vibecheck_state_pointer: None,

            //======================================
            // Async runtime
            async_rt,
        };

        state
    }

    /// Starts showing reported problems in the app. Needs the app handle.
    pub fn notice_handler_start(&mut self) -> Result<(), VibeCheckError> {
        let Some(app_handle) = self.app_handle.clone() else {
            logerr!("notice_handler_start() called but no app_handle was set");
            return Err(VibeCheckError::new(
                ErrorSource::Vcore(VcoreError::NoAppHandle),
                None,
            ));
        };
        let Some(notices) = self.notices.take() else {
            warn!("notice_handler_start() called twice");
            return Ok(());
        };
        self.notice_handler_thread = Some(self.async_rt.spawn(notice_handler(
            app_handle,
            notices,
            self.diagnostics.clone(),
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
            self.safety.clone(),
            self.diagnostics.clone(),
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

    pub fn set_app_handle(&mut self, app_handle: &AppHandle) {
        self.app_handle = Some(app_handle.clone());
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

        let bp_client_future = bluetooth::vc_toy_client_server_init("VibeCheck");

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
            self.reporter.clone(),
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

    /// Starts OSCQuery mode's advertising if the config is in OSCQuery mode,
    /// or stops it otherwise. Runs for the life of the app, whether
    /// VibeCheck is on or off.
    pub fn apply_oscquery(&self) {
        let networking = &self.config.networking;
        match (networking.mode(), networking.oscquery_port) {
            (OscMode::OscQuery, Some(port)) => {
                self.oscquery
                    .start(self.async_rt.handle(), port, self.diagnostics.clone())
            }
            _ => self.oscquery.stop(),
        }
    }

    /// Where VibeCheck sends OSC to VRChat right now: the manual remote
    /// address, or in OSCQuery mode the port VRChat advertises.
    pub fn osc_remote(&self) -> std::net::SocketAddrV4 {
        self.config.networking.remote_addr(self.oscquery.vrchat())
    }

    /// Restarts the off-state listener (if it's running) so it listens on
    /// the current address, e.g. after the connection settings changed. The
    /// new listener retries binding while the old one lets go of the port.
    pub fn restart_disabled_listener(&mut self) {
        if let Some(listener) = self.disabled_osc_listener_h_thread.take() {
            listener.abort();
            if let Err(e) = self.start_disabled_listener() {
                logerr!("Couldn't restart the off-state OSC listener: {:?}", e);
            }
        }
    }
}
