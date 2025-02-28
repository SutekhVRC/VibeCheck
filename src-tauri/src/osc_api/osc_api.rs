use crate::{osc::logic::recv_osc_cmd, toy_handling::ToySig};
use log::{info, trace};
use std::net::UdpSocket;
use tauri::{AppHandle, Manager};
use tokio::sync::broadcast::Sender as BSender;

use super::APIProcessor;

pub fn vibecheck_osc_api(
    bind_sock: &UdpSocket,
    app_handle: &AppHandle,
    toy_bcst_tx: &BSender<ToySig>,
) -> bool {
    match recv_osc_cmd(bind_sock) {
        Some(msg) => {
            // Stop toys on avatar change
            if msg.addr.starts_with("/avatar/change") {
                info!("Avatar Changed: Halting toy actions");
                {
                    let vc_pointer = app_handle
                        .state::<crate::vcore::state::VCStateMutex>()
                        .0
                        .clone();
                    let vc_lock = vc_pointer.lock();
                    vc_lock
                        .async_rt
                        .block_on(async {
                            vc_lock.bp_client.as_ref().unwrap().stop_all_devices().await
                        })
                        .unwrap();
                }
                true
            } else if msg.addr.starts_with("/avatar/parameters/vibecheck/api/") {
                trace!("[*] VibeCheck API: {:?}", msg);
                APIProcessor::parse(msg, app_handle);
                true
            } else {
                // Not a vibecheck OSC command, broadcast to toys
                if toy_bcst_tx.send(ToySig::OSCMsg(msg)).is_err() {
                    info!("BCST TX is disconnected. Shutting down toy input routine!");
                    // Shutting down handler_routine
                    false
                } else {
                    true
                }
            }
        }
        None => {
            if toy_bcst_tx.receiver_count() == 0 {
                info!("BCST TX is disconnected (RECV C=0). Shutting down toy input routine!");
                false
            } else {
                true
            }
        }
    }
}
