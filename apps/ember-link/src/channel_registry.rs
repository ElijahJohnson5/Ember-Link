use futures_util::lock::Mutex;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use crate::channel::actor::ChannelHandle;

#[derive(Default)]
pub struct ChannelRegistry {
    channels: Arc<Mutex<HashMap<String, ChannelHandle>>>,
}

impl ChannelRegistry {
    pub async fn get_or_create_channel(
        &self,
        channel_name: String,
        org_id: String,
    ) -> Result<ChannelHandle, String> {
        let mut channels = self.channels.lock().await;

        let key = format!("{}.{}", org_id, channel_name);

        let channel = match channels.entry(key.clone()) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            entry => self.create_channel(entry, channel_name, org_id),
        }?;

        Ok(channel)
    }

    fn create_channel(
        &self,
        entry: Entry<'_, String, ChannelHandle>,
        channel_name: String,
        org_id: String,
    ) -> Result<ChannelHandle, String> {
        let channel_handle = ChannelHandle::new();

        entry.or_insert(channel_handle.clone());

        Ok(channel_handle)
    }
}
