use self::toyops::{ProcessingModeValues, ToyParameter};

pub mod errors;
pub mod handling;
pub mod input_processor;
pub mod toy_manager;
pub mod toyops;

pub enum SmoothParser {
    Smoothed(f64),
    SkipZero(f64),
    Smoothing,
}

pub enum RateParser {
    RateCalculated(f64, bool),
    SkipZero,
}

#[derive(Clone, Debug)]
pub enum ToySig {
    //ToyCommand(ToyFeature),
    UpdateToy(crate::vcore::core::ToyUpdate),
    OSCMsg(rosc::OscMessage),
}

pub enum ModeProcessorInput<'processor> {
    InputProcessor((ModeProcessorInputType, &'processor mut ProcessingModeValues)),
    RawInput(ModeProcessorInputType, &'processor mut ToyParameter),
}

pub enum ModeProcessorInputType {
    Float(f64),
    Boolean(bool),
}
