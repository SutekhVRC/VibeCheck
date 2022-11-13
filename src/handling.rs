use buttplug::client::ButtplugClientDevice;
use buttplug::client::ButtplugClientEvent;
use buttplug::core::message::ActuatorType;
use buttplug::client::ScalarCommand::ScalarMap;
use buttplug::client::RotateCommand::RotateMap;
use futures::StreamExt;
use futures_timer::Delay;
use rosc::{self, OscMessage, OscPacket};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::{
    self,
    broadcast::{Receiver as BReceiver, Sender as BSender},
};
use tokio::task::JoinHandle;

use crate::config::OSCNetworking;
use crate::toyops::LevelTweaks;
use crate::toyops::VCFeatureType;
use crate::toyops::{VCToy, FeatureParamMap};
use crate::ui::ToyManagementEvent;
use crate::{ui::TmSig, ui::ToyUpdate, ui::VCError};

pub struct HandlerErr {
    pub id: i32,
    pub msg: String,
}

pub enum EventSig {
    ToyAdd(VCToy),
    ToyRemove(u32),
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum ToySig {
    //ToyCommand(ToyFeature),
    UpdateToy(ToyUpdate),
    OSCMsg(rosc::OscMessage),
}

/*
    This handler will handle the adding and removal of toys
    Needs Signals in and out to communicate with main thread
    - communicate errors and handler state (Errors to tell main thread its shutting down && State to receive shutdown from main thread) RECV/SEND
    - communicate toy events (add/remove) ONLY SEND?
*/
pub async fn client_event_handler(error_tx: Sender<VCError>, event_tx: Sender<EventSig>) {
    // Listen for toys and add them if it connects send add update
    // If a toy disconnects send remove update

    println!("[*] Initializing CEH..");

    let client = super::util::vc_toy_client_server_init("VibeCheck", false).await;
    let mut event_stream = client.event_stream();
    println!("[*] Connected to process");

    // Start scanning for toys
    if let Err(e) = client.start_scanning().await {
        let _ = error_tx.send(VCError::HandlingErr(HandlerErr {
            id: -2,
            msg: format!("Failed to scan for bluetooth devices. {}", e),
        }));
        println!("Failed to scan!!!!");
        return;
    }
    println!("[*] Handling Events..");
    loop {

        if let Some(event) = event_stream.next().await {
            match event {
                ButtplugClientEvent::DeviceAdded(dev) => {
                    Delay::new(Duration::from_secs(3)).await;
                    let battery_level = match dev.battery_level().await {
                        Ok(battery_lvl) => battery_lvl,
                        Err(_e) => 0.0,
                    };
                
                    let _ = event_tx.send(EventSig::ToyAdd(VCToy {
                        toy_id: dev.index(),
                        toy_name: dev.name().clone(),
                        battery_level,
                        toy_connected: dev.connected(),
                        toy_features: dev.message_attributes().clone(),
                        osc_params_list: vec![],
                        param_feature_map: FeatureParamMap::new(),
                        listening: false,
                        device_handle: dev.clone(),
                    }));

                    println!("[+] Device connected!!!!");
                }
                ButtplugClientEvent::DeviceRemoved(dev) => {
                    let _ = event_tx.send(EventSig::ToyRemove(dev.index()));
                    println!("[*] Sent dev discon to UI.");
                }
                ButtplugClientEvent::ScanningFinished => println!("[!] Scanning finished!"),
                ButtplugClientEvent::ServerDisconnect => {
                    let _ = event_tx.send(EventSig::Shutdown);
                    println!("[!] Server disconnected!");
                    let _ = client.stop_scanning().await;
                    let _ = client.disconnect().await;
                    break;
                }
                ButtplugClientEvent::PingTimeout => {
                    let _ = event_tx.send(EventSig::Shutdown);
                    println!("[!] Server timeout!");
                    let _ = client.stop_scanning().await;
                    let _ = client.disconnect().await;
                    break;
                }
                ButtplugClientEvent::Error(e) => {
                    println!("Client Event Error: {:?}", e);
                },
                ButtplugClientEvent::ServerConnect => {
                    println!("Server Connect");
                }
            }
        } else {
            println!("GOT NONE IN EVENT HANDLER");
        }
    }
    println!("[!] Event handler returning!");
}

// Parse scalar levels and logic for level tweaks
pub async fn scalar_parse_levels_send_toy_cmd(dev: &Arc<ButtplugClientDevice>, scalar_level: f64, feature_index: u32, actuator_type: ActuatorType, feature_levels: LevelTweaks) {
    // Floor or Ceiling a float if actuator type is Constrict
    /*if actuator_type == ActuatorType::Constrict {
        if scalar_level < 0.50 {
            scalar_level = scalar_level.floor();
        } else {
            scalar_level = scalar_level.ceil();
        }
    } dont need*/
    if scalar_level != 0.0 && scalar_level >= feature_levels.minimum_level && scalar_level <= feature_levels.maximum_level {
        //println!("{} {} {}", feature_index, actuator_type, scalar_level);
        let _e = dev.scalar(&ScalarMap(HashMap::from([(feature_index, (scalar_level, actuator_type))]))).await;
    } else if scalar_level == 0.0 {// if level is 0 put at idle
        let _e = dev.scalar(&buttplug::client::ScalarCommand::ScalarMap(HashMap::from([(feature_index, (feature_levels.idle_level, actuator_type))]))).await;
    }
}

/*
    This handler will send and receive updates to toys
    - communicate ToyUpdate to and from main thread SEND/RECV (Toys will be indexed on the main thread) (Connects and disconnect toy updates are handled by client event handler)
        + Keep a thread count of connected toys. Add/remove as recvs ToyUpdates from main thread
        + Send toy updates like (battery updates)
*/
pub async fn toy_management_handler(
    _tme_send: Sender<ToyManagementEvent>,
    tme_recv: Receiver<ToyManagementEvent>,
    mut toys: HashMap<u32, VCToy>,
    mut vc_config: OSCNetworking,
) {
    let f = |dev: Arc<ButtplugClientDevice>,
             mut toy_bcst_rx: BReceiver<ToySig>,
             mut feature_map: FeatureParamMap| {
        // Read toy config here?
        async move {
            
            while dev.connected() {
                match toy_bcst_rx.recv().await {
                    Ok(ts) => {
                        match ts {
                            ToySig::OSCMsg(mut msg) => {
                                // Parse OSC msgs to toys commands

                                // Does parameter name assign to a feature on this toy?

                                /*
                                    - Enumerate features for an OSC parameter
                                    - Clamp float to hundredths place and cast to 64 bit
                                    - Iterate through each feature
                                */

                                if let Some(features) =
                                    feature_map.get_features_from_param(&msg.addr)
                                {
                                    if let Some(lvl) = msg.args.pop().unwrap().float() {

                                        // Clamp float accuracy to hundredths and cast as 64 bit float
                                        let mut float_level = ((lvl * 100.0).round() / 100.0) as f64;

                                        // Iterate through features enumerated from OSC param
                                        for (feature_type, feature_index, feature_levels, smooth_enabled, smooth_entries) in features {
                                            
                                            // Smoothing enabled
                                            if smooth_enabled {

                                                // Reached smooth rate maximum and not a 0 value
                                                if smooth_entries.len() == feature_levels.smooth_rate as usize && float_level != 0.0 {

                                                    // Get Mean of all numbers in smoothing rate and then round to hundreths and cast as f64
                                                    float_level = ((smooth_entries.iter().sum::<f64>() as f64 / smooth_entries.len() as f64 * 100.0).round() / 100.0) as f64;
                                                    smooth_entries.clear();
    
                                                // Value is not 0 and have not reached smoothing maximum
                                                } else {
    
                                                    if float_level == 0.0 {
                                                        // let 0 through
                                                    } else {                                                
                                                        smooth_entries.push(float_level);
                                                        continue;
                                                    }
                                                }
                                            }

                                            match feature_type {
                                                VCFeatureType::Vibrator => {
                                                    scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Vibrate, feature_levels).await;
                                                },
                                                // We handle Rotator differently because it is not included in the Scalar feature set
                                                VCFeatureType::Rotator => {
                                                    if float_level != 0.0 && float_level >= feature_levels.minimum_level && float_level <= feature_levels.maximum_level {
                                                        let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (float_level, true))]))).await;
                                                    } else if float_level == 0.0 {
                                                        let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (feature_levels.idle_level, true))]))).await;
                                                    }
                                                },
                                                VCFeatureType::Constrict => {
                                                    scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Constrict, feature_levels).await;
                                                },
                                                VCFeatureType::Oscillate => {
                                                    scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Oscillate, feature_levels).await;
                                                },
                                                VCFeatureType::Position => {
                                                    scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Position, feature_levels).await;
                                                },
                                                VCFeatureType::Inflate => {
                                                    scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Inflate, feature_levels).await;
                                                },
                                                // We handle Linear differently because it is not included in the Scalar feature set
                                                VCFeatureType::Linear => {
                                                    if float_level != 0.0 && float_level >= feature_levels.minimum_level && float_level <= feature_levels.maximum_level {
                                                        let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (500, float_level))]))).await;
                                                    } else if float_level == 0.0 {
                                                        let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (500, feature_levels.idle_level))]))).await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            ToySig::UpdateToy(toy) => {
                                match toy {
                                    // Update feature map while toy running!
                                    ToyUpdate::AlterToy(new_toy) => {
                                        if new_toy.toy_id == dev.index() {
                                            feature_map = new_toy.param_feature_map;
                                        }
                                    }
                                    _ => {} // Remove and Add are handled internally from device connected state and management loop (listening)
                                }
                            }
                        }
                    }
                    Err(_e) => {}
                }
            }
            println!(
                "[*] Device {} disconnected! Leaving listening routine!",
                dev.index()
            );
        }
    }; // Toy listening routine

    let mut listening = false;

    // Management loop
    loop {
        // Recv event (not listening)
        match tme_recv.try_recv() {
            Ok(event) => {
                match event {
                    // Handle Toy Update Signals
                    ToyManagementEvent::Tu(tu) => match tu {
                        ToyUpdate::AddToy(toy) => {
                            toys.insert(toy.toy_id, toy);
                        }
                        ToyUpdate::RemoveToy(id) => {
                            toys.remove(&id);
                        }
                        ToyUpdate::AlterToy(toy) => {
                            toys.insert(toy.toy_id, toy);
                        }
                    },
                    // Handle Management Signals
                    ToyManagementEvent::Sig(tm_sig) => {
                        match tm_sig {
                            TmSig::StartListening(osc_net) => {
                                vc_config = osc_net;
                                listening = true;
                            }
                            TmSig::StopListening => {
                                // Already not listening
                            },
                            _ => {},
                        }
                    }
                } // Event handled
            }
            Err(_e) => {}
        }

        if listening {
            let toy_async_rt = Runtime::new().unwrap();
            println!("[*] Started listening!");
            // Recv events (listening)
            // Create toy bcst channel

            // Toy threads
            let mut running_toy_ths: HashMap<u32, JoinHandle<()>> = HashMap::new();

            // Broadcast channels for toy commands
            let (toy_bcst_tx, _toy_bcst_rx): (BSender<ToySig>, BReceiver<ToySig>) =
                sync::broadcast::channel(1024);

            // Create toy threads
            for toy in &toys {
                let f_run = f(
                    toy.1.device_handle.clone(),
                    toy_bcst_tx.subscribe(),
                    toy.1.param_feature_map.clone(),
                );
                running_toy_ths.insert(
                    *toy.0,
                    toy_async_rt.spawn(async move {
                        f_run.await;
                    }),
                );
                println!("[**] Toy: {} started listening..", *toy.0);
            }

            // Create OSC listener thread
            let toy_bcst_tx_osc = toy_bcst_tx.clone();
            println!("[+] Spawning OSC listener..");
            let vc_conf_clone = vc_config.clone();
            let tme_send_clone = _tme_send.clone();
            thread::spawn(move || toy_input_routine(toy_bcst_tx_osc, tme_send_clone, vc_conf_clone));

            loop {
                // Recv event (listening)
                let event = tme_recv.try_recv();
                match event {
                    Ok(event) => {
                        match event {
                            // Handle Toy Update Signals
                            ToyManagementEvent::Tu(tu) => {
                                match tu {
                                    ToyUpdate::AddToy(toy) => {
                                        toys.insert(toy.toy_id, toy.clone());
                                        let f_run = f(
                                            toy.device_handle,
                                            toy_bcst_tx.subscribe(),
                                            toy.param_feature_map.clone(),
                                        );
                                        running_toy_ths.insert(
                                            toy.toy_id,
                                            toy_async_rt.spawn(async move {
                                                f_run.await;
                                            }),
                                        );
                                        println!("[**] Toy: {} started listening..", toy.toy_id);
                                    }
                                    ToyUpdate::RemoveToy(id) => {
                                        // OSC Listener thread will only die on StopListening event
                                        if let Some(toy) = running_toy_ths.remove(&id) {
                                            toy.abort();
                                            let _ = toy.await;
                                            println!("[TOY ID: {}] Stopped listening. (ToyUpdate::RemoveToy)", id);
                                            running_toy_ths.remove(&id);
                                            toys.remove(&id);
                                        }
                                    }
                                    ToyUpdate::AlterToy(toy) => {
                                        let _ = toy_bcst_tx.send(ToySig::UpdateToy(
                                            ToyUpdate::AlterToy(toy.clone()),
                                        ));
                                        toys.insert(toy.toy_id, toy);
                                    }
                                }
                            }
                            // Handle Management Signals
                            ToyManagementEvent::Sig(tm_sig) => {
                                match tm_sig {
                                    TmSig::StartListening(osc_net) => {
                                        vc_config = osc_net;
                                        // Already listening
                                    }
                                    TmSig::StopListening => {
                                        // Stop listening on every device and clean running thread hashmap
                                        for toy in &mut running_toy_ths {
                                            toy.1.abort();
                                            let _ = toy.1.await;
                                            println!(
                                                "[TOY ID: {}] Stopped listening. (TMSIG)",
                                                toy.0
                                            );
                                        }
                                        running_toy_ths.clear();
                                        drop(_toy_bcst_rx); // Causes OSC listener to die
                                        toy_async_rt.shutdown_background();
                                        listening = false;
                                        break;
                                    },
                                    _ => {},
                                }
                            }
                        } // Event handled
                    }
                    Err(_e) => {}
                }
            }
        } //if listening
    } // Management loop
}

/*
    This subroutine
    Binds the OSC listen socket
    receives OSC messages
    broadcasts the OSC messages to each toy
*/
fn toy_input_routine(toy_bcst_tx: BSender<ToySig>, tme_send: Sender<ToyManagementEvent>, vc_config: OSCNetworking) {

    let bind_sock = match UdpSocket::bind(format!("{}:{}", vc_config.bind.0, vc_config.bind.1)) {
        Ok(s) => {
            let _ = tme_send.send(ToyManagementEvent::Sig(TmSig::Listening));
            s
        },
        Err(_e) => {
            let _ = tme_send.send(ToyManagementEvent::Sig(TmSig::BindError));
            return;
        }
    };
    println!("[+] Listen sock is bound");
    bind_sock.set_nonblocking(false).unwrap();
    let _ = bind_sock.set_read_timeout(Some(Duration::from_secs(1)));
    //bind_sock.set_read_timeout(Some(Duration::from_millis(20)));
    loop {
        // try recv OSC packet
        // parse OSC packet
        // Send address and arg to broadcast channel
        // Die when channel disconnects

        match recv_osc_cmd(&bind_sock) {
            Some(msg) => {
                if let Err(_) = toy_bcst_tx.send(ToySig::OSCMsg(msg)) {
                    println!("[*] BCST TX is disconnected. Shutting down toy input routine!");
                    return; // Shutting down handler_routine
                }
            }
            None => {
                if toy_bcst_tx.receiver_count() == 0 {
                    println!(
                        "[*] BCST TX is disconnected (RECV C=0). Shutting down toy input routine!"
                    );
                    return;
                }
            }
        }
    }
}

fn recv_osc_cmd(sock: &UdpSocket) -> Option<OscMessage> {
    let mut buf = [0u8; rosc::decoder::MTU];

    let (br, _a) = match sock.recv_from(&mut buf) {
        Ok((br, a)) => (br, a),
        Err(_e) => {
            return None;
        }
    };

    if br <= 0 {
        return None;
    } else {
        let pkt = match rosc::decoder::decode_udp(&buf) {
            Ok(pkt) => pkt,
            Err(_e) => {
                return None;
            }
        };

        match pkt.1 {
            OscPacket::Message(msg) => {
                return Some(msg);
            }
            _ => {
                return None;
            }
        }
    }
}
