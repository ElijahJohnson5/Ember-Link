use async_trait::async_trait;
use protocol::{StorageSyncMessage, StorageUpdateMessage};
use url::Url;
use yrs::{
    updates::{decoder::Decode, encoder::Encode},
    Doc, ReadTxn, StateVector, Transact, Update,
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
    async fn init_storage_from_endpoint(
        &self,
        channel_name: &String,
        storage_endpoint: &Option<String>,
        tenant_id: &Option<String>,
    ) -> Result<(), StorageError> {
        if let Some(storage_endpoint) = storage_endpoint {
            let mut url = Url::parse(&storage_endpoint)
                .map_err(|e| StorageError::EndpointError(Box::new(e)))?;

            if let Some(tenant_id) = tenant_id {
                url.query_pairs_mut().append_pair("tenant_id", tenant_id);
            }

            url.query_pairs_mut()
                .append_pair("channel_name", channel_name);

            let response = reqwest::get(url)
                .await
                .map_err(|e| StorageError::EndpointError(Box::new(e)))?;

            let response = response
                .error_for_status()
                .map_err(|e| StorageError::EndpointError(Box::new(e)))?
                .json::<protocol::StorageEndpointResponse>()
                .await
                .map_err(|e| StorageError::EndpointError(Box::new(e)))?;

            let mut transaction = Transact::transact_mut(&self.doc);

            for update in response.updates {
                let update = yrs::Update::decode_v1(&update)
                    .map_err(|e| StorageError::EndpointError(Box::new(e)))?;

                transaction
                    .apply_update(update)
                    .map_err(|e| StorageError::UpdateError(Box::new(e)))?;
            }
        }

        Ok(())
    }

    fn handle_sync_message(
        &self,
        message: &StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, StorageError> {
        tracing::info!("Got sync message of type: {}", message.sync_type);

        match message.sync_type.as_str() {
            "SyncStep1" => {
                let txn = self.doc.transact();

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
                let mut txn = self.doc.transact_mut();

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

    fn handle_update_message(&self, message: &StorageUpdateMessage) -> Result<(), StorageError> {
        let mut txn = self.doc.transact_mut();

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
