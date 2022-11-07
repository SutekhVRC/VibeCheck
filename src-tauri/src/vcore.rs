
//use buttplug::client::device::ClientDeviceMessageAttributesMap;
//use buttplug::client::ButtplugClientDevice;
//use buttplug::core::message::ClientDeviceMessageAttributes;
//use buttplug::core::messages::ButtplugCurrentSpecDeviceMessageType;


use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
//use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
//use std::os::windowsess::CommandExt;
use std::time::{Duration, Instant};
use buttplug::client::ButtplugClient;
use buttplug::util::in_process_client;
use futures_util::__private::async_await;
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use parking_lot::{RwLock, Mutex, RawMutex};
use parking_lot::MutexGuard;
use crate::config::{load_toy_config, save_toy_config};
use crate::handling::HandlerErr;
use crate::toyops::{alter_toy, VCFeatureType, VCToyFeature};
use crate::vcupdate::{VibeCheckUpdater, VERSION};
use crate::{
    util::{
        check_valid_ipv4,
        check_valid_port,
        get_user_home_dir
    },
    handling::{EventSig, client_event_handler, toy_management_handler},
    config::{
        VibeCheckConfig,
        OSCNetworking,
    },
    toyops::{
        VCToy,
    },
};

fn debug_out(s: &'static str) {
    println!("{}", s);
}

pub struct VCStateMutex(pub Mutex<VibeCheckState>);

pub struct VibeCheckState {

    pub config: VibeCheckConfig,

    pub bp_client: ButtplugClient,

    pub running: RunningState,
    pub toys: HashMap<u32, VCToy>,
    //================================================
    // Handlers error recvr
    //inner_channels: Arc<RwLock<innerChannels>>,
    pub error_rx: Receiver<VCError>,
    pub error_tx: Sender<VCError>,
    //================================================
    //
    //================================================
    // Client Event Handler
    pub client_eh_thread: JoinHandle<()>,
    pub client_eh_event_rx: Receiver<EventSig>,
    //================================================
    // Toy Management Handler
    pub toy_management_h_thread: JoinHandle<()>,
    pub tme_recv: Receiver<ToyManagementEvent>,
    pub tme_send: Sender<ToyManagementEvent>,
    //================================================
    //pub toy_input_h_thread: Option<JoinHandle<()>>,
    //================================================
    // Async Runtime for toy event handlers
    pub async_rt: Runtime,
    //================================================
    pub update_engine: VibeCheckUpdater,

    pub lovense_connect_toys: HashMap<String, crate::lovense::LovenseConnectToy>,
}

impl VibeCheckState {

    pub fn new(config: VibeCheckConfig) -> Self {

        // Toys hashmap
        let toys = HashMap::new();

        // Create error handling/passig channels
        let (error_tx, error_rx): (Sender<VCError>, Receiver<VCError>) = mpsc::channel();

        // Create async runtime for toy handling routines
        let async_rt = Runtime::new().unwrap();

        let bp_client = async_rt.block_on(async move {in_process_client("VibeCheck", false).await});
        println!("[*] Connected to process");
        let event_stream = bp_client.event_stream();

        // Client Event Handler Channels
        let (client_eh_event_tx, client_eh_event_rx): (Sender<EventSig>, Receiver<EventSig>) = mpsc::channel();
        let client_eh_thread = async_rt.spawn(client_event_handler(event_stream, error_tx.clone(), client_eh_event_tx));

        // Setup channels
        let (tme_recv_tx, tme_recv_rx): (Sender<ToyManagementEvent>, Receiver<ToyManagementEvent>) = mpsc::channel();
        let (tme_send_tx, tme_send_rx): (Sender<ToyManagementEvent>, Receiver<ToyManagementEvent>) = mpsc::channel();

        // Main thread toy management event bidirectional channels
        let tme_recv = tme_recv_rx;
        let tme_send = tme_send_tx;

        // Start toy management thread
        let toy_management_h_thread = async_rt.spawn(toy_management_handler(
            tme_recv_tx,
            tme_send_rx,
            toys.clone(),
            config.networking.clone(),
        ));

        // Timer prob remove idrc
        //let minute_sync = Instant::now();

        Self {
            config,

            bp_client,

            running: RunningState::Stopped,
            toys,

            //======================================
            // Error channels
            error_rx,
            error_tx,

            //======================================
            // Client Event Handler
            client_eh_thread,
            client_eh_event_rx,

            //======================================
            // Toy Management Handler
            toy_management_h_thread,
            tme_recv,
            tme_send,

            //======================================
            // Async runtime
            async_rt,

            //======================================
            // Update engine
            update_engine: VibeCheckUpdater::new(),

            //======================================
            // Lovense Connect toys
            lovense_connect_toys: HashMap::new(),
        }
    }
}


#[derive(Clone, Debug)]
pub enum ToyUpdate {
    AlterToy(VCToy),
    RemoveToy(u32),
    AddToy(VCToy),
}

pub enum TmSig {
    StopListening,
    StartListening(OSCNetworking),
    /*
    Running,
    Stopped,
    */
    Listening,
    BindError,
}

pub enum ToyManagementEvent {
    Tu(ToyUpdate),
    Sig(TmSig),
}

pub enum VCError {
    HandlingErr(crate::handling::HandlerErr),
}

pub enum RunningState {
    Running,
    Stopped,
    Error(String)
}

/*
fn update_vibecheck(&mut self) {
    if let RunningState::Running = self.running {
        self.disable_vibecheck();
    }
    let blob = self.update_engine.release_blob.take().unwrap();

    thread::spawn(move || {
        VibeCheckUpdater::update_vibecheck(blob);
    });
    thread::sleep(std::time::Duration::from_secs(1));
    std::process::exit(0);
}*/

pub async fn native_vibecheck_disable(vc_state: tauri::State<'_, VCStateMutex>) -> Result<(), &'static str> {

    let mut vc_lock = vc_state.0.lock();

    vc_lock.tme_send
    .send(ToyManagementEvent::Sig(TmSig::StopListening))
    .unwrap();

    let _ = vc_lock.bp_client.stop_all_devices().await;
    vc_lock.running = RunningState::Stopped;

    Ok(())
}

pub fn native_vibecheck_enable(vc_state: tauri::State<'_, VCStateMutex>) -> Result<(), &'static str> {
    // Send Start listening signal

    let mut vc_lock = vc_state.0.lock();
    vc_lock.tme_send.send(ToyManagementEvent::Sig(TmSig::StartListening(vc_lock.config.networking.clone()))).unwrap();
    
    // Check if listening succeded or not
    match vc_lock.tme_recv.recv() {
        Ok(tme) => {
            match tme {
                ToyManagementEvent::Sig(sig) => {
                    match sig {
                        TmSig::Listening => {
                            vc_lock.running = RunningState::Running;
                            Ok(())
                        },
                        TmSig::BindError => {

                            println!("[!] Bind Error: Sending shutdown signal!");

                            vc_lock.tme_send.send(ToyManagementEvent::Sig(TmSig::StopListening)).unwrap();
                            vc_lock.running = RunningState::Error("Bind Error! Set a different bind port in Settings!".to_string());
                            
                            debug_out("Bind Error in TME sig.");
                            return Err("Bind Error! Set a different bind port in Settings!");
                        },
                        _ => {//Did not get the correct signal oops
                            debug_out("Got incorrect TME signal.");
                            Err("Got incorrect TME signal.")
                        }, 
                    }
                },
                _ => {
                    debug_out("Got ToyUpdate in vc_enable().");
                    Err("Got ToyUpdate in vc_enable().")
                },// Got unexpected Sig
            }
        },
        Err(_e) => {
            debug_out("Failed to recv from TME receiver.");
            Err("Failed to recv from TME receiver.")
        },// Recv failed
    }// tme recv
}

pub async fn native_vibecheck_start_bt_scan(vc_state: tauri::State<'_, VCStateMutex>) {
    let vc_lock = vc_state.0.lock();
    // Start scanning for toys
    if let Err(e) = vc_lock.bp_client.start_scanning().await {
        let _ = vc_lock.error_tx.send(VCError::HandlingErr(HandlerErr {
            id: -2,
            msg: format!("Failed to scan for bluetooth devices. {}", e),
        }));
        println!("Failed to scan.");
        return;
    }
}

pub async fn native_vibecheck_stop_bt_scan(vc_state: tauri::State<'_, VCStateMutex>) {
    let vc_lock = vc_state.0.lock();
    // Start scanning for toys
    if let Err(e) = vc_lock.bp_client.stop_scanning().await {
        let _ = vc_lock.error_tx.send(VCError::HandlingErr(HandlerErr {
            id: -2,
            msg: format!("Failed to stop scan for bluetooth devices. {}", e),
        }));
        println!("Failed to stop scan.");
        return;
    }
}

/*
fn stop_client_event_handler(&mut self) {
    let th = match self.client_eh_thread.take() {
        Some(th) => th,
        None => return,
    };
    th.abort();
    let _ = self.async_rt.block_on(async {th.await});
    println!("[!] Aborted client event handler!");
}
*/



/*
fn stop_toy_management_handler(&mut self) {

    //self.tme_send.as_ref().unwrap().send(ToyManagementEvent::Sig(TmSig::StopListening));

    let tm_th = self.toy_management_h_thread.take().unwrap();

    tm_th.abort();
    let _ = self.async_rt.block_on(async {tm_th.await});

    println!("[*] Toy Management Handler shutdown!");
}
*/

fn refresh_lovense_connect(mut vc_lock: MutexGuard<VibeCheckState>) {
    if let Some(status) = crate::lovense::get_toys_from_natp_api() {
        vc_lock.lovense_connect_toys = status;
    }
}

/*
fn chk_valid_config_inputs(&mut self) -> bool {
    if !check_valid_ipv4(&self.config_edit.networking.bind.0) {
        return false;
    }

    if !check_valid_port(&self.config_edit.networking.bind.1) {
        return false;
    }

    true
}*/

pub fn native_get_vibecheck_config(vc_state: tauri::State<'_, VCStateMutex>) -> HashMap<&str, String> {

    let config = {
        let vc_lock = vc_state.0.lock();
        vc_lock.config.clone()
    };

    let mut map = HashMap::new();
    map.insert("host", config.networking.bind.0);
    map.insert("port", config.networking.bind.1);

    return map;
}
/*
fn save_config(&mut self) {
    fs::write(
        format!(
            "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck\\Config.json",
            get_user_home_dir()
        ),
        serde_json::to_string(&self.config).unwrap(),
    )
    .unwrap();
}*/

/*
fn list_toys(&mut self) {

    if self.toys.len() == 0 {
        /*
        ui.vertical_centered(|ui| {
            ui.add_space(90.);
            ui.heading("Connect a toy.. Please ;-;");
        });
        */
        return;
    }
    for toy in &mut self.toys {

        let features = toy.1.param_feature_map.features.clone();
        /*
        ui.horizontal_wrapped(|ui| {
            CollapsingHeader::new(RichText::new(format!(
                "{} [{}%]",
                toy.1.toy_name,
                (toy.1.battery_level * 100.).round()
            )).font(FontId::new(15., FontFamily::Monospace)))
            .show(ui, |ui| {

                for i in 0..features.len() {//Iterate through all features of toy
                    if self.toy_editing.contains_key(&(*toy.0, features[i].feature_type, features[i].feature_index)) {// Editing

                        let mut saved = false;
                        
                        ui.group(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            let fref = self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap();
                            
                            ui.vertical(|ui| {
                                ui.label(format!("{:?} - {}", fref.feature_type, features[i].feature_index));
                                ui.separator()
                            });

                            ui.with_layout(Layout::right_to_left(), |ui| {

                                if ui.button("Save").clicked() {
                                    // Saved
                                    self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().saved = true;
                                    // Set features to toy
                                    toy.1.param_feature_map.features[i] = self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().to_owned();
                                    // Take feature out of edit mode
                                    self.toy_editing.remove(&(*toy.0, features[i].feature_type, features[i].feature_index));
                                    
                                    // Send update toy message
                                    alter_toy(
                                        self.tme_send.as_ref().unwrap(),
                                        toy.1.clone(),
                                    );
                                    save_toy_config(
                                        &toy.1.toy_name,
                                        toy.1.param_feature_map.clone(),
                                    );
                                    saved = true;// Stop editing routine
                                }
                            });
                        });

                        if saved {
                            return;
                        }

                        let mut button_color = Color32::GREEN;
                        let mut button_text = "Enabled";
                        if !self.toy_editing.get(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_enabled {
                            button_color = Color32::RED;
                            button_text = "Disabled";
                        }

                        ui.checkbox(&mut self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_enabled, RichText::new(button_text).color(button_color));
                        ui.horizontal_wrapped(|ui| {
                            ui.label("OSC Parameter:");ui.text_edit_singleline(&mut self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().osc_parameter);
                        });
                        ui.checkbox(&mut self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().smooth_enabled, "Smoothing");
                        if self.toy_editing.get(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().smooth_enabled {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Smoothing Rate:"); ui.add(egui::Slider::new(&mut self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.smooth_rate, 1.0..=20.0)
                                .fixed_decimals(2));
                            });  
                        }
                        
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Idle:   "); ui.add(egui::Slider::new(&mut self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.idle_level, 0.0..=1.0)
                            .fixed_decimals(2));
                        });

                        ui.horizontal_wrapped(|ui| {
                            ui.label("Minimum:"); ui.add(egui::Slider::new(&mut self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.minimum_level, 0.0..=1.0)
                            .fixed_decimals(2));
                        });

                        // Minimum cant be more than maximum
                        if self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.minimum_level > self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.maximum_level {
                            self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.minimum_level = self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.maximum_level-0.01;
                        }
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Maximum:"); ui.add(egui::Slider::new(&mut self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.maximum_level, 0.0..=1.0).fixed_decimals(2));
                        });

                        // Maximum cant be less than minimum
                        if self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.maximum_level < self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.minimum_level {
                            self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.maximum_level = self.toy_editing.get_mut(&(*toy.0, features[i].feature_type, features[i].feature_index)).unwrap().feature_levels.minimum_level+0.01;
                        }

                    });
                    } else {// Saved
                        ui.group(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(format!("{:?}[{}]: {}", toy.1.param_feature_map.features[i].feature_type, toy.1.param_feature_map.features[i].feature_index, toy.1.param_feature_map.features[i].osc_parameter));
                            ui.with_layout(Layout::right_to_left(), |ui| {
                                if ui.button("Edit").clicked() {
                                    toy.1.param_feature_map.features[i].saved = false;
                                    self.toy_editing.insert((*toy.0, features[i].feature_type, features[i].feature_index), toy.1.param_feature_map.features[i].clone());
                                }
                                if toy.1.param_feature_map.features[i].feature_enabled {
                                    ui.colored_label(Color32::GREEN, "Enabled");
                                } else {
                                    ui.colored_label(Color32::RED, "Disabled");
                                }
                            });
                        });
                        });
                    }
                }

                    /*
                    if !self.toy_editing.contains_key(&toy.0) {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Features").font(FontId::new(14., FontFamily::Monospace)));
                            ui.with_layout(Layout::right_to_left(), |ui| {
                                if ui.button("Edit").clicked() {
                                    self.toy_editing.insert(*toy.0, *toy.0);
                                    return;
                                }
                            });
                        });
                        ui.separator();
                        // List toy features when not editing
                        for feature in toy.1.param_feature_map.features {
                            
                        }
                    } else {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Features"));
                            ui.with_layout(Layout::right_to_left(), |ui| {
                                if ui.button("Save").clicked() {
                                    if let Some(_) = self.toy_editing.remove(&toy.0) {
                                        alter_toy(
                                            self.tme_send.as_ref().unwrap(),
                                            toy.1.clone(),
                                        );
                                        save_toy_config(
                                            &toy.1.toy_name,
                                            toy.1.param_feature_map.clone(),
                                        );
                                        return;
                                    }
                                }
                            });
                        });
                        ui.separator();

                        // Edit parameters

                        
                    }*/
                //});
            });
        });
        */
        //                ui.separator();

        //});
        //ui.add_space(1.5);
    }
}*/

pub fn message_handling(mut vc_lock: MutexGuard<VibeCheckState>) {

    // Update Toys States
    match vc_lock.client_eh_event_rx.try_recv() {
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
                    vc_lock.tme_send
                        .send(ToyManagementEvent::Tu(ToyUpdate::AddToy(toy.clone())))
                        .unwrap();
                    // Load toy config for name of toy if it exists otherwise create the config for the toy name
                    vc_lock.toys.insert(toy.toy_id, toy.clone());
                    //println!("[+] Toy added: {} | {}", toy.toy_name, toy.toy_id);
                }
                EventSig::ToyRemove(id) => {
                    vc_lock.tme_send
                        .send(ToyManagementEvent::Tu(ToyUpdate::RemoveToy(id)))
                        .unwrap();
                    vc_lock.toys.remove(&id);
                    //println!("[!] Removed toy: {}", id);
                }
                EventSig::Shutdown => {}
            }
        }
        Err(_e) => {}
    }
}

/*
fn update_battery_percentages(&mut self) {
    for toy in &mut self.toys {
        if toy.1.device_handle.connected() {
            let f = toy.1.device_handle.battery_level();
            toy.1.battery_level = match self
                .async_rt
                .block_on(async { tokio::time::timeout(Duration::from_millis(500), f).await })
            {
                Ok(battery) => {
                    if let Ok(b) = battery {
                        b
                    } else {
                        println!(
                            "[!] Failed to get battery! Cancel toy call for {}.",
                            toy.1.toy_name
                        );
                        continue;
                    }
                }
                Err(_e) => {
                    println!(
                        "[!] Failed to get battery! Cancel toy call for {}.",
                        toy.1.toy_name
                    );
                    continue;
                }
            }
        }
    }
}*/
/*
 * Update Battery For Toys
 * Update Toy States
 * 
 */

/*
impl<'a> App for VibeCheckGUI<'a> {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dur = self.minute_sync.elapsed();
        if (dur.as_secs() % 120 == 0) && !self.battery_synced {
            self.update_battery_percentages();
            self.battery_synced = true;
        } else if self.battery_synced && (dur.as_secs() % 2 == 1) {
            self.battery_synced = false;
        }

        if self.data_update_inc == 120 {
            self.data_update_inc = 0;
        } else {
            self.data_update_inc += 1;
            self.update_toys();
        }

        self.set_tab(&ctx);
        CentralPanel::default().show(ctx, |ui| {
            ctx.request_repaint();

            self.gui_header(ui);

            match self.tab {
                VCGUITab::Main => {

                    ScrollArea::new([false, true]).show(ui, |ui| {
                        self.list_toys(ui);
                        ui.add_space(60.);
                    });
                }
                VCGUITab::Config => {
                    ui.horizontal_wrapped(|ui| {
                        ui.label("VibeCheck Settings");
                        ui.with_layout(Layout::right_to_left(), |ui| {
                            if ui.button("Save").clicked() {
                                if self.chk_valid_config_inputs() {
                                    println!("[!] Valid config inputs!");
                                    self.config = self.config_edit.clone();
                                    self.save_config();
                                } else {
                                    println!("[!] Invalid config inputs!");
                                    self.config_edit = self.config.clone();
                                }
                            }
                        });
                    });
                    ui.separator();
                    self.list_config(ui);
                },
                VCGUITab::LC => {
                    self.lovense_connect_status(ui);
                }
            }
            self.gui_footer(ctx);
        });
    }

    fn on_exit(&mut self, _ctx: &eframe::glow::Context) {

        let toys_sd = self.toys.clone();
        for toy in toys_sd {
            self.async_rt.block_on(async move {
                match toy.1.device_handle.stop().await {
                    Ok(_) => println!("[*] Stop command sent: {}", toy.1.toy_name),
                    Err(_e) => println!("[!] Err stopping device: {}", _e),
                }
            });
        }

        //self.stop_intiface_engine();
        self.save_config();
        std::process::exit(0);
    }
}
*/