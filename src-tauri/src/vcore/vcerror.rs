/*
 * Splitting errors into fronted native call errors and backend errors
 */

pub enum VCError {
    HandlingErr(crate::toy_handling::errors::HandlerErr),
}

pub mod frontend {
    use serde::Serialize;

    use super::backend::ToyAlterError;

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
