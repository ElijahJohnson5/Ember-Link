use std::sync::Arc;

use actor::{Participant, ParticipantArguments, ParticipantMessage};
use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use ractor::{Actor, ActorRef};
use tokio::task::JoinHandle;

use crate::auth::authenticator::AuthContext;
use crate::handler::MessageHandlerRegistry;

use super::channel::TokioChannel;

pub mod actor;

pub async fn start_participant(
    channel: TokioChannel,
    id: String,
    socket_write_sink: SplitSink<WebSocket, Message>,
    handlers: Arc<MessageHandlerRegistry>,
    auth: Option<AuthContext>,
) -> (ActorRef<ParticipantMessage>, JoinHandle<()>) {
    Actor::spawn(
        None,
        Participant,
        ParticipantArguments {
            id,
            channel,
            socket_write_sink,
            handlers,
            auth,
        },
    )
    .await
    .expect("Could not start participant processor")
}
