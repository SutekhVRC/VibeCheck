/*
 * Frontend type binding generation
 */
use serde::{Deserialize, Serialize};
use strum::Display;
use ts_rs::TS;

use crate::toy_handling::{
    input_processor::penetration_systems::PenetrationSystemType,
    toyops::{ProcessingMode, VCFeatureType},
    ToyPower,
};

use super::ToBackend;

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeVibeCheckConfig {
    pub networking: FeOSCNetworking,
    pub scan_on_disconnect: bool,
    pub minimize_on_exit: bool,
    pub desktop_notifications: bool,
    pub show_toy_advanced: bool,
    pub show_feature_advanced: bool,
    /// Master intensity, 0.0 to 1.0.
    pub master_intensity: f64,
    /// Bluetooth scan length in seconds; 0 scans until stopped.
    pub scan_length: u32,
    /// Theme id; one that doesn't exist shows the default theme.
    pub theme: String,
    /// Pressing the scan button turns VibeCheck on first.
    pub auto_enable_on_scan: bool,
    /// The sidebar is collapsed to icons.
    pub sidebar_collapsed: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeOSCNetworking {
    /// Manual mode: where VibeCheck listens.
    pub bind: String,
    /// Manual mode: where VibeCheck sends.
    pub remote: String,
    pub mode: FeOscMode,
    /// OSCQuery mode: the port VibeCheck listens on (picked once, kept).
    /// Read-only: the backend picks it.
    pub oscquery_port: Option<u16>,
}

/// How VibeCheck and VRChat find each other.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, TS)]
#[ts(export)]
pub enum FeOscMode {
    /// Automatic: VibeCheck advertises its own port over OSCQuery.
    OscQuery,
    /// The bind and remote addresses set by hand.
    Manual,
}

/// What OSCQuery mode is doing, for Settings > Connection.
#[derive(Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeOscQueryStatus {
    /// The port being advertised, once advertising started.
    pub advertising: Option<u16>,
    /// Where VRChat receives OSC (e.g. `127.0.0.1:9000`), while it's found.
    pub vrchat: Option<String>,
    /// Why advertising failed, if it did.
    pub error: Option<String>,
    /// Whether VibeCheck holds VRChat's default port (9001) so VRChat's sends
    /// there don't fail. False while another app has it.
    pub holding_default_port: bool,
}

#[derive(Serialize, Clone, TS, Display)]
#[ts(export)]
#[serde(tag = "kind", content = "data")]
pub enum FeToyEvent {
    Add(FeVCToy),
    Remove(u32),
    Update(FeVCToy),
    //OfflineSyncAll(Vec<FeVCToy>),
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub enum FeScanEvent {
    Start,
    /// The backend stopped a scan itself (its scan length ran out)
    Stop,
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub enum FeStateEvent {
    EnableAndScan,
    Disable,
    /// Turn on (tray menu)
    Enable,
    /// Look for toys while already on (tray menu)
    Scan,
}

#[derive(Serialize, Clone, TS, Display)]
#[ts(export)]
#[serde(tag = "kind", content = "data")]
pub enum FeCoreEvent {
    Scan(FeScanEvent),
    State(FeStateEvent),
    Safety(FeSafetyEvent),
    Config(FeConfigEvent),
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub enum FeConfigEvent {
    /// The settings changed outside the window (e.g. master intensity, see
    /// control.rs): load them again.
    Changed,
}

#[derive(Serialize, Clone, TS)]
#[ts(export)]
pub enum FeSafetyEvent {
    /// Panic stop: every toy stopped until resumed.
    Panicked,
    Resumed,
}

#[derive(Deserialize, Clone, TS)]
#[ts(export)]
pub enum FeBrowserLink {
    Github,
    GithubWiki,
    VRChatGroup,
    Discord,
    ToyOptions,
    FeatureOptions,
    /// The wiki's page on Intiface Central (Settings > Toys > Toy connection)
    IntifaceCentral,
}

#[derive(Serialize, Deserialize, Clone, TS, Debug)]
#[ts(export)]
pub enum FeVCToyAnatomy {
    Anus,
    //Any, I forgot what the point of this was?
    Breasts,
    Buttocks,
    Chest,
    Clitoris,
    Face,
    Feet,
    FootL,
    FootR,
    HandLeft,
    HandRight,
    Hands,
    Labia,
    Mouth,
    NA,
    Nipples,
    Penis,
    Perineum,
    Testicles,
    Thighs,
    Vagina,
    Vulva,
    Wrist,
}

#[derive(Debug, Deserialize, Serialize, Clone, TS)]
#[ts(export)]
pub struct FeVCToy {
    pub toy_id: Option<u32>,
    pub toy_name: String,
    pub toy_anatomy: FeVCToyAnatomy,
    pub toy_power: ToyPower,
    pub toy_connected: bool,
    pub features: Vec<FeVCToyFeature>,
    pub listening: bool,
    pub osc_data: bool,
    pub bt_update_rate: u64,
    pub sub_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, TS)]
#[ts(export)]
pub struct FeLevelTweaks {
    pub minimum_level: f64,
    pub maximum_level: f64,
    pub idle_level: f64,
    pub smooth_rate: f64,
    pub linear_position_speed: u32,
    /// Rate mode's wind-down speed
    pub rate_tune: f64,
    /// Rate mode's sensitivity
    pub rate_sensitivity: f64,
    pub constant_level: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum FeProcessingMode {
    Raw,
    Smooth,
    Rate,
    Constant,
}

impl ToBackend<ProcessingMode> for FeProcessingMode {
    type OutputType = ProcessingMode;

    fn to_backend(&self) -> Self::OutputType {
        match self {
            Self::Raw => ProcessingMode::Raw,
            Self::Smooth => ProcessingMode::Smooth,
            Self::Rate => ProcessingMode::Rate,
            Self::Constant => ProcessingMode::Constant,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeToyParameter {
    pub parameter: String,
    pub processing_mode: FeProcessingMode,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FePenetrationSystem {
    pub pen_system_type: PenetrationSystemType,
    pub pen_system_processing_mode: FeProcessingMode,
    pub pen_system_input_filter: Option<Vec<String>>,
}

impl ToBackend<(PenetrationSystemType, ProcessingMode)> for FePenetrationSystem {
    type OutputType = (PenetrationSystemType, ProcessingMode);

    fn to_backend(&self) -> Self::OutputType {
        (
            self.pen_system_type.clone(),
            self.pen_system_processing_mode.to_backend(),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FeVCToyFeature {
    pub feature_enabled: bool,
    pub feature_type: FeVCFeatureType,
    pub osc_parameters: Vec<FeToyParameter>,
    pub penetration_system: FePenetrationSystem,
    pub feature_index: u32,
    pub flip_input_float: bool,
    pub feature_levels: FeLevelTweaks,
    pub smooth_enabled: bool,
    pub rate_enabled: bool,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub enum FeToyAlter {
    Connected(FeVCToy),
    Disconnected(FeVCToy),
    //Feature(FeVCToyFeature),
    //Feature((u32, FeVCToyFeature)),
    //OSCData((u32, bool)),
    //Anatomy((u32, FeVCToyAnatomy)),
    //Offline(OfflineToy),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, TS)]
#[ts(export)]
pub enum FeVCFeatureType {
    Vibrator = 0,
    Rotator = 1,
    Linear = 2,
    Oscillate = 3,
    Constrict = 4,
    Inflate = 5,
    Position = 6,
    // Note no ScalarRotator bc conversion is done in vcore
}
/*
impl FeVCFeatureType {
    pub fn to_be(&self) -> VCFeatureType {
        match self {
            Self::Vibrator => VCFeatureType::Vibrator,
            Self::Rotator => VCFeatureType::Rotator,
            Self::Linear => VCFeatureType::Linear,
            Self::Oscillate => VCFeatureType::Oscillate,
            Self::Constrict => VCFeatureType::Constrict,
            Self::Inflate => VCFeatureType::Inflate,
            Self::Position => VCFeatureType::Position,
        }
    }
}*/

impl PartialEq<VCFeatureType> for FeVCFeatureType {
    fn eq(&self, other: &VCFeatureType) -> bool {
        *self as u32 == *other as u32
    }
}

/// One batch of live values for the diagnostics view, sent about 10 times a
/// second while it's open and VibeCheck is on.
#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
pub struct FeDiagnostics {
    /// Forget everything shown so far before applying this batch (a new
    /// session started, or values were cleared, e.g. on avatar change).
    pub reset: bool,
    /// Values on the signal plane that changed since the last batch (all of
    /// them after a reset).
    pub signals: Vec<FeDiagSignal>,
    /// Every running toy's graph values (always complete).
    pub toys: Vec<FeDiagToy>,
    /// New diagnostics log entries, oldest first.
    pub log: Vec<FeDiagLogEntry>,
}

/// A value an input wrote to the signal plane.
#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
pub struct FeDiagSignal {
    /// The module that wrote it, e.g. `osc`.
    pub module: String,
    /// Its port, e.g. an OSC address.
    pub port: String,
    pub value: f64,
    pub is_bool: bool,
    /// How many times it's been written since it was last cleared.
    #[ts(type = "number")]
    pub writes: u64,
    /// Read from VRChat's OSCQuery because VRChat hasn't sent it over OSC
    /// (yet).
    pub from_oscquery: bool,
}

/// The values inside one toy's graph.
#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
pub struct FeDiagToy {
    pub toy_id: u32,
    /// Each node output, in run order: input processor, parameter modes,
    /// levels.
    pub ports: Vec<FeDiagPort>,
    /// What was last sent to each motor, by each of its chains (the input
    /// processor and every parameter have their own).
    pub sent: Vec<FeDiagSent>,
}

#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
pub struct FeDiagSent {
    /// The node output it came from, e.g.
    /// `toy:lush:0/Vibrator_0/param0/levels/out`.
    pub from: String,
    /// The motor, e.g. `toy:lush:0/Vibrator_0`.
    pub motor: String,
    /// Before master intensity. Nothing yet if nothing has been sent.
    pub value: Option<f64>,
}

#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
pub struct FeDiagPort {
    /// e.g. `toy:lush:0/Vibrator_0/param0/mode/out`.
    pub name: String,
    /// Nothing yet, e.g. no input has arrived.
    pub value: Option<f64>,
}

/// An entry from the diagnostics log, e.g. an OSC message.
#[derive(Serialize, Clone, Debug, TS)]
#[ts(export)]
pub struct FeDiagLogEntry {
    #[ts(type = "number")]
    pub seq: u64,
    /// Milliseconds since diagnostics were turned on.
    #[ts(type = "number")]
    pub at_ms: u64,
    pub module: String,
    pub label: String,
    pub value: String,
}

/// The current avatar's parameters, read from VRChat's OSCQuery (see
/// osc/avatar_catalog.rs), for the parameter picker. Sent with
/// `get_avatar_catalog` and as `fe_avatar_event` whenever it changes.
#[derive(Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeAvatarCatalog {
    pub status: FeAvatarStatus,
    /// Empty unless `status` is `Ready`. Sorted by address.
    pub parameters: Vec<FeAvatarParameter>,
    /// The avatar's SPS zones, each once, in the order first seen. Empty
    /// unless `status` is `Ready`.
    pub zones: Vec<FeAvatarZone>,
    /// Whether the avatar has TPS parameters.
    pub tps: bool,
}

/// An SPS zone on the current avatar.
#[derive(Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeAvatarZone {
    /// e.g. a socket's name in the SPS menu. What input filter tags match.
    pub name: String,
    pub kind: FeZoneKind,
    pub format: FeZoneFormat,
}

#[derive(Serialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub enum FeZoneKind {
    Socket,
    Plug,
    Touch,
}

#[derive(Serialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub enum FeZoneFormat {
    /// SPS 3 and later
    Sps3,
    /// Older SPS
    Legacy,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, TS)]
#[ts(export)]
pub enum FeAvatarStatus {
    /// The first look for VRChat hasn't finished yet
    Looking,
    /// VRChat isn't found (closed, or OSC off)
    Searching,
    /// VRChat is found; reading the avatar
    Reading,
    Ready,
}

#[derive(Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeAvatarParameter {
    /// The full OSC address, e.g. `/avatar/parameters/Boop`.
    pub address: String,
    pub kind: FeParamKind,
    pub group: FeParamGroup,
}

#[derive(Serialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub enum FeParamKind {
    Float,
    Bool,
    /// VibeCheck can't use ints
    Int,
    Other,
}

#[derive(Serialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub enum FeParamGroup {
    Own,
    Sps,
    Tps,
    /// Generated by VRCFury; can change when the avatar is rebuilt
    VrcFury,
    BuiltIn,
}

/// What a feature's settings miss on the current avatar (`check_feature`).
#[derive(Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeFeatureCheck {
    pub missing: Vec<FeMissingParameter>,
    /// Parameters the avatar has but VibeCheck can't use (ints)
    pub unusable: Vec<FeAvatarParameter>,
    /// Input filter tags that match none of the avatar's SPS zones
    pub unmatched_tags: Vec<String>,
    /// The Processor's system, if the avatar has none of it
    pub system_missing: Option<PenetrationSystemType>,
    /// Each input filter tag and the SPS zones it matches
    pub tag_matches: Vec<FeTagMatch>,
}

#[derive(Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeTagMatch {
    pub tag: String,
    /// Names of the zones it matches; empty if none
    pub zones: Vec<String>,
}

#[derive(Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct FeMissingParameter {
    pub address: String,
    /// The avatar's parameter that was probably meant
    pub suggestion: Option<String>,
}

mod avatar_conversions {
    use super::*;
    use crate::osc::avatar_catalog::{
        AvatarParam, CatalogState, FeatureCheck, ParamGroup, ParamKind,
    };
    use crate::toy_handling::input_processor::penetration_systems::sps::{
        Format, ZoneInfo, ZoneKind,
    };

    impl From<&ZoneInfo> for FeAvatarZone {
        fn from(z: &ZoneInfo) -> Self {
            Self {
                name: z.name.clone(),
                kind: match z.kind {
                    ZoneKind::Socket => FeZoneKind::Socket,
                    ZoneKind::Plug => FeZoneKind::Plug,
                    ZoneKind::Touch => FeZoneKind::Touch,
                },
                format: match z.format {
                    Format::Sps3 => FeZoneFormat::Sps3,
                    Format::Legacy => FeZoneFormat::Legacy,
                },
            }
        }
    }

    impl From<&AvatarParam> for FeAvatarParameter {
        fn from(p: &AvatarParam) -> Self {
            Self {
                address: p.address.clone(),
                kind: match p.kind {
                    ParamKind::Float => FeParamKind::Float,
                    ParamKind::Bool => FeParamKind::Bool,
                    ParamKind::Int => FeParamKind::Int,
                    ParamKind::Other => FeParamKind::Other,
                },
                group: match p.group {
                    ParamGroup::Own => FeParamGroup::Own,
                    ParamGroup::Sps => FeParamGroup::Sps,
                    ParamGroup::Tps => FeParamGroup::Tps,
                    ParamGroup::VrcFury => FeParamGroup::VrcFury,
                    ParamGroup::BuiltIn => FeParamGroup::BuiltIn,
                },
            }
        }
    }

    impl From<&CatalogState> for FeAvatarCatalog {
        fn from(state: &CatalogState) -> Self {
            let empty = |status| Self {
                status,
                parameters: Vec::new(),
                zones: Vec::new(),
                tps: false,
            };
            match state {
                CatalogState::Looking => empty(FeAvatarStatus::Looking),
                CatalogState::Searching => empty(FeAvatarStatus::Searching),
                CatalogState::Reading => empty(FeAvatarStatus::Reading),
                CatalogState::Ready(avatar) => Self {
                    status: FeAvatarStatus::Ready,
                    parameters: avatar.params.iter().map(Into::into).collect(),
                    zones: avatar.zones.iter().map(Into::into).collect(),
                    tps: avatar.tps,
                },
            }
        }
    }

    impl From<FeatureCheck> for FeFeatureCheck {
        fn from(check: FeatureCheck) -> Self {
            Self {
                missing: check
                    .missing
                    .into_iter()
                    .map(|m| FeMissingParameter {
                        address: m.address,
                        suggestion: m.suggestion,
                    })
                    .collect(),
                unusable: check.unusable.iter().map(Into::into).collect(),
                unmatched_tags: check.unmatched_tags,
                system_missing: check.system_missing,
                tag_matches: check
                    .tag_matches
                    .into_iter()
                    .map(|m| FeTagMatch {
                        tag: m.tag,
                        zones: m.zones,
                    })
                    .collect(),
            }
        }
    }
}
