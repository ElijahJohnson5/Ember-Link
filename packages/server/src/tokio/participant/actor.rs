use crate::{channel::Channel, tokio::channel::TokioChannel};
use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures_util::{stream::SplitSink, SinkExt};
use protocol::{
    ClientPresenceMessage,
    ServerMessage, ServerPresenceMessage,
    StorageSyncMessage, StorageUpdateMessage,
};
use ractor::{Actor, ActorProcessingErr, ActorRef};

pub struct Participant;

pub struct ParticipantState {
    id: String,
    channel: TokioChannel,
    presence: Option<String>,
    socket_write_sink: SplitSink<WebSocket, Message>,
}

#[derive(Debug, Clone)]
pub enum ParticipantMessage {
    PingMessage { data: Bytes },
    TextPingMessage { data: String },
    MyPresence { data: ClientPresenceMessage },
    StorageUpdate { data: StorageUpdateMessage },
    StorageSync { data: StorageSyncMessage },
    ProviderSync { data: StorageSyncMessage },
    ProviderUpdate { data: StorageUpdateMessage },
    ServerMessage { data: String },
    ServerBinaryMessage { data: Vec<u8> }
}

pub struct ParticipantArguments {
    pub id: String,
    pub channel: TokioChannel,
    pub socket_write_sink: SplitSink<WebSocket, Message>,
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
                    .ok();
            }
            ParticipantMessage::TextPingMessage { data } => {
                state
                    .socket_write_sink
                    .send(Message::text(data))
                    .await
                    .ok();
            }
            ParticipantMessage::MyPresence { data } => {
                // TODO: Maybe keep an internal clock to make sure we should actually update the data
                state.presence.replace(data.presence.clone());

                state
                    .channel
                    .add_presence(state.id.clone(), data.presence.clone(), data.clock);

                state.channel.broadcast(
                    ServerMessage::ServerPresenceMessage(ServerPresenceMessage {
                        id: state.id.clone(),
                        clock: data.clock,
                        presence: Some(data.presence),
                    }),
                    Some(&state.id),
                );
            }
            ParticipantMessage::StorageUpdate { data } => {
                state
                    .channel
                    .handle_storage_update_message(data, &state.id)
                    .expect("Could not handle storage update");
            }
            ParticipantMessage::StorageSync { data } => {
                match state.channel.handle_storage_sync_message(data) {
                    Err(e) => {
                        tracing::error!("Could not sync storage: {}", e);
                    }
                    Ok(Some(msgs)) => {
                        for msg in msgs {
                            state
                                .socket_write_sink
                                .send(Message::binary(
                                    serde_bare::to_vec(
                                        &ServerMessage::StorageSyncMessage(msg),
                                    )
                                    .unwrap(),
                                ))
                                .await
                                .expect("Could not send response sync messages");
                        }
                    }
                    _ => {}
                }
            }
            ParticipantMessage::ProviderUpdate { data } => {
                state
                    .channel
                    .handle_provider_update_message(data, &state.id)
                    .expect("Could not handle storage update");
            }
            ParticipantMessage::ProviderSync { data } => {
                match state.channel.handle_provider_sync_message(data) {
                    Err(e) => {
                        tracing::error!("Could not sync storage: {}", e);
                    }
                    Ok(Some(msgs)) => {
                        for msg in msgs {
                            state
                                .socket_write_sink
                                .send(Message::binary(
                                    serde_bare::to_vec(
                                        &ServerMessage::ProviderSyncMessage(msg),
                                    )
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
                match state
                    .socket_write_sink
                    .send(Message::text(data))
                    .await
                {
                    Err(e) => {
                        tracing::warn!(
                            error = e.to_string(),
                            "Could not send message to participant removing them from the channel"
                        );
                    }
                    Ok(()) => {}
                }
            }
            ParticipantMessage::ServerBinaryMessage { data } => {
                match state
                    .socket_write_sink
                    .send(Message::binary(data))
                    .await
                {
                    Err(e) => {
                        tracing::warn!(
                            error = e.to_string(),
                            "Could not send message to participant removing them from the channel"
                        );
                    }
                    Ok(()) => {}
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
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
