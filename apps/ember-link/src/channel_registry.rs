#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use futures_util::lock::Mutex;
use protocol::{CloseChannel, NewChannel, NewParticipant, RemoveParticipant, WebhookMessage};
use ractor::ActorRef;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::instrument;

use crate::{
    channel::{Channel, WeakChannel},
    webhook_processor::actor::WebhookProcessorMessage,
};

pub struct ChannelRegistry {
    channels: Arc<Mutex<HashMap<String, WeakChannel>>>,
    webhook_processor: Option<ActorRef<WebhookProcessorMessage>>,
}

impl ChannelRegistry {
    pub fn new(webhook_processor: Option<ActorRef<WebhookProcessorMessage>>) -> Self {
        Self {
            channels: Arc::default(),
            webhook_processor: webhook_processor.clone(),
        }
    }

    #[instrument(skip(self))]
    pub async fn get_or_create_channel(
        &self,
        channel_name: String,
        tenant_id: Option<String>,
    ) -> Channel {
        let mut channels = self.channels.lock().await;

        let len = channels.len();

        let channel = match channels.entry(channel_name.clone()) {
            Entry::Occupied(entry) => match entry.get().upgrade() {
                Some(channel) => {
                    tracing::info!("Found existing channel");
                    channel
                }
                None => self.create_channel(Entry::Occupied(entry), channel_name, tenant_id, len),
            },
            entry => self.create_channel(entry, channel_name, tenant_id, len),
        };

        channel
    }

    #[instrument(skip(self, entry, old_num_channels))]
    fn create_channel(
        &self,
        entry: Entry<'_, String, WeakChannel>,
        channel_name: String,
        tenant_id: Option<String>,
        old_num_channels: usize,
    ) -> Channel {
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
            tracing::info!("Setting up webhook callbacks");

            channel
                .on_participant_added({
                    let id = channel_name.clone();
                    let webhook_processor = webhook_processor.clone();
                    let tenant_id = tenant_id.clone();

                    move |participant_id, num_participants| {
                        let start = SystemTime::now();
                        let since_the_epoch = start
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards");

                        let webhook_message = WebhookMessage::NewParticipant(NewParticipant {
                            id: uuid::Uuid::new_v4().into(),
                            channel_id: id.clone(),
                            timestamp: since_the_epoch.as_millis(),
                            participant_id: participant_id.clone(),
                            num_pariticipants: *num_participants,
                        });

                        let webhook_processor_message = WebhookProcessorMessage {
                            msg: webhook_message,
                            tenant_id: tenant_id.clone(),
                        };

                        webhook_processor
                            .cast(webhook_processor_message)
                            .expect("Could not send new participant message to webhook processor")
                    }
                })
                .detach();

            channel
                .on_participant_removed({
                    let id = channel_name.clone();
                    let webhook_processor = webhook_processor.clone();
                    let tenant_id = tenant_id.clone();

                    move |participant_id, num_participants| {
                        let start = SystemTime::now();
                        let since_the_epoch = start
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards");

                        let webhook_message =
                            WebhookMessage::RemoveParticipant(RemoveParticipant {
                                id: uuid::Uuid::new_v4().into(),
                                channel_id: id.clone(),
                                timestamp: since_the_epoch.as_millis(),
                                participant_id: participant_id.clone(),
                                num_pariticipants: *num_participants,
                            });

                        let webhook_processor_message = WebhookProcessorMessage {
                            msg: webhook_message,
                            tenant_id: tenant_id.clone(),
                        };

                        webhook_processor.cast(webhook_processor_message).expect(
                            "Could not send remove participant message to webhook processor",
                        )
                    }
                })
                .detach();

            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");

            let webhook_message = WebhookMessage::NewChannel(NewChannel {
                id: uuid::Uuid::new_v4().into(),
                channel_id: channel_name.clone(),
                timestamp: since_the_epoch.as_millis(),
                num_channels: old_num_channels + 1,
            });

            let webhook_processor_message = WebhookProcessorMessage {
                msg: webhook_message,
                tenant_id: tenant_id.clone(),
            };

            webhook_processor
                .cast(webhook_processor_message)
                .expect("Could not send message to webhook processor")
        }

        channel
            .on_close({
                let id = channel_name.clone();
                let channels = self.channels.clone();
                let webhook_processor = self.webhook_processor.clone();
                let tenant_id = tenant_id.clone();

                move || {
                    tokio::spawn(async move {
                        {
                            let start = SystemTime::now();
                            let since_the_epoch = start
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards");

                            let num = {
                                let mut channels = channels.lock().await;
                                channels.remove(&id);

                                channels.len()
                            };

                            let webhook_message = WebhookMessage::CloseChannel(CloseChannel {
                                id: uuid::Uuid::new_v4().into(),
                                channel_id: id,
                                timestamp: since_the_epoch.as_millis(),
                                num_channels: num,
                            });

                            let webhook_processor_message = WebhookProcessorMessage {
                                msg: webhook_message,
                                tenant_id: tenant_id,
                            };

                            if let Some(webhook_processor) = webhook_processor {
                                webhook_processor.cast(webhook_processor_message).expect(
                                    "Could not send close channel message to webhook processor",
                                );
                            }
                        }
                    });
                }
            })
            .detach();

        channel
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::time::Duration;

    use ractor::{Actor, ActorProcessingErr};

    use crate::channel::tests::create_participant;

    use super::*;

    struct TestWebhookActor;

    #[derive(Default)]
    struct TestWebhookActorState {
        new_channel_message: Option<NewChannel>,
        new_participant_message: Option<NewParticipant>,
        remove_participant_message: Option<RemoveParticipant>,
    }

    impl Actor for TestWebhookActor {
        type Msg = WebhookProcessorMessage;
        type Arguments = Arc<Mutex<TestWebhookActorState>>;
        type State = Arc<Mutex<TestWebhookActorState>>;

        async fn pre_start(
            &self,
            _this_actor: ActorRef<Self::Msg>,
            args: Self::Arguments,
        ) -> Result<Self::State, ActorProcessingErr> {
            Ok(args)
        }

        async fn handle(
            &self,
            _myself: ActorRef<Self::Msg>,
            message: Self::Msg,
            state: &mut Self::State,
        ) -> Result<(), ActorProcessingErr> {
            match message.msg {
                WebhookMessage::NewChannel(data) => {
                    state.lock().await.new_channel_message.replace(data);
                }
                WebhookMessage::NewParticipant(data) => {
                    state.lock().await.new_participant_message.replace(data);
                }
                WebhookMessage::RemoveParticipant(data) => {
                    state.lock().await.remove_participant_message.replace(data);
                }
                _ => {}
            }

            Ok(())
        }
    }

    #[tokio::test]
    async fn it_creates_new_channel() {
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, Arc::default())
                .await
                .expect("Actor failed to start");

        let channel_registry = ChannelRegistry::new(Some(webhook_processor.clone()));

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));
        }

        // Give time for the drop handler to process before we close the processor to stop a panic inside of the channel registry callback for channel close
        tokio::time::sleep(Duration::from_nanos(1)).await;
        webhook_processor.drain().unwrap();
        webhook_processor_handle.await.unwrap();
    }

    #[tokio::test]
    async fn it_uses_existing_channel_if_exists() {
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, Arc::default())
                .await
                .expect("Actor failed to start");

        let channel_registry = ChannelRegistry::new(Some(webhook_processor.clone()));

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));

            let _ = channel_registry
                .get_or_create_channel("Test".into(), None)
                .await;

            assert_eq!(channel_registry.channels.lock().await.len(), 1);
        }

        // Give time for the drop handler to process before we close the processor to stop a panic inside of the channel registry callback for channel close
        tokio::time::sleep(Duration::from_nanos(1)).await;
        webhook_processor.drain().unwrap();
        webhook_processor_handle.await.unwrap();
    }

    #[tokio::test]
    async fn it_creates_new_channel_if_old_was_dropped() {
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, Arc::default())
                .await
                .expect("Actor failed to start");

        let channel_registry = ChannelRegistry::new(Some(webhook_processor.clone()));

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));
            assert_eq!(channel_registry.channels.lock().await.len(), 1);
        }

        assert!(channel_registry.channels.lock().await.contains_key("Test"));

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));
            assert_eq!(channel_registry.channels.lock().await.len(), 1);
        }

        // Give time for the drop handler to process before we close the processor to stop a panic inside of the channel registry callback for channel close
        tokio::time::sleep(Duration::from_nanos(1)).await;
        webhook_processor.drain().unwrap();
        webhook_processor_handle.await.unwrap();
    }

    #[tokio::test]
    async fn it_sets_callbacks_for_participant_added() {
        let webhook_processor_state: Arc<Mutex<TestWebhookActorState>> = Arc::default();
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, webhook_processor_state.clone())
                .await
                .expect("Actor failed to start");

        let channel_registry = ChannelRegistry::new(Some(webhook_processor.clone()));

        {
            let channel = channel_registry
                .get_or_create_channel("Test".into(), None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));

            let (participant, _reciever) = create_participant("participant");

            channel.add_participant(participant.id.clone(), participant.downgrade());

            // Let the webhook processor do its thing
            tokio::time::sleep(Duration::from_nanos(1)).await;

            let webhook_processor_state = webhook_processor_state.lock().await;

            assert!(webhook_processor_state.new_participant_message.is_some());

            assert_eq!(
                webhook_processor_state
                    .new_participant_message
                    .clone()
                    .unwrap()
                    .channel_id,
                "Test"
            );

            assert_eq!(
                webhook_processor_state
                    .new_participant_message
                    .clone()
                    .unwrap()
                    .participant_id,
                "participant"
            );
        }

        // Give time for the drop handler to process before we close the processor to stop a panic inside of the channel registry callback for channel close
        tokio::time::sleep(Duration::from_nanos(1)).await;
        webhook_processor.drain().unwrap();
        webhook_processor_handle.await.unwrap();
    }

    #[tokio::test]
    async fn it_sets_callbacks_for_participant_removed() {
        let webhook_processor_state: Arc<Mutex<TestWebhookActorState>> = Arc::default();
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, webhook_processor_state.clone())
                .await
                .expect("Actor failed to start");

        let channel_registry = ChannelRegistry::new(Some(webhook_processor.clone()));

        {
            let channel = channel_registry
                .get_or_create_channel("Test".into(), None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));

            let (participant, _reciever) = create_participant("participant");

            channel.add_participant(participant.id.clone(), participant.downgrade());
            channel.remove_participant(&participant.id);

            // Let the webhook processor do its thing
            tokio::time::sleep(Duration::from_nanos(1)).await;

            let webhook_processor_state = webhook_processor_state.lock().await;

            assert!(webhook_processor_state.remove_participant_message.is_some());

            assert_eq!(
                webhook_processor_state
                    .remove_participant_message
                    .clone()
                    .unwrap()
                    .channel_id,
                "Test"
            );

            assert_eq!(
                webhook_processor_state
                    .remove_participant_message
                    .clone()
                    .unwrap()
                    .participant_id,
                "participant"
            );
        }

        // Give time for the drop handler to process before we close the processor to stop a panic inside of the channel registry callback for channel close
        tokio::time::sleep(Duration::from_nanos(1)).await;
        webhook_processor.drain().unwrap();
        webhook_processor_handle.await.unwrap();
    }
}
