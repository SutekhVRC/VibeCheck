use thiserror::Error;

use crate::{
    util::errors::UtilError, vcore::errors::VcoreError,
};

pub mod reporter;

/// An error from starting up a part of VibeCheck, with what was being done
/// when it happened.
#[derive(Debug, Error)]
#[error("{}{source}", context(message))]
pub struct VibeCheckError {
    source: ErrorSource,
    message: Option<&'static str>,
}

fn context(message: &Option<&'static str>) -> String {
    message.map(|m| format!("{m}: ")).unwrap_or_default()
}

impl VibeCheckError {
    pub fn new(source: ErrorSource, message: Option<&'static str>) -> Self {
        Self { source, message }
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ErrorSource {
    #[error(transparent)]
    Util(UtilError),
    #[error(transparent)]
    Vcore(VcoreError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn says_what_went_wrong() {
        let e = VibeCheckError::new(
            ErrorSource::Util(UtilError::BPIOInit),
            Some("Failed to initialize bpio"),
        );
        assert_eq!(
            e.to_string(),
            "Failed to initialize bpio: Couldn't start the toy connection service (Buttplug)."
        );
        let e = VibeCheckError::new(ErrorSource::Vcore(VcoreError::NoAppHandle), None);
        assert_eq!(e.to_string(), VcoreError::NoAppHandle.to_string());
    }
}
