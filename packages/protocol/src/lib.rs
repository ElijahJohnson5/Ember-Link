// pub mod client;
// pub mod server;
mod generated;
pub use generated::*;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
    pub timestamp: u64,
    pub num_channels: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct CloseChannel {
    pub id: String,
    pub channel_name: String,
    pub timestamp: u64,
    pub num_channels: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct NewParticipant {
    pub id: String,
    pub channel_name: String,
    pub timestamp: u64,
    pub participant_id: String,
    pub num_pariticipants: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct RemoveParticipant {
    pub id: String,
    pub channel_name: String,
    pub timestamp: u64,
    pub participant_id: String,
    pub num_pariticipants: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct StorageUpdated {
    pub id: String,
    pub channel_name: String,
    pub timestamp: u64,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize)]
pub enum WebSocketCloseCode {
    TokenNotFound = 4000,
    InvalidToken = 4001,
    InvalidSignerKey = 4002,
    ChannelCreationFailed = 4003,
    #[cfg(feature = "multi-tenant")]
    MissingTenantId = 4004,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct StorageEndpointResponse {
    pub updates: Vec<Vec<u8>>,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct SignerKeyResponse {
    pub public_signer_key: String,
}
