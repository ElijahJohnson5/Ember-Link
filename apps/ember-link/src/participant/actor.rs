use futures_util::{stream::SplitSink, SinkExt};
use protocol::{
    client::ClientPresenceMessage,
    server::{ServerMessage, ServerPresenceMessage},
    StorageSyncMessage, StorageUpdateMessage,
};
use serde_json::Value;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    tungstenite::{Bytes, Message},
    WebSocketStream,
};

use crate::channel::Channel;

#[derive(Clone)]
pub struct ParticipantHandle {
    pub id: String,
    pub sender: mpsc::UnboundedSender<ParticipantMessage>,
}

#[derive(Clone)]
pub struct WeakParticipantHandle {
    pub id: String,
    pub sender: mpsc::WeakUnboundedSender<ParticipantMessage>,
}

#[derive(Debug)]
pub enum ParticipantMessage {
    PingMessage { data: Bytes },
    TextPingMessage { data: String },
    MyPresence { data: ClientPresenceMessage<Value> },
    StorageUpdate { data: StorageUpdateMessage },
    StorageSync { data: StorageSyncMessage },
    ServerMessage { data: ServerMessage<Value> },
}

impl ParticipantHandle {
    pub fn new(
        id: uuid::Uuid,
        channel: Channel,
        socket_write_sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let new = Self {
            id: id.to_string(),
            sender: sender.clone(),
        };

        channel.add_participant(id.to_string(), new.downgrade());

        let participant = Participant::new(id.to_string(), receiver, channel, socket_write_sink);

        tokio::spawn(run_participant(participant));

        new
    }

    pub fn downgrade(&self) -> WeakParticipantHandle {
        WeakParticipantHandle {
            id: self.id.clone(),
            sender: self.sender.downgrade(),
        }
    }
}

impl WeakParticipantHandle {
    pub fn upgrade(&self) -> Option<ParticipantHandle> {
        self.sender.upgrade().map(|sender| ParticipantHandle {
            sender,
            id: self.id.clone(),
        })
    }
}

pub struct Participant {
    id: String,
    receiver: mpsc::UnboundedReceiver<ParticipantMessage>,
    channel: Channel,
    socket_write_sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    presence: Option<Value>,
}

impl Participant {
    pub fn new(
        id: String,
        receiver: mpsc::UnboundedReceiver<ParticipantMessage>,
        channel: Channel,
        socket_write_sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    ) -> Self {
        Self {
            id,
            receiver,
            channel,
            socket_write_sink,
            presence: None,
        }
    }

    async fn handle_message(&mut self, msg: ParticipantMessage) {
        match msg {
            ParticipantMessage::PingMessage { data } => {
                self.socket_write_sink
                    .send(Message::Pong(data))
                    .await
                    .unwrap();
            }
            ParticipantMessage::TextPingMessage { data } => {
                self.socket_write_sink
                    .send(Message::text(data))
                    .await
                    .unwrap();
            }
            ParticipantMessage::MyPresence { data } => {
                // TODO: Maybe keep an internal clock to make sure we should actually update the data
                self.presence.replace(data.data.clone());

                self.channel.broadcast(
                    ServerMessage::Presence(ServerPresenceMessage {
                        id: self.id.clone(),
                        clock: data.clock,
                        data: Some(data.data.clone()),
                    }),
                    Some(&self.id),
                );

                self.channel
                    .add_presence(self.id.clone(), data.data, data.clock);
            }
            ParticipantMessage::StorageUpdate { data } => {
                self.channel
                    .handle_update_message(data, self.id.clone())
                    .await
                    .expect("Could not handle storage update");
            }
            ParticipantMessage::StorageSync { data } => {
                match self.channel.handle_sync_message(data).await {
                    Err(e) => {
                        tracing::error!("Could not sync storage: {}", e);
                    }
                    Ok(Some(msgs)) => {
                        for msg in msgs {
                            self.socket_write_sink
                                .send(Message::text(
                                    serde_json::to_string(&ServerMessage::<Value>::StorageSync(
                                        msg,
                                    ))
                                    .unwrap(),
                                ))
                                .await
                                .expect("Could not send response sync messages");
                        }
                    }
                    _ => {}
                }
            }
            ParticipantMessage::ServerMessage { data } => {
                self.socket_write_sink
                    .send(Message::text(serde_json::to_string(&data).unwrap()))
                    .await
                    .expect("Could not send message");
            }
        }
    }
}

async fn run_participant(mut participant: Participant) {
    loop {
        tokio::select! {
            opt_msg = participant.receiver.recv() => {
                let msg = match opt_msg {
                    Some(msg) => msg,
                    None => break,
                };

                participant.handle_message(msg).await
            }
        }
    }

    participant.channel.remove_participant(&participant.id);

    tracing::info!("Participant {} finished", participant.id);
}
