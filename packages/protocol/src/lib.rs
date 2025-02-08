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

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub enum WebhookMessage {
    NewChannel(NewChannel),
    CloseChannel(CloseChannel),
    NewParticipant(NewParticipant),
    RemoveParticipant(RemoveParticipant),
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct NewChannel {
    pub channel_id: String,
    pub tenant_id: Option<String>,
    pub num_channels: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct CloseChannel {
    pub channel_id: String,
    pub tenant_id: Option<String>,
    pub num_channels: usize,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct NewParticipant {
    pub channel_id: String,
    pub tenant_id: Option<String>,
    pub participant_id: String,
}

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct RemoveParticipant {
    pub channel_id: String,
    pub tenant_id: Option<String>,
    pub participant_id: String,
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize)]
pub enum WebSocketCloseCode {
    TokenNotFound = 3000,
    InvalidToken = 3001,
    InvalidSignerKey = 3002,
}
