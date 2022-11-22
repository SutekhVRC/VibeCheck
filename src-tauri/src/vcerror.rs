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

    #[derive(Serialize)]
    pub enum VibeCheckConfigError {
        SerializeFailure,
        WriteFailure,
    }
}