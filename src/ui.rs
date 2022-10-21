//use buttplug::client::device::ClientDeviceMessageAttributesMap;
//use buttplug::client::ButtplugClientDevice;
//use buttplug::core::message::ClientDeviceMessageAttributes;
//use buttplug::core::messages::ButtplugCurrentSpecDeviceMessageType;
use eframe::CreationContext;
use eframe::egui::CollapsingHeader;
//use eframe::egui::style::{WidgetVisuals, Widgets};
use eframe::egui::{
    style::Visuals, Color32, Context, Hyperlink, Layout, RichText, ScrollArea, Style, TextStyle,
    TopBottomPanel,
};
use eframe::epaint::{FontId, FontFamily};

use eframe::{
    egui::{self, CentralPanel},
    App,
};

use std::collections::HashMap;
use std::fs;
//use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
//use std::os::windows::process::CommandExt;
use std::time::{Duration, Instant};
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use crate::config::{load_toy_config, save_toy_config};
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

pub enum VCGUITab {
    Main,
    Config,
    LC,
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

pub enum RunningState<'a> {
    Running,
    Stopped,
    Error(&'a str)
}

pub struct VibeCheckGUI<'a> {
    pub config: VibeCheckConfig,
    pub config_edit: VibeCheckConfig,

    // toy_id, FeatureEditState
    pub toy_editing: HashMap<(u32, VCFeatureType, u32), VCToyFeature>,

    pub battery_synced: bool,
    pub minute_sync: Instant,

    pub tab: VCGUITab,
    pub running: RunningState<'a>,
    pub toys: HashMap<u32, VCToy>,
    //================================================
    // Handlers error recvr
    pub error_rx: Option<Receiver<VCError>>,
    pub error_tx: Option<Sender<VCError>>,
    //================================================
    //
    //================================================
    // Client Event Handler
    pub client_eh_thread: Option<JoinHandle<()>>,
    pub eh_sig_recvr: Option<Receiver<EventSig>>,
    //================================================
    // Toy Management Handler
    pub toy_management_h_thread: Option<JoinHandle<()>>,
    pub tme_recv: Option<Receiver<ToyManagementEvent>>,
    pub tme_send: Option<Sender<ToyManagementEvent>>,
    //================================================
    pub toy_input_h_thread: Option<JoinHandle<()>>,
    //================================================
    pub data_update_inc: u64,
    pub async_rt: Runtime,
    //================================================
    pub update_engine: VibeCheckUpdater,

    pub lovense_connect_toys: HashMap<String, crate::lovense::LovenseConnectToy>,
}

impl VibeCheckGUI<'_> {
    pub fn new(config: VibeCheckConfig, cc: &CreationContext<'_>) -> Self {
        let config_edit = config.clone();

        // Set fonts
        let mut style: Style = (*cc.egui_ctx.style()).clone();
        style.override_text_style = Some(TextStyle::Monospace);
        cc.egui_ctx.set_style(style);
        let mut visuals = Visuals::default();
        visuals.override_text_color = Some(Color32::from_rgb(0xef, 0x98, 0xff));
        visuals.hyperlink_color = Color32::from_rgb(0xef, 0x98, 0xff);

        /*
        let wv = WidgetVisuals {
            bg_fill: Color32::from_rgb(0xef, 0x98, 0xff),
            bg_stroke: Stroke::new(0.1, Color32::WHITE),
            rounding: Rounding::none(),
            fg_stroke: Stroke::new(0.1, Color32::WHITE),
            expansion: 0.1,
        };//Color32::from_rgb(0xef, 0x98, 0xff);

        let widgets = Widgets {
            noninteractive: wv,
            inactive: wv,
            hovered: wv,
            active: wv,
            open: wv,
        };
        */

        //visuals.widgets = widgets;

        cc.egui_ctx.set_visuals(visuals);

        let sys = System::new_all();

        let procs = sys.processes();
        for proc in procs {
            if proc.1.name() == "IntifaceCLI.exe" {
                if !proc.1.kill() {
                    println!("[!] Failed to kill IntifaceCLI. Shutting down..");
                    std::process::exit(0);
                } else {
                    println!("[*] Sent intiface kill.");
                }
            }
        }

        let minute_sync = Instant::now();



        let mut init = Self {
            config,
            config_edit,

            toy_editing: HashMap::new(),

            battery_synced: false,
            minute_sync,
            tab: VCGUITab::Main,
            running: RunningState::Stopped,
            toys: HashMap::new(),
            //======================================
            // Error channels
            error_rx: None,
            error_tx: None,
            //======================================
            //======================================
            // Client Event Handler
            client_eh_thread: None,
            eh_sig_recvr: None,
            //======================================
            // Toy Management Handler
            toy_management_h_thread: None,
            tme_recv: None,
            tme_send: None,
            //======================================
            toy_input_h_thread: None,
            data_update_inc: 0,
            async_rt: Runtime::new().unwrap(),
            //======================================
            update_engine: VibeCheckUpdater::new(),

            lovense_connect_toys: HashMap::new(),
        };

        init.set_error_handling_channels();
        //init.start_intiface_engine();
        //thread::sleep(Duration::from_secs(2));
        init.start_client_event_handler();
        init.start_toy_management_handler();
        init
    }

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
    }

    fn disable_vibecheck(&mut self) {
        self.tme_send
        .as_ref()
        .unwrap()
        .send(ToyManagementEvent::Sig(TmSig::StopListening))
        .unwrap();
        let toys_sd = self.toys.clone();
        for toy in toys_sd {
            self.async_rt.block_on(async move {
                match toy.1.device_handle.stop().await {
                    Ok(_) => println!("[*] Stop command sent: {}", toy.1.toy_name),
                    Err(_e) => println!("[!] Err stopping device: {}", _e),
                }
            });
        }

        self.running = RunningState::Stopped;
    }

    fn enable_vibecheck(&mut self) {
        // Send Start listening signal
        self.tme_send.as_ref().unwrap().send(ToyManagementEvent::Sig(TmSig::StartListening(self.config.networking.clone()))).unwrap();
        
        // Check if listening succeded or not
        match self.tme_recv.as_ref().unwrap().recv() {
            Ok(tme) => {
                match tme {
                    ToyManagementEvent::Sig(sig) => {
                        match sig {
                            TmSig::Listening => {
                                self.running = RunningState::Running;
                            },
                            TmSig::BindError => {
                                println!("[!] Bind Error: Sending shutdown signal!");

                                self.tme_send.as_ref().unwrap().send(ToyManagementEvent::Sig(TmSig::StopListening)).unwrap();
                                self.running = RunningState::Error("Bind Error! Set a different bind port in Settings!");
                            },
                            _ => {},// 
                        }
                    },
                    _ => {},// Got unexpected Sig
                }
            },
            Err(_e) => {},// Recv failed
        }// tme recv
    }

    fn exec_handler(&mut self, ui: &mut egui::Ui) {

        if let RunningState::Stopped = self.running {
            let ed_button = ui.button(RichText::new("Enable").font(FontId::new(15., FontFamily::Monospace)));
                if ed_button.clicked() {
                    self.enable_vibecheck();
                }
        }

        if let RunningState::Running = self.running {
            if ui.button("Disable").clicked() {
                self.disable_vibecheck();
            }
        }

        if let RunningState::Error(err) = self.running {
            let ed_button = ui.button(RichText::new("Enable").font(FontId::new(15., FontFamily::Monospace)));
            if ed_button.clicked() {
                self.enable_vibecheck();
            }
            let err_msg = err.clone();
            ui.label(RichText::new(err_msg).color(Color32::RED));
        }
    }

    // Start CEH
    fn start_client_event_handler(&mut self) {
        // Client Event Handler Channels
        let (client_eh_event_tx, client_eh_event_rx): (Sender<EventSig>, Receiver<EventSig>) =
            mpsc::channel();
        self.eh_sig_recvr = Some(client_eh_event_rx);
        self.client_eh_thread = Some(self.async_rt.spawn(client_event_handler(
            self.error_tx.as_ref().unwrap().clone(),
            client_eh_event_tx,
        )));
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

    fn start_toy_management_handler(&mut self) {
        // Setup channels
        let (tme_recv_tx, tme_recv_rx): (Sender<ToyManagementEvent>, Receiver<ToyManagementEvent>) =
            mpsc::channel();
        let (tme_send_tx, tme_send_rx): (Sender<ToyManagementEvent>, Receiver<ToyManagementEvent>) =
            mpsc::channel();

        // Main thread channels
        self.tme_recv = Some(tme_recv_rx);
        self.tme_send = Some(tme_send_tx);

        

        self.toy_management_h_thread = Some(self.async_rt.spawn(toy_management_handler(
            tme_recv_tx,
            tme_send_rx,
            self.toys.clone(),
            self.config.networking.clone(),
        )));
    }

    /*
    fn stop_toy_management_handler(&mut self) {

        //self.tme_send.as_ref().unwrap().send(ToyManagementEvent::Sig(TmSig::StopListening));

        let tm_th = self.toy_management_h_thread.take().unwrap();

        tm_th.abort();
        let _ = self.async_rt.block_on(async {tm_th.await});

        println!("[*] Toy Management Handler shutdown!");
    }
    */

    fn set_error_handling_channels(&mut self) {
        // Handler error comm channels
        let (error_tx, error_rx): (Sender<VCError>, Receiver<VCError>) = mpsc::channel();
        self.error_rx = Some(error_rx);
        self.error_tx = Some(error_tx);
    }

    fn set_tab(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Toys").clicked() {
                            self.tab = VCGUITab::Main;
                        }
                        ui.separator();

                        if ui.button("Settings").clicked() {
                            self.tab = VCGUITab::Config;
                        }
                        ui.separator();
                        if ui.button("LC Debug").clicked() {
                            self.tab = VCGUITab::LC;
                        }

                        if !self.update_engine.up_to_date {
                            ui.separator();
                            if ui
                                .button(
                                    RichText::new("Update").color(Color32::GREEN).monospace(),
                                )
                                .clicked()
                            {
                                self.update_vibecheck();
                                std::thread::sleep(std::time::Duration::from_secs(5));
                            }
                        }
                    });
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    let dur_secs = self.minute_sync.elapsed().as_secs();
                    ui.label(format!(
                        "Horny for: {:02}:{:02}:{:02}",
                        (dur_secs / 60) / 60,
                        (dur_secs / 60) % 60,
                        dur_secs % 60,
                    ));
                });
            });
        });
    }

    fn gui_header(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("VibeCheck");
        });
        ui.horizontal_wrapped(|ui| {
            ui.with_layout(Layout::right_to_left(), |ui| {
                self.exec_handler(ui);
            });
        });
        ui.separator();
    }

    fn gui_footer(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                ui.add(Hyperlink::from_label_and_url(
                    "Join Discord",
                    "https://discord.gg/g6kUFtMtpw",
                ));
                ui.add(Hyperlink::from_label_and_url(
                    RichText::new("Made by Sutekh")
                        .monospace()
                        .color(Color32::WHITE),
                    "https://github.com/SutekhVRC",
                ));
                ui.label(VERSION);
                ui.add_space(5.0);
            });
        });
    }

    fn refresh_lovense_connect(&mut self) {
        if let Some(status) = crate::lovense::get_toys_from_natp_api() {
            self.lovense_connect_toys = status;
        }
    }
    fn lovense_connect_status(&mut self, ui: &mut egui::Ui) {

        /*
            pub struct LovenseConnectToy {
                nickName: String,
                hVersion: String,
                fVersion: u64,
                name: String,
                id: String,
                battery: u8,
                version: String,
                status: u64,
            }
        */
        ui.horizontal(|ui| {
        ui.heading("Detected Lovense Connect Toys");

            ui.with_layout(Layout::right_to_left(), |ui| {
                if ui.button("Refresh").clicked() {
                    self.refresh_lovense_connect();
                }
            });
        });
        ui.separator();
        
        ScrollArea::new([true, true])
        .id_source("lc_debug")
        .max_height(ui.available_height())
        .auto_shrink([false, false])
        .stick_to_right()
        .show(ui, |ui| {

            for toy in &self.lovense_connect_toys {

                CollapsingHeader::new(RichText::new(format!(
                    "{}({}) [{}]",
                    toy.1.name,
                    toy.1.nickName,
                    toy.0
                )).font(FontId::new(15., FontFamily::Monospace)))
                .show(ui, |ui| {
                    ui.label(format!("Toy: {}", toy.0));
                    ui.label(format!("Name: {}", toy.1.name));
                    ui.label(format!("NickName: {}", toy.1.nickName));
                    ui.label(format!("hVersion: {}", toy.1.hVersion));
                    ui.label(format!("fVersion: {}", toy.1.fVersion));
                    ui.label(format!("ID: {}", toy.1.id));
                    ui.label(format!("Battery: {}", toy.1.battery));
                    ui.label(format!("Version: {}", toy.1.version));
                    ui.label(format!("Status: {}", toy.1.status));
                ui.separator();
                });
            }
        });

    }

    fn chk_valid_config_inputs(&mut self) -> bool {
        if !check_valid_ipv4(&self.config_edit.networking.bind.0) {
            return false;
        }

        if !check_valid_port(&self.config_edit.networking.bind.1) {
            return false;
        }

        true
    }

    fn list_config(&mut self, ui: &mut egui::Ui) {

        ui.horizontal_wrapped(|ui| {
            ui.label("OSC Bind Host: ");
            ui.text_edit_singleline(&mut self.config_edit.networking.bind.0);
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("OSC Bind Port: ");
            ui.text_edit_singleline(&mut self.config_edit.networking.bind.1);
        });
    }

    fn save_config(&mut self) {
        fs::write(
            format!(
                "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck\\Config.json",
                get_user_home_dir()
            ),
            serde_json::to_string(&self.config).unwrap(),
        )
        .unwrap();
    }

    fn list_toys(&mut self, ui: &mut egui::Ui) {

        if self.toys.len() == 0 {
            ui.vertical_centered(|ui| {
                ui.add_space(90.);
                ui.heading("Connect a toy.. Please ;-;");
            });
            return;
        }
        for toy in &mut self.toys {

            let features = toy.1.param_feature_map.features.clone();
            
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

            //                ui.separator();

            //});
            ui.add_space(1.5);
        }
    }

    fn update_toys(&mut self) {
        if let Some(ref tu_channel) = self.eh_sig_recvr {
            match tu_channel.try_recv() {
                Ok(tu) => {
                    match tu {
                        EventSig::ToyAdd(mut toy) => {
                            // Load toy config for name of toy if it exists otherwise create the config for the toy name

                            // Load config with toy name
                            if let Some(fmap) = load_toy_config(&toy.toy_name) {
                                toy.populate_toy_feature_param_map(Some(fmap));
                            } else {
                                toy.populate_toy_feature_param_map(None);
                            }
                            //println!("[TOY FEATURES]\n{:?}", toy.param_feature_map);
                            self.tme_send
                                .as_ref()
                                .unwrap()
                                .send(ToyManagementEvent::Tu(ToyUpdate::AddToy(toy.clone())))
                                .unwrap();
                            // Load toy config for name of toy if it exists otherwise create the config for the toy name
                            self.toys.insert(toy.toy_id, toy.clone());
                            //println!("[+] Toy added: {} | {}", toy.toy_name, toy.toy_id);
                        }
                        EventSig::ToyRemove(id) => {
                            self.tme_send
                                .as_ref()
                                .unwrap()
                                .send(ToyManagementEvent::Tu(ToyUpdate::RemoveToy(id)))
                                .unwrap();
                            self.toys.remove(&id);
                            //println!("[!] Removed toy: {}", id);
                        }
                        EventSig::Shutdown => {}
                    }
                }
                Err(_e) => {}
            }
        }
    }

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
    }
}

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
