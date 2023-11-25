pub mod errors;
pub mod handling;
pub mod toy_manager;
pub mod toyops;
pub mod penetration_systems;

pub enum SmoothParser {
    Smoothed,
    SkipZero,
    Smoothing,
}

pub enum RateParser {
    RateCalculated(bool),
    SkipZero,
}

#[derive(Clone, Debug)]
pub enum ToySig {
    //ToyCommand(ToyFeature),
    UpdateToy(crate::vcore::core::ToyUpdate),
    OSCMsg(rosc::OscMessage),
}