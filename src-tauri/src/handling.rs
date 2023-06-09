use buttplug::client::ButtplugClientDevice;
use buttplug::client::ButtplugClientEvent;
use buttplug::core::message::ActuatorType;
use buttplug::client::ScalarCommand::ScalarMap;
use buttplug::client::RotateCommand::RotateMap;
use futures::StreamExt;
use futures_timer::Delay;
use log::debug;
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
use std::time::Instant;
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
use crate::osc_api::osc_api::vibecheck_osc_api;
use crate::toy_manager::ToyManager;
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
                    let battery_level: Option<f64> = match dev.battery_level().await {
                        Ok(battery_lvl) => Some(battery_lvl),
                        Err(_e) => {
                            warn!("Device battery_level error: {:?}", _e);
                            match _e {
                                buttplug::client::ButtplugClientError::ButtplugError(bpe) => {
                                    match bpe {
                                        buttplug::core::errors::ButtplugError::ButtplugDeviceError(bde) => {
                                            match bde {
                                                buttplug::core::errors::ButtplugDeviceError::MessageNotSupported(_) => None,
                                                _ => Some(0.0)// Not Message not supported error
                                            }// msg not supported
                                        },// buttplug device error
                                        _ => Some(0.0)// Not device error
                                    }// buttplug error
                                },// buttplug error
                                _ => Some(0.0),
                            }
                        },
                    };

                    let sub_id = {
                        let vc_lock = vibecheck_state_pointer.lock();
                        let mut toy_dup_count = 0;
                        vc_lock.core_toy_manager.as_ref().unwrap().online_toys.iter().for_each(|toy| {
                            if &toy.1.toy_name == dev.name() {
                                toy_dup_count += 1;
                            }
                        });
                        toy_dup_count
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
                        sub_id,
                    };

                    // Load config with toy name
                    match toy.load_toy_config() {
                        Ok(()) => info!("Toy config loaded successfully."),
                        Err(e) => warn!("Toy config failed to load: {:?}", e),
                    }
                    toy.populate_toy_config();
                    
                    {
                        let mut vc_lock = vibecheck_state_pointer.lock();
                        vc_lock.core_toy_manager.as_mut().unwrap().online_toys.insert(toy.toy_id, toy.clone());
                    }
                    trace!("Toy inserted into VibeCheckState toys");

                    tme_send.send(ToyManagementEvent::Tu(ToyUpdate::AddToy(toy.clone()))).unwrap();
                    
                    let _ = app_handle.emit_all("fe_toy_event",
                        FeToyEvent::Add ({
                            FeVCToy {
                                toy_id: Some(toy.toy_id),
                                toy_name: toy.toy_name.clone(),
                                toy_anatomy: toy.config.as_ref().unwrap().anatomy.to_fe(),
                                battery_level,
                                toy_connected: toy.toy_connected,
                                features: toy.param_feature_map.to_fe(),
                                listening: toy.listening,
                                osc_data: toy.osc_data,
                                sub_id: toy.sub_id,
                            }
                        }),
                    );

                    {
                        let vc_lock = vibecheck_state_pointer.lock();
                        if vc_lock.config.desktop_notifications {
                            let _ = Notification::new(identifier.clone())
                            .title("Toy Connected")
                            .body(format!("{} ({}%)", toy.toy_name, (100.0 * toy.battery_level.unwrap_or(0.0))).as_str())
                            .show();
                        }
                    }

                    info!("Toy Connected: {} | {}", toy.toy_name, toy.toy_id);
                }
                ButtplugClientEvent::DeviceRemoved(dev) => {

                    // Get scan on disconnect and toy
                    let (sod, toy) = {
                        let mut vc_lock = vibecheck_state_pointer.lock();
                        (vc_lock.config.scan_on_disconnect, vc_lock.core_toy_manager.as_mut().unwrap().online_toys.remove(&dev.index()))
                    };
                    
                    // Check if toy is valid
                    if let Some(toy) = toy {
                        trace!("Removed toy from VibeCheckState toys");
                        tme_send.send(ToyManagementEvent::Tu(ToyUpdate::RemoveToy(dev.index()))).unwrap();

                        let _ = app_handle.emit_all("fe_toy_event", FeToyEvent::Remove(dev.index()));

                        {
                            let vc_lock = vibecheck_state_pointer.lock();
                            if vc_lock.config.desktop_notifications {
                                let _ = Notification::new(identifier.clone())
                                .title("Toy Disconnected")
                                .body(format!("{}", toy.toy_name).as_str())
                                .show();
                            }
                        }

                        if sod {
                            info!("Scan on disconnect is enabled.. Starting scan.");
                            let vc_lock = vibecheck_state_pointer.lock();
                            if vc_lock.bp_client.is_some() && vc_lock.config.scan_on_disconnect {
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
    info!("Event handler returning!");
}

// Parse scalar levels and logic for level tweaks
pub async fn scalar_parse_levels_send_toy_cmd(dev: &Arc<ButtplugClientDevice>, scalar_level: f64, feature_index: u32, actuator_type: ActuatorType, flip_float: bool, feature_levels: LevelTweaks) {

    if !flip_float {
        if scalar_level != 0.0 && scalar_level >= feature_levels.minimum_level && scalar_level <= feature_levels.maximum_level {
        
            info!("SENDING FI[{}] AT[{}] SL[{}]", feature_index, actuator_type, scalar_level);
            match dev.scalar(&ScalarMap(HashMap::from([(feature_index, (scalar_level, actuator_type))]))).await {
                Ok(()) => {},
                Err(e) => {
                    logerr!("Send scalar to device error: {}", e);
                }
            }
        
        } else if scalar_level == 0.0 {// if level is 0 put at idle

            info!("IDLE FI[{}] AT[{}] SL[{}]", feature_index, actuator_type, feature_levels.idle_level);
            let _e = dev.scalar(&buttplug::client::ScalarCommand::ScalarMap(HashMap::from([(feature_index, (feature_levels.idle_level, actuator_type))]))).await;

        } else if scalar_level > feature_levels.maximum_level {
            let _e = dev.scalar(&ScalarMap(HashMap::from([(feature_index, (feature_levels.maximum_level, actuator_type))]))).await;
        } else if scalar_level < feature_levels.minimum_level {
            let _e = dev.scalar(&ScalarMap(HashMap::from([(feature_index, (feature_levels.minimum_level, actuator_type))]))).await;
        }
    } else {// FLOAT FLIPPED
        
        let flipped_lvl = flip_float64(scalar_level);
        //debug!("Flipped: {:.5}", flipped_lvl);
        // Reverse logic here for flipped float
        if flipped_lvl != 1.0 && flipped_lvl <= flip_float64(feature_levels.minimum_level) && flipped_lvl >= flip_float64(feature_levels.maximum_level) {

            info!("SENDING FI[{}] AT[{}] SL[{:.5}]", feature_index, actuator_type, flipped_lvl);
            let _e = dev.scalar(&ScalarMap(HashMap::from([(feature_index, (flipped_lvl, actuator_type))]))).await;

        } else if flipped_lvl == 1.0 {// if flipped level is 1.0 put at idle

            info!("IDLE FI[{}] AT[{}] SL[{}]", feature_index, actuator_type, flip_float64(feature_levels.idle_level));
            let _e = dev.scalar(&buttplug::client::ScalarCommand::ScalarMap(HashMap::from([(feature_index, (flip_float64(feature_levels.idle_level), actuator_type))]))).await;
        } else if flipped_lvl < flip_float64(feature_levels.maximum_level) {
            let _e = dev.scalar(&ScalarMap(HashMap::from([(feature_index, (flip_float64(feature_levels.maximum_level), actuator_type))]))).await;
        } else if flipped_lvl > flip_float64(feature_levels.minimum_level) {
            let _e = dev.scalar(&ScalarMap(HashMap::from([(feature_index, (flip_float64(feature_levels.minimum_level), actuator_type))]))).await;
        }
    }
}

#[inline]
pub fn flip_float64(orig: f64) -> f64 {
    //1.00 - orig
    ((1.00 - orig) * 100.0).round() / 100.0
}

enum SmoothParser {
    Smoothed,
    SkipZero,
    Smoothing,
}

#[inline]
fn parse_smoothing(smooth_queue: &mut Vec<f64>, feature_levels: LevelTweaks, float_level: &mut f64, flip_float: bool) -> SmoothParser {
    debug!("!flip_float && *float_level == 0.0: [{}] || [{}] flip_float && *float_level == 1.0\nCOMBINED: [{}]", !flip_float && *float_level == 0.0, flip_float && *float_level == 1.0,
    smooth_queue.len() == feature_levels.smooth_rate as usize && (!flip_float && *float_level == 0.0 || flip_float && *float_level == 1.0)
);
    // Reached smooth rate maximum and not a 0 value
    if smooth_queue.len() == feature_levels.smooth_rate as usize {
        debug!("smooth_queue: {}", smooth_queue.len());
        if !flip_float && *float_level == 0.0 || flip_float && *float_level == 1.0 {

            // Don't return just set to 0
            debug!("float level is 0 but will be forgotten!");

            // Clear smooth queue bc restarting from 0
            smooth_queue.clear();

        } else {
            debug!("Setting float_level with smoothed float");
            // Get Mean of all numbers in smoothing rate and then round to hundredths and cast as f64
            *float_level = ((smooth_queue.iter().sum::<f64>() as f64 / smooth_queue.len() as f64 * 100.0).round() / 100.0) as f64;
            smooth_queue.clear();

            smooth_queue.push(*float_level);
            return SmoothParser::Smoothed;   
        }

    // Have not reached smoothing maximum
    }

    // Maybe move this to be before queue is full check?
    if !flip_float && *float_level == 0.0 || flip_float && *float_level == 1.0 {
        debug!("Bypassing smoother: {:.5}", float_level);
        // let 0 through
        return SmoothParser::SkipZero;
    }                              

    debug!("Adding float {} to smoothing.. queue size: {}", *float_level, smooth_queue.len());
    smooth_queue.push(*float_level);
    // Continue receiving smooth floats
    SmoothParser::Smoothing
}

enum RateParser {
    RateCalculated(bool),
    SkipZero,
}

#[inline]
fn parse_rate(rate_internal_level: &mut f64, rate_saved_osc_input: &mut f64, rate_timestamp: &mut Option<Instant>, decrement_rate: f64, float_level: &mut f64, flip_float: bool) -> RateParser {

    // Skip because got 0 value to stop toy.
    if !flip_float && *float_level <= 0.0 || flip_float && *float_level >= 1.0 {
        debug!("Bypassing rate input");
        *rate_internal_level = *float_level;
        *rate_saved_osc_input = *float_level;
        return RateParser::SkipZero;

    } else {// Increase toy level
        
        // Store new input then get the distance of the new input from the last input
        // Add that distance to the internal float level
        
        // get distance between newest input and last input
        // Set the distance between as the new motor speed
        if *rate_saved_osc_input > *float_level {
            *rate_internal_level += (*rate_saved_osc_input - *float_level).clamp(0.00, 1.0);
        } else {
            *rate_internal_level += (*float_level - *rate_saved_osc_input).clamp(0.00, 1.0);
        }

        // Dont let internal level go over 1.0
        *rate_internal_level = rate_internal_level.clamp(0.00, 1.00);

        // Set the newest input as the recent input
        *rate_saved_osc_input = *float_level;
        
        // Set the internal rate state to the float level
        *float_level = *rate_internal_level;

        // Save the internal motor speed
        //*rate_internal_level += *float_level;

        trace!("float level rate increased");
    }

    // Decrement testing
    if let Some(instant) = rate_timestamp {

        // Decrease tick
        if instant.elapsed().as_secs_f64() >= 0.2 {

            // Decrease the internal rate level
            // This decrease rate should be tuneable
            *rate_internal_level = (*rate_internal_level - decrement_rate).clamp(0.00, 1.0);
            debug!("internal level after decrement: {}", rate_internal_level);
            
            // Set float level to decremented internal rate
            *float_level = *rate_internal_level;
            
            trace!("decrease timer reset");
            return RateParser::RateCalculated(true);
        }
    }

    RateParser::RateCalculated(false)
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
    mut core_toy_manager: ToyManager,
    mut vc_config: OSCNetworking,
    app_handle: AppHandle,
) {
    let f = |dev: Arc<ButtplugClientDevice>,
             mut toy_bcst_rx: BReceiver<ToySig>,
             mut feature_map: FeatureParamMap| {
        // Read toy config here?
        async move {

            // Put smooth_queue here
            // Put rate tracking here
            // Time tracking here?
            // Async runtime wrapped in Option for rate updating here????

            while dev.connected() {
                //trace!("Toy recv loop start");
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
                                        //debug!("Received and cast float lvl: {:.5}", float_level);

                                        // Iterate through features enumerated from OSC param
                                        for (feature_type, feature_index, flip_float, feature_levels, smooth_enabled, smooth_queue, rate_enabled, rate_saved_level, rate_saved_osc_input, rate_timestamp) in features {
                                            
                                            // Smoothing enabled
                                            if smooth_enabled && !rate_enabled {
                                                //trace!("parse_moothing()");
                                                match parse_smoothing(smooth_queue, feature_levels, &mut float_level, flip_float) {
                                                    SmoothParser::SkipZero | SmoothParser::Smoothed => command_toy(dev.clone(), feature_type, float_level, feature_index, flip_float, feature_levels).await,
                                                    SmoothParser::Smoothing => continue,
                                                }
                                            } else if rate_enabled && !smooth_enabled {

                                                //trace!("parse_rate()");
                                                // Need to set rate_timestamp when feature enabled
                                                if let None = rate_timestamp {
                                                    *rate_timestamp = Some(Instant::now());
                                                }
                                                match parse_rate(rate_saved_level, rate_saved_osc_input, rate_timestamp, feature_levels.rate_tune, &mut float_level, flip_float) {
                                                    RateParser::SkipZero => command_toy(dev.clone(), feature_type, float_level, feature_index, flip_float, feature_levels).await,
                                                    RateParser::RateCalculated(reset_timer) => {
                                                        if reset_timer {
                                                            *rate_timestamp = Some(Instant::now())
                                                        }
                                                        command_toy(dev.clone(), feature_type, float_level, feature_index, flip_float, feature_levels).await;
                                                    },
                                                }
                                            } else {
                                                command_toy(dev.clone(), feature_type, float_level, feature_index, flip_float, feature_levels).await;
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
            info!(
                "Device {} disconnected! Leaving listening routine!",
                dev.index()
            );
        }
    }; // Toy listening routine

    let mut listening = false;

    // Management loop
    loop {
        // Recv event (not listening)
        match tme_recv.recv().await {
            Some(event) => {
                match event {
                    // Handle Toy Update Signals
                    ToyManagementEvent::Tu(tu) => match tu {
                        ToyUpdate::AddToy(toy) => {
                            core_toy_manager.online_toys.insert(toy.toy_id, toy);
                        }
                        ToyUpdate::RemoveToy(id) => {
                            core_toy_manager.online_toys.remove(&id);
                        }
                        ToyUpdate::AlterToy(toy) => {
                            core_toy_manager.online_toys.insert(toy.toy_id, toy);
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
                                info!("StopListening but not listening");
                            },
                            TmSig::TMHReset => {
                                info!("TMHReset but not listening");
                            }
                            _ => {},
                        }
                    }
                } // Event handled
            }
            None => {}
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
            for toy in &core_toy_manager.online_toys {
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
            let app_handle_clone = app_handle.clone();
            thread::spawn(move || toy_input_routine(toy_bcst_tx_osc, tme_send_clone, app_handle_clone, vc_conf_clone));

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
                                        core_toy_manager.online_toys.insert(toy.toy_id, toy.clone());
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
                                            match toy.await {
                                                Ok(()) => info!("Toy {} thread finished", id),
                                                Err(e) => warn!("Toy {} thread failed to reach completion: {}", id, e),
                                            }
                                            info!("[TOY ID: {}] Stopped listening. (ToyUpdate::RemoveToy)", id);
                                            running_toy_ths.remove(&id);
                                            core_toy_manager.online_toys.remove(&id);
                                        }
                                    }
                                    ToyUpdate::AlterToy(toy) => {
                                        match toy_bcst_tx.send(ToySig::UpdateToy(
                                            ToyUpdate::AlterToy(toy.clone()),
                                        )) {
                                            Ok(receivers) => info!("Sent ToyUpdate broadcast to {} toys", receivers-1),
                                            Err(e) => logerr!("Failed to send UpdateToy: {}", e),
                                        }
                                        core_toy_manager.online_toys.insert(toy.toy_id, toy);
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
                                            match toy.1.await {
                                                Ok(()) => info!("Toy {} thread finished", toy.0),
                                                Err(e) => warn!("Toy {} thread failed to reach completion: {}", toy.0, e),
                                            }
                                            info!(
                                                "[TOY ID: {}] Stopped listening. (TMSIG)",
                                                toy.0
                                            );
                                        }
                                        running_toy_ths.clear();
                                        drop(_toy_bcst_rx); // Causes OSC listener to die
                                        toy_async_rt.shutdown_background();
                                        listening = false;
                                        info!("Toys: {}", core_toy_manager.online_toys.len());
                                        break;//Stop Listening
                                    },
                                    TmSig::TMHReset => {
                                        // Stop listening on every device and clean running thread hashmap
                                        info!("TMHReset");

                                        for toy in &mut running_toy_ths {
                                            toy.1.abort();
                                            match toy.1.await {
                                                Ok(()) => info!("Toy {} thread finished", toy.0),
                                                Err(e) => warn!("Toy {} thread failed to reach completion: {}", toy.0, e),
                                            }
                                            info!(
                                                "[TOY ID: {}] Stopped listening. (TMSIG)",
                                                toy.0
                                            );
                                        }
                                        running_toy_ths.clear();
                                        drop(_toy_bcst_rx); // Causes OSC listener to die
                                        toy_async_rt.shutdown_background();
                                        listening = false;
                                        info!("Toys: {}", core_toy_manager.online_toys.len());
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
 * Sends commands to toys
 */
pub async fn command_toy(
    dev: Arc<ButtplugClientDevice>,
    feature_type: VCFeatureType,
    float_level: f64,
    feature_index: u32,
    flip_float: bool,
    feature_levels: LevelTweaks,
) {
    
    match feature_type {
        VCFeatureType::Vibrator => {
            scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Vibrate, flip_float, feature_levels).await;
        },
        // We handle Rotator differently because it is not included in the Scalar feature set
        VCFeatureType::Rotator => {

            // I think im going to convert this to match
            if !flip_float {
                if float_level != 0.0 && float_level >= feature_levels.minimum_level && float_level <= feature_levels.maximum_level {
                    // Do normal input
                    let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (float_level, true))]))).await;
                } else if float_level == 0.0 {// if level is 0 put at idle
                    let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (feature_levels.idle_level, true))]))).await;
                } else if float_level > feature_levels.maximum_level {
                    // Do max
                    let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (feature_levels.maximum_level, true))]))).await;
                } else if float_level < feature_levels.minimum_level {
                    let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (feature_levels.minimum_level, true))]))).await;
                }
            } else {// FLOAT FLIPPED
                
                let flipped_lvl = flip_float64(float_level);
                // Reverse logic here for flipped float
                if flipped_lvl != 1.0 && flipped_lvl <= flip_float64(feature_levels.minimum_level) && flipped_lvl >= flip_float64(feature_levels.maximum_level) {
                    let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (flipped_lvl, true))]))).await;
                } else if flipped_lvl == 1.0 {// if flipped level is 1.0 put at idle
                    let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (flip_float64(feature_levels.idle_level), true))]))).await;
                } else if flipped_lvl < flip_float64(feature_levels.maximum_level) {
                    let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (flip_float64(feature_levels.maximum_level), true))]))).await;
                } else if flipped_lvl > flip_float64(feature_levels.minimum_level) {
                    let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (flip_float64(feature_levels.minimum_level), true))]))).await;
                }
            }
        },
        VCFeatureType::Constrict => {
            scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Constrict, flip_float, feature_levels).await;
        },
        VCFeatureType::Oscillate => {
            scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Oscillate, flip_float, feature_levels).await;
        },
        VCFeatureType::Position => {
            scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Position, flip_float, feature_levels).await;
        },
        VCFeatureType::Inflate => {
            scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Inflate, flip_float, feature_levels).await;
        },
        // We handle Linear differently because it is not included in the Scalar feature set
        VCFeatureType::Linear => {

            if !flip_float {
                if float_level != 0.0 && float_level >= feature_levels.minimum_level && float_level <= feature_levels.maximum_level {
                    let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (feature_levels.linear_position_speed, float_level))]))).await;
                } else if float_level == 0.0 {// if level is 0 put at idle
                    let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (feature_levels.linear_position_speed, feature_levels.idle_level))]))).await;
                } else if float_level > feature_levels.maximum_level {
                    let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (feature_levels.linear_position_speed, feature_levels.maximum_level))]))).await;
                } else if float_level < feature_levels.minimum_level {
                    let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (feature_levels.linear_position_speed, feature_levels.minimum_level))]))).await;
                }
            } else {// FLOAT FLIPPED
                
                let flipped_lvl = flip_float64(float_level);
                // Reverse logic here for flipped float
                if flipped_lvl != 1.0 && flipped_lvl <= flip_float64(feature_levels.minimum_level) && flipped_lvl >= flip_float64(feature_levels.maximum_level) {
                    let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (feature_levels.linear_position_speed, flip_float64(float_level)))]))).await;
                } else if flipped_lvl == 1.0 {// if flipped level is 1.0 put at idle
                    let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (feature_levels.linear_position_speed, flip_float64(feature_levels.idle_level)))]))).await;
                } else if flipped_lvl < flip_float64(feature_levels.maximum_level) {
                    let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (feature_levels.linear_position_speed, flip_float64(feature_levels.maximum_level)))]))).await;
                } else if flipped_lvl > flip_float64(feature_levels.minimum_level) {
                    let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (feature_levels.linear_position_speed, flip_float64(feature_levels.minimum_level)))]))).await;
                }
            }
        }
        VCFeatureType::ScalarRotator => {
            scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Rotate, flip_float, feature_levels).await;
        },
    }
}

/*
    This subroutine
    Binds the OSC listen socket
    receives OSC messages
    broadcasts the OSC messages to each toy
*/
fn toy_input_routine(toy_bcst_tx: BSender<ToySig>, tme_send: UnboundedSender<ToyManagementEvent>, app_handle: AppHandle, vc_config: OSCNetworking) {

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

        if vibecheck_osc_api(&bind_sock, &app_handle, &toy_bcst_tx) {
            continue;
        } else {
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
        },
        Err(_e) => {
            logerr!("Failed to bind UDP socket for disabled cmd listening.. Retries remaining: {}", retries);
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

        if br <= 0 {
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
                                let _ = app_handle.emit_all("fe_core_event", FeCoreEvent::State(crate::frontend_types::FeStateEvent::EnableAndScan));
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


/* FUTURE MAYBE
 * Toy update loop every 1 sec maybe 5
 * How to do parameter structure
 * /avatar/parameters/toy_name
 * INT SIGS:
 * 0 - 100: toy.battery_level
 * -1: connected
 * -2: disconnected
 * 
 * ATM this only sends a battery life OSC address/value.
 */

pub async fn toy_refresh(vibecheck_state_pointer: Arc<Mutex<VibeCheckState>>, app_handle: AppHandle) {

    loop {
        Delay::new(Duration::from_secs(15)).await;


        let (toys, remote) = {
            let vc_lock = vibecheck_state_pointer.lock();
            if !vc_lock.core_toy_manager.as_ref().unwrap().online_toys.is_empty() {
                (vc_lock.core_toy_manager.as_ref().unwrap().online_toys.clone(), vc_lock.config.networking.remote)
            } else {
                continue;
            }
        };

        let sock = tUdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();
        info!("Bound toy_refresh sender sock to {}", sock.local_addr().unwrap());
        sock.connect(remote).await.unwrap();
        for (.., mut toy) in toys {

            let b_level: Option<f64> = match toy.device_handle.battery_level().await {
                Ok(battery_lvl) => Some(battery_lvl),
                Err(_e) => {
                    warn!("Failed to get battery for toy: {}", toy.toy_name);
                    None
                },
            };

            toy.battery_level = b_level;

            let _ = app_handle.emit_all("fe_toy_event",
                        FeToyEvent::Update ({
                            FeVCToy {
                                toy_id: Some(toy.toy_id),
                                toy_name: toy.toy_name.clone(),
                                toy_anatomy: toy.config.as_ref().unwrap().anatomy.to_fe(),
                                battery_level: b_level,
                                toy_connected: toy.toy_connected,
                                features: toy.param_feature_map.to_fe(),
                                listening: toy.listening,
                                osc_data: toy.osc_data,
                                sub_id: toy.sub_id,
                            }
                        }),
                    );
            
            if toy.osc_data {

                trace!("Sending OSC data for toy: {}", toy.toy_name);

                let battery_level_msg = encoder::encode(&OscPacket::Message(OscMessage {
                    addr: format!("/avatar/parameters/vibecheck/osc_data/{}/{}/battery", toy.toy_name.replace("Lovense Connect", "lovense").replace(" ", "_").to_lowercase(), toy.sub_id),
                    args: vec![OscType::Float(b_level.unwrap_or(0.0) as f32)]
                })).unwrap();

                let batt_send_err = sock.send(&battery_level_msg).await;
                if batt_send_err.is_err(){warn!("Failed to send battery_level to {}", remote.to_string());}
                else{info!("Sent battery_level: {} to {}", b_level.unwrap_or(0.0) as f32, toy.toy_name);}
            } else {
                trace!("OSC data disabled for toy {}", toy.toy_name);
            }
        }
    }
}