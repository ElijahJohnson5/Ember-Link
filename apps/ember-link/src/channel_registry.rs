use futures_util::lock::Mutex;
use protocol::{CloseChannel, NewChannel, NewParticipant, RemoveParticipant, WebhookMessage};
use ractor::ActorRef;
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

pub struct ChannelRegistry {
    channels: Arc<Mutex<HashMap<String, WeakChannel>>>,
    handlers: Handlers,
    webhook_processor: Option<ActorRef<WebhookMessage>>,
}

impl ChannelRegistry {
    pub fn new(webhook_processor: Option<ActorRef<WebhookMessage>>) -> Self {
        let new = Self {
            channels: Arc::default(),
            handlers: Handlers::default(),
            webhook_processor: webhook_processor.clone(),
        };

        if let Some(webhook_processor) = webhook_processor {
            new.on_channel_created({
                move |channel_id, num| {
                    webhook_processor
                        .cast(WebhookMessage::NewChannel(NewChannel {
                            channel_id: channel_id.clone(),
                            num_channels: num.clone(),
                        }))
                        .expect("Could not send message to webhook processor")
                }
            })
            .detach();
        }

        new
    }

    pub async fn get_or_create_channel(&self, channel_name: String) -> Result<Channel, String> {
        let mut channels = self.channels.lock().await;

        let len = channels.len();

        let channel = match channels.entry(channel_name.clone()) {
            Entry::Occupied(entry) => match entry.get().upgrade() {
                Some(channel) => Ok(channel),
                None => self.create_channel(Entry::Occupied(entry), channel_name, len),
            },
            entry => self.create_channel(entry, channel_name, len),
        }?;

        Ok(channel)
    }

    fn create_channel(
        &self,
        entry: Entry<'_, String, WeakChannel>,
        channel_name: String,
        old_num_channels: usize,
    ) -> Result<Channel, String> {
        let channel = Channel::new(channel_name.clone());

        match entry {
            Entry::Occupied(mut entry) => {
                entry.insert(channel.downgrade());
            }
            Entry::Vacant(entry) => {
                entry.insert(channel.downgrade());
            }
        }

        if let Some(webhook_processor) = self.webhook_processor.as_ref() {
            channel
                .on_participant_added({
                    let id = channel_name.clone();
                    let webhook_processor = webhook_processor.clone();

                    move |participant_id| {
                        webhook_processor
                            .cast(WebhookMessage::NewParticipant(NewParticipant {
                                channel_id: id.clone(),
                                participant_id: participant_id.clone(),
                            }))
                            .expect("Could not send new participant message to webhook processor")
                    }
                })
                .detach();

            channel
                .on_participant_removed({
                    let id = channel_name.clone();
                    let webhook_processor = webhook_processor.clone();

                    move |participant_id| {
                        webhook_processor
                            .cast(WebhookMessage::RemoveParticipant(RemoveParticipant {
                                channel_id: id.clone(),
                                participant_id: participant_id.clone(),
                            }))
                            .expect(
                                "Could not send remove participant message to webhook processor",
                            )
                    }
                })
                .detach();
        }

        channel
            .on_close({
                let id = channel_name.clone();
                let channels = self.channels.clone();
                let webhook_processor = self.webhook_processor.clone();

                move || {
                    tokio::spawn(async move {
                        {
                            let num = {
                                let mut channels = channels.lock().await;
                                channels.remove(&id);

                                channels.len()
                            };

                            if let Some(webhook_processor) = webhook_processor {
                                webhook_processor
                                    .cast(WebhookMessage::CloseChannel(CloseChannel {
                                        channel_id: id,
                                        num_channels: num,
                                    }))
                                    .expect(
                                        "Could not send close channel message to webhook processor",
                                    );
                            }
                        }
                    });
                }
            })
            .detach();

        self.handlers
            .channel_created
            .call_simple(&channel_name, &(old_num_channels + 1));

        Ok(channel)
    }

    pub fn on_channel_created<F: Fn(&String, &usize) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) -> HandlerId {
        self.handlers.channel_created.add(Arc::new(callback))
    }
}
