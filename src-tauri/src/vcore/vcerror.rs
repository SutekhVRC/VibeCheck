/*
 * Splitting errors into fronted native call errors and backend errors
 */

pub mod frontend {
    use serde::Serialize;

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
        UnsetLCOverrideFailure,
        SetLCOverrideFailure,
        InvalidLCHost,

        ToyManagerNotReady,
    }

    #[derive(Serialize)]
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

    pub enum ToyAlterError {
        //NoFeatureIndex,
        //NoToyIndex,
        TMESendFailure,
    }
}
