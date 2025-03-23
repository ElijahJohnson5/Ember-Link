use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CustomMessage, StorageSyncMessage, StorageUpdateMessage};

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/index.ts")]
#[serde(
    rename_all_fields = "camelCase",
    rename_all = "camelCase",
    tag = "type"
)]
pub enum ServerMessage<P, C> {
    Presence(ServerPresenceMessage<P>),
    InitialPresence(InitialPresenceMessage<P>),
    AssignId(AssignIdMessage),
    StorageUpdate(StorageUpdateMessage),
    StorageSync(StorageSyncMessage),
    Custom(CustomMessage<C>),
}

#[derive(Clone, Deserialize, Debug, Serialize, TS, PartialEq, Eq)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct ServerPresenceMessage<P> {
    pub id: String,
    pub clock: i32,
    pub data: Option<P>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS, PartialEq, Eq)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct InitialPresenceMessage<P> {
    pub presences: Vec<ServerPresenceMessage<P>>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/server/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct AssignIdMessage {
    pub id: String,
}
