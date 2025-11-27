use strum::Display;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Clone, TS, Display, Debug)]
#[ts(export)]
#[serde(tag = "kind", content = "data")]
pub enum FrontendError {
    Error(String),
    Warning(String),
    Info(String),
}
