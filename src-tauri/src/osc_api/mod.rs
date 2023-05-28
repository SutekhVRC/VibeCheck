use log::{info, trace, debug};
use rosc::OscMessage;
use tauri::{AppHandle, Manager};

use crate::{frontend_types::FeCoreEvent, config::toy::VCToyAnatomy, vcore};

pub mod osc_api;

struct APIProcessor;

impl APIProcessor {
    pub fn parse(mut endpoint: OscMessage, app_handle: &AppHandle) {
        
        let mut api_tokenize = endpoint.addr.split("/").map(|t| t.to_string()).collect::<Vec<String>>();
        api_tokenize.retain(|token| token.len() > 0);
        debug!("[*] API tokenization: {:?}", api_tokenize);
        if api_tokenize.len() == 5 {
            if api_tokenize[2] == "vibecheck" && api_tokenize[3] == "api" {
                if api_tokenize[4] == "state" {// /avatar/parameters/vibecheck/api/state
                    if let Some(state_bool) = endpoint.args.pop().unwrap().bool() {
                        if !state_bool {
                            info!("State false: Sending Disable event");
                            let _ = app_handle.emit_all("fe_core_event", FeCoreEvent::State(crate::frontend_types::FeStateEvent::Disable));
                        }
                    }
                }
            }
        } else if api_tokenize.len() == 7 {
            if api_tokenize[4] == "anatomy" && api_tokenize[6] == "enabled" {// /avatar/parameters/vibecheck/api/anatomy/Anal/enabled

                trace!("[*] Checking anatomy token: {}", api_tokenize[5]);

                let anatomy = VCToyAnatomy::get_anatomy(&api_tokenize[5]);
                let mut altered_toys = Vec::new();

                if let Some(state_bool) = endpoint.args.pop().unwrap().bool() {

                    let vc_pointer = app_handle.state::<crate::vcore::VCStateMutex>().0.clone();
                    let mut vc_lock = vc_pointer.lock();

                    vc_lock.core_toy_manager.as_mut().unwrap().online_toys.iter_mut().for_each(|toy| {
                        if toy.1.mutate_state_by_anatomy(&anatomy, state_bool) {
                            trace!("[*] Mutating feature state from anatomy for toy: {}", toy.1.toy_name);
                            altered_toys.push(toy.1.clone());
                        }
                    });
                }

                altered_toys.iter().for_each(|toy| {
                    let _ = vcore::native_alter_toy(app_handle.state::<crate::vcore::VCStateMutex>(), app_handle.clone(), toy.clone());
                });
            }
        }
    }
}