use buttplug::client::ButtplugClientDevice;
use buttplug::client::ButtplugClientEvent;
use buttplug::core::message::ActuatorType;
use buttplug::client::ScalarCommand::ScalarMap;
use buttplug::client::RotateCommand::RotateMap;
use futures::StreamExt;
use futures_timer::Delay;
use parking_lot::Mutex;
use rosc::OscType;
use rosc::encoder;
use rosc::{self, OscMessage, OscPacket};
use tauri::AppHandle;
use tauri::Manager;
use tokio::net::UdpSocket as tUdpSocket;
use tokio::sync::mpsc::UnboundedReceiver;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;
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
use crate::frontend_types::FeCoreEvent;
use crate::frontend_types::FeToyEvent;
use crate::frontend_types::FeVCToy;
use crate::frontend_types::FeScanEvent;
use crate::toyops::LevelTweaks;
use crate::toyops::VCFeatureType;
use crate::toyops::{VCToy, FeatureParamMap};
use crate::vcore::ToyManagementEvent;
use crate::vcore::VibeCheckState;
use crate::{vcore::TmSig, vcore::ToyUpdate, vcore::VCError};
use tokio::sync::mpsc::UnboundedSender;
use tauri::api::notification::Notification;
use log::{error as logerr, warn, info, trace};

pub struct HandlerErr {
    pub id: i32,
    pub msg: String,
}

#[derive(Clone, Debug)]
pub enum ToySig {
    //ToyCommand(ToyFeature),
    UpdateToy(ToyUpdate),
    OSCMsg(rosc::OscMessage),
}

// Uses TME send channel and CEH recv channel
/*
pub async fn message_handling(
    vibecheck_state_pointer: Arc<Mutex<VibeCheckState>>,
    tme_send: UnboundedSender<ToyManagementEvent>,
    identifier: String) {

    loop {
        let ceh_ = {
            let mut lock = vibecheck_state_pointer.lock();
            //println!("[+] Got event recv lock!");
            // Can optimize this by Arc<> around the receiver and cloning pointer from lock and receiving from it
            // that way can release lock and wait for packets
            lock.client_eh_event_rx.try_recv()
        };
        //println!("Event: {:?}", event);
        // Update Toys States
        match event {
            Ok(tu) => {
                match tu {
                    EventSig::ToyAdd(mut toy) => {
                        // Load toy config for name of toy if it exists otherwise create the config for the toy name

                        // Load config with toy name
                        let toy_config = load_toy_config(&toy.toy_name);
                        if toy_config.is_some() {
                            toy.populate_toy_feature_param_map(toy_config);
                        } else {
                            toy.populate_toy_feature_param_map(None);
                        }
                        //println!("[TOY FEATURES]\n{:?}", toy.param_feature_map);
                        {
                            let mut vc_lock = vibecheck_state_pointer.lock();
                            //println!("[+] Got toy add lock!");
                            vc_lock.tme_send
                            .send(ToyManagementEvent::Tu(ToyUpdate::AddToy(toy.clone())))
                            .unwrap();
                        // Load toy config for name of toy if it exists otherwise create the config for the toy name
                            vc_lock.toys.insert(toy.toy_id, toy.clone());
                        }

                        let _ = Notification::new(identifier.clone())
                        .title("Toy Connected")
                        .body(format!("{} ({}%)", toy.toy_name, (100.0 * toy.battery_level)).as_str())
                        .show();

                        println!("[+] Toy added: {} | {}", toy.toy_name, toy.toy_id);
                    }
                    EventSig::ToyRemove(id, toy_name) => {
                        let mut vc_lock = vibecheck_state_pointer.lock();
                        //println!("[+] Got toy remove recv lock!");
                        vc_lock.tme_send
                            .send(ToyManagementEvent::Tu(ToyUpdate::RemoveToy(id)))
                            .unwrap();
                        vc_lock.toys.remove(&id);

                        
                        let _ = Notification::new(identifier.clone())
                        .title("Toy Disconnected")
                        .body(format!("{}", toy_name).as_str())
                        .show();
                        
                        //println!("[!] Removed toy: {}", id);
                    }
                    EventSig::Shutdown => {}
                }
            },
            Err(_e) => {
                Delay::new(Duration::from_secs(1)).await;
            }
        }
    }
}
*/

/*
    This handler will handle the adding and removal of toys
    Needs Signals in and out to communicate with main thread
    - communicate errors and handler state (Errors to tell main thread its shutting down && State to receive shutdown from main thread) RECV/SEND
    - communicate toy events (add/remove) ONLY SEND?
*/
// Uses CEH send channel 
pub async fn client_event_handler(
    mut event_stream: impl futures::Stream<Item = ButtplugClientEvent> + std::marker::Unpin,
    vibecheck_state_pointer: Arc<Mutex<VibeCheckState>>,
    identifier: String,
    app_handle: AppHandle,
    tme_send: UnboundedSender<ToyManagementEvent>,
    _error_tx: Sender<VCError>
) {
    // Listen for toys and add them if it connects send add update
    // If a toy disconnects send remove update


    trace!("BP Client Event Handler Handling Events..");
    loop {

        if let Some(event) = event_stream.next().await {
            match event {
                ButtplugClientEvent::DeviceAdded(dev) => {

                    Delay::new(Duration::from_secs(3)).await;
                    let battery_level = match dev.battery_level().await {
                        Ok(battery_lvl) => battery_lvl,
                        Err(_e) => 0.0,
                    };

                    // Load toy config for name of toy if it exists otherwise create the config for the toy name
                    let mut toy = VCToy {
                        toy_id: dev.index(),
                        toy_name: dev.name().clone(),
                        battery_level,
                        toy_connected: dev.connected(),
                        toy_features: dev.message_attributes().clone(),
                        param_feature_map: FeatureParamMap::new(),
                        osc_data: false,
                        listening: false,
                        device_handle: dev.clone(),
                        config: None,
                    };

                    // Load config with toy name
                    match toy.load_toy_config() {
                        Ok(()) => info!("Toy config loaded successfully."),
                        Err(e) => warn!("Toy config failed to load: {:?}", e),
                    }
                    toy.populate_toy_feature_param_map();
                    
                    {
                        let mut vc_lock = vibecheck_state_pointer.lock();
                        vc_lock.toys.insert(toy.toy_id, toy.clone());
                    }
                    trace!("Toy inserted into VibeCheckState toys");

                    tme_send.send(ToyManagementEvent::Tu(ToyUpdate::AddToy(toy.clone()))).unwrap();
                    
                    let _ = app_handle.emit_all("fe_toy_event",
                        FeToyEvent::Add ({
                            FeVCToy {
                                toy_id: toy.toy_id,
                                toy_name: toy.toy_name.clone(),
                                battery_level: toy.battery_level,
                                toy_connected: toy.toy_connected,
                                features: toy.param_feature_map.to_fe(),
                                listening: toy.listening,
                                osc_data: toy.osc_data,
                            }
                        }),
                    );


                    let _ = Notification::new(identifier.clone())
                    .title("Toy Connected")
                    .body(format!("{} ({}%)", toy.toy_name, (100.0 * toy.battery_level)).as_str())
                    .show();

                    info!("Toy Connected: {} | {}", toy.toy_name, toy.toy_id);
                }
                ButtplugClientEvent::DeviceRemoved(dev) => {

                    // Get scan on disconnect and toy
                    let (sod, toy) = {
                        let mut vc_lock = vibecheck_state_pointer.lock();
                        (vc_lock.config.scan_on_disconnect, vc_lock.toys.remove(&dev.index()))
                    };
                    
                    // Check is toy is valid
                    if let Some(toy) = toy {
                        trace!("Removed toy from VibeCheckState toys");
                        tme_send.send(ToyManagementEvent::Tu(ToyUpdate::RemoveToy(dev.index()))).unwrap();

                        let _ = app_handle.emit_all("fe_toy_event", FeToyEvent::Remove(dev.index()));

                        let _ = Notification::new(identifier.clone())
                        .title("Toy Disconnected")
                        .body(format!("{}", toy.toy_name).as_str())
                        .show();

                        if sod {
                            info!("Scan on disconnect is enabled.. Starting scan.");
                            let vc_lock = vibecheck_state_pointer.lock();
                            if vc_lock.bp_client.is_some() {
                                let _ = vc_lock.async_rt.spawn(vc_lock.bp_client.as_ref().unwrap().start_scanning());
                            }
                            let _ = app_handle.emit_all("fe_core_event", FeCoreEvent::Scan(FeScanEvent::Start));
                        }
                    }

                }
                ButtplugClientEvent::ScanningFinished => info!("Scanning finished!"),
                ButtplugClientEvent::ServerDisconnect => break,
                ButtplugClientEvent::PingTimeout => break,
                ButtplugClientEvent::Error(e) => {
                    logerr!("Client Event Error: {:?}", e);
                },
                ButtplugClientEvent::ServerConnect => {
                    info!("Server Connect");
                },
            }
        } else {
            warn!("GOT NONE IN EVENT HANDLER: THIS SHOULD NEVER HAPPEN LOL");
        }
    }
    info!("[!] Event handler returning!");
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
// Uses TME send and recv channel
pub async fn toy_management_handler(
    tme_send: UnboundedSender<ToyManagementEvent>,
    mut tme_recv: UnboundedReceiver<ToyManagementEvent>,
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
                                            info!("Altered toy: {}", new_toy.toy_id);
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
                            TmSig::TMHReset => {
                                info!("TMHReset but not listening");
                            }
                            _ => {},
                        }
                    }
                } // Event handled
            }
            Err(_e) => {}
        }

        if listening {
            // This is a nested runtime maybe remove
            // Would need to pass toy thread handles to VibeCheckState
            let toy_async_rt = Runtime::new().unwrap();
            info!("Started listening!");
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
                info!("Toy: {} started listening..", *toy.0);
            }

            // Create OSC listener thread
            let toy_bcst_tx_osc = toy_bcst_tx.clone();
            info!("Spawning OSC listener..");
            let vc_conf_clone = vc_config.clone();
            let tme_send_clone = tme_send.clone();
            thread::spawn(move || toy_input_routine(toy_bcst_tx_osc, tme_send_clone, vc_conf_clone));

            loop {
                // Recv event (listening)
                let event = tme_recv.recv().await;
                match event {
                    Some(event) => {
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
                                        info!("Toy: {} started listening..", toy.toy_id);
                                    }
                                    ToyUpdate::RemoveToy(id) => {
                                        // OSC Listener thread will only die on StopListening event
                                        if let Some(toy) = running_toy_ths.remove(&id) {
                                            toy.abort();
                                            let _ = toy.await;
                                            info!("[TOY ID: {}] Stopped listening. (ToyUpdate::RemoveToy)", id);
                                            running_toy_ths.remove(&id);
                                            toys.remove(&id);
                                        }
                                    }
                                    ToyUpdate::AlterToy(toy) => {
                                        match toy_bcst_tx.send(ToySig::UpdateToy(
                                            ToyUpdate::AlterToy(toy.clone()),
                                        )) {
                                            Ok(receivers) => info!("Sent ToyUpdate broadcast to {} toys", receivers-1),
                                            Err(e) => logerr!("Failed to send UpdateToy: {}", e),
                                        }
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
                                            info!(
                                                "[TOY ID: {}] Stopped listening. (TMSIG)",
                                                toy.0
                                            );
                                        }
                                        running_toy_ths.clear();
                                        drop(_toy_bcst_rx); // Causes OSC listener to die
                                        toy_async_rt.shutdown_background();
                                        listening = false;
                                        break;//Stop Listening
                                    },
                                    TmSig::TMHReset => {
                                        // Stop listening on every device and clean running thread hashmap
                                        info!("TMHReset");
                                        toys.clear();
                                        for toy in &mut running_toy_ths {
                                            toy.1.abort();
                                            let _ = toy.1.await;
                                            info!(
                                                "[TOY ID: {}] Stopped listening. (TMSIG)",
                                                toy.0
                                            );
                                        }
                                        running_toy_ths.clear();
                                        drop(_toy_bcst_rx); // Causes OSC listener to die
                                        toy_async_rt.shutdown_background();
                                        listening = false;
                                        break;//Stop Listening
                                    }
                                    _ => {},
                                }
                            }
                        } // Event handled
                    }
                    None => {}
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
fn toy_input_routine(toy_bcst_tx: BSender<ToySig>, tme_send: UnboundedSender<ToyManagementEvent>, vc_config: OSCNetworking) {

    let bind_sock = match UdpSocket::bind(format!("{}:{}", vc_config.bind.ip(), vc_config.bind.port())) {
        Ok(s) => {
            let _ = tme_send.send(ToyManagementEvent::Sig(TmSig::Listening));
            s
        },
        Err(_e) => {
            let _ = tme_send.send(ToyManagementEvent::Sig(TmSig::BindError));
            return;
        }
    };
    info!("Listen sock is bound");
    bind_sock.set_nonblocking(false).unwrap();
    let _ = bind_sock.set_read_timeout(Some(Duration::from_secs(1)));

    loop {
        // try recv OSC packet
        // parse OSC packet
        // Send address and arg to broadcast channel
        // Die when channel disconnects

        match recv_osc_cmd(&bind_sock) {
            Some(msg) => {
                if let Err(_) = toy_bcst_tx.send(ToySig::OSCMsg(msg)) {
                    info!("BCST TX is disconnected. Shutting down toy input routine!");
                    return; // Shutting down handler_routine
                }
            }
            None => {
                if toy_bcst_tx.receiver_count() == 0 {
                    info!(
                        "BCST TX is disconnected (RECV C=0). Shutting down toy input routine!"
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


/*
 * Toy update loop every 1 sec maybe 5
 * How to do parameter structure
 * /avatar/parameters/toy_name
 * INT SIGS:
 * 0 - 100: toy.battery_level
 * -1: connected
 * -2: disconnected
 * 
 */

pub async fn toy_refresh(vibecheck_state_pointer: Arc<Mutex<VibeCheckState>>) {

    loop {
        Delay::new(Duration::from_secs(60)).await;


        let (toys, remote) = {
            let vc_lock = vibecheck_state_pointer.lock();
            if !vc_lock.toys.is_empty() {
                (vc_lock.toys.clone(), vc_lock.config.networking.remote)
            } else {
                continue;
            }
        };

        let sock = tUdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();
        info!("Bound toy_refresh sender sock to {}", sock.local_addr().unwrap());
        sock.connect(remote).await.unwrap();
        for (.., mut toy) in toys {

            toy.battery_level = match toy.device_handle.battery_level().await {
                Ok(battery_lvl) => battery_lvl,
                Err(_e) => 0.0,
            };
            
            if toy.osc_data {

                let battery_level_msg = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: format!("/avatar/parameters/{}", toy.toy_name.to_lowercase().replace(" ", "_")),
                    args: vec![OscType::Int(toy.battery_level as i32 * 100)]
                })).unwrap();

                let batt_send_err = sock.send(&battery_level_msg).await;
                if batt_send_err.is_err(){warn!("Failed to send battery_level to {}", remote.to_string());}
                else{info!("Sent battery_level: {} to {}", toy.battery_level, toy.toy_name);}

            }
        }
    }
}