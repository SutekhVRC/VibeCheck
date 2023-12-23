//use ts_rs::TS;

pub mod error;
pub mod frontend_native;
pub mod frontend_types;

pub trait ToBackend<B> {
    type OutputType;
    fn to_backend(&self) -> Self::OutputType;
}

pub trait ToFrontend<F> {
    type OutputType;
    fn to_frontend(&self) -> Self::OutputType;
}

pub trait FromFrontend<F> {
    type OutputType;
    fn from_frontend(&mut self, frontend_type: F) -> Self::OutputType;
}

pub trait FromBackend<B> {
    type OutputType;
    fn from_backend(&mut self, backend_type: B) -> Self::OutputType;
}
