use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::StorageUpdateMessage;

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(
    rename_all_fields = "camelCase",
    rename_all = "camelCase",
    tag = "type"
)]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub enum ClientMessage<T> {
    Presence(ClientPresenceMessage<T>),
    StorageUpdate(StorageUpdateMessage),
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/client/index.ts")]
pub struct ClientPresenceMessage<T> {
    pub clock: i32,
    pub data: T,
}
