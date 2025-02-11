use async_trait::async_trait;
use protocol::{StorageSyncMessage, StorageUpdateMessage};
use yrs::{
    updates::{decoder::Decode, encoder::Encode},
    AsyncTransact, Doc, ReadTxn, StateVector, Update,
};

use super::{Storage, StorageError};

pub struct YjsStorage {
    doc: Doc,
}

impl YjsStorage {
    pub fn new(doc: Doc) -> Self {
        Self { doc }
    }
}

#[async_trait]
impl Storage for YjsStorage {
    async fn handle_sync_message(
        &self,
        message: &StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, super::StorageError> {
        tracing::info!("Got sync message of type: {}", message.sync_type);

        match message.sync_type.as_str() {
            "SyncStep1" => {
                let txn = self.doc.transact().await;

                let sv = match StateVector::decode_v1(&message.data) {
                    Err(e) => return Err(StorageError::Sync(Box::new(e))),
                    Ok(sv) => sv,
                };

                let update = txn.encode_state_as_update_v1(&sv);

                let step_1_update = txn.state_vector().encode_v1();

                return Ok(Some(vec![
                    StorageSyncMessage {
                        data: update,
                        sync_type: "SyncStep2".to_string(),
                    },
                    StorageSyncMessage {
                        data: step_1_update,
                        sync_type: "SyncStep1".to_string(),
                    },
                ]));
            }
            "SyncStep2" => {
                let mut txn = self.doc.transact_mut().await;

                let update = match Update::decode_v1(&message.data) {
                    Err(e) => return Err(StorageError::Sync(Box::new(e))),
                    Ok(sv) => sv,
                };

                match txn.apply_update(update) {
                    Err(e) => return Err(StorageError::Sync(Box::new(e))),
                    _ => {}
                }

                return Ok(Some(vec![StorageSyncMessage {
                    data: vec![],
                    sync_type: "SyncDone".to_string(),
                }]));
            }
            &_ => {
                tracing::warn!("Unknown sync message type: {}", message.sync_type);
            }
        }

        Ok(None)
    }

    async fn handle_update_message(
        &self,
        message: &StorageUpdateMessage,
    ) -> Result<(), StorageError> {
        let mut txn = self.doc.transact_mut().await;

        let update = match Update::decode_v1(&message.update) {
            Err(e) => return Err(StorageError::Sync(Box::new(e))),
            Ok(sv) => sv,
        };

        match txn.apply_update(update) {
            Err(e) => return Err(StorageError::Sync(Box::new(e))),
            _ => {}
        }

        Ok(())
    }
}

pub fn init_storage() -> YjsStorage {
    let doc = Doc::new();

    YjsStorage::new(doc)
}
