/*
 * Errors commands return to the frontend (`frontend`) and errors from
 * backend work (`backend`).
 *
 * Every error has a message for the user (its Display, via thiserror) and a
 * code naming the problem (its innermost variant's name). Errors a command
 * returns serialize as `{ "code": ..., "message": ... }`: the frontend shows
 * the message and can check the code for cases it handles itself.
 */

use serde::{Serialize, Serializer};
use strum::IntoStaticStr;
use thiserror::Error;

#[derive(Debug, Serialize, Error, IntoStaticStr)]
pub enum VcoreError {
    #[error("VibeCheck isn't fully started yet.")]
    NoAppHandle,
    #[error("VibeCheck's toy list isn't ready yet.")]
    NoToyManager,
    #[error("VibeCheck isn't fully started yet.")]
    NoStatePointer,
    #[error("The toy event handler is already running.")]
    CehAlreadyInitialized,
    #[error(
        "Couldn't start listening for VRChat while off: the previous listener is still running."
    )]
    DisabledOscListenerThreadRunning,
}

/// What a command's error looks like to the frontend.
#[derive(Serialize)]
struct CommandErrorJson<'a> {
    code: &'a str,
    message: String,
}

fn serialize_command_error<S: Serializer>(
    code: &str,
    error: &impl std::fmt::Display,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    CommandErrorJson {
        code,
        message: error.to_string(),
    }
    .serialize(serializer)
}

pub mod frontend {
    use serde::{Serialize, Serializer};
    use strum::IntoStaticStr;
    use thiserror::Error;

    use crate::toy_handling::errors::ToyHandlingError;

    use super::{backend::ToyAlterError, serialize_command_error, VcoreError};

    #[derive(Debug, Error, IntoStaticStr)]
    pub enum VCFeError {
        #[error(transparent)]
        AlterToyFailure(ToyAlterError),
        // App State Errors
        #[error(
            "Couldn't turn VibeCheck on. Try again, or restart VibeCheck if it keeps happening."
        )]
        EnableFailure,
        #[error("Couldn't turn VibeCheck on: another app is using its OSC port. Close the other app, or change the port in Settings > Connection.")]
        EnableBindFailure,
        #[error("VibeCheck is already off.")]
        DisableFailure,
        #[error("VibeCheck's toy connection service didn't start, so toys can't connect. Restart VibeCheck.")]
        ToyServiceNotRunning,
        #[error("Couldn't start looking for toys: {0}")]
        StartScanFailure(String),
        #[error("Couldn't stop looking for toys: {0}")]
        StopScanFailure(String),

        // Config Errors
        #[error("The OSC Bind address isn't valid. Use an address and port, like 127.0.0.1:9001.")]
        InvalidBindEndpoint,
        #[error(
            "The OSC Remote address isn't valid. Use an address and port, like 127.0.0.1:9000."
        )]
        InvalidRemoteEndpoint,
        #[error("Couldn't save settings: they couldn't be converted for saving.")]
        SerializeFailure,
        #[error("Couldn't save settings: VibeCheck couldn't write its config file.")]
        WriteFailure,
        #[error("Couldn't save settings: VibeCheck couldn't find its config folder.")]
        ConfigDirFailure,
        #[error("Couldn't save this toy's settings.")]
        SaveOfflineToyConfig,
        //InvalidIpv4Host,
        #[error("VibeCheck's toy list isn't ready yet. Try again in a moment.")]
        ToyManagerNotReady,
        #[error(transparent)]
        ToyManager(ToyHandlingError),

        #[error(transparent)]
        Vcore(VcoreError),
    }

    impl VCFeError {
        /// The problem's name, e.g. "ToyOnline": wrapped errors give their
        /// own variant's name.
        pub fn code(&self) -> &'static str {
            match self {
                Self::AlterToyFailure(e) => e.into(),
                Self::ToyManager(e) => e.into(),
                Self::Vcore(e) => e.into(),
                other => other.into(),
            }
        }
    }

    impl Serialize for VCFeError {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serialize_command_error(self.code(), self, serializer)
        }
    }
}

pub mod backend {
    use serde::{Serialize, Serializer};
    use strum::IntoStaticStr;
    use thiserror::Error;

    use super::serialize_command_error;

    #[derive(Serialize, Debug, Error, IntoStaticStr)]
    pub enum VibeCheckConfigError {
        //ReadFailure,
        //DeserializeError,
        #[error("VibeCheck's settings couldn't be converted for saving.")]
        SerializeError,
        #[error("VibeCheck couldn't write its config file.")]
        WriteFailure,
        #[error("VibeCheck couldn't find its config folder.")]
        ConfigDirFail,
    }

    #[derive(Serialize, Debug, Error, IntoStaticStr)]
    pub enum VibeCheckToyConfigError {
        #[error("The toy's saved settings file couldn't be opened.")]
        ReadFailure,
        #[error("The toy's saved settings couldn't be read.")]
        DeserializeError,
        #[error("This toy has no saved settings.")]
        OfflineToyConfigNotFound,
        #[error("The toy's settings couldn't be converted for saving.")]
        SerializeError,
        //WriteFailure,
        #[error("VibeCheck couldn't find its config folder.")]
        ConfigDirFail,
        #[error(transparent)]
        FSFailure(VibeCheckFSError),
    }

    #[derive(Debug, Error, IntoStaticStr)]
    pub enum VibeCheckFSError {
        #[error("Couldn't read the folder.")]
        ReadDirFailure,
        #[error("Couldn't read a file's location in the folder.")]
        ReadDirPathFailure,
        #[error("Couldn't delete the files.")]
        RemoveDirsFailure,
        #[error("Couldn't write the file.")]
        FileWriteFailure,
        /// No VRChat OSC folder was found (VRChat not installed, never
        /// used with OSC, or installed somewhere unusual)
        #[error("Couldn't find VRChat's OSC folder.")]
        VRChatOscNotFound,
    }

    impl Serialize for VibeCheckFSError {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serialize_command_error(self.into(), self, serializer)
        }
    }

    #[derive(Serialize, Debug, Error, IntoStaticStr)]
    pub enum ToyAlterError {
        #[error("That feature doesn't exist on this toy any more.")]
        NoFeatureIndex,
        #[error("That toy isn't connected any more.")]
        NoToyIndex,
        #[error("Couldn't pass the change to the running toy. Turn VibeCheck off and on again.")]
        TMESendFailure,
        #[error("The toy is connected, so this change has to go to the connected toy.")]
        ToyConnected,
        #[error("The toy isn't connected.")]
        ToyDisconnected,
        #[error("This toy has no saved settings.")]
        OfflineToyNotExist,
        #[error("That feature doesn't exist in this toy's saved settings.")]
        OfflineToyNoFeatureIndex,
    }
}

#[cfg(test)]
mod tests {
    use super::{backend::*, frontend::VCFeError};
    use crate::toy_handling::errors::ToyHandlingError;

    fn json(e: &impl serde::Serialize) -> serde_json::Value {
        serde_json::to_value(e).unwrap()
    }

    #[test]
    fn command_errors_serialize_as_code_and_message() {
        let e = VCFeError::InvalidBindEndpoint;
        assert_eq!(json(&e)["code"], "InvalidBindEndpoint");
        assert_eq!(json(&e)["message"], e.to_string());
    }

    #[test]
    fn wrapped_errors_use_the_inner_code_and_message() {
        let e = VCFeError::ToyManager(ToyHandlingError::ToyOnline);
        assert_eq!(json(&e)["code"], "ToyOnline");
        assert_eq!(json(&e)["message"], ToyHandlingError::ToyOnline.to_string());

        let e = VCFeError::AlterToyFailure(ToyAlterError::NoToyIndex);
        assert_eq!(json(&e)["code"], "NoToyIndex");
    }

    #[test]
    fn fs_errors_serialize_the_same_way() {
        assert_eq!(
            json(&VibeCheckFSError::VRChatOscNotFound)["code"],
            "VRChatOscNotFound"
        );
    }
}
