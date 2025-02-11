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
}

#[async_trait]
pub trait Storage {
    async fn handle_sync_message(
        &self,
        message: &StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, StorageError>;

    async fn handle_update_message(
        &self,
        message: &StorageUpdateMessage,
    ) -> Result<(), StorageError>;
}
