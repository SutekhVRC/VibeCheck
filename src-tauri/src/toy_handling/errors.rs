use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ToyHandlingError {
    PopulateConfigFailure,
}

impl Error for ToyHandlingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for ToyHandlingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ToyHandlingError")
    }
}

pub struct HandlerErr {
    pub id: i32,
    pub msg: String,
}
