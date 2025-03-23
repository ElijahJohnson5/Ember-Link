use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{StorageSyncMessage, StorageUpdateMessage};

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/index.ts")]
#[serde(
    rename_all_fields = "camelCase",
    rename_all = "camelCase",
    tag = "type"
)]
pub enum ServerMessage<T> {
    Presence(ServerPresenceMessage<T>),
    InitialPresence(InitialPresenceMessage<T>),
    AssignId(AssignIdMessage),
    StorageUpdate(StorageUpdateMessage),
    StorageSync(StorageSyncMessage),
    Custom(serde_json::Value),
}

#[derive(Clone, Deserialize, Debug, Serialize, TS, PartialEq, Eq)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct ServerPresenceMessage<T> {
    pub id: String,
    pub clock: i32,
    pub data: Option<T>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS, PartialEq, Eq)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct InitialPresenceMessage<T> {
    pub presences: Vec<ServerPresenceMessage<T>>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct AssignIdMessage {
    pub id: String,
}
