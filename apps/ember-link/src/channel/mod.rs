#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Weak},
};

use parking_lot::Mutex;
use protocol::server::{InitialPresenceMessage, ServerMessage, ServerPresenceMessage};
use serde_json::Value;

use crate::{
    event_listener_primitives::{Bag, BagOnce, HandlerId},
    participant::actor::{ParticipantMessage, WeakParticipantHandle},
};

#[derive(Default)]
struct Handlers {
    participant_added: Bag<Arc<dyn Fn(&String) + Send + Sync>, String>,
    participant_removed: Bag<Arc<dyn Fn(&String) + Send + Sync>, String>,
    closed: BagOnce<Box<dyn FnOnce() + Send>>,
}

struct Inner {
    id: String,
    participant_handles: Mutex<HashMap<String, WeakParticipantHandle>>,
    participant_presence_state: Mutex<HashMap<String, (Value, i32)>>,
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
pub struct Channel {
    inner: Arc<Inner>,
}

impl Channel {
    pub fn new(id: String) -> Self {
        tracing::info!("Creating channel {}", id);

        Self {
            inner: Arc::new(Inner {
                id,
                participant_handles: Mutex::default(),
                participant_presence_state: Mutex::default(),
                handlers: Handlers::default(),
            }),
        }
    }

    pub fn broadcast(&self, message: ServerMessage<Value>, exclude_id: Option<&String>) {
        for (key, value) in self.inner.participant_handles.lock().iter() {
            if exclude_id.is_some_and(|id| *id == *key) {
                continue;
            }

            match value.upgrade() {
                Some(handle) => {
                    // Send message to each participant
                    handle
                        .sender
                        .send(ParticipantMessage::ServerMessage {
                            data: message.clone(),
                        })
                        .unwrap();
                }
                None => {
                    // TODO: Remove stale handles
                }
            }
        }
    }

    pub fn add_presence(&self, participant_id: String, state: Value, clock: i32) {
        self.inner
            .participant_presence_state
            .lock()
            .insert(participant_id, (state, clock));
    }

    pub fn add_participant(&self, participant_id: String, participant: WeakParticipantHandle) {
        self.inner
            .participant_handles
            .lock()
            .insert(participant_id.clone(), participant.clone());

        self.inner
            .handlers
            .participant_added
            .call_simple(&participant_id);

        match participant.upgrade() {
            Some(participant) => {
                participant
                    .sender
                    .send(ParticipantMessage::ServerMessage {
                        data: ServerMessage::InitialPresence(self.initial_presence_message()),
                    })
                    .unwrap();
            }
            _ => {}
        }
    }

    pub fn remove_participant(&self, participant_id: &str) {
        self.inner.participant_handles.lock().remove(participant_id);
        let state = self
            .inner
            .participant_presence_state
            .lock()
            .remove(participant_id);

        self.inner
            .handlers
            .participant_removed
            .call_simple(&participant_id.to_string());

        match state {
            Some((_, clock)) => {
                self.broadcast(
                    ServerMessage::Presence(ServerPresenceMessage {
                        clock: clock,
                        data: None,
                        id: participant_id.to_string(),
                    }),
                    Some(&participant_id.to_string()),
                );
            }
            None => {}
        }
    }

    pub fn on_participant_added<F: Fn(&String) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) -> HandlerId {
        self.inner
            .handlers
            .participant_added
            .add(Arc::new(callback))
    }

    pub fn on_participant_removed<F: Fn(&String) + Send + Sync + 'static>(
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

    fn initial_presence_message(&self) -> InitialPresenceMessage<Value> {
        let mut presences: Vec<ServerPresenceMessage<Value>> = Vec::default();
        for (key, (state, clock)) in self.inner.participant_presence_state.lock().iter() {
            presences.push(ServerPresenceMessage {
                id: key.clone(),
                clock: clock.clone(),
                data: Some(state.clone()),
            });
        }

        InitialPresenceMessage { presences }
    }

    /// Get `WeakChannel` that can later be upgraded to `Channel`, but will not prevent channel from
    /// being destroyed
    pub fn downgrade(&self) -> WeakChannel {
        WeakChannel {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

/// Similar to `Channel`, but doesn't prevent channel from being destroyed
#[derive(Debug, Clone)]
pub struct WeakChannel {
    inner: Weak<Inner>,
}

impl WeakChannel {
    /// Upgrade `WeakChannel` to `Channel`, may return `None` if underlying Channel was destroyed already
    pub fn upgrade(&self) -> Option<Channel> {
        self.inner.upgrade().map(|inner| Channel { inner })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod tests {
    use crate::participant::actor::ParticipantHandle;
    use protocol::StorageUpdateMessage;
    use tokio::sync::mpsc::{self, UnboundedReceiver};

    use super::*;

    pub fn create_participant<T: Into<String>>(
        id: T,
    ) -> (ParticipantHandle, UnboundedReceiver<ParticipantMessage>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (
            ParticipantHandle {
                id: id.into(),
                sender,
            },
            receiver,
        )
    }

    fn get_nth_message(
        receiver: &mut UnboundedReceiver<ParticipantMessage>,
        n: usize,
    ) -> Result<ParticipantMessage, mpsc::error::TryRecvError> {
        for _i in 0..n - 1 {
            receiver.try_recv()?;
        }

        receiver.try_recv()
    }

    #[tokio::test]
    async fn it_adds_a_participant_and_sends_initial_presence() {
        let channel = Channel::new("test".to_string());
        let (participant, mut receiver) = create_participant("participant");

        channel.add_participant(participant.id.clone(), participant.downgrade());

        assert_eq!(channel.inner.participant_handles.lock().len(), 1);
        assert!(channel
            .inner
            .participant_handles
            .lock()
            .get(&participant.id.clone())
            .is_some());

        let message = get_nth_message(&mut receiver, 1);

        assert!(message.is_ok());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match data {
                ServerMessage::InitialPresence(data) => {
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
        let channel = Channel::new("test".to_string());
        let (participant, _receiver) = create_participant("participant");
        let handler_counter = Arc::new(Mutex::new(HandlerCounter { count: 0 }));

        channel
            .on_participant_added({
                let handler_counter = handler_counter.clone();
                move |_participant_id| {
                    handler_counter.lock().count += 1;
                }
            })
            .detach();

        channel.add_participant(participant.id.clone(), participant.downgrade());

        assert_eq!(handler_counter.lock().count, 1);
    }

    #[tokio::test]
    async fn it_calls_participant_removed_handler() {
        let channel = Channel::new("test".to_string());
        let (participant, _receiver) = create_participant("participant");
        let handler_counter = Arc::new(Mutex::new(HandlerCounter { count: 0 }));

        channel
            .on_participant_removed({
                let handler_counter = handler_counter.clone();
                move |_participant_id| {
                    handler_counter.lock().count += 1;
                }
            })
            .detach();

        channel.add_participant(participant.id.clone(), participant.downgrade());
        channel.remove_participant(&participant.id);

        assert_eq!(handler_counter.lock().count, 1);
    }

    #[tokio::test]
    async fn it_calls_on_close_when_dropped() {
        let handler_counter = Arc::new(Mutex::new(HandlerCounter { count: 0 }));

        {
            let channel = Channel::new("test".to_string());

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
        let channel = Channel::new("test".to_string());
        let (participant1, mut receiver1) = create_participant("participant1");
        let (participant2, mut receiver2) = create_participant("participant2");

        channel.add_participant(participant1.id.clone(), participant1.downgrade());
        channel.add_participant(participant2.id.clone(), participant2.downgrade());

        channel.broadcast(
            ServerMessage::StorageUpdate(StorageUpdateMessage { update: vec![] }),
            None,
        );

        let message = get_nth_message(&mut receiver1, 2);

        assert!(message.is_ok());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match data {
                ServerMessage::StorageUpdate(data) => {
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

        let message = get_nth_message(&mut receiver2, 2);

        assert!(message.is_ok());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match data {
                ServerMessage::StorageUpdate(data) => {
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
        let channel = Channel::new("test".to_string());
        let (participant1, mut receiver1) = create_participant("participant1");
        let (participant2, mut receiver2) = create_participant("participant2");

        channel.add_participant(participant1.id.clone(), participant1.downgrade());
        channel.add_participant(participant2.id.clone(), participant2.downgrade());

        channel.broadcast(
            ServerMessage::StorageUpdate(StorageUpdateMessage { update: vec![] }),
            Some(&participant2.id),
        );

        let message = get_nth_message(&mut receiver1, 2);

        assert!(message.is_ok());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match data {
                ServerMessage::StorageUpdate(data) => {
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

        let message = get_nth_message(&mut receiver2, 2);

        assert!(message.is_err());

        match message.unwrap_err() {
            mpsc::error::TryRecvError::Empty => {}
            _ => {
                panic!("Message error was not empty")
            }
        }
    }

    #[tokio::test]
    async fn it_removes_participant() {
        let channel = Channel::new("test".to_string());
        let (participant, _receiver) = create_participant("participant");

        channel.add_participant(participant.id.clone(), participant.downgrade());

        channel.remove_participant(&participant.id);

        assert_eq!(channel.inner.participant_handles.lock().len(), 0);
        assert!(channel
            .inner
            .participant_handles
            .lock()
            .get(&participant.id.clone())
            .is_none());
    }

    #[tokio::test]
    async fn it_broadcasts_removed_participant_presence() {
        let channel = Channel::new("test".to_string());
        let (participant1, _receiver) = create_participant("participant1");
        let (participant2, mut receiver2) = create_participant("participant2");

        channel.add_participant(participant1.id.clone(), participant1.downgrade());
        channel.add_participant(participant2.id.clone(), participant2.downgrade());
        channel.add_presence(participant1.id.clone(), Value::Null, 0);

        channel.remove_participant(&participant1.id);

        assert_eq!(channel.inner.participant_handles.lock().len(), 1);
        assert!(channel
            .inner
            .participant_handles
            .lock()
            .get(&participant1.id.clone())
            .is_none());

        let message = get_nth_message(&mut receiver2, 2);

        assert!(message.is_ok());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match data {
                ServerMessage::Presence(data) => {
                    assert_eq!(
                        data,
                        ServerPresenceMessage {
                            clock: 0,
                            data: None,
                            id: participant1.id.clone()
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
        let channel = Channel::new("test".to_string());

        let downgrade = channel.downgrade();

        assert!(downgrade.upgrade().is_some());
    }

    #[test]
    fn it_returns_none_when_trying_to_upgrade_dropped_channel() {
        let downgrade = {
            let channel = Channel::new("test".to_string());

            channel.downgrade()
        };

        assert!(downgrade.upgrade().is_none());
    }

    #[tokio::test]
    async fn it_sends_all_current_presences_to_new_participant() {
        let channel = Channel::new("test".to_string());
        let (participant1, _receiver) = create_participant("participant1");
        let (participant2, mut receiver2) = create_participant("participant2");

        channel.add_participant(participant1.id.clone(), participant1.downgrade());
        channel.add_presence(participant1.id.clone(), Value::Null, 0);

        channel.add_participant(participant2.id.clone(), participant2.downgrade());

        assert_eq!(channel.inner.participant_handles.lock().len(), 2);

        let message = get_nth_message(&mut receiver2, 1);

        assert!(message.is_ok());

        match message.unwrap() {
            ParticipantMessage::ServerMessage { data } => match data {
                ServerMessage::InitialPresence(data) => {
                    assert_eq!(
                        data,
                        InitialPresenceMessage {
                            presences: vec![ServerPresenceMessage {
                                clock: 0,
                                data: Some(Value::Null),
                                id: participant1.id.clone()
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
    }
}
