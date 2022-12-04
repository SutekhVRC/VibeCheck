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
        DisableFailure,
        StartScanFailure(String),
        StopScanFailure(String),

        // Config Errors
        InvalidBindEndpoint,
        InvalidRemoteEndpoint,
        SerializeFailure,
        WriteFailure,
        //InvalidIpv4Host,
        UnsetLCOverrideFailure,
        SetLCOverrideFailure,
        InvalidLCHost,
    }

    #[derive(Serialize)]
    pub enum ToyAlterError {
        NoFeatureIndex,
        NoToyIndex,
        TMESendFailure
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
        //SerializeError,
        //WriteFailure,
    }
}