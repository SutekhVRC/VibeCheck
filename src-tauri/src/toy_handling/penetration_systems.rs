use serde::{Serialize, Deserialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PenetrationSystem {
    NONE,
    TPS,
    SPS,
}