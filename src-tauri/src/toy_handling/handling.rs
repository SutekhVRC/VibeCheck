use buttplug::client::ButtplugClientDevice;
use buttplug::client::ButtplugClientEvent;
use buttplug::core::message::ActuatorType;
use buttplug::client::ScalarCommand::ScalarMap;
use buttplug::client::RotateCommand::RotateMap;
use futures::StreamExt;
use futures_timer::Delay;
use log::debug;
use parking_lot::Mutex;

use tauri::AppHandle;
use tauri::Manager;

use tokio::sync::mpsc::UnboundedReceiver;
use std::collections::HashMap;

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
use crate::frontend::frontend_types::FeCoreEvent;
use crate::frontend::frontend_types::FeToyEvent;
use crate::frontend::frontend_types::FeVCToy;
use crate::frontend::frontend_types::FeScanEvent;

use crate::osc::logic::toy_input_routine;
use crate::toy_handling::ToySig;
use crate::toy_handling::toy_manager::ToyManager;
use crate::toy_handling::toyops::LevelTweaks;
use crate::toy_handling::toyops::VCFeatureType;
use crate::toy_handling::toyops::{VCToy, VCToyFeatures};
use crate::vcore::core::ToyManagementEvent;
use crate::vcore::core::VibeCheckState;
use crate::{vcore::core::TmSig, vcore::core::ToyUpdate, vcore::core::VCError};
use tokio::sync::mpsc::UnboundedSender;
use tauri::api::notification::Notification;
use log::{error as logerr, warn, info, trace};

use super::RateParser;
use super::SmoothParser;


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

                    let mut battery_level: Option<f64> = None;
                    
                    // Can use this to differ between toys with batteries and toys without!
                    if dev.has_battery_level() {
                        battery_level = 
                        match dev.battery_level().await {
                            Ok(battery_lvl) => Some(battery_lvl),
                            Err(_e) => {
                                warn!("Device battery_level error: {:?}", _e);
                                Some(0.0)
                            },
                        };
                    }

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
                        parsed_toy_features: VCToyFeatures::new(),
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

                    if let None = toy.config { // First time toy load
                        toy.populate_toy_config();
                        let mut vc_lock = vibecheck_state_pointer.lock();
                        vc_lock.core_toy_manager.as_mut().unwrap().populate_configs();
                    } else {
                        toy.populate_toy_config();
                    }
                    
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
                                features: toy.parsed_toy_features.to_fe(),
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
#[inline]
pub async fn scalar_parse_levels_send_toy_cmd(dev: &Arc<ButtplugClientDevice>, scalar_level: f64, feature_index: u32, actuator_type: ActuatorType, flip_float: bool, feature_levels: LevelTweaks) {

    let new_level = clamp_and_flip(scalar_level, flip_float, feature_levels);
    #[cfg(debug_assertions)] {
        let message_prefix = if scalar_level == 0.0 { "IDLE" } else { "SENDING" };
        info!("{} FI[{}] AT[{}] SL[{}]", message_prefix, feature_index, actuator_type, new_level);
    }
    match dev.scalar(&ScalarMap(HashMap::from([(feature_index, (new_level, actuator_type))]))).await {
        Ok(()) => {},
        Err(e) => {
            logerr!("Send scalar to device error: {}", e);
        }
    }

}

#[inline]
fn clamp_and_flip(value: f64, flip: bool, levels: LevelTweaks) -> f64 {

    let mut new_value;
    if value == 0.0 {
        new_value  = levels.idle_level;
    } else {
        new_value  = value.clamp(levels.minimum_level, levels.maximum_level);
    }
    if flip {
        new_value = flip_float64(new_value)
    }
    return new_value

}

#[inline]
pub fn flip_float64(orig: f64) -> f64 {
    //1.00 - orig
    ((1.00 - orig) * 100.0).round() / 100.0
}



#[inline(always)]
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



#[inline(always)]
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
        if instant.elapsed().as_secs_f64() >= 0.15 {

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
             mut vc_toy_features: VCToyFeatures| {
        // Read toy config here?
        async move {

            // Put smooth_queue here
            // Put rate tracking here
            // Time tracking here?
            // Async runtime wrapped in Option for rate updating here????

            // Lock this to a user-set HZ value
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

                                /*
                                 * Get feature objects that correspond to parameter.
                                 * enum(Feature) ?
                                 */

                                if let Some(features) =
                                    vc_toy_features.get_features_from_param(&msg.addr)
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
                                            vc_toy_features = new_toy.parsed_toy_features;
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
                    toy.1.parsed_toy_features.clone(),
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
                                            toy.parsed_toy_features.clone(),
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

            let new_level = clamp_and_flip(float_level, flip_float, feature_levels);
            let _ = dev.rotate(&RotateMap(HashMap::from([(feature_index, (new_level, true))]))).await;

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

            let new_level = clamp_and_flip(float_level, flip_float, feature_levels);
            let _ = dev.linear(&buttplug::client::LinearCommand::LinearMap(HashMap::from([(feature_index, (feature_levels.linear_position_speed, new_level))]))).await;

        }
        VCFeatureType::ScalarRotator => {
            scalar_parse_levels_send_toy_cmd(&dev, float_level, feature_index, ActuatorType::Rotate, flip_float, feature_levels).await;
        },
    }
}

