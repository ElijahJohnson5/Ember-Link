use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex, Weak},
};

use protocol::server::ServerMessage;

use crate::participant::actor::{ParticipantHandle, ParticipantMessage, WeakParticipantHandle};

struct Inner {
    id: String,
    participant_handles: Mutex<HashMap<String, WeakParticipantHandle>>,
}

impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Inner").field("id", &self.id).finish()
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        println!("Channel {} closed", self.id);
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
            }),
        }
    }

    pub fn broadcast(&self, message: ServerMessage, exclude_id: Option<&String>) {
        for (key, value) in self
            .inner
            .participant_handles
            .lock()
            .expect("Could not get lock for participant handles")
            .iter()
        {
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

    pub fn add_participant(&self, participant_id: String, participant: WeakParticipantHandle) {
        self.inner
            .participant_handles
            .lock()
            .expect("Could not get lock for participant handles")
            .insert(participant_id, participant);
    }

    pub fn remove_participant(&self, participant_id: &str) {
        self.inner
            .participant_handles
            .lock()
            .expect("Could not get lock for participant handles")
            .remove(participant_id);
    }

    /// Get `WeakChannel` that can later be upgraded to `Channel`, but will not prevent channel from
    /// being destroyed
    pub fn downgrade(&self) -> WeakChannel {
        WeakChannel {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

/// Similar to `Room`, but doesn't prevent room from being destroyed
#[derive(Debug, Clone)]
pub struct WeakChannel {
    inner: Weak<Inner>,
}

impl WeakChannel {
    /// Upgrade `WeakRoom` to `Room`, may return `None` if underlying room was destroyed already
    pub fn upgrade(&self) -> Option<Channel> {
        self.inner.upgrade().map(|inner| Channel { inner })
    }
}
