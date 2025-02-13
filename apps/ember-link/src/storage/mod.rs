use async_trait::async_trait;
use protocol::{StorageSyncMessage, StorageUpdateMessage};
use std::error::Error as StdError;

pub mod yjs;

pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StorageError {
    #[error("Sync encountered an error: {0}")]
    Sync(#[source] BoxDynError),

    #[error("Unable to call storage endpoint to sync data: {0}")]
    EndpointError(#[source] BoxDynError),

    #[error("Update could not be applyed: {0}")]
    UpdateError(#[source] BoxDynError),
}

#[async_trait]
pub trait Storage {
    async fn init_storage_from_endpoint(
        &self,
        channel_name: &String,
        storage_endpoint: &Option<String>,
        tenant_id: &Option<String>,
    ) -> Result<(), StorageError>;

    fn handle_sync_message(
        &self,
        message: &StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, StorageError>;

    fn handle_update_message(&self, message: &StorageUpdateMessage) -> Result<(), StorageError>;
}
