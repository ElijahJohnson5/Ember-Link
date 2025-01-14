pub mod client;
pub mod server;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct PresenceState {
    #[ts(type = "Record<string, unknown>")]
    custom: HashMap<String, Value>,
}
