use actor::{Participant, ParticipantArguments, ParticipantMessage};
use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use ractor::{Actor, ActorRef};
use tokio::task::JoinHandle;

use crate::channel::Channel;

pub mod actor;

pub async fn start_participant(
    channel: Channel,
    id: String,
    socket_write_sink: SplitSink<WebSocket, Message>,
) -> (ActorRef<ParticipantMessage>, JoinHandle<()>) {
    Actor::spawn(
        None,
        Participant,
        ParticipantArguments {
            id,
            channel,
            socket_write_sink,
        },
    )
    .await
    .expect("Could not start participant processor")
}
