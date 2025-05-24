mod api;
mod channel;
mod channel_registry;
mod config;
mod environment;
mod event_listener_primitives;
mod participant;
#[cfg(feature = "webhook")]
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
use channel_registry::{ChannelRegistry, ChannelRegistryBuilder};
use config::TokioConfig;
use envconfig::Envconfig;
use environment::Environment;
use futures_util::SinkExt;
use futures_util::StreamExt;
use participant::actor::ParticipantMessage;
use participant::start_participant;
use protocol::ClientMessage;
use protocol::{AssignIdMessage, ServerMessage};
use protocol::StorageType;
use protocol::WebSocketCloseCode;
use ractor::ActorRef;
use std::error::Error as StdError;
use tokio::signal;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::instrument;

#[cfg(feature = "multi-tenant")]
use tokio::sync::Mutex;

use crate::auth::{validate_token, AuthData};
use crate::channel::create_channel_name;
use crate::AppState;

pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[derive(Clone)]
pub struct TokioAppState {
    pub config: TokioConfig,
    pub channel_registry: Arc<ChannelRegistry>,
    // TODO: Move this to redis or have an option to use redis
    #[cfg(feature = "multi-tenant")]
    pub jwt_signer_key_cache: Arc<Mutex<HashMap<String, String>>>,
}

#[async_trait::async_trait]
impl AppState for TokioAppState {
    #[cfg(feature = "multi-tenant")]
    fn jwt_signer_key_endpoint(&self) -> Option<String> {
        self.config.base_config.jwt_signer_key_endpoint.clone()
    }

    fn jwt_signer_key(&self) -> Option<String> {
        self.config.base_config.jwt_signer_key.clone()
    }

    #[cfg(feature = "multi-tenant")]
    async fn get_cached_key(&self, tenant_id: &String) -> Option<String> {
        let cache = self.jwt_signer_key_cache.lock().await;

        return cache.get(tenant_id).cloned();
    }

    #[cfg(feature = "multi-tenant")]
    async fn cache_key(&self, tenant_id: String, key: String) {
        let mut cache = self.jwt_signer_key_cache.lock().await;

        cache.insert(tenant_id, key);
    }
}

pub struct Server {
    router: axum::Router<TokioAppState>,
    app_state: TokioAppState,
    environment: Environment,
}

impl Server {
    pub async fn new() -> Self {
        match dotenvy::dotenv() {
            Err(_e) => {
                tracing::info!("Could not find .env file")
            }
            _ => {}
        }

        let config = TokioConfig::init_from_env().unwrap();

        let environment = Environment::from_config(&config).await;

        #[cfg(feature = "webhook")]
        let channel_registry = ChannelRegistryBuilder::new(config.clone())
            .with_webhook_processor(environment.webhook_processor())
            .build();

        #[cfg(not(feature = "webhook"))]
        let channel_registry = ChannelRegistryBuilder::new(config.clone()).build();

        let channel_registry: Arc<ChannelRegistry> = Arc::new(channel_registry);

        let app_state = TokioAppState {
            config,
            channel_registry,
            #[cfg(feature = "multi-tenant")]
            jwt_signer_key_cache: Arc::default(),
        };

        let router = Router::new()
            .route("/ws", any(ws_handler))
            .nest("/api", api::api_routes())
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            );

        Self {
            router,
            app_state,
            environment,
        }
    }

    pub async fn serve(
        mut self,
        host: Option<String>,
        port: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(host) = host {
            self.app_state.config.host = host;
        }

        if let Some(port) = port {
            self.app_state.config.port = port;
        }

        let tcp_listener_addr = format!(
            "{}:{}",
            self.app_state.config.host, self.app_state.config.port
        );

        let listener = tokio::net::TcpListener::bind(tcp_listener_addr)
            .await
            .unwrap();
        tracing::info!("listening on {}", listener.local_addr().unwrap());

        let router = self.router.with_state(self.app_state.clone());

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

        self.environment.cleanup().await;

        Ok(())
    }

    pub fn router_mut(&mut self) -> &mut axum::Router<TokioAppState> {
        &mut self.router
    }
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
        _ = ctrl_c => {
            tracing::info!("Gracefully shutting down server");
        },
        _ = terminate => {
            tracing::info!("Gracefully shutting down server");
        },
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
    State(app_state): State<TokioAppState>,
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
    app_state: TokioAppState,
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

    if !app_state.config.base_config.allow_unauthorized && !query_params.contains_key("token") {
        tracing::warn!("Could not find token for authorization");
        socket.send(ws::Message::Close(Some(ws::CloseFrame {
            code: WebSocketCloseCode::TokenNotFound as u16,
            reason: "Token was not found in params for authorization and ALLOW_UNAUTHORIZED is not set on the server".into(),
        }))).await.expect("Could not close websocket");
        return;
    }

    let mut token_payload: Option<AuthData> = None;

    let tenant_id = {
        #[cfg(feature = "multi-tenant")]
        {
            query_params.get("tenant_id").cloned()
        }

        #[cfg(not(feature = "multi-tenant"))]
        {
            None
        }
    };

    if query_params.contains_key("token") {
        let payload =
            match validate_token(&query_params["token"], tenant_id.clone(), &app_state).await {
                Ok(payload) => payload,
                Err(auth_error) => {
                    let auth_error_string = auth_error.to_string();
                    tracing::error!("{}", auth_error_string);

                    let websocket_close_code: WebSocketCloseCode = auth_error.into();

                    socket
                        .send(ws::Message::Close(Some(ws::CloseFrame {
                            code: websocket_close_code as u16,
                            reason: auth_error_string.into(),
                        })))
                        .await
                        .expect("Could not close websocket");
                    return;
                }
            };

        token_payload.replace(payload);
    }

    tracing::debug!(
        "New WebSocket connection: {}",
        who,
    );

    let channel_name = query_params["channel_name"].to_string();

    let name = match create_channel_name(&channel_name, &tenant_id) {
        Ok(name) => name,
        Err(e) => {
            let close_code: WebSocketCloseCode = e.into();

            socket
                .send(ws::Message::Close(Some(ws::CloseFrame {
                    code: close_code as u16,
                    reason: "Channel creation failed".into(),
                })))
                .await
                .expect("Could not close websocket");
            return;
        }
    };

    let channel = match app_state
        .channel_registry
        .get_or_create_channel(name, channel_name, storage_type, tenant_id)
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

    match write
        .send(ws::Message::text(
            serde_json::to_string(&ServerMessage::AssignIdMessage(AssignIdMessage {
                id: participant_id.to_string(),
            }))
            .unwrap(),
        ))
        .await
         {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("Error sending assign id: {}", e);
                return;
            }
        }
         

    match write.send(ws::Message::Ping("".into())).await {
        Ok(_) => {}
        Err(e) => {
            tracing::warn!("Error sending ping: {}", e);
            return;
        }
    }

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
                                match handle_message(&participant, msg).await {
                                    Err(error) => {
                                        tracing::info!(error = error.to_string(), "Could not handle message");
                                    }
                                    Ok(()) => {}
                                }
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

    tracing::debug!("Disconnected");
}

async fn handle_message(
    participant: &ActorRef<ParticipantMessage>,
    msg: ws::Message,
) -> Result<(), String> {
    match msg {
        ws::Message::Ping(data) => {
            participant
                .cast(ParticipantMessage::PingMessage { data: data })
                .ok();
        }
        ws::Message::Text(data) => {
            let data = data.to_string();

            if data == "ping" {
                participant
                    .cast(ParticipantMessage::TextPingMessage {
                        data: "pong".into(),
                    })
                    .ok();
            } else {
                match serde_json::from_str(&data) {
                    Ok(message) => {
                        match handle_client_message(participant, message) {
                            Err(e) => {
                                tracing::error!(
                                    error = e.to_string(),
                                    "Could not send message to participant"
                                )
                            }
                            Ok(()) => {}
                        };
                    }

                    Err(e) => {
                        tracing::error!(data = data, "Could not parse message: {}", e);
                    }
                };
            }
        }
        ws::Message::Binary(data) => {
            let message: ClientMessage = match serde_bare::from_slice(&data) {
                Ok(message) => message,
                Err(e) => {
                    tracing::error!("Could not parse message: {}", e);
                    return Err("Could not parse binary message".into());
                }
            };

            match handle_client_message(participant, message) {
                Err(e) => {
                    tracing::error!(
                        error = e.to_string(),
                        "Could not send message to participant"
                    )
                }
                Ok(()) => {}
            };
        }
        _ => {}
    }

    Ok(())
}

fn handle_client_message(
    participant: &ActorRef<ParticipantMessage>,
    msg: ClientMessage,
) -> Result<(), ractor::MessagingErr<ParticipantMessage>> {
    match msg {
        ClientMessage::ClientPresenceMessage(msg) => {
            participant.cast(ParticipantMessage::MyPresence { data: msg })
        }
        ClientMessage::StorageUpdateMessage(msg) => {
            participant.cast(ParticipantMessage::StorageUpdate { data: msg })
        }
        ClientMessage::StorageSyncMessage(msg) => {
            participant.cast(ParticipantMessage::StorageSync { data: msg })
        }
        ClientMessage::ProviderSyncMessage(msg) => {
            participant.cast(ParticipantMessage::ProviderSync { data: msg })
        }
        ClientMessage::ProviderUpdateMessage(msg) => {
            participant.cast(ParticipantMessage::ProviderUpdate { data: msg })
        }
        ClientMessage::CustomMessage(msg) => participant.cast(ParticipantMessage::ServerMessage {
            data: serde_json::to_string(&ServerMessage::CustomMessage(msg)).unwrap(),
        }),
    }
}
