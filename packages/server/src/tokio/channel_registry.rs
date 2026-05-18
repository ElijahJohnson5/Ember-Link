use futures_util::lock::Mutex;
use protocol::StorageType;

use protocol::{CloseChannel, NewChannel};
use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error as StdError,
    sync::Arc,
};
use tracing::instrument;

use crate::{
    handler::{MessageHandler, MessageHandlerRegistry},
    observer::{ChannelEvent, ObserverContext},
    storage::{yjs::init_storage, Storage},
    tokio::channel::{now_millis, TokioChannel, WeakTokioChannel},
    tokio::config::TokioConfig,
    tokio::observer_bus::ObserverBus,
};

pub struct ChannelRegistry {
    channels: Arc<Mutex<HashMap<String, WeakTokioChannel>>>,
    observers: ObserverBus,
    message_handlers: Arc<MessageHandlerRegistry>,
    config: TokioConfig,
}

impl ChannelRegistry {
    pub fn message_handlers(&self) -> Arc<MessageHandlerRegistry> {
        self.message_handlers.clone()
    }
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
    observers: ObserverBus,
    handlers: MessageHandlerRegistry,
}

impl ChannelRegistryBuilder {
    #[must_use]
    pub fn new(config: TokioConfig) -> Self {
        Self {
            config,
            observers: ObserverBus::noop(),
            handlers: MessageHandlerRegistry::new(),
        }
    }

    pub(crate) fn with_observers(mut self, observers: ObserverBus) -> Self {
        self.observers = observers;
        self
    }

    pub fn with_handler(mut self, handler: Arc<dyn MessageHandler>) -> Self {
        self.handlers.insert(handler);
        self
    }

    pub fn build(self) -> ChannelRegistry {
        ChannelRegistry {
            channels: Arc::default(),
            observers: self.observers,
            message_handlers: Arc::new(self.handlers),
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
                    tracing::debug!("Found existing channel");
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

        let channel = TokioChannel::new(
            unique_name,
            channel_name.clone(),
            tenant_id.clone(),
            storage,
            self.observers.clone(),
        );

        match entry {
            Entry::Occupied(mut entry) => {
                entry.insert(channel.downgrade());
            }
            Entry::Vacant(entry) => {
                entry.insert(channel.downgrade());
            }
        }

        let observer_ctx = ObserverContext {
            tenant_id: tenant_id.clone(),
        };

        self.observers.emit(
            ChannelEvent::NewChannel(NewChannel {
                id: uuid::Uuid::new_v4().into(),
                channel_name: channel_name.clone(),
                timestamp: now_millis(),
                num_channels: old_num_channels + 1,
            }),
            observer_ctx.clone(),
        );

        channel
            .on_close({
                let channels = self.channels.clone();
                let observers = self.observers.clone();
                let close_channel_name = channel_name.clone();
                let close_observer_ctx = observer_ctx;

                move || {
                    tokio::spawn(async move {
                        // The channel is in Drop right now, so every weak ref
                        // to it has already become invalid. Sweep the map of
                        // any such stale entries to recover the slot.
                        let num = {
                            let mut channels = channels.lock().await;
                            channels.retain(|_, weak_ref| weak_ref.upgrade().is_some());
                            channels.len()
                        };

                        observers.emit(
                            ChannelEvent::CloseChannel(CloseChannel {
                                id: uuid::Uuid::new_v4().into(),
                                channel_name: close_channel_name,
                                timestamp: now_millis(),
                                num_channels: num,
                            }),
                            close_observer_ctx,
                        );
                    });
                }
            })
            .detach();

        channel
    }
}

#[cfg(test)]
mod tests {
    use crate::observer::Observer;
    use async_trait::async_trait;
    use envconfig::Envconfig;
    use parking_lot::Mutex as PlMutex;
    use protocol::{NewParticipant, RemoveParticipant};
    use tokio::task::yield_now;

    use crate::tokio::observer_bus::start_observer_bus;

    use super::*;

    #[derive(Default)]
    struct TestObserverState {
        new_channel: Option<NewChannel>,
        new_participant: Option<NewParticipant>,
        remove_participant: Option<RemoveParticipant>,
    }

    struct TestObserver {
        state: Arc<PlMutex<TestObserverState>>,
    }

    #[async_trait]
    impl Observer for TestObserver {
        async fn on_event(&self, event: ChannelEvent, _ctx: ObserverContext) {
            let mut state = self.state.lock();
            match event {
                ChannelEvent::NewChannel(data) => {
                    state.new_channel.replace(data);
                }
                ChannelEvent::NewParticipant(data) => {
                    state.new_participant.replace(data);
                }
                ChannelEvent::RemoveParticipant(data) => {
                    state.remove_participant.replace(data);
                }
                _ => {}
            }
        }
    }

    fn create_config() -> TokioConfig {
        let mut config_values = HashMap::new();

        #[cfg(feature = "webhook")]
        {
            config_values.insert("WEBHOOK_URL".into(), "fakeurl".into());
            config_values.insert("WEBHOOK_SECRET_KEY".into(), "secret".into());
        }

        TokioConfig::init_from_hashmap(&config_values).unwrap()
    }

    fn create_channel_registry_with_observer(
        observer_state: Arc<PlMutex<TestObserverState>>,
    ) -> (ChannelRegistry, crate::tokio::observer_bus::ObserverBusHandle) {
        let (bus, handle) = start_observer_bus(vec![Arc::new(TestObserver {
            state: observer_state,
        })]);
        let registry = ChannelRegistryBuilder::new(create_config())
            .with_observers(bus)
            .build();
        (registry, handle)
    }

    fn create_channel_registry() -> ChannelRegistry {
        ChannelRegistryBuilder::new(create_config()).build()
    }

    #[tokio::test]
    async fn it_creates_new_channel() {
        let channel_registry = create_channel_registry();

        let _ = channel_registry
            .get_or_create_channel("Test".into(), "Test".into(), None, None)
            .await;

        assert!(channel_registry.channels.lock().await.contains_key("Test"));
        yield_now().await;
    }

    #[tokio::test]
    async fn it_uses_existing_channel_if_exists() {
        let channel_registry = create_channel_registry();

        let _ = channel_registry
            .get_or_create_channel("Test".into(), "Test".into(), None, None)
            .await;
        let _ = channel_registry
            .get_or_create_channel("Test".into(), "Test".into(), None, None)
            .await;

        assert_eq!(channel_registry.channels.lock().await.len(), 1);
        yield_now().await;
    }

    #[tokio::test]
    async fn it_creates_new_channel_if_old_was_dropped() {
        let channel_registry = create_channel_registry();

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), "Test".into(), None, None)
                .await;
            assert_eq!(channel_registry.channels.lock().await.len(), 1);
        }

        {
            let _ = channel_registry
                .get_or_create_channel("Test".into(), "Test".into(), None, None)
                .await;
            assert_eq!(channel_registry.channels.lock().await.len(), 1);
        }
        yield_now().await;
    }

    #[tokio::test]
    async fn it_emits_new_channel_to_observer() {
        let observer_state: Arc<PlMutex<TestObserverState>> = Arc::default();
        let (channel_registry, _bus_handle) =
            create_channel_registry_with_observer(observer_state.clone());

        let _ = channel_registry
            .get_or_create_channel("Test".into(), "Test".into(), None, None)
            .await;

        for _ in 0..10 {
            yield_now().await;
            if observer_state.lock().new_channel.is_some() {
                break;
            }
        }

        assert!(observer_state.lock().new_channel.is_some());
        assert_eq!(
            observer_state
                .lock()
                .new_channel
                .as_ref()
                .unwrap()
                .channel_name,
            "Test"
        );
    }

    #[tokio::test]
    async fn it_emits_participant_added_to_observer() {
        use crate::tokio::participant::actor::tests::create_participant;
        use std::time::Duration;

        let observer_state: Arc<PlMutex<TestObserverState>> = Arc::default();
        let (channel_registry, _bus_handle) =
            create_channel_registry_with_observer(observer_state.clone());

        let channel = channel_registry
            .get_or_create_channel("Test".into(), "Test".into(), None, None)
            .await
            .unwrap();

        let (participant, _state) = create_participant().await;
        channel.add_participant("participant".into(), participant);

        for _ in 0..10 {
            yield_now().await;
            if observer_state.lock().new_participant.is_some() {
                break;
            }
        }

        let state = observer_state.lock();
        assert!(state.new_participant.is_some());
        let msg = state.new_participant.as_ref().unwrap();
        assert_eq!(msg.channel_name, "Test");
        assert_eq!(msg.participant_id, "participant");

        drop(state);
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    #[tokio::test]
    async fn it_emits_participant_removed_to_observer() {
        use crate::tokio::participant::actor::tests::create_participant;

        let observer_state: Arc<PlMutex<TestObserverState>> = Arc::default();
        let (channel_registry, _bus_handle) =
            create_channel_registry_with_observer(observer_state.clone());

        let channel = channel_registry
            .get_or_create_channel("Test".into(), "Test".into(), None, None)
            .await
            .unwrap();

        let (participant, _state) = create_participant().await;
        channel.add_participant("participant".into(), participant);
        channel.remove_participant("participant");

        for _ in 0..10 {
            yield_now().await;
            if observer_state.lock().remove_participant.is_some() {
                break;
            }
        }

        let state = observer_state.lock();
        assert!(state.remove_participant.is_some());
        let msg = state.remove_participant.as_ref().unwrap();
        assert_eq!(msg.channel_name, "Test");
        assert_eq!(msg.participant_id, "participant");
    }
}
