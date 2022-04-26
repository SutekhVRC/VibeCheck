use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};
use std::task::Context;
use std::time::Duration;
use buttplug::client::{ButtplugClient, ButtplugClientEvent, ButtplugClientDeviceMessageType};
use buttplug::client::ButtplugClientDevice;
use eframe::egui::Button;
use futures::{StreamExt, FutureExt, Stream};
use futures_timer::Delay;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use std::thread;
use tokio::sync::{self, broadcast::{Sender as BSender, Receiver as BReceiver}};
use rosc::{self, OscMessage, OscPacket, OscType};

use crate::ui::{ToyManagementEvent, ToyMode, FeatureParamMap, FeatureMode};
use crate::{
    get_user_home_dir,
    ui::ToyUpdate,
    ui::VCToy,
    ui::ToyFeature,
    ui::VCError,
    ui::TmSig,
};

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
pub async fn client_event_handler(
    error_tx: Sender<VCError>,
    event_tx: Sender<EventSig>,
) {

    // Listen for toys and add them if it connects send add update
    // If a toy disconnects send remove update

    println!("[*] Initializing CEH..");

    Delay::new(Duration::from_secs(3)).await;

    let client = ButtplugClient::new("VibeCheck");
    let mut event_stream = client.event_stream();

    // Create in-process connector
    match client.connect_in_process(None).await {
        Ok(_) => {},
        Err(_e) => {
            let _ = error_tx.send(VCError::HandlingErr(HandlerErr{id: -1, msg: format!("Failed to connect in process. {}", _e)}));
            println!("CON PROC ERR: {}", _e);
            return;
        }
    }
    println!("[*] Connected to process");

    // Start scanning for toys
    if let Err(e) = client.start_scanning().await {
        let _ = error_tx.send(VCError::HandlingErr(HandlerErr{id: -2, msg: format!("Failed to scan for bluetooth devices. {}", e)}));
        return;
    }

    loop {
/*
        loop {
            let c = event_stream.size_hint();
            println!("{:?}", c);
            if c.0 != 0 {
                break;
            }
            Delay::new(Duration::from_secs(1)).await;
        }
        /*
            Make event handler a new thread and have loop that reads from mpsc channel and abort 
        */
*/
        if let Some(event) = event_stream.next().await {
            match event {
                ButtplugClientEvent::DeviceAdded(dev) => {

                    Delay::new(Duration::from_secs(3)).await;
                    let battery_level = dev.battery_level().await.unwrap();
 
                    let _ = event_tx.send(EventSig::ToyAdd(VCToy{
                        toy_id: dev.index(),
                        toy_name: dev.name.clone(),
                        battery_level,
                        toy_connected: dev.connected(),
                        toy_features: dev.allowed_messages.clone(),
                        osc_params_list: vec![],
                        toy_param_mode: ToyMode::Auto("".to_string()),
                        param_feature_map: FeatureParamMap::new(),
                        listening: false,
                        device_handle: dev.clone(),
                    }));

                    println!("[+] Device connected!!!!");
                },
                ButtplugClientEvent::DeviceRemoved(dev) => {
    
                    event_tx.send(EventSig::ToyRemove(dev.index()));
                    println!("[*] Sent dev discon to UI.");
                },
                ButtplugClientEvent::ScanningFinished => println!("[!] Scanning finished!"),
                ButtplugClientEvent::ServerDisconnect => {
                    event_tx.send(EventSig::Shutdown);
                    println!("[!] Server disconnected!");
                    client.stop_scanning().await;
                    client.disconnect().await;
                    break;
                },
                ButtplugClientEvent::PingTimeout => {
                    event_tx.send(EventSig::Shutdown);
                    println!("[!] Server timeout!");
                    client.stop_scanning().await;
                    client.disconnect().await;
                    break;
                },
                _ => {},
            }
        } else {println!("GOT NONE IN EVENT HANDLER");}
    }
    println!("[!] Event handler returning!");
}

/*
    This handler will send and receive updates to toys
    - communicate ToyUpdate to and from main thread SEND/RECV (Toys will be indexed on the main thread) (Connects and disconnect toy updates are handled by client event handler)
        + Keep a thread count of connected toys. Add/remove as recvs ToyUpdates from main thread
        + Send toy updates like (battery updates)
*/
pub async fn toy_management_handler(tme_send: Sender<ToyManagementEvent>, tme_recv: Receiver<ToyManagementEvent>, mut toys: HashMap<u32, VCToy>) {

    let f = |dev: Arc<ButtplugClientDevice>, mut toy_bcst_rx: BReceiver<ToySig>, mut feature_map: FeatureParamMap| {
        // Read toy config here?
        async move {
            while dev.connected() {
                match toy_bcst_rx.recv().await {
                    Ok(ts) => {
                        match ts {
                            ToySig::OSCMsg(mut msg) => {// Parse OSC msgs to toys commands
                                
                                // Does parameter name assign to a feature on this toy?
                                
                                // Parse param get Vec of Features
                                // these vec items will match the param
                                // Toy feature Auto/Custom
                                // Parse if Auto or Custom
                                // if Auto Speed() if Custom get index from param hashmap
                                
                                if let Some(features) = feature_map.get_features_from_param(&msg.addr) {
                                    if let Some(lvl) = msg.args.pop().unwrap().float() {
                                        for feature in features {
                                        
                                            match feature {
                                        
                                                ToyFeature::Vibrator(fm) => {
                                                    let vibe_level = ((lvl * 100.0).round() / 100.0) as f64;
                                                    if let FeatureMode::Custom(fi) = fm {
                                                        dev.vibrate(buttplug::client::VibrateCommand::SpeedMap(HashMap::from([(fi, vibe_level)]))).await;
                                                    } else {
                                                        dev.vibrate(buttplug::client::VibrateCommand::Speed(vibe_level)).await;
                                                    }
                                                },
                                                ToyFeature::Rotator(fm) => {
                                                    let rotate_level = ((lvl * 100.0).round() / 100.0) as f64;
                                                    if let FeatureMode::Custom(fi) = fm {
                                                        dev.rotate(buttplug::client::RotateCommand::RotateMap(HashMap::from([(fi, (rotate_level,true))]))).await;
                                                    } else {
                                                        dev.rotate(buttplug::client::RotateCommand::Rotate(rotate_level, true)).await;
                                                    }
                                                },
                                                ToyFeature::Linear(fm) => {
                                                    let linear_level = ((lvl * 100.0).round() / 100.0) as f64;
                                                    if let FeatureMode::Custom(fi) = fm {
                                                        dev.linear(buttplug::client::LinearCommand::LinearMap(HashMap::from([(fi, (500, linear_level))]))).await;
                                                    } else {
                                                        dev.linear(buttplug::client::LinearCommand::Linear(500, linear_level)).await;
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
                                    },
                                    _ => {}// Remove and Add are handled internally from device connected state and management loop (listening)
                                }
                            }
                        }
                    },
                    Err(_e) => {},
                }
            }
            println!("[*] Device {} disconnected! Leaving listening routine!", dev.index());
        }
    };// Toy listening routine

    let mut listening = false;

    // Management loop
    loop {
        // Recv event (not listening)
        match tme_recv.try_recv() {
            Ok(event) => {
                match event {
                    // Handle Toy Update Signals
                    ToyManagementEvent::Tu(tu) => {
                        match tu {
                            ToyUpdate::AddToy(toy) => {
                                toys.insert(toy.toy_id, toy);
                            },
                            ToyUpdate::RemoveToy(id) => {
                                toys.remove(&id);
                            },
                            ToyUpdate::AlterToy(toy) => {
                                toys.insert(toy.toy_id, toy);
                            }
                        }
                    },
                    // Handle Management Signals
                    ToyManagementEvent::Sig(tm_sig) => {
                        match tm_sig {
                            TmSig::StartListening => {
                                listening = true;
                            },
                            TmSig::StopListening => {
                                // Already not listening
                            }
                        }
                    }
                }// Event handled
            },
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
            let (toy_bcst_tx, _toy_bcst_rx): (BSender<ToySig>, BReceiver<ToySig>) = sync::broadcast::channel(1024);

            // Create toy treads
            for toy in &toys {
                let f_run = f(toy.1.device_handle.clone(), toy_bcst_tx.subscribe(), toy.1.param_feature_map.clone());
                running_toy_ths.insert(*toy.0, toy_async_rt.spawn(async move {f_run.await;}));
                println!("[**] Toy: {} started listening..", *toy.0);
            }

            // Create OSC listener thread
            let toy_bcst_tx_osc = toy_bcst_tx.clone();
            println!("[+] Spawning OSC listener..");
            thread::spawn(move || toy_input_routine(toy_bcst_tx_osc));

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
                                        let f_run = f(toy.device_handle, toy_bcst_tx.subscribe(), toy.param_feature_map.clone());
                                        running_toy_ths.insert(toy.toy_id, toy_async_rt.spawn(async move {f_run.await;}));
                                        println!("[**] Toy: {} started listening..", toy.toy_id);
                                    },
                                    ToyUpdate::RemoveToy(id) => {
                                        // OSC Listener thread will only die on StopListening event
                                        if let Some(toy) = running_toy_ths.remove(&id) {
                                            toy.abort();
                                            toy.await;
                                            println!("[TOY ID: {}] Stopped listening. (ToyUpdate::RemoveToy)", id);
                                            running_toy_ths.remove(&id);
                                            toys.remove(&id);
                                        }
                                    },
                                    ToyUpdate::AlterToy(toy) => {
                                        toy_bcst_tx.send(ToySig::UpdateToy(ToyUpdate::AlterToy(toy.clone())));
                                        toys.insert(toy.toy_id, toy);
                                    }
                                }
                            },
                            // Handle Management Signals
                            ToyManagementEvent::Sig(tm_sig) => {
                                match tm_sig {
                                    TmSig::StartListening => {
                                        // Already listening
                                    },
                                    TmSig::StopListening => {
                                        // Stop listening on every device and clean running thread hashmap
                                        for toy in &mut running_toy_ths {
                                            toy.1.abort();
                                            toy.1.await;
                                            println!("[TOY ID: {}] Stopped listening. (TMSIG)", toy.0);
                                        }
                                        running_toy_ths.clear();
                                        drop(_toy_bcst_rx);// Causes OSC listener to die
                                        toy_async_rt.shutdown_background();
                                        listening = false;
                                        break;
                                    }
                                }
                            }
                        }// Event handled
                    },
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
fn toy_input_routine(toy_bcst_tx: BSender<ToySig>) {
    // I think I need a toy structure here to parse the inputted OSC args
    let bind_sock = UdpSocket::bind("127.0.0.1:10069").unwrap();
    println!("Listen sock is bound");
    bind_sock.set_nonblocking(true).unwrap();
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
                    return;// Shutting down handler_routine
                }
            },
            None => {
                if toy_bcst_tx.receiver_count() == 0 {
                    println!("[*] BCST TX is disconnected (RECV C=0). Shutting down toy input routine!");
                    return;
                }
            },
        }
    }
}

fn recv_osc_cmd(sock: &UdpSocket) -> Option<OscMessage> {
    let mut buf = [0u8; rosc::decoder::MTU];

    let (br, _a) = match sock.recv_from(&mut buf) {
        Ok((br, a)) => (br, a),
        Err(_e) => {return None;}
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
            },
            _ => {return None;}
        }
    }
}