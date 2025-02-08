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
use josekit::jws;
use josekit::jwt;
use participant::actor::{ParticipantHandle, ParticipantMessage};
use protocol::client::ClientMessage;
use protocol::server::{AssignIdMessage, ServerMessage};
use protocol::WebSocketCloseCode;
use serde::Deserialize;
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::WebSocketStream;
use tracing::instrument;
use tracing_subscriber::FmtSubscriber;
use url::Url;

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
        let _handle = tokio::spawn(accept_connection(
            stream,
            channel_registry.clone(),
            config.clone(),
        ));
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
async fn accept_connection(
    stream: TcpStream,
    channel_registry: Arc<ChannelRegistry>,
    config: Config,
) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");

    let (ws_stream, params) = handle_raw_socket(stream).await;

    let mut ws_stream = match ws_stream {
        Err(e) => {
            tracing::error!("Error during the websocket handshake occurred: {e}");
            return;
        }
        Ok(ws_stream) => ws_stream,
    };

    let params = params.expect("Could not get query params from connection");

    if !params.contains_key("channel_name") {
        tracing::warn!("Could not find channel name in query params");
        return;
    }

    if !config.allow_unauthorized && !params.contains_key("token") {
        tracing::warn!("Could not find token for authorization");
        ws_stream
            .close(Some(CloseFrame {
                code: CloseCode::Library(WebSocketCloseCode::TokenNotFound as u16),
                reason: "Token was not found in params for authorization and ALLOW_UNAUTHORIZED is not set on the server".into(),
            }))
            .await
            .expect("Could not close websocket");
        return;
    }
    let mut token_payload: Option<jwt::JwtPayload> = None;

    if params.contains_key("token") {
        let payload = match validate_token(
            params["token"].clone(),
            params.get("tenant_id").cloned(),
            config.clone(),
        )
        .await
        {
            Ok(payload) => payload,
            Err((trace_error_message, websocket_close_code)) => {
                tracing::error!("{trace_error_message}");
                ws_stream
                    .close(Some(CloseFrame {
                        code: CloseCode::Library(websocket_close_code as u16),
                        reason: trace_error_message.into(),
                    }))
                    .await
                    .expect("Could not close websocket");
                return;
            }
        };

        token_payload.replace(payload);
    }

    tracing::info!("Token payload {:?}", token_payload);

    tracing::info!(
        "New WebSocket connection: {}, query params: {:?}, channel_name: {}",
        addr,
        params,
        params["channel_name"]
    );

    let (mut write, mut read) = ws_stream.split();

    let channel = channel_registry
        .get_or_create_channel(
            params["channel_name"].to_string(),
            params.get("tenant_id").cloned(),
        )
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
                                tracing::info!(error = error.to_string());
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignerKeyResponse {
    public_signer_key: String,
}

async fn validate_token(
    token: String,
    tenant_id: Option<String>,
    config: Config,
) -> Result<jwt::JwtPayload, (String, WebSocketCloseCode)> {
    if let Some(tenant_id) = tenant_id {
        if let Some(jwt_signer_key_endpoint) = config.jwt_signer_key_endpoint {
            let mut url = Url::parse(&jwt_signer_key_endpoint).map_err(|_e| {
                (
                    "JWT Signer Key Endpoint is invalid".into(),
                    WebSocketCloseCode::InvalidSignerKey,
                )
            })?;

            url.query_pairs_mut().append_pair("tenant_id", &tenant_id);

            tracing::info!("{:?}", url);

            let response = reqwest::get(url).await.map_err(|e| {
                tracing::error!("Error: {e}");
                (
                    "JWT Signer Key Endpoint is invalid".into(),
                    WebSocketCloseCode::InvalidSignerKey,
                )
            })?;

            tracing::info!("Signing Key Endpoint status: {}", response.status());
            let response = response.json::<SignerKeyResponse>().await.map_err(|e| {
                tracing::error!("Error: {e}");
                (
                    "JWT Signer Key Endpoint is invalid".into(),
                    WebSocketCloseCode::InvalidSignerKey,
                )
            })?;

            return verify_token(&token, response.public_signer_key);
        } else {
            Err((
                "JWT Signer Key Endpoint is required if you are validating tokens with multiple tenants".into(),
                WebSocketCloseCode::InvalidSignerKey,
            ))
        }
    } else if let Some(jwt_signer_key) = config.jwt_signer_key {
        // Validate signature of token
        return verify_token(&token, jwt_signer_key);
    } else {
        Err((
            "JWT Signer Key is required if you are validating tokens".into(),
            WebSocketCloseCode::InvalidSignerKey,
        ))
    }
}

fn verify_token(
    token: &String,
    signer_key: String,
) -> Result<jwt::JwtPayload, (String, WebSocketCloseCode)> {
    // Validate signature of token
    match jws::RS256.verifier_from_pem(signer_key) {
        Ok(jwt_verifier) => match jwt::decode_with_verifier(token, &jwt_verifier) {
            Ok((payload, _header)) => Ok(payload),
            Err(e) => Err((
                format!("Could not verify token: {e}").into(),
                WebSocketCloseCode::InvalidToken,
            )),
        },
        Err(e) => Err((
            format!("JWT Signer Key is invalid: {e}").into(),
            WebSocketCloseCode::InvalidSignerKey,
        )),
    }
}
