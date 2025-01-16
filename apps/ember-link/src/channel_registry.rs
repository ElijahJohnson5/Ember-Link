use futures_util::lock::Mutex;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use crate::channel::{Channel, WeakChannel};

#[derive(Default)]
pub struct ChannelRegistry {
    channels: Arc<Mutex<HashMap<String, WeakChannel>>>,
}

impl ChannelRegistry {
    pub async fn get_or_create_channel(
        &self,
        channel_name: String,
        org_id: String,
    ) -> Result<Channel, String> {
        let mut channels = self.channels.lock().await;

        let key = format!("{}.{}", org_id, channel_name);

        let channel = match channels.entry(key.clone()) {
            Entry::Occupied(entry) => match entry.get().upgrade() {
                Some(channel) => Ok(channel),
                None => self.create_channel(Entry::Occupied(entry), channel_name, org_id, key),
            },
            entry => self.create_channel(entry, channel_name, org_id, key),
        }?;

        Ok(channel)
    }

    fn create_channel(
        &self,
        entry: Entry<'_, String, WeakChannel>,
        channel_name: String,
        org_id: String,
        combined_id: String,
    ) -> Result<Channel, String> {
        let channel = Channel::new(combined_id.clone());

        match entry {
            Entry::Occupied(mut entry) => {
                entry.insert(channel.downgrade());
            }
            Entry::Vacant(entry) => {
                entry.insert(channel.downgrade());
            }
        }

        channel
            .on_close({
                let id = combined_id;
                let channels = self.channels.clone();

                move || {
                    tokio::spawn(async move {
                        {
                            channels.lock().await.remove(&id);
                        }
                    });
                }
            })
            .detach();

        Ok(channel)
    }
}
