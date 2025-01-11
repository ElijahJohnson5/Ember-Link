mod channel;
mod channel_registry;
mod participant;

use std::sync::Arc;

use channel::actor::{ChannelHandle, ChannelMessage};
use channel_registry::ChannelRegistry;
use futures_util::SinkExt;
use futures_util::{future, StreamExt, TryStreamExt};
use participant::actor::{ParticipantHandle, ParticipantMessage};
use protocol::client::ClientMessage;
use regex::Regex;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::WebSocketStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:9000").await.unwrap();

    let channel_registry: Arc<ChannelRegistry> = Arc::default();

    let channel_name_regex = Regex::new("/channel/(?<channel_name>[a-zA-Z0-9_-]+)")
        .expect("Channel name regex is invalid");

    while let Ok((stream, _)) = listener.accept().await {
        let handle = tokio::spawn(accept_connection(
            stream,
            channel_registry.clone(),
            channel_name_regex.clone(),
        ));
    }

    Ok(())
}

async fn handle_raw_socket(
    stream: TcpStream,
) -> (
    Result<WebSocketStream<TcpStream>, tungstenite::Error>,
    Option<String>,
) {
    let mut path = None;

    let callback = |req: &tungstenite::handshake::server::Request,
                    res: tungstenite::handshake::server::Response|
     -> Result<
        tungstenite::handshake::server::Response,
        tungstenite::handshake::server::ErrorResponse,
    > {
        path = Some(req.uri().path().to_string());
        Ok(res)
    };

    (
        tokio_tungstenite::accept_hdr_async(stream, callback).await,
        path,
    )
}

async fn accept_connection(
    stream: TcpStream,
    channel_registry: Arc<ChannelRegistry>,
    channel_name_regex: Regex,
) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");

    let (ws_stream, path) = handle_raw_socket(stream).await;

    let path = path.expect("Could not get path from connection");

    let Some(caps) = channel_name_regex.captures(&path) else {
        println!("Could not find channel name in path: {}", path);
        return;
    };

    let ws_stream = ws_stream.expect("Error during the websocket handshake occurred");

    println!(
        "New WebSocket connection: {}, path: {}, channel_name: {}",
        addr, path, &caps["channel_name"]
    );

    let channel_handle = channel_registry
        .get_or_create_channel(caps["channel_name"].to_string(), "test".into())
        .await
        .expect("Could not create or get channel");

    let participant_handle = ParticipantHandle::new();

    let (mut write, mut read) = ws_stream.split();

    write.send(Message::Ping("".into())).await.unwrap();

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(msg) => {
                        match msg {
                            Ok(msg) => match msg {
                                Message::Ping(data) => write.send(Message::Pong(data)).await.unwrap(),
                                Message::Pong(data) => println!("Received pong"),
                                Message::Text(data) => {
                                    let data = data.to_string();

                                    println!("{}", data);

                                    if data == "ping" {
                                        write.send(Message::Text("pong".into())).await.unwrap();
                                    } else {
                                        let message: ClientMessage = serde_json::from_str(&data).unwrap();

                                        println!("{:?}", message);
                                    }
                                },
                                Message::Binary(data) => {
                                },

                                _ => break,
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

    println!("Disconnected");
}
