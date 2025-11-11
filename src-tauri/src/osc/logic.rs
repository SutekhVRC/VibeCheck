use futures_timer::Delay;
use log::{error as logerr, info, trace, warn};
use parking_lot::Mutex;
use rosc::encoder;
use rosc::OscType;
use rosc::{self, OscMessage, OscPacket};

use tauri::AppHandle;
use tokio::net::UdpSocket as tUdpSocket;

use tokio::sync::broadcast::Sender as BSender;
use tokio::sync::mpsc::UnboundedSender;

use std::net::Ipv4Addr;
use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

use crate::frontend::frontend_types::FeCoreEvent;
use crate::frontend::frontend_types::FeToyEvent;
use crate::frontend::frontend_types::FeVCToy;
use crate::frontend::ToFrontend;
use crate::osc_api::api::vibecheck_osc_api;
use crate::toy_handling::ToyPower;
use crate::toy_handling::ToySig;
use crate::vcore::ipc::call_plane::TmSig;
use crate::vcore::ipc::call_plane::ToyManagementEvent;
use crate::vcore::ipc::emit_plane::emit_core_event;
use crate::vcore::ipc::emit_plane::emit_toy_event;
use crate::vcore::state::VibeCheckState;

use super::OSCNetworking;

/*
    This subroutine
    Binds the OSC listen socket
    receives OSC messages
    broadcasts the OSC messages to each toy
*/
pub fn toy_input_routine(
    toy_bcst_tx: BSender<ToySig>,
    tme_send: UnboundedSender<ToyManagementEvent>,
    app_handle: AppHandle,
    vc_config: OSCNetworking,
) {
    let bind_sock =
        match UdpSocket::bind(format!("{}:{}", vc_config.bind.ip(), vc_config.bind.port())) {
            Ok(s) => {
                let _ = tme_send.send(ToyManagementEvent::Sig(TmSig::Listening));
                s
            }
            Err(_e) => {
                let _ = tme_send.send(ToyManagementEvent::Sig(TmSig::BindError));
                return;
            }
        };
    info!(
        "Listen sock is bound {} : {}",
        vc_config.bind.ip(),
        vc_config.bind.port()
    );
    bind_sock.set_nonblocking(false).unwrap();
    let _ = bind_sock.set_read_timeout(Some(Duration::from_secs(1)));

    loop {
        // try recv OSC packet
        // parse OSC packet
        // Send address and arg to broadcast channel
        // Die when channel disconnects

        if !vibecheck_osc_api(&bind_sock, &app_handle, &toy_bcst_tx) {
            return;
        }
    }
}

pub async fn vc_disabled_osc_command_listen(app_handle: AppHandle, vc_config: OSCNetworking) {
    info!("Listening for OSC commands while disabled");
    let mut retries = 3;
    let sock;
    loop {
        Delay::new(Duration::from_secs(1)).await;
        match tUdpSocket::bind(format!("{}:{}", vc_config.bind.ip(), vc_config.bind.port())).await {
            Ok(s) => {
                info!("Listening while disabled");
                sock = s;
                break;
            }
            Err(_e) => {
                logerr!(
                    "Failed to bind UDP socket for disabled cmd listening.. Retries remaining: {}",
                    retries
                );
                if retries == 0 {
                    return;
                }
                retries -= 1;
                continue;
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
                Err(_e) => {
                    logerr!("Failed to parse OSC packet");
                    continue;
                }
            };

            match pkt.1 {
                OscPacket::Message(mut msg) => {
                    if msg.addr == "/avatar/parameters/vibecheck/state" {
                        if let Some(state_bool) = msg.args.pop().unwrap().bool() {
                            if state_bool {
                                info!("Sending EnableAndScan event");
                                emit_core_event(&app_handle, FeCoreEvent::State(crate::frontend::frontend_types::FeStateEvent::EnableAndScan));
                            }
                        }
                    }
                }
                _ => {
                    info!("Didn't get OscPacket::Message, skipping..");
                }
            }
        }
    }
}

#[inline]
pub fn recv_osc_cmd(sock: &UdpSocket) -> Option<OscMessage> {
    let mut buf = [0u8; rosc::decoder::MTU];

    let (br, _a) = match sock.recv_from(&mut buf) {
        Ok((br, a)) => (br, a),
        Err(_e) => {
            return None;
        }
    };

    if br == 0 {
        return None;
    }
    let pkt = rosc::decoder::decode_udp(&buf).ok()?;
    match pkt.1 {
        OscPacket::Message(msg) => Some(msg),
        _ => None,
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

        let (toys, remote) = {
            let vc_lock = vibecheck_state_pointer.lock();
            if !vc_lock
                .core_toy_manager
                .as_ref()
                .unwrap()
                .online_toys
                .is_empty()
            {
                (
                    vc_lock
                        .core_toy_manager
                        .as_ref()
                        .unwrap()
                        .online_toys
                        .clone(),
                    vc_lock.config.networking.remote,
                )
            } else {
                continue;
            }
        };

        let sock = tUdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();
        info!(
            "Bound toy_refresh sender sock to {}",
            sock.local_addr().unwrap()
        );
        sock.connect(remote).await.unwrap();
        for (.., mut toy) in toys {
            // Can use this to differ between toys with batteries and toys without!
            let toy_power = if toy.device_handle.has_battery_level() {
                match toy.device_handle.battery_level().await {
                    Ok(battery_lvl) => ToyPower::Battery(battery_lvl),
                    Err(_e) => {
                        warn!("Device battery_level() error: {:?}", _e);
                        ToyPower::Pending
                    }
                }
            } else {
                ToyPower::NoBattery
            };

            toy.toy_power = toy_power.clone();

            emit_toy_event(
                &app_handle,
                FeToyEvent::Update({
                    FeVCToy {
                        toy_id: Some(toy.toy_id),
                        toy_name: toy.toy_name.clone(),
                        toy_anatomy: toy.config.as_ref().unwrap().anatomy.to_fe(),
                        toy_power: toy_power.clone(),
                        toy_connected: toy.toy_connected,
                        features: toy.parsed_toy_features.features.to_frontend(),
                        listening: toy.listening,
                        osc_data: toy.osc_data,
                        sub_id: toy.sub_id,
                    }
                }),
            );

            if toy.osc_data {
                trace!("Sending OSC data for toy: {}", toy.toy_name);

                let battery_level_msg = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: format!(
                        "/avatar/parameters/vibecheck/osc_data/{}/{}/battery",
                        toy.toy_name.replace(" ", "_").to_lowercase(),
                        toy.sub_id
                    ),
                    args: vec![OscType::Float(toy_power.to_float() as f32)],
                }))
                .unwrap();

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
