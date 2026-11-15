use std::{error::Error, fmt};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum ToyHandlingError {
    PopulateConfigFailure,
    /// The toy is connected, so its settings are in use
    ToyOnline,
    /// No saved settings for that toy
    ToyNotFound,
    RemoveFailure,
}

impl Error for ToyHandlingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for ToyHandlingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ToyHandlingError")
    }
}

