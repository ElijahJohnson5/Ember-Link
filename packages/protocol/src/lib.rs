pub mod client;
pub mod server;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Deserialize, Debug, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct StorageUpdateMessage {
    pub update: Vec<u8>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct StorageSyncMessage {
    pub sync_type: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub enum WebhookMessage {
    NewChannel(NewChannel),
    CloseChannel(CloseChannel),
    NewParticipant(NewParticipant),
    RemoveParticipant(RemoveParticipant),
    StorageUpdated(StorageUpdated),
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct NewChannel {
    pub id: String,
    pub channel_name: String,
    pub timestamp: u128,
    pub num_channels: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct CloseChannel {
    pub id: String,
    pub channel_name: String,
    pub timestamp: u128,
    pub num_channels: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct NewParticipant {
    pub id: String,
    pub channel_name: String,
    pub timestamp: u128,
    pub participant_id: String,
    pub num_pariticipants: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct RemoveParticipant {
    pub id: String,
    pub channel_name: String,
    pub timestamp: u128,
    pub participant_id: String,
    pub num_pariticipants: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct StorageUpdated {
    pub id: String,
    pub channel_name: String,
    pub timestamp: u128,
    pub data: Vec<u8>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/index.ts")]
#[serde(rename_all = "camelCase")]
pub struct CustomMessage<C> {
    pub data: C,
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize)]
pub enum WebSocketCloseCode {
    TokenNotFound = 3000,
    InvalidToken = 3001,
    InvalidSignerKey = 3002,
    ChannelCreationFailed = 3003,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
// Add more once we have more implementations, this gets sent to the server for the server to be able to handle
// sync messages correctly
pub enum StorageType {
    Yjs,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct StorageEndpointResponse {
    pub updates: Vec<Vec<u8>>,
}
