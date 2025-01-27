mod channel;
mod channel_registry;
mod config;
mod environment;
mod event_listener_primitives;
mod participant;
mod webhook_processor;

use std::collections::HashMap;
use std::sync::Arc;

use channel_registry::ChannelRegistry;
use config::Config;
use envconfig::Envconfig;
use environment::Environment;
use futures_util::SinkExt;
use futures_util::StreamExt;
use participant::actor::{ParticipantHandle, ParticipantMessage};
use protocol::client::ClientMessage;
use protocol::server::{AssignIdMessage, ServerMessage};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::WebSocketStream;
use tracing::instrument;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let listener = TcpListener::bind("127.0.0.1:9000").await.unwrap();
    tracing::info!("Starting server on 127.0.0.1:9000");

    let config = Config::init_from_env().unwrap();

    let environment = Environment::from_config(&config).await;

    let channel_registry: Arc<ChannelRegistry> =
        Arc::new(ChannelRegistry::new(environment.webhook_processor()));

    while let Ok((stream, _)) = listener.accept().await {
        let _handle = tokio::spawn(accept_connection(stream, channel_registry.clone()));
    }

    environment.cleanup().await;

    Ok(())
}

async fn handle_raw_socket(
    stream: TcpStream,
) -> (
    Result<WebSocketStream<TcpStream>, tungstenite::Error>,
    Option<HashMap<String, String>>,
) {
    let mut params: Option<HashMap<String, String>> = None;

    let callback = |req: &tungstenite::handshake::server::Request,
                    res: tungstenite::handshake::server::Response|
     -> Result<
        tungstenite::handshake::server::Response,
        tungstenite::handshake::server::ErrorResponse,
    > {
        let decoded_params: HashMap<String, String> = req
            .uri()
            .query()
            .map(|v| {
                url::form_urlencoded::parse(v.as_bytes())
                    .into_owned()
                    .collect()
            })
            .unwrap_or_else(HashMap::new);

        params.replace(decoded_params);
        Ok(res)
    };

    (
        tokio_tungstenite::accept_hdr_async(stream, callback).await,
        params,
    )
}

#[instrument(skip_all)]
async fn accept_connection(stream: TcpStream, channel_registry: Arc<ChannelRegistry>) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");

    let (ws_stream, params) = handle_raw_socket(stream).await;

    let params = params.expect("Could not get query params from connection");

    if !params.contains_key(&"channel_name".to_string()) {
        tracing::warn!("Could not find channel name in query params");
        return;
    }

    let ws_stream = match ws_stream {
        Err(e) => {
            tracing::error!("Error during the websocket handshake occurred: {e}");
            return;
        }
        Ok(ws_stream) => ws_stream,
    };

    tracing::info!(
        "New WebSocket connection: {}, query params: {:?}, channel_name: {}",
        addr,
        params,
        params["channel_name"]
    );

    let (mut write, mut read) = ws_stream.split();

    let channel = channel_registry
        .get_or_create_channel(params["channel_name"].to_string())
        .await;

    let participant_id = uuid::Uuid::new_v4();

    write
        .send(Message::text(
            serde_json::to_string(&ServerMessage::AssignId::<Value>(AssignIdMessage {
                id: participant_id.to_string(),
            }))
            .unwrap(),
        ))
        .await
        .unwrap();

    write.send(Message::Ping("".into())).await.unwrap();

    let participant_handle = ParticipantHandle::new(participant_id, channel, write);

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(msg) => {
                        match msg {
                            Ok(msg) => {
                                handle_message(&participant_handle, msg).await.unwrap()
                            }
                            Err(error) => {
                                println!("{}", error.to_string());
                                break;
                            }
                        };
                    }
                    None => break,
                }
            }
        }
    }

    tracing::info!("Disconnected");
}

async fn handle_message(
    participant: &ParticipantHandle,
    msg: tokio_tungstenite::tungstenite::Message,
) -> Result<(), String> {
    match msg {
        Message::Ping(data) => {
            participant
                .sender
                .send(ParticipantMessage::PingMessage { data: data })
                .unwrap();
        }
        Message::Text(data) => {
            let data = data.to_string();

            if data == "ping" {
                participant
                    .sender
                    .send(ParticipantMessage::TextPingMessage {
                        data: "pong".into(),
                    })
                    .unwrap();
            } else {
                match serde_json::from_str(&data) {
                    Ok(message) => {
                        handle_client_message(participant, message);
                    }

                    Err(e) => {
                        tracing::error!(data = data, "Could not parse message: {}", e);
                    }
                };
            }
        }
        Message::Binary(_data) => {}
        _ => {}
    }

    Ok(())
}

fn handle_client_message(participant: &ParticipantHandle, msg: ClientMessage<Value>) {
    match msg {
        ClientMessage::Presence(msg) => {
            // TODO broadcast to the channel and store in the channel
            participant
                .sender
                .send(ParticipantMessage::MyPresence { data: msg })
                .unwrap();
        }
        ClientMessage::StorageUpdate(msg) => {
            participant
                .sender
                .send(ParticipantMessage::StorageUpdate { data: msg })
                .unwrap();
        }
    }
}
