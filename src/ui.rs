use buttplug::client::device::ClientDeviceMessageAttributesMap;
use buttplug::client::ButtplugClientDevice;
use buttplug::core::messages::ButtplugCurrentSpecDeviceMessageType;
use eframe::egui::CollapsingHeader;
//use eframe::egui::style::{WidgetVisuals, Widgets};
use eframe::egui::{
    style::Visuals, Color32, Context, Hyperlink, Layout, RichText, ScrollArea, Style, TextStyle,
    TopBottomPanel,
};
//use eframe::epaint::{Stroke, Rounding};
use core::fmt;
use eframe::{
    egui::{self, CentralPanel},
    epi::App,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::os::windows::process::CommandExt;
use std::time::{Duration, Instant};
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use crate::file_exists;
use crate::{
    check_valid_ipv4, check_valid_port, get_user_home_dir,
    handling::EventSig,
    handling::{client_event_handler, toy_management_handler},
    VibeCheckConfig,
};

pub enum VCGUITab {
    Main,
    Config,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub struct LevelTweaks {
    pub minimum_level: f64,
    pub maximum_level: f64,
    pub idle_level: f64,
}

impl Default for LevelTweaks {
    fn default() -> Self {
        LevelTweaks { minimum_level: 0., maximum_level: 100., idle_level: 0. }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Vibrators {
    Auto(String, LevelTweaks),
    Custom(Vec<(String, u32, LevelTweaks)>),
}

impl ToyFeatureTrait for Vibrators {
    fn get_param_mode_str(&self) -> String {
        match self {
            Self::Auto(..) => "Auto".to_string(),
            Self::Custom(_) => "Custom".to_string(),
        }
    }

    fn get_auto_mut_ref(&mut self) -> Option<&mut String> {
        match self {
            Self::Auto(ref mut p, _) => Some(p),
            _ => None,
        }
    }

    fn get_custom_mut_refs(&mut self) -> Option<Vec<(&mut String, u32, LevelTweaks)>> {
        match self {
            Self::Custom(cmap) => {
                let mut ret_vec = vec![];

                for map in cmap {
                    ret_vec.push((&mut map.0, map.1, map.2));
                }
                if ret_vec.is_empty() {
                    None
                } else {
                    Some(ret_vec)
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Rotators {
    Auto(String, LevelTweaks),
    Custom(Vec<(String, u32, LevelTweaks)>),
}

impl ToyFeatureTrait for Rotators {
    fn get_param_mode_str(&self) -> String {
        match self {
            Self::Auto(..) => "Auto".to_string(),
            Self::Custom(_) => "Custom".to_string(),
        }
    }

    fn get_auto_mut_ref(&mut self) -> Option<&mut String> {
        match self {
            Self::Auto(ref mut p, _) => Some(p),
            _ => None,
        }
    }

    fn get_custom_mut_refs(&mut self) -> Option<Vec<(&mut String, u32, LevelTweaks)>> {
        match self {
            Self::Custom(cmap) => {
                let mut ret_vec = vec![];

                for map in cmap {
                    ret_vec.push((&mut map.0, map.1, map.2));
                }
                if ret_vec.is_empty() {
                    None
                } else {
                    Some(ret_vec)
                }
            }
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Linears {
    Auto(String, LevelTweaks),
    Custom(Vec<(String, u32, LevelTweaks)>),
}

impl ToyFeatureTrait for Linears {
    fn get_param_mode_str(&self) -> String {
        match self {
            Self::Auto(..) => "Auto".to_string(),
            Self::Custom(_) => "Custom".to_string(),
        }
    }

    fn get_auto_mut_ref(&mut self) -> Option<&mut String> {
        match self {
            Self::Auto(ref mut p, _) => Some(p),
            _ => None,
        }
    }

    fn get_custom_mut_refs(&mut self) -> Option<Vec<(&mut String, u32, LevelTweaks)>> {
        match self {
            Self::Custom(cmap) => {
                let mut ret_vec = vec![];

                for map in cmap {
                    ret_vec.push((&mut map.0, map.1, map.2));
                }
                if ret_vec.is_empty() {
                    None
                } else {
                    Some(ret_vec)
                }
            }
            _ => None,
        }
    }
}

trait ToyFeatureTrait {
    fn get_param_mode_str(&self) -> String;
    fn get_auto_mut_ref(&mut self) -> Option<&mut String>;
    fn get_custom_mut_refs(&mut self) -> Option<Vec<(&mut String, u32, LevelTweaks)>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureParamMap {
    // Vibrators
    pub v: Option<Vibrators>,
    pub v_custom: Vec<(String, u32, LevelTweaks)>,
    pub v_auto: (String, LevelTweaks),
    // Rotators
    pub r: Option<Rotators>,
    pub r_custom: Vec<(String, u32, LevelTweaks)>,
    pub r_auto: (String, LevelTweaks),
    // Linears
    pub l: Option<Linears>,
    pub l_custom: Vec<(String, u32, LevelTweaks)>,
    pub l_auto: (String, LevelTweaks),
}

impl fmt::Display for FeatureParamMap {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(v) = &self.v {
            match v {
                Vibrators::Auto(p, _) => {
                    write!(f, "Vibrators | Auto: {}", p);
                }
                Vibrators::Custom(cmap) => {
                    let mut base_str = String::from("Vibrators | Custom");
                    for map in cmap {
                        base_str.push_str(format!("\nVibrator ({}): {}", map.1, map.0).as_str());
                    }
                    write!(f, "{}", base_str);
                }
            }
        }

        if let Some(r) = &self.r {
            write!(f, "\n");
            match r {
                Rotators::Auto(p, _) => {
                    write!(f, "Rotators  | Auto: {}", p);
                }
                Rotators::Custom(cmap) => {
                    let mut base_str = String::from("Rotators  | Custom");
                    for map in cmap {
                        base_str.push_str(format!("\nRotator  ({}): {}", map.1, map.0).as_str());
                    }
                    write!(f, "{}", base_str);
                }
            }
        }

        if let Some(l) = &self.l {
            write!(f, "\n");
            match l {
                Linears::Auto(p, _) => {
                    write!(f, "Linears  | Auto: {}", p);
                }
                Linears::Custom(cmap) => {
                    let mut base_str = String::from("Linears  | Custom");
                    for map in cmap {
                        base_str.push_str(format!("\nLinear  ({}): {}", map.1, map.0).as_str());
                    }
                    write!(f, "{}", base_str);
                }
            }
        }
        write!(f, "")
    }
}

impl FeatureParamMap {
    pub fn new() -> Self {
        FeatureParamMap {
            v: None,
            v_custom: vec![],
            v_auto: (String::new(), LevelTweaks::default()),
            r: None,
            r_custom: vec![],
            r_auto: (String::new(), LevelTweaks::default()),
            l: None,
            l_custom: vec![],
            l_auto: (String::new(), LevelTweaks::default()),
        }
    }

    pub fn get_features_from_param(&self, param: &String) -> Option<Vec<ToyFeature>> {
        let mut features = vec![];
        if let Some(v) = &self.v {
            match v {
                Vibrators::Auto(p, l) => {
                    if *p == *param {
                        features.push(ToyFeature::Vibrator(FeatureMode::Auto(l.clone())));
                    }
                }
                Vibrators::Custom(cmap) => {
                    for map in cmap {
                        if map.0 == *param {
                            features.push(ToyFeature::Vibrator(FeatureMode::Custom(map.1, map.2)));
                        }
                    }
                }
            }
        }

        if let Some(r) = &self.r {
            match r {
                Rotators::Auto(p, l) => {
                    if *p == *param {
                        features.push(ToyFeature::Rotator(FeatureMode::Auto(l.clone())));
                    }
                }
                Rotators::Custom(cmap) => {
                    for map in cmap {
                        if map.0 == *param {
                            features.push(ToyFeature::Rotator(FeatureMode::Custom(map.1, map.2)));
                        }
                    }
                }
            }
        }

        if let Some(l) = &self.l {
            match l {
                Linears::Auto(p, l) => {
                    if *p == *param {
                        features.push(ToyFeature::Linear(FeatureMode::Auto(l.clone())));
                    }
                }
                Linears::Custom(cmap) => {
                    for map in cmap {
                        if map.0 == *param {
                            features.push(ToyFeature::Linear(FeatureMode::Custom(map.1, map.2)));
                        }
                    }
                }
            }
        }

        if features.is_empty() {
            return None;
        } else {
            return Some(features);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ToyFeature {
    /*
        FEATURE(INDEX)
    */
    // Vibrator (vibrator index)
    Vibrator(FeatureMode),
    // Rotator (index)
    Rotator(FeatureMode),
    // Linear (index)
    Linear(FeatureMode),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeatureMode {
    Auto(LevelTweaks),
    Custom(u32, LevelTweaks),
}

/*
impl fmt::Display for ToyFeature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ToyFeature::Vibrator(id) => write!(f, "Vibrator ({})", id),
            ToyFeature::Rotator(id) => write!(f, "Rotator ({})", id),
            ToyFeature::Linear(id) => write!(f, "Mode: {}, Linear ({})", ),
        }
    }
}*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToyMode {
    Auto(String),
    Custom,
}

#[derive(Clone, Debug)]
pub struct VCToy {
    pub toy_id: u32,
    pub toy_name: String,
    pub battery_level: f64,
    pub toy_connected: bool,
    pub toy_features: ClientDeviceMessageAttributesMap,
    pub osc_params_list: Vec<String>,
    pub toy_param_mode: ToyMode,
    pub param_feature_map: FeatureParamMap,
    pub listening: bool,
    pub device_handle: Arc<ButtplugClientDevice>,
}

#[derive(Clone, Debug)]
pub enum ToyUpdate {
    AlterToy(VCToy),
    RemoveToy(u32),
    AddToy(VCToy),
}

pub enum TmSig {
    StopListening,
    StartListening,
}

pub enum ToyManagementEvent {
    Tu(ToyUpdate),
    Sig(TmSig),
}

pub enum VCError {
    HandlingErr(crate::handling::HandlerErr),
}

pub struct VibeCheckGUI {
    pub config: VibeCheckConfig,
    pub config_edit: VibeCheckConfig,

    pub editing: HashMap<u32, u32>,

    pub battery_synced: bool,
    pub minute_sync: Instant,

    pub tab: VCGUITab,
    pub running: bool,
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
    //
    pub data_update_inc: u64,
    pub async_rt: Runtime,
    pub intiface_child_proc_h: Option<Child>,
}

impl VibeCheckGUI {
    pub fn new(config: VibeCheckConfig) -> Self {
        let config_edit = config.clone();

        Self {
            config,
            config_edit,

            editing: HashMap::new(),

            battery_synced: false,
            minute_sync: Instant::now(),
            tab: VCGUITab::Main,
            running: false,
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
            intiface_child_proc_h: None,
        }
    }

    fn exec_handler(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(Layout::right_to_left(), |ui| {
            if ui.button("Intiface Restart").clicked() {
                self.stop_intiface_engine();
                thread::sleep(Duration::from_secs(2));
                self.start_intiface_engine();
            }

            if !self.running {
                if ui.button("Enable").clicked() {
                    self.tme_send
                        .as_ref()
                        .unwrap()
                        .send(ToyManagementEvent::Sig(TmSig::StartListening))
                        .unwrap();
                    
                    self.running = true;
                }
            } else {
                if ui.button("Disable").clicked() {
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

                    self.running = false;
                }
            }
        });
    }

    fn start_intiface_engine(&mut self) {
        // Start intiface-cli
        println!("[*] Starting intiface");
        self.intiface_child_proc_h = match Command::new(format!(
            "{}\\AppData\\Local\\IntifaceDesktop\\engine\\IntifaceCLI.exe",
            get_user_home_dir()
        ))
        .args([
            "--wsinsecureport",
            format!("{}", self.config.intiface_config.0).as_str(),
            "--stayopen",
            "--log",
            "1",
        ])
        .creation_flags(0x08000000)
        .spawn()
        {
            Ok(p) => Some(p),
            Err(_e) => {
                // Cant start intiface
                return;
            }
        };

        println!("[*] Started intiface");
    }

    fn stop_intiface_engine(&mut self) {
        let mut icph = self.intiface_child_proc_h.take();
        match icph {
            Some(ref mut ph) => {
                if let Ok(()) = ph.kill() {
                    println!("[*] Intiface Engine Killed.");
                } else {
                    println!("[!] Intiface was not running.");
                }
            }
            None => {
                println!("[!] Got None for intiface process handle.");
            }
        }
        println!("[*] Stopped Intiface");
    }

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
                        if ui.button("Main").clicked() {
                            self.tab = VCGUITab::Main;
                        }
                        ui.separator();

                        if ui.button("Config").clicked() {
                            self.tab = VCGUITab::Config;
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
            ui.add_space(3.);
        });
        ui.separator();
    }

    fn gui_footer(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                ui.add(Hyperlink::from_label_and_url(
                    "VibeCheck",
                    "https://github.com/SutekhVRC/VibeCheck",
                ));
                ui.label("0.0.20-alpha");
                ui.add(Hyperlink::from_label_and_url(
                    RichText::new("Made by Sutekh")
                        .monospace()
                        .color(Color32::WHITE),
                    "https://github.com/SutekhVRC",
                ));
                ui.add_space(5.0);
            });
        });
    }

    fn chk_valid_config_inputs(&mut self) -> bool {
        if !check_valid_ipv4(&self.config_edit.networking.bind.0) {
            return false;
        }

        if !check_valid_port(&self.config_edit.networking.bind.1) {
            return false;
        }

        if !check_valid_port(&self.config_edit.intiface_config.0) {
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

        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("Intiface WS Port: ");
            ui.text_edit_singleline(&mut self.config_edit.intiface_config.0);
        });

        //ui.label(format!("Bind: {}:{}", self.config.networking.bind.0,self.config.networking.bind.1));
        //ui.label(format!("VRChat: {}:{}", self.config.networking.vrchat.0,self.config.networking.vrchat.1));
        //ui.label(format!("Intiface WS Port: {}", self.config.intiface_config.0));
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

            ui.horizontal_wrapped(|ui| {
                CollapsingHeader::new(format!(
                    "{} [{}%]",
                    toy.1.toy_name,
                    (toy.1.battery_level * 100.).round()
                ))
                .show(ui, |ui| {
                    ui.group(|ui| {
                        if !self.editing.contains_key(&toy.0) {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("Features"));
                                ui.with_layout(Layout::right_to_left(), |ui| {
                                    if ui.button("Edit").clicked() {
                                        self.editing.insert(*toy.0, *toy.0);
                                        return;
                                    }
                                });
                            });
                            ui.separator();
                            // List toy features
                            ui.label(format!("{}", toy.1.param_feature_map));
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("Features"));
                                ui.with_layout(Layout::right_to_left(), |ui| {
                                    if ui.button("Save").clicked() {
                                        if let Some(_) = self.editing.remove(&toy.0) {
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

                            match toy.1.param_feature_map.v {
                                Some(ref mut vibrators) => {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("Vibrator(s)"));
                                        ui.separator();
                                        egui::ComboBox::from_id_source(format!(
                                            "{} Vibrator Mode ({})",
                                            toy.1.toy_name, toy.1.toy_id
                                        ))
                                        .selected_text(vibrators.get_param_mode_str())
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                ui.selectable_value(
                                                    vibrators,
                                                    Vibrators::Auto(
                                                        toy.1.param_feature_map.v_auto.0.clone(), toy.1.param_feature_map.v_auto.1
                                                    ),
                                                    "Auto",
                                                );
                                                ui.selectable_value(
                                                    vibrators,
                                                    Vibrators::Custom(
                                                        toy.1.param_feature_map.v_custom.clone(),
                                                    ),
                                                    "Custom",
                                                );
                                            },
                                        );
                                    });

                                    match vibrators {
                                        Vibrators::Auto(..) => {
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label("Parameter: ");
                                                ui.text_edit_singleline(
                                                    &mut toy.1.param_feature_map.v_auto.0,
                                                );
                                            });
                                            ui.add(egui::Slider::new(&mut toy.1.param_feature_map.v_auto.1.idle_level, 0.0..=1.0).text("Idle Level"));
                                            ui.add(egui::Slider::new(&mut toy.1.param_feature_map.v_auto.1.minimum_level, 0.0..=1.0).text("Minimum Level"));
                                            if toy.1.param_feature_map.v_auto.1.minimum_level > toy.1.param_feature_map.v_auto.1.maximum_level {
                                                toy.1.param_feature_map.v_auto.1.minimum_level = toy.1.param_feature_map.v_auto.1.maximum_level-0.01;
                                            }
                                            ui.add(egui::Slider::new(&mut toy.1.param_feature_map.v_auto.1.maximum_level, 0.0..=1.0).text("Maximum Level"));
                                            if toy.1.param_feature_map.v_auto.1.maximum_level < toy.1.param_feature_map.v_auto.1.minimum_level {
                                                toy.1.param_feature_map.v_auto.1.maximum_level = toy.1.param_feature_map.v_auto.1.minimum_level+0.01;
                                            }
                                            toy.1.param_feature_map.v = Some(Vibrators::Auto(
                                                toy.1.param_feature_map.v_auto.0.clone(),
                                                toy.1.param_feature_map.v_auto.1,
                                            ));
                                        }
                                        Vibrators::Custom(cmap) => {
                                            for i in 0..cmap.len() {
                                                if i > 0 {
                                                    ui.add_space(0.1);
                                                }
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.label(format!(
                                                        "Vibrator ({}): ",
                                                        toy.1.param_feature_map.v_custom[i].1
                                                    ));
                                                    ui.text_edit_singleline(
                                                        &mut toy.1.param_feature_map.v_custom[i].0,
                                                    );
                                                });

                                                ui.add(egui::Slider::new(&mut toy.1.param_feature_map.v_custom[i].2.idle_level, 0.0..=1.0).text("Idle Level"));
                                                ui.add(egui::Slider::new(&mut toy.1.param_feature_map.v_custom[i].2.minimum_level, 0.0..=1.0).text("Minimum Level"));
                                                if toy.1.param_feature_map.v_custom[i].2.minimum_level > toy.1.param_feature_map.v_custom[i].2.maximum_level {
                                                    toy.1.param_feature_map.v_custom[i].2.minimum_level = toy.1.param_feature_map.v_custom[i].2.maximum_level-0.01;
                                                }
                                                ui.add(egui::Slider::new(&mut toy.1.param_feature_map.v_custom[i].2.maximum_level, 0.0..=1.0).text("Maximum Level"));
                                                if toy.1.param_feature_map.v_custom[i].2.maximum_level < toy.1.param_feature_map.v_custom[i].2.minimum_level {
                                                    toy.1.param_feature_map.v_custom[i].2.maximum_level = toy.1.param_feature_map.v_custom[i].2.minimum_level+0.01;
                                                }
                                            }
                                            toy.1.param_feature_map.v = Some(Vibrators::Custom(
                                                toy.1.param_feature_map.v_custom.clone(),
                                            ));
                                        }
                                    }
                                }
                                None => {}
                            }

                            match toy.1.param_feature_map.r {
                                Some(ref mut rotators) => {
                                    ui.separator();
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("Rotator(s)"));
                                        ui.separator();
                                        egui::ComboBox::from_id_source(format!(
                                            "{} Rotator Mode ({})",
                                            toy.1.toy_name, toy.1.toy_id
                                        ))
                                        .selected_text(rotators.get_param_mode_str())
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                ui.selectable_value(
                                                    rotators,
                                                    Rotators::Auto(
                                                        toy.1.param_feature_map.r_auto.0.clone(),
                                                        toy.1.param_feature_map.r_auto.1
                                                    ),
                                                    "Auto",
                                                );
                                                ui.selectable_value(
                                                    rotators,
                                                    Rotators::Custom(
                                                        toy.1.param_feature_map.r_custom.clone(),
                                                    ),
                                                    "Custom",
                                                );
                                            },
                                        );
                                    });

                                    match rotators {
                                        Rotators::Auto(..) => {
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label("Parameter: ");
                                                ui.text_edit_singleline(
                                                    &mut toy.1.param_feature_map.r_auto.0,
                                                );
                                            });
                                            ui.add(egui::Slider::new(&mut toy.1.param_feature_map.r_auto.1.idle_level, 0.0..=1.0).text("Idle Level"));
                                            ui.add(egui::Slider::new(&mut toy.1.param_feature_map.r_auto.1.minimum_level, 0.0..=1.0).text("Minimum Level"));
                                            if toy.1.param_feature_map.r_auto.1.minimum_level > toy.1.param_feature_map.r_auto.1.maximum_level {
                                                toy.1.param_feature_map.r_auto.1.minimum_level = toy.1.param_feature_map.r_auto.1.maximum_level-0.01;
                                            }
                                            ui.add(egui::Slider::new(&mut toy.1.param_feature_map.r_auto.1.maximum_level, 0.0..=1.0).text("Maximum Level"));
                                            if toy.1.param_feature_map.r_auto.1.maximum_level < toy.1.param_feature_map.r_auto.1.minimum_level {
                                                toy.1.param_feature_map.r_auto.1.maximum_level = toy.1.param_feature_map.r_auto.1.minimum_level+0.01;
                                            }
                                            toy.1.param_feature_map.r = Some(Rotators::Auto(
                                                toy.1.param_feature_map.r_auto.0.clone(), toy.1.param_feature_map.r_auto.1
                                            ));
                                        }
                                        Rotators::Custom(cmap) => {
                                            for i in 0..cmap.len() {
                                                if i > 0 {
                                                    ui.add_space(0.1);
                                                }
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.label(format!(
                                                        "Rotator ({}): ",
                                                        toy.1.param_feature_map.r_custom[i].1
                                                    ));
                                                    ui.text_edit_singleline(
                                                        &mut toy.1.param_feature_map.r_custom[i].0,
                                                    );
                                                });

                                                ui.add(egui::Slider::new(&mut toy.1.param_feature_map.r_custom[i].2.idle_level, 0.0..=1.0).text("Idle Level"));
                                                ui.add(egui::Slider::new(&mut toy.1.param_feature_map.r_custom[i].2.minimum_level, 0.0..=1.0).text("Minimum Level"));
                                                if toy.1.param_feature_map.r_custom[i].2.minimum_level > toy.1.param_feature_map.r_custom[i].2.maximum_level {
                                                    toy.1.param_feature_map.r_custom[i].2.minimum_level = toy.1.param_feature_map.r_custom[i].2.maximum_level-0.01;
                                                }
                                                ui.add(egui::Slider::new(&mut toy.1.param_feature_map.r_custom[i].2.maximum_level, 0.0..=1.0).text("Maximum Level"));
                                                if toy.1.param_feature_map.r_custom[i].2.maximum_level < toy.1.param_feature_map.r_custom[i].2.minimum_level {
                                                    toy.1.param_feature_map.r_custom[i].2.maximum_level = toy.1.param_feature_map.r_custom[i].2.minimum_level+0.01;
                                                }
                                            }
                                            toy.1.param_feature_map.r = Some(Rotators::Custom(
                                                toy.1.param_feature_map.r_custom.clone(),
                                            ));
                                        }
                                    }
                                }
                                None => {}
                            }

                            match toy.1.param_feature_map.l {
                                Some(ref mut linears) => {
                                    ui.separator();
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(RichText::new("Linear(s)"));
                                        ui.separator();
                                        egui::ComboBox::from_id_source(format!(
                                            "{} Linear Mode ({})",
                                            toy.1.toy_name, toy.1.toy_id
                                        ))
                                        .selected_text(linears.get_param_mode_str())
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                ui.selectable_value(
                                                    linears,
                                                    Linears::Auto(
                                                        toy.1.param_feature_map.l_auto.0.clone(),
                                                        toy.1.param_feature_map.l_auto.1
                                                    ),
                                                    "Auto",
                                                );
                                                ui.selectable_value(
                                                    linears,
                                                    Linears::Custom(
                                                        toy.1.param_feature_map.l_custom.clone(),
                                                    ),
                                                    "Custom",
                                                );
                                            },
                                        );
                                    });

                                    match linears {
                                        Linears::Auto(..) => {
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label("Auto Parameter: ");
                                                ui.text_edit_singleline(
                                                    &mut toy.1.param_feature_map.l_auto.0,
                                                );
                                            });
                                            ui.add(egui::Slider::new(&mut toy.1.param_feature_map.l_auto.1.idle_level, 0.0..=1.0).text("Idle Level"));
                                            if toy.1.param_feature_map.l_auto.1.minimum_level > toy.1.param_feature_map.l_auto.1.maximum_level {
                                                toy.1.param_feature_map.l_auto.1.minimum_level = toy.1.param_feature_map.l_auto.1.maximum_level-0.01;
                                            }
                                            ui.add(egui::Slider::new(&mut toy.1.param_feature_map.l_auto.1.minimum_level, 0.0..=1.0).text("Minimum Level"));
                                            if toy.1.param_feature_map.l_auto.1.maximum_level < toy.1.param_feature_map.l_auto.1.minimum_level {
                                                toy.1.param_feature_map.l_auto.1.maximum_level = toy.1.param_feature_map.l_auto.1.minimum_level+0.01;
                                            }
                                            ui.add(egui::Slider::new(&mut toy.1.param_feature_map.l_auto.1.maximum_level, 0.0..=1.0).text("Maximum Level"));
                                            toy.1.param_feature_map.l = Some(Linears::Auto(
                                                toy.1.param_feature_map.l_auto.0.clone(),
                                                toy.1.param_feature_map.l_auto.1
                                            ));
                                        }
                                        Linears::Custom(cmap) => {
                                            for i in 0..cmap.len() {
                                                if i > 0 {
                                                    ui.add_space(0.1);
                                                }
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.label(format!(
                                                        "Linear ({}): ",
                                                        toy.1.param_feature_map.l_custom[i].1
                                                    ));
                                                    ui.text_edit_singleline(
                                                        &mut toy.1.param_feature_map.l_custom[i].0,
                                                    );
                                                });

                                                ui.add(egui::Slider::new(&mut toy.1.param_feature_map.l_custom[i].2.idle_level, 0.0..=1.0).text("Idle Level"));
                                                ui.add(egui::Slider::new(&mut toy.1.param_feature_map.l_custom[i].2.minimum_level, 0.0..=1.0).text("Minimum Level"));
                                                if toy.1.param_feature_map.l_custom[i].2.minimum_level > toy.1.param_feature_map.l_custom[i].2.maximum_level {
                                                    toy.1.param_feature_map.l_custom[i].2.minimum_level = toy.1.param_feature_map.l_custom[i].2.maximum_level-0.01;
                                                }
                                                ui.add(egui::Slider::new(&mut toy.1.param_feature_map.l_custom[i].2.maximum_level, 0.0..=1.0).text("Maximum Level"));
                                                if toy.1.param_feature_map.l_custom[i].2.maximum_level < toy.1.param_feature_map.l_custom[i].2.minimum_level {
                                                    toy.1.param_feature_map.l_custom[i].2.maximum_level = toy.1.param_feature_map.l_custom[i].2.minimum_level+0.01;
                                                }
                                            }

                                            toy.1.param_feature_map.l = Some(Linears::Custom(
                                                toy.1.param_feature_map.l_custom.clone(),
                                            ));
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    });
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
                                populate_toy_feature_param_map(&mut toy, Some(fmap));
                            } else {
                                populate_toy_feature_param_map(&mut toy, None);
                            }
                            println!("[TOY FEATURES]\n{:?}", toy.param_feature_map);
                            self.tme_send
                                .as_ref()
                                .unwrap()
                                .send(ToyManagementEvent::Tu(ToyUpdate::AddToy(toy.clone())))
                                .unwrap();
                            // Load toy config for name of toy if it exists otherwise create the config for the toy name
                            self.toys.insert(toy.toy_id, toy.clone());
                            println!("[+] Toy added: {} | {}", toy.toy_name, toy.toy_id);
                        }
                        EventSig::ToyRemove(id) => {
                            self.tme_send
                                .as_ref()
                                .unwrap()
                                .send(ToyManagementEvent::Tu(ToyUpdate::RemoveToy(id)))
                                .unwrap();
                            self.toys.remove(&id);
                            println!("[!] Removed toy: {}", id);
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

#[derive(Serialize, Deserialize)]
struct ToyConfig {
    feature_param_map: HashMap<String, ToyFeature>,
}

fn load_toy_config(toy_name: &String) -> Option<FeatureParamMap> {
    let config_path = format!(
        "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck\\ToyConfigs\\{}.json",
        get_user_home_dir(),
        toy_name
    );

    if !file_exists(&config_path) {
        return None;
    } else {
        let con = fs::read_to_string(config_path).unwrap();

        let feature_param_map: FeatureParamMap = match serde_json::from_str(&con) {
            Ok(fpm) => fpm,
            Err(_) => {
                return None;
            }
        };
        return Some(feature_param_map);
    }
}

fn save_toy_config(toy_name: &String, feature_param_map: FeatureParamMap) {
    let config_path = format!(
        "{}\\AppData\\LocalLow\\VRChat\\VRChat\\OSC\\VibeCheck\\ToyConfigs\\{}.json",
        get_user_home_dir(),
        toy_name
    );

    fs::write(
        &config_path,
        serde_json::to_string(&feature_param_map).unwrap(),
    )
    .unwrap();
}

fn alter_toy(tme_send: &Sender<ToyManagementEvent>, altered_toy: VCToy) {
    let _ = tme_send.send(ToyManagementEvent::Tu(ToyUpdate::AlterToy(altered_toy)));
}

/*
    Parse configs of toy names and populate toys on ToyAdd
    If no config put toy to Auto params
*/
fn populate_toy_feature_param_map(toy: &mut VCToy, param_feature_map: Option<FeatureParamMap>) {
    // If a param feature map is passed configure the toy
    // If None is passed set toy to auto defaults

    if let Some(map) = param_feature_map {
        toy.param_feature_map = map;
    } else {
        let features = toy.toy_features.clone();
        println!(
            "Populating toy: {} | feature count: {}",
            toy.toy_id,
            toy.toy_features.len()
        );

        for feature in features {
            // When in Auto there only needs to be one auto param per feature type
            match feature.0 {
                ButtplugCurrentSpecDeviceMessageType::VibrateCmd => {
                    for i in 0..feature.1.feature_count.unwrap() {
                        toy.param_feature_map
                            .v_custom
                            .push((format!("/avatar/parameters/custom_vibrate_{}", i), i, LevelTweaks::default()));
                    }
                    toy.param_feature_map.v_auto =
                        ("/avatar/parameters/vibrate_auto_default".to_string(), LevelTweaks::default());
                    toy.param_feature_map.v = Some(Vibrators::Auto(
                        "/avatar/parameters/vibrate_auto_default".to_string(), LevelTweaks::default(),
                    ));
                }
                ButtplugCurrentSpecDeviceMessageType::RotateCmd => {
                    for i in 0..feature.1.feature_count.unwrap() {
                        toy.param_feature_map
                            .r_custom
                            .push((format!("/avatar/parameters/custom_rotate_{}", i), i, LevelTweaks::default()));
                    }
                    toy.param_feature_map.r_auto =
                        ("/avatar/parameters/rotate_auto_default".to_string(), LevelTweaks::default());
                    toy.param_feature_map.r = Some(Rotators::Auto(
                        "/avatar/parameters/rotate_auto_default".to_string(), LevelTweaks::default()
                    ));
                }
                ButtplugCurrentSpecDeviceMessageType::LinearCmd => {
                    for i in 0..feature.1.feature_count.unwrap() {
                        toy.param_feature_map
                            .l_custom
                            .push((format!("/avatar/parameters/custom_linear_{}", i), i, LevelTweaks::default()));
                    }
                    toy.param_feature_map.l_auto =
                        ("/avatar/parameters/linear_auto_default".to_string(), LevelTweaks::default());
                    toy.param_feature_map.l = Some(Linears::Auto(
                        "/avatar/parameters/linear_auto_default".to_string(), LevelTweaks::default()
                    ));
                }
                _ => {}
            }
        }
        // Save toy on first time add
        save_toy_config(&toy.toy_name, toy.param_feature_map.clone());
    }
}

impl App for VibeCheckGUI {
    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &eframe::epi::Frame,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        // Set fonts
        let mut style: Style = (*ctx.style()).clone();
        style.override_text_style = Some(TextStyle::Monospace);
        ctx.set_style(style);
        let mut visuals = Visuals::default();
        visuals.override_text_color = Some(Color32::from_rgb(0xef, 0x98, 0xff));

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

        ctx.set_visuals(visuals);

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

        // Set horny time
        println!("[*] Horny Timer Loaded: {}", self.config.horny_timer);
        let sync_now = Instant::now().checked_sub(Duration::from_secs(self.config.horny_timer)).unwrap();
        println!("[*] Time Sync: {}", sync_now.elapsed().as_secs());
        self.minute_sync = sync_now;

        self.set_error_handling_channels();
        self.start_intiface_engine();
        //thread::sleep(Duration::from_secs(2));
        self.start_client_event_handler();
        self.start_toy_management_handler();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &eframe::epi::Frame) {
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
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Main");
                        self.exec_handler(ui);
                    });
                    ui.separator();
                    ScrollArea::new([false, true]).show(ui, |ui| {
                        self.list_toys(ui);
                        ui.add_space(60.);
                    });
                    //self.main_tab(ui);
                }
                VCGUITab::Config => {
                    ui.horizontal_wrapped(|ui| {
                        ui.label("VibeCheck Config");
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
                }
            }
            self.gui_footer(ctx);
        });
    }

    fn on_exit(&mut self) {

        let toys_sd = self.toys.clone();
        for toy in toys_sd {
            self.async_rt.block_on(async move {
                match toy.1.device_handle.stop().await {
                    Ok(_) => println!("[*] Stop command sent: {}", toy.1.toy_name),
                    Err(_e) => println!("[!] Err stopping device: {}", _e),
                }
            });
        }
        thread::sleep(Duration::from_secs(1));
        self.stop_intiface_engine();
        self.config.horny_timer = self.minute_sync.elapsed().as_secs();
        self.save_config();
        std::process::exit(0);
    }

    fn name(&self) -> &str {
        "VibeCheck"
    }
}
