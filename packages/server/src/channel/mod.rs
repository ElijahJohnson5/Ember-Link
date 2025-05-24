use protocol::{ServerMessage, StorageSyncMessage, StorageUpdateMessage};

use crate::storage::StorageError;
use crate::EmberLinkError;

pub trait Channel {
    fn broadcast(&self, message: ServerMessage, exclude_id: Option<&String>);

    fn handle_storage_sync_message(
        &self,
        message: StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, StorageError>;

    fn handle_storage_update_message(
        &self,
        message: StorageUpdateMessage,
        participant_id: &String,
    ) -> Result<(), StorageError>;

    fn handle_provider_sync_message(
        &self,
        message: StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, StorageError>;

    fn handle_provider_update_message(
        &self,
        message: StorageUpdateMessage,
        participant_id: &String,
    ) -> Result<(), StorageError>;
}

#[cfg(feature = "multi-tenant")]
pub fn create_channel_name(
    channel_name: &String,
    tenant_id: &Option<String>,
) -> Result<String, EmberLinkError> {
    if let None = tenant_id {
        return Err(EmberLinkError::MissingTenantId);
    }

    let tenant_id = tenant_id.clone().unwrap();

    Ok(format!("{}:{}", channel_name, tenant_id))
}

#[cfg(not(feature = "multi-tenant"))]
pub fn create_channel_name(
    channel_name: &String,
    _tenant_id: &Option<String>,
) -> Result<String, EmberLinkError> {
    Ok(channel_name.clone())
}
