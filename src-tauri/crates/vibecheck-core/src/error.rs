//! The error type modules and the runtime return.

use std::fmt;

use crate::address::PortId;

/// What went wrong in a module or the runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A module referred to a port that does not exist.
    UnknownPort(PortId),
    /// An output could not reach its device or service.
    OutputUnavailable(String),
    /// A graph was wired incorrectly (unknown port, cycle, ...).
    Graph(String),
    /// Anything else a module wants to report.
    Module(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownPort(port) => write!(f, "unknown port {port}"),
            Error::OutputUnavailable(why) => write!(f, "output unavailable: {why}"),
            Error::Graph(why) => write!(f, "graph error: {why}"),
            Error::Module(why) => write!(f, "module error: {why}"),
        }
    }
}

impl std::error::Error for Error {}

/// A `Result` with [`Error`] as the default error.
pub type Result<T, E = Error> = std::result::Result<T, E>;
