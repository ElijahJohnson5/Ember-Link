use crate::channel::Channel;
use futures_util::{stream::SplitSink, SinkExt};
use protocol::{
    client::ClientPresenceMessage,
    server::{ServerMessage, ServerPresenceMessage},
    StorageSyncMessage, StorageUpdateMessage,
};
use ractor::{Actor, ActorProcessingErr, ActorRef};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{Bytes, Message},
    WebSocketStream,
};

pub struct Participant;

pub struct ParticipantState {
    id: String,
    channel: Channel,
    presence: Option<Value>,
    socket_write_sink: SplitSink<WebSocketStream<TcpStream>, Message>,
}

#[derive(Debug, Clone)]
pub enum ParticipantMessage {
    PingMessage { data: Bytes },
    TextPingMessage { data: String },
    MyPresence { data: ClientPresenceMessage<Value> },
    StorageUpdate { data: StorageUpdateMessage },
    StorageSync { data: StorageSyncMessage },
    ServerMessage { data: ServerMessage<Value> },
}

pub struct ParticipantArguments {
    pub id: String,
    pub channel: Channel,
    pub socket_write_sink: SplitSink<WebSocketStream<TcpStream>, Message>,
}

impl Actor for Participant {
    type State = ParticipantState;
    type Msg = ParticipantMessage;
    type Arguments = ParticipantArguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        args.channel.add_participant(args.id.clone(), myself);

        Ok(ParticipantState {
            channel: args.channel,
            id: args.id,
            presence: None,
            socket_write_sink: args.socket_write_sink,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ParticipantMessage::PingMessage { data } => {
                state
                    .socket_write_sink
                    .send(Message::Pong(data))
                    .await
                    .unwrap();
            }
            ParticipantMessage::TextPingMessage { data } => {
                state
                    .socket_write_sink
                    .send(Message::text(data))
                    .await
                    .unwrap();
            }
            ParticipantMessage::MyPresence { data } => {
                // TODO: Maybe keep an internal clock to make sure we should actually update the data
                state.presence.replace(data.data.clone());

                state.channel.broadcast(
                    ServerMessage::Presence(ServerPresenceMessage {
                        id: state.id.clone(),
                        clock: data.clock,
                        data: Some(data.data.clone()),
                    }),
                    Some(&state.id),
                );

                state
                    .channel
                    .add_presence(state.id.clone(), data.data, data.clock);
            }
            ParticipantMessage::StorageUpdate { data } => {
                state
                    .channel
                    .handle_update_message(data, state.id.clone())
                    .expect("Could not handle storage update");
            }
            ParticipantMessage::StorageSync { data } => {
                match state.channel.handle_sync_message(data) {
                    Err(e) => {
                        tracing::error!("Could not sync storage: {}", e);
                    }
                    Ok(Some(msgs)) => {
                        for msg in msgs {
                            state
                                .socket_write_sink
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
                state
                    .socket_write_sink
                    .send(Message::text(serde_json::to_string(&data).unwrap()))
                    .await
                    .expect("Could not send message");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub mod tests {
    use std::sync::Arc;

    use ractor::{Actor, ActorProcessingErr, ActorRef};
    use tokio::sync::Mutex;

    use super::ParticipantMessage;

    struct TestParticipantActor;

    #[derive(Default)]
    pub struct TestParticipantActorState {
        pub messages: Vec<ParticipantMessage>,
    }

    impl Actor for TestParticipantActor {
        type Msg = ParticipantMessage;
        type Arguments = Arc<Mutex<TestParticipantActorState>>;
        type State = Arc<Mutex<TestParticipantActorState>>;

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
            state.lock().await.messages.push(message);

            Ok(())
        }
    }

    pub async fn create_participant() -> (
        ActorRef<ParticipantMessage>,
        Arc<Mutex<TestParticipantActorState>>,
    ) {
        let state: Arc<Mutex<TestParticipantActorState>> = Arc::default();
        let (actor, _) = Actor::spawn(None, TestParticipantActor, state.clone())
            .await
            .expect("Could not start test participant actor");

        return (actor, state);
    }
}
