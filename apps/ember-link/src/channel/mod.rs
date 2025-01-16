use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Weak},
};

use parking_lot::Mutex;
use protocol::{
    server::{InitialPresenceMessage, NewPresenceMessage, ServerMessage},
    PresenceState,
};

use crate::{
    event_listener_primitives::{handler_id::HandlerId, once::BagOnce, regular::Bag},
    participant::{
        self,
        actor::{ParticipantMessage, WeakParticipantHandle},
    },
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
    participant_presence_state: Mutex<HashMap<String, (PresenceState, i32)>>,
    handlers: Handlers,
}

impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Inner").field("id", &self.id).finish()
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        println!("Channel {} closed", self.id);

        self.handlers.closed.call_simple();
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    inner: Arc<Inner>,
}

impl Channel {
    pub fn new(id: String) -> Self {
        println!("Creating channel {}", id);

        Self {
            inner: Arc::new(Inner {
                id,
                participant_handles: Mutex::default(),
                participant_presence_state: Mutex::default(),
                handlers: Handlers::default(),
            }),
        }
    }

    pub fn broadcast(&self, message: ServerMessage, exclude_id: Option<&String>) {
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

    pub fn add_presence(&self, participant_id: String, state: PresenceState, clock: i32) {
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
        self.inner
            .participant_presence_state
            .lock()
            .remove(participant_id);

        self.inner
            .handlers
            .participant_removed
            .call_simple(&participant_id.to_string());
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

    fn initial_presence_message(&self) -> InitialPresenceMessage {
        let mut presences: Vec<NewPresenceMessage> = Vec::default();
        for (key, (state, clock)) in self.inner.participant_presence_state.lock().iter() {
            presences.push(NewPresenceMessage {
                id: key.clone(),
                clock: clock.clone(),
                data: state.clone(),
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
