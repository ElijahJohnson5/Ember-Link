use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub enum ClientMessage {
    NewPresence { client_id: i32 },
}
