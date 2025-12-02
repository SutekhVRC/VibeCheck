use log::error as logerr;

use tauri::{AppHandle, Manager};

use crate::{frontend::{error::FrontendError, frontend_types::{FeCoreEvent, FeToyEvent}}};

pub fn emit_toy_event(app_handle: &AppHandle, event: FeToyEvent) {
    match app_handle.emit_all("fe_toy_event", &event) {
        Ok(()) => (),
        Err(e) => logerr!("Emit Toy Event [{}] failed: {}", event, e),
    }
}

pub fn emit_core_event(app_handle: &AppHandle, event: FeCoreEvent) {
    match app_handle.emit_all("fe_core_event", &event) {
        Ok(()) => (),
        Err(e) => logerr!("Emit Core Event [{}] failed: {}", event, e),
    }
}

pub fn emit_error(app_handle: &AppHandle, event: FrontendError) {
    match app_handle.emit_all("fe_error", &event) {
        Ok(()) => (),
        Err(e) => logerr!("Emit Error Event [{}] failed: {}", event, e),
    }
}