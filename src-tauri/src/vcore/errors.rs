/*
 * Splitting errors into fronted native call errors and backend errors
 */

use std::{error::Error, fmt};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum VcoreError {
    NoAppHandle,
    NoToyManager,
    NoStatePointer,
    CehAlreadyInitialized,
    DisabledOscListenerThreadRunning,
}

impl Error for VcoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for VcoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VcoreError")
    }
}

pub enum VCError {
    HandlingErr(crate::toy_handling::errors::HandlerErr),
}

pub mod frontend {
    use serde::Serialize;

    use crate::toy_handling::errors::ToyHandlingError;

    use super::{backend::ToyAlterError, VcoreError};

    #[derive(Serialize)]
    pub enum VCFeError {
        AlterToyFailure(ToyAlterError),
        // App State Errors
        EnableFailure,
        EnableBindFailure,
        DisableFailure,
        StartScanFailure(String),
        StopScanFailure(String),

        // Config Errors
        InvalidBindEndpoint,
        InvalidRemoteEndpoint,
        OSCQueryFailure(&'static str),
        SerializeFailure,
        WriteFailure,
        //InvalidIpv4Host,
        ToyManagerNotReady,
        ToyManager(ToyHandlingError),

        Vcore(VcoreError),
    }
}

pub mod backend {
    use serde::Serialize;

    #[derive(Serialize, Debug)]
    pub enum VibeCheckConfigError {
        //ReadFailure,
        //DeserializeError,
        SerializeError,
        WriteFailure,
    }

    #[derive(Serialize, Debug)]
    pub enum VibeCheckToyConfigError {
        //ReadFailure,
        DeserializeError,
        OfflineToyConfigNotFound,
        //SerializeError,
        //WriteFailure,
    }

    #[derive(Serialize, Debug)]
    pub enum VibeCheckFSError {
        ReadDirFailure,
        ReadDirPathFailure,
        RemoveDirsFailure,
    }

    #[derive(Serialize, Debug)]
    pub enum ToyAlterError {
        NoFeatureIndex,
        NoToyIndex,
        TMESendFailure,
        ToyConnected,
        ToyDisconnected,
        OfflineToyNotExist,
        OfflineToyNoFeatureIndex,
    }
}
