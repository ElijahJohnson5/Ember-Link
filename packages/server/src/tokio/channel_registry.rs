use futures_util::lock::Mutex;
use protocol::StorageType;

#[cfg(feature = "webhook")]
use protocol::{
    CloseChannel, NewChannel, NewParticipant, RemoveParticipant, StorageUpdated, WebhookMessage,
};
#[cfg(feature = "webhook")]
use ractor::ActorRef;
use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error as StdError,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::instrument;

use crate::{
    storage::{yjs::init_storage, Storage},
    tokio::channel::{TokioChannel, WeakTokioChannel},
    tokio::config::TokioConfig,
};

#[cfg(feature = "webhook")]
use crate::tokio::webhook_processor::actor::WebhookProcessorMessage;

pub struct ChannelRegistry {
    channels: Arc<Mutex<HashMap<String, WeakTokioChannel>>>,
    #[cfg(feature = "webhook")]
    webhook_processor: ActorRef<WebhookProcessorMessage>,
    config: TokioConfig,
}

pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ChannelError {
    #[error("Channel creation failed: {0}")]
    CreationError(#[source] BoxDynError),
}

pub struct ChannelRegistryBuilder {
    config: TokioConfig,
    #[cfg(feature = "webhook")]
    webhook_processor: Option<ActorRef<WebhookProcessorMessage>>,
}

impl ChannelRegistryBuilder {
    #[must_use]
    pub fn new(config: TokioConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "webhook")]
            webhook_processor: None,
        }
    }

    #[cfg(feature = "webhook")]
    pub fn with_webhook_processor(
        mut self,
        webhook_processor: ActorRef<WebhookProcessorMessage>,
    ) -> Self {
        self.webhook_processor.replace(webhook_processor);
        self
    }

    pub fn build(self) -> ChannelRegistry {
        ChannelRegistry {
            channels: Arc::default(),
            #[cfg(feature = "webhook")]
            webhook_processor: self
                .webhook_processor
                .expect("You must provide a webhook processor when the webhook feature is enabled"),
            config: self.config,
        }
    }
}

impl ChannelRegistry {
    #[instrument(skip(self))]
    pub async fn get_or_create_channel(
        &self,
        unique_name: String,
        channel_name: String,
        storage_type: Option<StorageType>,
        tenant_id: Option<String>,
    ) -> Result<TokioChannel, ChannelError> {
        let mut channels = self.channels.lock().await;

        let len = channels.len();

        let channel = match channels.entry(unique_name.clone()) {
            Entry::Occupied(entry) => match entry.get().upgrade() {
                Some(channel) => {
                    tracing::info!("Found existing channel");
                    channel
                }
                None => {
                    let channel = self.create_channel(
                        Entry::Occupied(entry),
                        unique_name,
                        channel_name,
                        storage_type,
                        &tenant_id,
                        len,
                    );

                    channel
                        .init_storage(&self.config.base_config.storage_endpoint, &tenant_id)
                        .await
                        .map_err(|e| ChannelError::CreationError(Box::new(e)))?;

                    channel
                }
            },
            entry => {
                let channel = self.create_channel(
                    entry,
                    unique_name,
                    channel_name,
                    storage_type,
                    &tenant_id,
                    len,
                );

                channel
                    .init_storage(&self.config.base_config.storage_endpoint, &tenant_id)
                    .await
                    .map_err(|e| ChannelError::CreationError(Box::new(e)))?;

                channel
            }
        };

        Ok(channel)
    }

    #[instrument(skip(self))]
    pub async fn get_channel(&self, channel_name: &String) -> Option<WeakTokioChannel> {
        let channels = self.channels.lock().await;

        channels.get(channel_name).cloned()
    }

    #[instrument(skip(self, entry, old_num_channels))]
    fn create_channel(
        &self,
        entry: Entry<'_, String, WeakTokioChannel>,
        unique_name: String,
        channel_name: String,
        storage_type: Option<StorageType>,
        tenant_id: &Option<String>,
        old_num_channels: usize,
    ) -> TokioChannel {
        let storage = storage_type.map(|t| match t {
            StorageType::Yjs => {
                let yjs_storage: Box<dyn Storage + Send + Sync> = Box::new(init_storage());

                yjs_storage
            }
        });

        let channel = TokioChannel::new(channel_name.clone(), storage);

        match entry {
            Entry::Occupied(mut entry) => {
                entry.insert(channel.downgrade());
            }
            Entry::Vacant(entry) => {
                entry.insert(channel.downgrade());
            }
        }

        #[cfg(feature = "webhook")]
        self.setup_webhook_callbacks(&channel, &channel_name, tenant_id, old_num_channels);

        channel
            .on_close({
                let channel_name = channel_name.clone();
                let channels = self.channels.clone();
                #[cfg(feature = "webhook")]
                let webhook_processor = self.webhook_processor.clone();
                #[cfg(feature = "webhook")]
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
                                channels.remove(&channel_name);

                                channels.len()
                            };

                            #[cfg(feature = "webhook")]
                            {
                                let webhook_message = WebhookMessage::CloseChannel(CloseChannel {
                                    id: uuid::Uuid::new_v4().into(),
                                    channel_name,
                                    timestamp: since_the_epoch.as_millis() as u64,
                                    num_channels: num,
                                });

                                let webhook_processor_message = WebhookProcessorMessage {
                                    msg: webhook_message,
                                    tenant_id: tenant_id,
                                };

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

    #[cfg(feature = "webhook")]
    fn setup_webhook_callbacks(
        &self,
        channel: &TokioChannel,
        channel_name: &String,
        tenant_id: &Option<String>,
        old_num_channels: usize,
    ) {
        let webhook_processor = &self.webhook_processor;

        tracing::info!("Setting up webhook callbacks");

        channel
            .on_participant_added({
                let channel_name = channel_name.clone();
                let webhook_processor = webhook_processor.clone();
                let tenant_id = tenant_id.clone();

                move |participant_id, num_participants| {
                    let start = SystemTime::now();
                    let since_the_epoch = start
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");

                    let webhook_message = WebhookMessage::NewParticipant(NewParticipant {
                        id: uuid::Uuid::new_v4().into(),
                        channel_name: channel_name.clone(),
                        timestamp: since_the_epoch.as_millis() as u64,
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
                let channel_name = channel_name.clone();
                let webhook_processor = webhook_processor.clone();
                let tenant_id = tenant_id.clone();

                move |participant_id, num_participants| {
                    let start = SystemTime::now();
                    let since_the_epoch = start
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");

                    let webhook_message = WebhookMessage::RemoveParticipant(RemoveParticipant {
                        id: uuid::Uuid::new_v4().into(),
                        channel_name: channel_name.clone(),
                        timestamp: since_the_epoch.as_millis() as u64,
                        participant_id: participant_id.clone(),
                        num_pariticipants: *num_participants,
                    });

                    let webhook_processor_message = WebhookProcessorMessage {
                        msg: webhook_message,
                        tenant_id: tenant_id.clone(),
                    };

                    webhook_processor
                        .cast(webhook_processor_message)
                        .expect("Could not send remove participant message to webhook processor")
                }
            })
            .detach();

        channel
            .on_storage_updated({
                let channel_name = channel_name.clone();
                let webhook_processor = webhook_processor.clone();
                let tenant_id = tenant_id.clone();

                move |update| {
                    let start = SystemTime::now();
                    let since_the_epoch = start
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");

                    let webhook_message = WebhookMessage::StorageUpdated(StorageUpdated {
                        id: uuid::Uuid::new_v4().into(),
                        channel_name: channel_name.clone(),
                        timestamp: since_the_epoch.as_millis() as u64,
                        data: update.clone(),
                    });

                    let webhook_processor_message = WebhookProcessorMessage {
                        msg: webhook_message,
                        tenant_id: tenant_id.clone(),
                    };

                    webhook_processor
                        .cast(webhook_processor_message)
                        .expect("Could not send remove participant message to webhook processor")
                }
            })
            .detach();

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let webhook_message = WebhookMessage::NewChannel(NewChannel {
            id: uuid::Uuid::new_v4().into(),
            channel_name: channel_name.clone(),
            timestamp: since_the_epoch.as_millis() as u64,
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
}

#[cfg(test)]
mod tests {
    use envconfig::Envconfig;
    #[cfg(feature = "webhook")]
    use ractor::{Actor, ActorProcessingErr};
    use tokio::task::yield_now;

    use super::*;

    struct TestWebhookActor;

    #[derive(Default)]
    #[cfg(feature = "webhook")]
    struct TestWebhookActorState {
        new_channel_message: Option<NewChannel>,
        new_participant_message: Option<NewParticipant>,
        remove_participant_message: Option<RemoveParticipant>,
    }

    #[cfg(feature = "webhook")]
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

    fn create_config() -> TokioConfig {
        let config_values = HashMap::new();

        TokioConfig::init_from_hashmap(&config_values).unwrap()
    }

    fn create_channel_registry(
        #[cfg(feature = "webhook")] webhook_processor: ActorRef<WebhookProcessorMessage>,
    ) -> ChannelRegistry {
        let config = create_config();
        let builder = ChannelRegistryBuilder::new(config);

        #[cfg(feature = "webhook")]
        let builder = builder.with_webhook_processor(webhook_processor.clone());

        builder.build()
    }

    #[tokio::test]
    async fn it_creates_new_channel() {
        #[cfg(feature = "webhook")]
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, Arc::default())
                .await
                .expect("Actor failed to start");

        #[cfg(not(feature = "webhook"))]
        let channel_registry = create_channel_registry();

        #[cfg(feature = "webhook")]
        let channel_registry = create_channel_registry(webhook_processor.clone());

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), "Test".into(), None, None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));
        }

        // Give time for the drop handler to process before we close the processor to stop a panic inside of the channel registry callback for channel close
        yield_now().await;

        #[cfg(feature = "webhook")]
        {
            webhook_processor.drain().unwrap();
            webhook_processor_handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn it_uses_existing_channel_if_exists() {
        #[cfg(feature = "webhook")]
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, Arc::default())
                .await
                .expect("Actor failed to start");

        #[cfg(not(feature = "webhook"))]
        let channel_registry = create_channel_registry();

        #[cfg(feature = "webhook")]
        let channel_registry = create_channel_registry(webhook_processor.clone());

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), "Test".into(), None, None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));

            let _ = channel_registry
                .get_or_create_channel("Test".into(), "Test".into(), None, None)
                .await;

            assert_eq!(channel_registry.channels.lock().await.len(), 1);
        }

        // Give time for the drop handler to process before we close the processor to stop a panic inside of the channel registry callback for channel close
        yield_now().await;

        #[cfg(feature = "webhook")]
        {
            webhook_processor.drain().unwrap();
            webhook_processor_handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn it_creates_new_channel_if_old_was_dropped() {
        #[cfg(feature = "webhook")]
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, Arc::default())
                .await
                .expect("Actor failed to start");

        #[cfg(not(feature = "webhook"))]
        let channel_registry = create_channel_registry();

        #[cfg(feature = "webhook")]
        let channel_registry = create_channel_registry(webhook_processor.clone());

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), "Test".into(), None, None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));
            assert_eq!(channel_registry.channels.lock().await.len(), 1);
        }

        assert!(channel_registry.channels.lock().await.contains_key("Test"));

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), "Test".into(), None, None)
                .await;

            assert!(channel_registry.channels.lock().await.contains_key("Test"));
            assert_eq!(channel_registry.channels.lock().await.len(), 1);
        }

        // Give time for the drop handler to process before we close the processor to stop a panic inside of the channel registry callback for channel close
        yield_now().await;

        #[cfg(feature = "webhook")]
        {
            webhook_processor.drain().unwrap();
            webhook_processor_handle.await.unwrap();
        }
    }

    #[tokio::test]
    #[cfg(feature = "webhook")]
    async fn it_sets_callbacks_for_participant_added() {
        use crate::tokio::{participant::actor::tests::create_participant, webhook_processor};
        use std::time::Duration;

        let webhook_processor_state: Arc<Mutex<TestWebhookActorState>> = Arc::default();
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, Arc::default())
                .await
                .expect("Actor failed to start");

        let channel_registry = create_channel_registry(webhook_processor.clone());

        {
            let channel = channel_registry
                .get_or_create_channel("Test".into(), "Test".into(), None, None)
                .await
                .unwrap();

            assert!(channel_registry.channels.lock().await.contains_key("Test"));

            let (participant, _state) = create_participant().await;

            let participant_id: String = "participant".into();

            channel.add_participant(participant_id, participant);

            // Let the webhook processor do its thing
            yield_now().await;

            let webhook_processor_state = webhook_processor_state.lock().await;

            assert!(webhook_processor_state.new_participant_message.is_some());

            assert_eq!(
                webhook_processor_state
                    .new_participant_message
                    .clone()
                    .unwrap()
                    .channel_name,
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
    #[cfg(feature = "webhook")]
    async fn it_sets_callbacks_for_participant_removed() {
        use crate::tokio::{participant::actor::tests::create_participant, webhook_processor};
        use std::time::Duration;

        let webhook_processor_state: Arc<Mutex<TestWebhookActorState>> = Arc::default();
        let (webhook_processor, webhook_processor_handle) =
            Actor::spawn(None, TestWebhookActor, webhook_processor_state.clone())
                .await
                .expect("Actor failed to start");

        let channel_registry = create_channel_registry(webhook_processor.clone());

        {
            let channel = channel_registry
                .get_or_create_channel("Test".into(), "Test".into(), None, None)
                .await
                .unwrap();

            assert!(channel_registry.channels.lock().await.contains_key("Test"));

            let (participant, _state) = create_participant().await;

            let participant_id: String = "participant".into();

            channel.add_participant(participant_id.clone(), participant);
            channel.remove_participant(&participant_id);

            // Let the webhook processor do its thing
            yield_now().await;

            let webhook_processor_state = webhook_processor_state.lock().await;

            assert!(webhook_processor_state.remove_participant_message.is_some());

            assert_eq!(
                webhook_processor_state
                    .remove_participant_message
                    .clone()
                    .unwrap()
                    .channel_name,
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
        yield_now().await;
        webhook_processor.drain().unwrap();
        webhook_processor_handle.await.unwrap();
    }
}
