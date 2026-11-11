use std::fmt;

use crate::address::PortId;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A module referred to a port that does not exist.
    UnknownPort(PortId),
    /// An output could not reach its device or service.
    OutputUnavailable(String),
    /// Anything else a module wants to report.
    Module(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownPort(port) => write!(f, "unknown port {port}"),
            Error::OutputUnavailable(why) => write!(f, "output unavailable: {why}"),
            Error::Module(why) => write!(f, "module error: {why}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;
