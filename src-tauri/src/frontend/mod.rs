//use ts_rs::TS;

pub mod frontend_native;
pub mod frontend_types;
pub mod error;

/* Trait for frontend type conversions? maybe
pub trait VCFrontend<F, B> 
where F: TS, B: TS,
{
    fn to_frontend(&self, backend_type: B) -> F;
    fn to_backend(&self, frontend_type: F) -> B;
}*/