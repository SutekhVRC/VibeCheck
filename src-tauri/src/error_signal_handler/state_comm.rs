use crate::{
    error_signal_handler::VibeCheckError,
    frontend::error::FrontendError,
    vcore::{errors::VCError, ipc::emit_plane::emit_error, state::VibeCheckState},
};
use log::error as logerr;
use parking_lot::lock_api::Mutex;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};
use tauri::AppHandle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/*
pub struct ErrorSignalAggregator {
    // These should be a trait that each module has to implement to send messages to it
    error_rx: Option<UnboundedReceiver<VCError>>,
    error_tx: UnboundedSender<VCError>,
}

impl ErrorSignalAggregator {
    pub fn new(rx: UnboundedReceiver<VCError>, tx: UnboundedSender<VCError>) -> Self {
        Self {
            error_rx: Some(rx),
            error_tx: tx,
        }
    }

    pub fn get_new_tx(&self) -> UnboundedSender<VCError> {
        self.error_tx.clone()
    }

    pub fn send_err(&self, err: VCError) {
        if self.error_tx.send(err).is_err() {
            logerr!("ErrorSignalAggregator::send_err() failed!");
        }
    }
}*/

pub async fn error_message_handler(
    app_handle: AppHandle,
    mut error_rx: UnboundedReceiver<VCError>,
) {
    loop {
        if let Some(err_msg) = error_rx.recv().await {
            match err_msg {
                VCError::HandlingErr(e) => {
                    let s = format!("{}: {}", e.msg, e.id);
                    logerr!("{}", s);
                    emit_error(&app_handle, FrontendError::Error(s));
                }
            }
        }
    }
}
