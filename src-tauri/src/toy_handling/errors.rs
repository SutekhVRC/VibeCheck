use serde::Serialize;
use strum::IntoStaticStr;
use thiserror::Error;

#[derive(Debug, Serialize, Error, IntoStaticStr)]
pub enum ToyHandlingError {
    #[error("Couldn't load the saved toys.")]
    PopulateConfigFailure,
    /// The toy is connected, so its settings are in use
    #[error("The toy is connected. Disconnect it first.")]
    ToyOnline,
    /// No saved settings for that toy
    #[error("This toy has no saved settings.")]
    ToyNotFound,
    #[error("Couldn't delete the toy's saved settings.")]
    RemoveFailure,
}
