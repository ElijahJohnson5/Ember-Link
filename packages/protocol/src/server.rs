use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{PresenceState, StorageUpdateMessage};

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/index.ts")]
#[serde(
    rename_all_fields = "camelCase",
    rename_all = "camelCase",
    tag = "type"
)]
pub enum ServerMessage {
    Presence(ServerPresenceMessage),
    InitialPresence(InitialPresenceMessage),
    AssignId(AssignIdMessage),
    StorageUpdate(StorageUpdateMessage),
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct ServerPresenceMessage {
    pub id: String,
    pub clock: i32,
    pub data: Option<PresenceState>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct InitialPresenceMessage {
    pub presences: Vec<ServerPresenceMessage>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct AssignIdMessage {
    pub id: String,
}
