use futures_util::{stream::SplitSink, SinkExt};
use protocol::server::ServerMessage;
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
    BroadcastMessage { data: ServerMessage },
    ServerMessage { data: ServerMessage },
}

impl ParticipantHandle {
    pub fn new(
        id: uuid::Uuid,
        channel: Channel,
        socketWriteSink: SplitSink<WebSocketStream<TcpStream>, Message>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let new = Self {
            id: id.to_string(),
            sender: sender.clone(),
        };

        channel.add_participant(id.to_string(), new.downgrade());

        let participant = Participant::new(id.to_string(), receiver, channel, socketWriteSink);

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
    socketWriteSink: SplitSink<WebSocketStream<TcpStream>, Message>,
}

impl Participant {
    pub fn new(
        id: String,
        receiver: mpsc::UnboundedReceiver<ParticipantMessage>,
        channel: Channel,
        socketWriteSink: SplitSink<WebSocketStream<TcpStream>, Message>,
    ) -> Self {
        Self {
            id,
            receiver,
            channel,
            socketWriteSink,
        }
    }

    async fn handle_message(&mut self, msg: ParticipantMessage) {
        match msg {
            ParticipantMessage::PingMessage { data } => {
                self.socketWriteSink
                    .send(Message::Pong(data))
                    .await
                    .unwrap();
            }
            ParticipantMessage::TextPingMessage { data } => {
                self.socketWriteSink
                    .send(Message::text(data))
                    .await
                    .unwrap();
            }
            ParticipantMessage::BroadcastMessage { data } => {
                println!("Broadcasting message {:?}", data);
                self.channel.broadcast(data, Some(&self.id));
            }
            ParticipantMessage::ServerMessage { data } => {
                println!("Sending Server message {:?}", data);
                self.socketWriteSink
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

    println!("Participant {} finished", participant.id);
}
