use std::sync::Arc;

use crate::{
    auth::authenticator::AuthContext,
    channel::Channel,
    handler::{HandlerAction, MessageContext, MessageHandlerRegistry},
    tokio::channel::TokioChannel,
};
use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures_util::{stream::SplitSink, SinkExt};
use protocol::{
    ClientPresenceMessage, CustomMessage, ServerMessage, ServerPresenceMessage, StorageSyncMessage,
    StorageUpdateMessage,
};
use ractor::{Actor, ActorProcessingErr, ActorRef};

pub struct Participant;

pub struct ParticipantState {
    id: String,
    channel: TokioChannel,
    presence: Option<String>,
    socket_write_sink: SplitSink<WebSocket, Message>,
    handlers: Arc<MessageHandlerRegistry>,
    auth: Option<AuthContext>,
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
    CustomMessage { data: CustomMessage },
    ServerMessage { data: String },
    ServerBinaryMessage { data: Vec<u8> },
}

pub struct ParticipantArguments {
    pub id: String,
    pub channel: TokioChannel,
    pub socket_write_sink: SplitSink<WebSocket, Message>,
    pub handlers: Arc<MessageHandlerRegistry>,
    pub auth: Option<AuthContext>,
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
            handlers: args.handlers,
            auth: args.auth,
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
                state.socket_write_sink.send(Message::Pong(data)).await.ok();
            }
            ParticipantMessage::TextPingMessage { data } => {
                state.socket_write_sink.send(Message::text(data)).await.ok();
            }
            ParticipantMessage::MyPresence { data } => {
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
                                    serde_bare::to_vec(&ServerMessage::StorageSyncMessage(msg))
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
                                    serde_bare::to_vec(&ServerMessage::ProviderSyncMessage(msg))
                                        .unwrap(),
                                ))
                                .await
                                .expect("Could not send response sync messages");
                        }
                    }
                    _ => {}
                }
            }
            ParticipantMessage::CustomMessage { data } => {
                let ctx = MessageContext {
                    channel_name: state.channel.channel_name().to_string(),
                    participant_id: state.id.clone(),
                    tenant_id: state.channel.tenant_id().cloned(),
                    auth: state.auth.clone(),
                };

                match state.handlers.dispatch(&data.message, ctx).await {
                    None => {
                        // No matching handler — fall back to legacy behavior:
                        // broadcast as-is to everyone except sender.
                        state.channel.broadcast(
                            ServerMessage::CustomMessage(data),
                            Some(&state.id),
                        );
                    }
                    Some(Ok(action)) => {
                        if let Err(e) =
                            dispatch_handler_action(state, action).await
                        {
                            tracing::warn!(error = e, "Failed to dispatch handler action");
                        }
                    }
                    Some(Err(e)) => {
                        tracing::warn!(error = e.to_string(), "Handler returned error");
                    }
                }
            }
            ParticipantMessage::ServerMessage { data } => {
                match state.socket_write_sink.send(Message::text(data)).await {
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
                match state.socket_write_sink.send(Message::binary(data)).await {
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

async fn dispatch_handler_action(
    state: &mut ParticipantState,
    action: HandlerAction,
) -> Result<(), String> {
    match action {
        HandlerAction::Broadcast(value) => {
            let payload = serde_json::to_string(&value)
                .map_err(|e| format!("serialize broadcast payload: {}", e))?;
            state.channel.broadcast(
                ServerMessage::CustomMessage(CustomMessage { message: payload }),
                Some(&state.id),
            );
        }
        HandlerAction::BroadcastIncludingSender(value) => {
            let payload = serde_json::to_string(&value)
                .map_err(|e| format!("serialize broadcast payload: {}", e))?;
            state
                .channel
                .broadcast(ServerMessage::CustomMessage(CustomMessage { message: payload }), None);
        }
        HandlerAction::ReplyOnly(value) => {
            let payload = serde_json::to_string(&value)
                .map_err(|e| format!("serialize reply payload: {}", e))?;
            state
                .socket_write_sink
                .send(Message::text(
                    serde_json::to_string(&ServerMessage::CustomMessage(CustomMessage {
                        message: payload,
                    }))
                    .map_err(|e| format!("serialize ServerMessage: {}", e))?,
                ))
                .await
                .map_err(|e| format!("send reply: {}", e))?;
        }
        HandlerAction::Drop => {}
    }

    Ok(())
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
