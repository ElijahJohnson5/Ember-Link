use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export, export_to = "index.ts")]
pub enum ServerMessage {
    UpdatePresence { client_id: i64 },
    NewPresence { client_id: i64 },
}
