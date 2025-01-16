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
    NewPresence(NewPresenceMessage),
    InitialPresence(InitialPresenceMessage),
    AssignId(AssignIdMessage),
    StorageUpdate(StorageUpdateMessage),
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct NewPresenceMessage {
    pub id: String,
    pub clock: i32,
    pub data: PresenceState,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct InitialPresenceMessage {
    pub presences: Vec<NewPresenceMessage>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct AssignIdMessage {
    pub id: String,
}
