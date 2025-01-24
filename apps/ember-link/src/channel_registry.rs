use futures_util::lock::Mutex;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use crate::{
    channel::{Channel, WeakChannel},
    event_listener_primitives::{Bag, HandlerId},
};

#[derive(Default)]
struct Handlers {
    channel_created: Bag<Arc<dyn Fn(&String, &usize) + Send + Sync>, String, usize>,
}

#[derive(Default)]
pub struct ChannelRegistry {
    channels: Arc<Mutex<HashMap<String, WeakChannel>>>,
    handlers: Handlers,
}

impl ChannelRegistry {
    pub async fn get_or_create_channel(
        &self,
        channel_name: String,
        org_id: String,
    ) -> Result<Channel, String> {
        let mut channels = self.channels.lock().await;

        let len = channels.len();

        let key = format!("{}.{}", org_id, channel_name);

        let channel = match channels.entry(key.clone()) {
            Entry::Occupied(entry) => match entry.get().upgrade() {
                Some(channel) => Ok(channel),
                None => self.create_channel(Entry::Occupied(entry), channel_name, org_id, key, len),
            },
            entry => self.create_channel(entry, channel_name, org_id, key, len),
        }?;

        Ok(channel)
    }

    fn create_channel(
        &self,
        entry: Entry<'_, String, WeakChannel>,
        channel_name: String,
        org_id: String,
        combined_id: String,
        old_num_channels: usize,
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
                let id = combined_id.clone();
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

        self.handlers
            .channel_created
            .call_simple(&combined_id, &(old_num_channels + 1));

        Ok(channel)
    }

    pub fn on_channel_created<F: Fn(&String, &usize) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) -> HandlerId {
        self.handlers.channel_created.add(Arc::new(callback))
    }
}
