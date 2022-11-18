/*
 * Splitting errors into fronted native call errors and backend errors
 */


pub mod frontend {
    use serde::Serialize;


    #[derive(Serialize)]
    pub enum VCFeError {
        
        // Toy Errors
        AlterToyFailure,

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
}

pub mod backend {
    #[derive(serde::Serialize)]
    pub enum VibeCheckConfigError {
        InvalidHost,
        InvalidPort,
        SerializeFailure,
        WriteFailure,
    }

}