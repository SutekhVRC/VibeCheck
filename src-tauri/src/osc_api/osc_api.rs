use std::net::UdpSocket;
use log::{info,trace};
use tauri::{AppHandle, Manager};
use crate::{handling::{recv_osc_cmd, ToySig}};
use tokio::sync::broadcast::Sender as BSender;

use super::APIProcessor;

pub fn vibecheck_osc_api(bind_sock: &UdpSocket, app_handle: &AppHandle, toy_bcst_tx: &BSender<ToySig>) -> bool {
    match recv_osc_cmd(&bind_sock) {

        Some(msg) => {

            // Stop toys on avatar change
            if msg.addr.starts_with("/avatar/change") {
                info!("Avatar Changed: Halting toy actions");
                {
                    let vc_pointer = app_handle.state::<crate::vcore::VCStateMutex>().0.clone();
                    let vc_lock = vc_pointer.lock();
                    let _ = vc_lock.async_rt.block_on(async {vc_lock.bp_client.as_ref().unwrap().stop_all_devices().await}).unwrap();
                }
                return true;
            } else if msg.addr.starts_with("/avatar/parameters/vibecheck/api/") {
                trace!("[*] VibeCheck API: {:?}", msg);
                APIProcessor::parse(msg, &app_handle);
                return true;
            } else {// Not a vibecheck OSC command, broadcast to toys
                if let Err(_) = toy_bcst_tx.send(ToySig::OSCMsg(msg)) {
                    info!("BCST TX is disconnected. Shutting down toy input routine!");
                    return false; // Shutting down handler_routine
                }
                return true;
            }
        },
        None => {
            if toy_bcst_tx.receiver_count() == 0 {
                info!(
                    "BCST TX is disconnected (RECV C=0). Shutting down toy input routine!"
                );
                return false;
            }
            return true;
        }
    }
}