use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CustomMessage, StorageSyncMessage, StorageUpdateMessage};

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(
    rename_all_fields = "camelCase",
    rename_all = "camelCase",
    tag = "type"
)]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub enum ClientMessage<P, C> {
    Presence(ClientPresenceMessage<P>),
    StorageUpdate(StorageUpdateMessage),
    StorageSync(StorageSyncMessage),
    ProviderSync(StorageSyncMessage),
    ProviderUpdate(StorageUpdateMessage),
    Custom(CustomMessage<C>),
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/client/index.ts")]
pub struct ClientPresenceMessage<P> {
    pub clock: i32,
    pub data: P,
}
