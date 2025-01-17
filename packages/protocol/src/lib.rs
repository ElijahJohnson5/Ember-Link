pub mod client;
pub mod server;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Deserialize, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../src/bindings/index.ts")]
pub struct StorageUpdateMessage {
    pub update: Vec<u8>,
}
