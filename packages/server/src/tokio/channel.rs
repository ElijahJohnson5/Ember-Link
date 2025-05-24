use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Weak},
};

use parking_lot::{Mutex, RwLock};
use protocol::{
    InitialPresenceMessage, ServerMessage, ServerPresenceMessage,
    StorageSyncMessage, StorageUpdateMessage,
};
use ractor::ActorRef;

use crate::{
    channel::Channel,
    storage::{yjs::YjsStorage, Storage, StorageError},
    tokio::{
        event_listener_primitives::{Bag, BagOnce, HandlerId},
        participant::actor::ParticipantMessage,
    },
};

#[derive(Default)]
struct Handlers {
    participant_added: Bag<Arc<dyn Fn(&String, &usize) + Send + Sync>, String, usize>,
    participant_removed: Bag<Arc<dyn Fn(&String, &usize) + Send + Sync>, String, usize>,
    storage_updated: Bag<Arc<dyn Fn(&Vec<u8>) + Send + Sync>, Vec<u8>>,
    closed: BagOnce<Box<dyn FnOnce() + Send>>,
}

struct Inner {
    id: String,
    storage: Option<Box<dyn Storage + Sync + Send + 'static>>,
    yjs_provider_storage: YjsStorage,
    participant_refs: RwLock<HashMap<String, ActorRef<ParticipantMessage>>>,
    participant_presence_state: Mutex<HashMap<String, (String, i32)>>,
    handlers: Handlers,
}

impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Inner").field("id", &self.id).finish()
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        tracing::info!("Channel {} closed", self.id);

        self.handlers.closed.call_simple();
    }
}

#[derive(Debug, Clone)]
pub struct TokioChannel {
    inner: Arc<Inner>,
}

impl Channel for TokioChannel {
    fn broadcast(&self, message: ServerMessage, exclude_id: Option<&String>) {
        let participant_refs = self.inner.participant_refs.read();

        let data = serde_bare::to_vec(&message).unwrap();

        for (key, value) in participant_refs.iter() {
            if exclude_id.is_some_and(|id| *id == *key) {
                continue;
            }
            
            match value.cast(ParticipantMessage::ServerBinaryMessage {
                data: data.clone(),
            }) {
                Err(e) => {
                    tracing::warn!(
                        error = e.to_string(),
                        participant_id = key,
                        "Could not send message to participant in channel",
                    );
                }
                Ok(()) => {}
            }
        }
    }

    fn handle_storage_sync_message(
        &self,
        message: StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, StorageError> {
        if let Some(storage) = &self.inner.storage {
            return storage.handle_sync_message(&message);
        }

        Ok(None)
    }

    fn handle_storage_update_message(
        &self,
        message: StorageUpdateMessage,
        participant_id: &String,
    ) -> Result<(), StorageError> {
        if let Some(storage) = &self.inner.storage {
            storage.handle_update_message(&message)?;
        }

        self.inner
            .handlers
            .storage_updated
            .call_simple(&message.update);

        self.broadcast(ServerMessage::StorageUpdateMessage(message), Some(&participant_id));

        Ok(())
    }

    fn handle_provider_sync_message(
        &self,
        message: StorageSyncMessage,
    ) -> Result<Option<Vec<StorageSyncMessage>>, StorageError> {
        return self
            .inner
            .yjs_provider_storage
            .handle_sync_message(&message);
    }

    fn handle_provider_update_message(
        &self,
        message: StorageUpdateMessage,
        participant_id: &String,
    ) -> Result<(), StorageError> {
        self.inner
            .yjs_provider_storage
            .handle_update_message(&message)?;

        self.inner
            .handlers
            .storage_updated
            .call_simple(&message.update);

        self.broadcast(
            ServerMessage::ProviderUpdateMessage(message),
            Some(&participant_id),
        );

        Ok(())
    }
}

impl TokioChannel {
    pub fn new(id: String, storage: Option<Box<dyn Storage + Send + Sync>>) -> Self {
        tracing::info!("Creating channel {}", id);

        Self {
            inner: Arc::new(Inner {
                id,
                storage,
                yjs_provider_storage: YjsStorage::new(yrs::Doc::new()),
                participant_refs: RwLock::new(HashMap::new()),
                participant_presence_state: Mutex::default(),
                handlers: Handlers::default(),
            }),
        }
    }

    pub fn add_presence(&self, participant_id: String, state: String, clock: i32) {
        self.inner
            .participant_presence_state
            .lock()
            .insert(participant_id, (state, clock));
    }

    pub fn add_participant(
        &self,
        participant_id: String,
        participant: ActorRef<ParticipantMessage>,
    ) {
        let num_participants = {
            let mut pariticpants = self.inner.participant_refs.write();

            pariticpants.insert(participant_id.clone(), participant.clone());

            pariticpants.len()
        };

        self.inner
            .handlers
            .participant_added
            .call_simple(&participant_id, &num_participants);

        participant
            .cast(ParticipantMessage::ServerMessage {
                data: serde_json::to_string(&ServerMessage::InitialPresenceMessage(self.initial_presence_message())).unwrap(),
            })
            .expect("Could not send message to participant");
    }

    pub fn remove_participant(&self, participant_id: &str) {
        let num_participants = {
            let mut pariticpants = self.inner.participant_refs.write();

            pariticpants.remove(participant_id);

            pariticpants.len()
        };

        let state = { self
            .inner
            .participant_presence_state
            .lock()
            .remove(participant_id)
        };

        self.inner
            .handlers
            .participant_removed
            .call_simple(&participant_id.to_string(), &num_participants);

        match state {
            Some((_, clock)) => {
                self.broadcast(
                    ServerMessage::ServerPresenceMessage(ServerPresenceMessage {
                        clock: clock,
                        presence: None,
                        id: participant_id.to_string(),
                    }),
                    Some(&participant_id.to_string()),
                );
            }
            None => {}
        }
    }

    pub fn on_participant_added<F: Fn(&String, &usize) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) -> HandlerId {
        self.inner
            .handlers
            .participant_added
            .add(Arc::new(callback))
    }

    pub fn on_storage_updated<F: Fn(&Vec<u8>) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) -> HandlerId {
        self.inner.handlers.storage_updated.add(Arc::new(callback))
    }

    pub fn on_participant_removed<F: Fn(&String, &usize) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) -> HandlerId {
        self.inner
            .handlers
            .participant_removed
            .add(Arc::new(callback))
    }

    pub fn on_close<F: FnOnce() + Send + 'static>(&self, callback: F) -> HandlerId {
        self.inner.handlers.closed.add(Box::new(callback))
    }

    pub async fn init_storage(
        &self,
        storage_endpoint: &Option<String>,
        tenant_id: &Option<String>,
    ) -> Result<(), StorageError> {
        if let Some(storage) = &self.inner.storage {
            storage
                .init_storage_from_endpoint(&self.inner.id, storage_endpoint, tenant_id)
                .await?;
        }

        Ok(())
    }

    fn initial_presence_message(&self) -> InitialPresenceMessage {
        let mut presences: Vec<ServerPresenceMessage> = Vec::default();

        let participant_presence_state = {
            self.inner.participant_presence_state.lock().clone()
        };

        for (key, (state, clock)) in participant_presence_state.iter() {
            presences.push(ServerPresenceMessage {
                id: key.clone(),
                clock: clock.clone(),
                presence: Some(state.clone()),
            });
        }

        InitialPresenceMessage { presences }
    }

    /// Get `WeakChannel` that can later be upgraded to `Channel`, but will not prevent channel from
    /// being destroyed
    pub fn downgrade(&self) -> WeakTokioChannel {
        WeakTokioChannel {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

/// Similar to `TokioChannel`, but doesn't prevent channel from being destroyed
#[derive(Debug, Clone)]
pub struct WeakTokioChannel {
    inner: Weak<Inner>,
}

impl WeakTokioChannel {
    /// Upgrade `WeakChannel` to `Channel`, may return `None` if underlying Channel was destroyed already
    pub fn upgrade(&self) -> Option<TokioChannel> {
        self.inner.upgrade().map(|inner| TokioChannel { inner })
    }
}

#[cfg(test)]
pub mod tests {
    use crate::tokio::participant::actor::tests::{create_participant, TestParticipantActorState};
    use tokio::{sync::Mutex, task::yield_now};

    use protocol::StorageUpdateMessage;

    use super::*;

    async fn get_nth_message(
        state: Arc<Mutex<TestParticipantActorState>>,
        n: usize,
    ) -> Option<ParticipantMessage> {
        state.lock().await.messages.get(n).cloned()
    }

    #[tokio::test]
    async fn it_adds_a_participant_and_sends_initial_presence() {
        let channel = TokioChannel::new("test".to_string(), None);
        let (participant, state) = create_participant().await;

        let participant_id: String = "participant".into();

        channel.add_participant(participant_id.clone(), participant);

        assert_eq!(channel.inner.participant_refs.read().len(), 1);
        assert!(channel
            .inner
            .participant_refs
            .read()
            .get(&participant_id)
            .is_some());

        yield_now().await;

        let message = get_nth_message(state, 0).await;

        assert!(message.is_some());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match serde_json::from_str::<protocol::ServerMessage>(&data).unwrap() {
                ServerMessage::InitialPresenceMessage(data) => {
                    assert_eq!(data.presences.len(), 0);
                }
                _ => {
                    panic!("Message was not an initial presence message")
                }
            },
            _ => {
                panic!("Message was not a server message")
            }
        }
    }

    struct HandlerCounter {
        count: u16,
    }

    #[tokio::test]
    async fn it_calls_participant_added_handler() {
        let channel = TokioChannel::new("test".to_string(), None);
        let (participant, _) = create_participant().await;

        let participant_id: String = "participant".into();

        let handler_counter = Arc::new(parking_lot::Mutex::new(HandlerCounter { count: 0 }));

        channel
            .on_participant_added({
                let handler_counter = handler_counter.clone();
                move |_participant_id, _size| {
                    handler_counter.lock().count += 1;
                }
            })
            .detach();

        channel.add_participant(participant_id, participant);

        assert_eq!(handler_counter.lock().count, 1);
    }

    #[tokio::test]
    async fn it_calls_participant_removed_handler() {
        let channel = TokioChannel::new("test".to_string(), None);
        let (participant, _) = create_participant().await;

        let participant_id: String = "participant".into();

        let handler_counter = Arc::new(parking_lot::Mutex::new(HandlerCounter { count: 0 }));

        channel
            .on_participant_removed({
                let handler_counter = handler_counter.clone();
                move |_participant_id, _size| {
                    handler_counter.lock().count += 1;
                }
            })
            .detach();

        channel.add_participant(participant_id.clone(), participant);
        channel.remove_participant(&participant_id);

        assert_eq!(handler_counter.lock().count, 1);
    }

    #[tokio::test]
    async fn it_calls_on_close_when_dropped() {
        let handler_counter = Arc::new(parking_lot::Mutex::new(HandlerCounter { count: 0 }));

        {
            let channel = TokioChannel::new("test".to_string(), None);

            channel
                .on_close({
                    let handler_counter = handler_counter.clone();
                    move || {
                        handler_counter.lock().count += 1;
                    }
                })
                .detach();
        }

        assert_eq!(handler_counter.lock().count, 1);
    }

    #[tokio::test]
    async fn it_broadcasts_to_all_participants() {
        let channel = TokioChannel::new("test".to_string(), None);
        let (participant1, state) = create_participant().await;

        let participant_id1: String = "participant1".into();

        let (participant2, state2) = create_participant().await;

        let participant_id2: String = "participant2".into();

        channel.add_participant(participant_id1.clone(), participant1);
        channel.add_participant(participant_id2.clone(), participant2);

        channel.broadcast(
            ServerMessage::StorageUpdateMessage(StorageUpdateMessage { update: vec![] }),
            None,
        );

        yield_now().await;

        let message = get_nth_message(state, 1).await;

        assert!(message.is_some());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match serde_json::from_str::<protocol::ServerMessage>(&data).unwrap() {
                ServerMessage::StorageUpdateMessage(data) => {
                    assert_eq!(data, StorageUpdateMessage { update: vec![] });
                }
                _ => {
                    panic!("Message is not ServerMessage::StorageUpdate")
                }
            },
            _ => {
                panic!("Message is not a ParticipantMessage::ServerMessage")
            }
        }

        let message = get_nth_message(state2, 1).await;

        assert!(message.is_some());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match serde_json::from_str::<protocol::ServerMessage>(&data).unwrap() {
                ServerMessage::StorageUpdateMessage(data) => {
                    assert_eq!(data, StorageUpdateMessage { update: vec![] });
                }
                _ => {
                    panic!("Message is not ServerMessage::StorageUpdate")
                }
            },
            _ => {
                panic!("Message is not a ParticipantMessage::ServerMessage")
            }
        }
    }

    #[tokio::test]
    async fn it_boradcasts_to_all_participants_except_excluded() {
        let channel = TokioChannel::new("test".to_string(), None);
        let (participant1, state) = create_participant().await;

        let participant_id1: String = "participant1".into();

        let (participant2, state2) = create_participant().await;

        let participant_id2: String = "participant2".into();

        channel.add_participant(participant_id1.clone(), participant1);
        channel.add_participant(participant_id2.clone(), participant2);

        channel.broadcast(
            ServerMessage::StorageUpdateMessage(StorageUpdateMessage { update: vec![] }),
            Some(&participant_id2),
        );

        yield_now().await;

        let message = get_nth_message(state, 1).await;

        assert!(message.is_some());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match serde_json::from_str::<protocol::ServerMessage>(&data).unwrap() {
                ServerMessage::StorageUpdateMessage(data) => {
                    assert_eq!(data, StorageUpdateMessage { update: vec![] });
                }
                _ => {
                    panic!("Message is not ServerMessage::StorageUpdate")
                }
            },
            _ => {
                panic!("Message is not a ParticipantMessage::ServerMessage")
            }
        }

        let message = get_nth_message(state2, 2).await;

        assert!(message.is_none());
    }

    #[tokio::test]
    async fn it_removes_participant() {
        let channel = TokioChannel::new("test".to_string(), None);
        let (participant, _) = create_participant().await;

        let participant_id: String = "participant".into();

        channel.add_participant(participant_id.clone(), participant);

        channel.remove_participant(&participant_id);

        assert_eq!(channel.inner.participant_refs.read().len(), 0);
        assert!(channel
            .inner
            .participant_refs
            .read()
            .get(&participant_id.clone())
            .is_none());
    }

    #[tokio::test]
    async fn it_broadcasts_removed_participant_presence() {
        let channel = TokioChannel::new("test".to_string(), None);
        let (participant1, _) = create_participant().await;

        let participant_id1: String = "participant1".into();

        let (participant2, state2) = create_participant().await;

        let participant_id2: String = "participant2".into();

        channel.add_participant(participant_id1.clone(), participant1);
        channel.add_participant(participant_id2.clone(), participant2);
        channel.add_presence(participant_id1.clone(), "null".into(), 0);

        channel.remove_participant(&participant_id1);

        assert_eq!(channel.inner.participant_refs.read().len(), 1);
        assert!(channel
            .inner
            .participant_refs
            .read()
            .get(&participant_id1.clone())
            .is_none());

        yield_now().await;

        let message = get_nth_message(state2, 1).await;

        assert!(message.is_some());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match serde_json::from_str::<protocol::ServerMessage>(&data).unwrap() {
                ServerMessage::ServerPresenceMessage(data) => {
                    assert_eq!(
                        data,
                        ServerPresenceMessage {
                            clock: 0,
                            presence: None,
                            id: participant_id1.clone()
                        }
                    );
                }
                _ => {
                    panic!("Message is not ServerMessage::Presence")
                }
            },
            _ => {
                panic!("Message is not a ParticipantMessage::ServerMessage")
            }
        }
    }

    #[test]
    fn it_can_downgrade_and_upgrade() {
        let channel = TokioChannel::new("test".to_string(), None);

        let downgrade = channel.downgrade();

        assert!(downgrade.upgrade().is_some());
    }

    #[test]
    fn it_returns_none_when_trying_to_upgrade_dropped_channel() {
        let downgrade = {
            let channel = TokioChannel::new("test".to_string(), None);

            channel.downgrade()
        };

        assert!(downgrade.upgrade().is_none());
    }

    #[tokio::test]
    async fn it_sends_all_current_presences_to_new_participant() {
        let channel = TokioChannel::new("test".to_string(), None);
        let (participant1, _) = create_participant().await;

        let participant_id1: String = "participant1".into();

        let (participant2, state2) = create_participant().await;

        let participant_id2: String = "participant2".into();

        channel.add_participant(participant_id1.clone(), participant1.clone());
        channel.add_presence(participant_id2.clone(), "null".into(), 0);

        channel.add_participant(participant_id2.clone(), participant2.clone());

        assert_eq!(channel.inner.participant_refs.read().len(), 2);

        yield_now().await;

        let message = get_nth_message(state2, 0).await;

        assert!(message.is_some());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match serde_json::from_str::<protocol::ServerMessage>(&data).unwrap() {
                ServerMessage::InitialPresenceMessage(data) => {
                    assert_eq!(
                        data,
                        InitialPresenceMessage {
                            presences: vec![ServerPresenceMessage {
                                clock: 0,
                                presence: Some("null".into()),
                                id: participant_id2.clone()
                            }]
                        }
                    );
                }
                _ => {
                    panic!("Message is not ServerMessage::InitialPresence")
                }
            },
            _ => {
                panic!("Message is not a ParticipantMessage::ServerMessage")
            }
        }

        participant1.drain().unwrap();
        participant2.drain().unwrap();
    }
}
