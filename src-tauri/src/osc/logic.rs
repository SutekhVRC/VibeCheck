use futures_timer::Delay;
use log::{debug, error as logerr, info, trace, warn};
use parking_lot::Mutex;
use rosc::encoder;
use rosc::OscType;
use rosc::{self, OscMessage, OscPacket};

use tauri::AppHandle;
use tokio::net::UdpSocket as tUdpSocket;

use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use crate::frontend::frontend_types::FeCoreEvent;
use crate::frontend::frontend_types::FeToyEvent;
use crate::frontend::frontend_types::FeVCToy;
use crate::frontend::ToFrontend;
use crate::toy_handling::ToyPower;
use crate::vcore::ipc::emit_plane::emit_core_event;
use crate::vcore::ipc::emit_plane::emit_toy_event;
use crate::vcore::state::VibeCheckState;

use super::OSCNetworking;

/// How often, and how many times, the off-state listener retries binding
/// while the port is still held (about 5 seconds in all).
const BIND_RETRY_EVERY: Duration = Duration::from_millis(100);
const BIND_RETRIES: u32 = 50;

pub async fn vc_disabled_osc_command_listen(app_handle: AppHandle, vc_config: OSCNetworking) {
    info!("Listening for OSC commands while disabled");
    // Bind straight away: while nothing holds the port, VRChat's sends to it
    // fail, and through Proton enough failures make VRChat stop sending OSC
    // for the session. If the session's OSC input is still letting go of the
    // port, retry quickly so the gap stays short.
    let mut retries = BIND_RETRIES;
    let sock;
    loop {
        // The same address the session listens on (all interfaces in
        // OSCQuery mode), so the port stays held while VibeCheck is off
        match tUdpSocket::bind(vc_config.listen_addr()).await {
            Ok(s) => {
                info!("Listening while disabled on {}", vc_config.listen_addr());
                sock = s;
                break;
            }
            Err(e) => {
                if retries == 0 {
                    logerr!(
                        "Couldn't bind {} for listening while disabled: {}",
                        vc_config.listen_addr(),
                        e
                    );
                    return;
                }
                retries -= 1;
                Delay::new(BIND_RETRY_EVERY).await;
            }
        };
    }

    loop {
        let mut buf = [0u8; rosc::decoder::MTU];

        let (br, _a) = match sock.recv_from(&mut buf).await {
            Ok((br, a)) => (br, a),
            Err(_e) => {
                logerr!("Failed to receive data");
                continue;
            }
        };

        if br == 0 {
            continue;
        } else {
            let pkt = match rosc::decoder::decode_udp(&buf) {
                Ok(pkt) => pkt,
                Err(e) => {
                    // Per packet: debug only, so a bad sender can't flood the log
                    debug!("Failed to parse OSC packet: {:?}", e);
                    continue;
                }
            };

            match pkt.1 {
                OscPacket::Message(mut msg) => {
                    // The same parameter turns VibeCheck off while it's on
                    // (osc_api). A message without a value is ignored.
                    if msg.addr == "/avatar/parameters/vibecheck/api/state" {
                        if let Some(state_bool) = msg.args.pop().and_then(|arg| arg.bool()) {
                            if state_bool {
                                info!("Sending EnableAndScan event");
                                emit_core_event(&app_handle, FeCoreEvent::State(crate::frontend::frontend_types::FeStateEvent::EnableAndScan));
                            }
                        }
                    }
                }
                _ => {
                    trace!("Didn't get OscPacket::Message, skipping..");
                }
            }
        }
    }
}

/* FUTURE MAYBE
 * Toy update loop every 1 sec maybe 5
 * How to do parameter structure
 * /avatar/parameters/toy_name
 *
 * ATM this only sends a battery life OSC address/value.
 */

pub async fn toy_refresh(
    vibecheck_state_pointer: Arc<Mutex<VibeCheckState>>,
    app_handle: AppHandle,
) {
    loop {
        Delay::new(Duration::from_secs(15)).await;

        let (toys, remote, reporter) = {
            let vc_lock = vibecheck_state_pointer.lock();
            let toys = &vc_lock.toy_manager().online_toys;
            if toys.is_empty() {
                continue;
            }
            (toys.clone(), vc_lock.osc_remote(), vc_lock.reporter.clone())
        };

        // Battery levels for VRChat. If this fails, toys still update in the
        // app; the next round tries again.
        let sock = match battery_socket(remote).await {
            Ok(sock) => Some(sock),
            Err(e) => {
                reporter.warning(
                    "toys",
                    "battery_socket",
                    format!("Couldn't send battery levels to VRChat at {remote}: {e}"),
                );
                None
            }
        };
        for (.., mut toy) in toys {
            let toy_power = ToyPower::read(&toy.device_handle).await;

            toy.toy_power = toy_power.clone();

            emit_toy_event(
                &app_handle,
                FeToyEvent::Update({
                    FeVCToy {
                        toy_id: Some(toy.toy_id),
                        toy_name: toy.toy_name.clone(),
                        toy_anatomy: toy.anatomy(),
                        toy_power: toy_power.clone(),
                        toy_connected: toy.toy_connected,
                        features: toy.parsed_toy_features.features.to_frontend(),
                        listening: toy.listening,
                        osc_data: toy.osc_data,
                        bt_update_rate: toy.bt_update_rate,
                        sub_id: toy.sub_id,
                    }
                }),
            );

            let Some(sock) = sock.as_ref() else {
                continue;
            };
            if toy.osc_data {
                trace!("Sending OSC data for toy: {}", toy.toy_name);

                let battery_level_msg = match encoder::encode(&OscPacket::Message(OscMessage {
                    addr: format!(
                        "/avatar/parameters/vibecheck/osc_data/{}/{}/battery",
                        toy.toy_name.replace(" ", "_").to_lowercase(),
                        toy.sub_id
                    ),
                    args: vec![OscType::Float(toy_power.to_float() as f32)],
                })) {
                    Ok(msg) => msg,
                    Err(e) => {
                        warn!(
                            "Couldn't encode the battery level for {}: {}",
                            toy.toy_name, e
                        );
                        continue;
                    }
                };

                let batt_send_err = sock.send(&battery_level_msg).await;
                if batt_send_err.is_err() {
                    warn!("Failed to send battery_level to {}", remote.to_string());
                } else {
                    info!(
                        "Sent battery_level: {} to {}",
                        toy_power.to_float() as f32,
                        toy.toy_name
                    );
                }
            } else {
                trace!("OSC data disabled for toy {}", toy.toy_name);
            }
        }
    }
}

/// A socket for sending battery levels to `remote`.
async fn battery_socket(remote: std::net::SocketAddrV4) -> std::io::Result<tUdpSocket> {
    let sock = tUdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await?;
    sock.connect(remote).await?;
    if let Ok(local) = sock.local_addr() {
        info!("Bound toy_refresh sender sock to {}", local);
    }
    Ok(sock)
}
