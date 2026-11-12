/*
 * The value type SPS/TPS input processors take.
 *
 * The processing modes themselves (Raw, Smooth, Rate, Constant) live in
 * vibecheck-core (modules::modes) and are built per toy in
 * runtime/toy_pipeline.rs.
 */

use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, TS, Serialize, Deserialize, Copy)]
pub enum ModeProcessorInputType {
    Float(f64),
    Boolean(bool),
}

impl ModeProcessorInputType {
    pub fn try_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn try_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}
