use core::fmt;
use std::error::Error;

use crate::{
    frontend::error::FrontendError, osc::errors::OscError, osc_api::OscApiError,
    toy_handling::errors::ToyHandlingError, util::errors::UtilError, vcore::errors::VcoreError,
};

pub mod state_comm;

#[derive(Debug)]
pub struct VibeCheckError {
    source: ErrorSource,
    message: Option<&'static str>,
}

impl VibeCheckError {
    pub fn new(source: ErrorSource, message: Option<&'static str>) -> Self {
        Self { source, message }
    }
}

impl Error for VibeCheckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl fmt::Display for VibeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VibeCheckError")
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum ErrorSource {
    Frontend(FrontendError),
    Osc(OscError),
    OscApi(OscApiError),
    ToyHandling(ToyHandlingError),
    Util(UtilError),
    Vcore(VcoreError),
}

impl Error for ErrorSource {}

impl fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VibeCheckError")
    }
}
