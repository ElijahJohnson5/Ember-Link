mod channel;
mod channel_registry;
mod config;
mod environment;
mod event_listener_primitives;
mod participant;
mod storage;
mod webhook_processor;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::extract::{ws, ws::WebSocket, ConnectInfo, Query, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::any;
use axum::Router;
use axum_extra::headers;
use axum_extra::TypedHeader;
use channel_registry::ChannelRegistry;
use config::Config;
use envconfig::Envconfig;
use environment::Environment;
use futures_util::SinkExt;
use futures_util::StreamExt;
use josekit::jws;
use josekit::jwt;
use participant::actor::ParticipantMessage;
use participant::start_participant;
use protocol::client::ClientMessage;
use protocol::server::{AssignIdMessage, ServerMessage};
use protocol::StorageType;
use protocol::WebSocketCloseCode;
use ractor::ActorRef;
use serde::Deserialize;
use serde_json::Value;
use tokio::signal;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::instrument;
use tracing_subscriber::FmtSubscriber;
use url::Url;

#[derive(Clone)]
struct AppState {
    config: Config,
    channel_registry: Arc<ChannelRegistry>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let config = Config::init_from_env().unwrap();

    let environment = Environment::from_config(&config).await;

    let channel_registry: Arc<ChannelRegistry> = Arc::new(ChannelRegistry::new(
        environment.webhook_processor(),
        config.clone(),
    ));

    let app_state = AppState {
        config,
        channel_registry,
    };

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    environment.cleanup().await;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// The handler for the HTTP request (this gets called when the HTTP request lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    Query(params): Query<HashMap<String, String>>,
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::debug!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, params, addr, app_state))
}

#[instrument(skip_all)]
async fn handle_socket(
    mut socket: WebSocket,
    query_params: HashMap<String, String>,
    who: SocketAddr,
    app_state: AppState,
) {
    if !query_params.contains_key("channel_name") {
        tracing::warn!("Could not find channel name in query params");
        return;
    }
    let storage_type: Option<StorageType> = query_params
        .get("storage_type")
        .map(|storage_type| format!("\"{storage_type}\""))
        .map(|storage_type| serde_json::from_str(&storage_type).ok())
        .flatten();

    if !app_state.config.allow_unauthorized && !query_params.contains_key("token") {
        tracing::warn!("Could not find token for authorization");
        socket.send(ws::Message::Close(Some(ws::CloseFrame {
            code: WebSocketCloseCode::TokenNotFound as u16,
            reason: "Token was not found in params for authorization and ALLOW_UNAUTHORIZED is not set on the server".into(),
        }))).await.expect("Could not close websocket");
        return;
    }
    let mut token_payload: Option<jwt::JwtPayload> = None;

    if query_params.contains_key("token") {
        let payload = match validate_token(
            &query_params["token"],
            query_params.get("tenant_id").cloned(),
            app_state.config.clone(),
        )
        .await
        {
            Ok(payload) => payload,
            Err((trace_error_message, websocket_close_code)) => {
                tracing::error!("{trace_error_message}");
                socket
                    .send(ws::Message::Close(Some(ws::CloseFrame {
                        code: websocket_close_code as u16,
                        reason: trace_error_message.into(),
                    })))
                    .await
                    .expect("Could not close websocket");
                return;
            }
        };

        token_payload.replace(payload);
    }

    tracing::info!("Token payload {:?}", token_payload);

    tracing::info!(
        "New WebSocket connection: {}, query params: {:?}",
        who,
        query_params,
    );

    let channel = match app_state
        .channel_registry
        .get_or_create_channel(
            query_params["channel_name"].to_string(),
            storage_type,
            query_params.get("tenant_id").cloned(),
        )
        .await
    {
        Err(e) => {
            tracing::error!("{}", e);
            socket
                .send(ws::Message::Close(Some(ws::CloseFrame {
                    code: WebSocketCloseCode::ChannelCreationFailed as u16,
                    reason: "Channel creation failed".into(),
                })))
                .await
                .expect("Could not close websocket");
            return;
        }

        Ok(channel) => channel,
    };

    let (mut write, mut read) = socket.split();

    let participant_id = uuid::Uuid::new_v4();

    write
        .send(ws::Message::text(
            serde_json::to_string(&ServerMessage::AssignId::<Value>(AssignIdMessage {
                id: participant_id.to_string(),
            }))
            .unwrap(),
        ))
        .await
        .unwrap();

    write.send(ws::Message::Ping("".into())).await.unwrap();

    let weak_channel = channel.downgrade();

    let (participant, handle) =
        start_participant(channel, participant_id.clone().to_string(), write).await;

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(msg) => {
                        match msg {
                            Ok(msg) => {
                                handle_message(&participant, msg).await.unwrap()
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

    match weak_channel.upgrade() {
        None => {}
        Some(channel) => {
            channel.remove_participant(&participant_id.to_string());
        }
    }

    participant.stop(None);
    handle
        .await
        .expect("Could not await participant join handle");

    tracing::info!("Disconnected");
}

async fn handle_message(
    participant: &ActorRef<ParticipantMessage>,
    msg: ws::Message,
) -> Result<(), String> {
    match msg {
        ws::Message::Ping(data) => {
            participant
                .cast(ParticipantMessage::PingMessage { data: data })
                .expect("Could not send message to participant");
        }
        ws::Message::Text(data) => {
            let data = data.to_string();

            if data == "ping" {
                participant
                    .cast(ParticipantMessage::TextPingMessage {
                        data: "pong".into(),
                    })
                    .expect("Could not send message to participant");
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
        ws::Message::Binary(_data) => {}
        _ => {}
    }

    Ok(())
}

fn handle_client_message(participant: &ActorRef<ParticipantMessage>, msg: ClientMessage<Value>) {
    match msg {
        ClientMessage::Presence(msg) => {
            // TODO broadcast to the channel and store in the channel
            participant
                .cast(ParticipantMessage::MyPresence { data: msg })
                .expect("Could not send message to participant");
        }
        ClientMessage::StorageUpdate(msg) => {
            participant
                .cast(ParticipantMessage::StorageUpdate { data: msg })
                .expect("Could not send message to participant");
        }
        ClientMessage::StorageSync(msg) => {
            participant
                .cast(ParticipantMessage::StorageSync { data: msg })
                .expect("Could not send message to participant");
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignerKeyResponse {
    public_signer_key: String,
}

async fn validate_token(
    token: &String,
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
