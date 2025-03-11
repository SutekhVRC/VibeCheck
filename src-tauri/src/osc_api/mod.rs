use std::rc::Rc;

use log::{debug, info, trace};
use rosc::OscMessage;
use tauri::{AppHandle, Manager};

use crate::{
    config::toy::VCToyAnatomy,
    frontend::frontend_types::FeCoreEvent,
    vcore::ipc::{call_plane::native_alter_toy, emit_plane::emit_core_event},
};

#[derive(Debug)]
pub struct OscApiError {}

pub mod osc_api;

struct APIProcessor;

impl APIProcessor {
    pub fn parse(mut endpoint: OscMessage, app_handle: &AppHandle) {
        let mut api_tokenize = endpoint
            .addr
            .split('/')
            .map(|t| t.to_string())
            .collect::<Vec<String>>();
        api_tokenize.retain(|token| !token.is_empty());
        debug!("[*] API tokenization: {:?}", api_tokenize);
        if api_tokenize.len() == 5
            && api_tokenize[2] == "vibecheck"
            && api_tokenize[3] == "api"
            && api_tokenize[4] == "state"
        {
            // /avatar/parameters/vibecheck/api/state
            let Some(false) = endpoint.args.pop().unwrap().bool() else {
                return;
            };
            info!("State false: Sending Disable event");

            emit_core_event(
                &Rc::new(app_handle),
                FeCoreEvent::State(crate::frontend::frontend_types::FeStateEvent::Disable),
            );
        } else if api_tokenize.len() == 7
            && api_tokenize[4] == "anatomy"
            && api_tokenize[6] == "enabled"
        {
            // /avatar/parameters/vibecheck/api/anatomy/Anal/enabled
            trace!("[*] Checking anatomy token: {}", api_tokenize[5]);
            let anatomy = VCToyAnatomy::get_anatomy(&api_tokenize[5]);
            let mut altered_toys = Vec::new();

            if let Some(state_bool) = endpoint.args.pop().unwrap().bool() {
                let vc_pointer = app_handle
                    .state::<crate::vcore::state::VCStateMutex>()
                    .0
                    .clone();
                let mut vc_lock = vc_pointer.lock();

                vc_lock
                    .core_toy_manager
                    .as_mut()
                    .unwrap()
                    .online_toys
                    .iter_mut()
                    .for_each(|toy| {
                        if toy.1.mutate_state_by_anatomy(&anatomy, state_bool) {
                            trace!(
                                "[*] Mutating feature state from anatomy for toy: {}",
                                toy.1.toy_name
                            );
                            altered_toys.push(toy.1.clone());
                        }
                    });
            }

            altered_toys.iter().for_each(|toy| {
                let _ = native_alter_toy(
                    app_handle.state::<crate::vcore::state::VCStateMutex>(),
                    app_handle.clone(),
                    toy.clone(),
                );
            });
        }
    }
}
