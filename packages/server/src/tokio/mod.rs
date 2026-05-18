mod api;
mod channel;
mod channel_registry;
mod config;
mod environment;
mod event_listener_primitives;
mod observer_bus;
mod participant;
#[cfg(feature = "webhook")]
mod webhook_processor;

use std::collections::HashMap;
use std::future::Future;
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
pub use config::TokioConfig;
use envconfig::Envconfig;
use environment::Environment;
use futures_util::SinkExt;
use futures_util::StreamExt;
use observer_bus::{start_observer_bus, ObserverBusHandle};
use participant::actor::ParticipantMessage;
use participant::start_participant;
use protocol::ClientMessage;
use protocol::StorageType;
use protocol::WebSocketCloseCode;
use protocol::{AssignIdMessage, ServerMessage};
use ractor::ActorRef;
use std::error::Error as StdError;
use tokio::signal;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::instrument;

use crate::auth::authenticator::{
    AllowAllAuthenticator, Authenticator, AuthenticatorFn, HasAuthenticator, JwtAuthenticator,
};
#[cfg(feature = "multi-tenant")]
use crate::auth::authenticator::MultiTenantJwtAuthenticator;
use crate::auth::AuthError;
use crate::channel::create_channel_name;
use crate::handler::{HandlerAction, HandlerError, MessageContext, MessageHandler};
use crate::observer::{ChannelEvent, Observer, ObserverContext};

pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[derive(Clone)]
pub struct TokioAppState {
    pub config: TokioConfig,
    pub channel_registry: Arc<ChannelRegistry>,
    pub(crate) authenticator: Arc<dyn Authenticator>,
}

impl HasAuthenticator for TokioAppState {
    fn authenticator(&self) -> &Arc<dyn Authenticator> {
        &self.authenticator
    }
}

fn default_authenticator(config: &TokioConfig) -> Arc<dyn Authenticator> {
    if config.base_config.allow_unauthorized {
        return Arc::new(AllowAllAuthenticator);
    }

    #[cfg(feature = "multi-tenant")]
    {
        if let Some(endpoint) = config.base_config.jwt_signer_key_endpoint.clone() {
            return Arc::new(MultiTenantJwtAuthenticator::new(endpoint));
        }
    }

    if let Some(key) = config.base_config.jwt_signer_key.clone() {
        return Arc::new(JwtAuthenticator::new(key));
    }

    // No auth configured and ALLOW_UNAUTHORIZED is false. Connections will be
    // rejected with `SignerKeyMissing` when they attempt to authenticate.
    Arc::new(JwtAuthenticator::new(String::new()))
}

pub struct Server {
    router: axum::Router<TokioAppState>,
    app_state: TokioAppState,
    bus_handle: ObserverBusHandle,
    bind_addr: Option<SocketAddr>,
}

/// Fluent builder for [`Server`]. Plug in a custom authenticator, observers,
/// or message handlers before starting.
///
/// Defaults loaded from environment variables (`JWT_SIGNER_KEY`, `WEBHOOK_URL`,
/// `HOST`, `PORT`, …) still apply unless explicitly overridden, so
/// `Server::builder().build().await.serve().await` remains a viable one-liner.
#[derive(Default)]
pub struct ServerBuilder {
    config: Option<TokioConfig>,
    authenticator: Option<Arc<dyn Authenticator>>,
    observers: Vec<Arc<dyn Observer>>,
    handlers: Vec<Arc<dyn MessageHandler>>,
    bind_addr: Option<SocketAddr>,
}

impl ServerBuilder {
    /// Replace the env-derived [`TokioConfig`]. Useful for embedding tests or
    /// when you want to drive configuration from your own source of truth.
    pub fn with_config(mut self, config: TokioConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Override the bind address; equivalent to setting `HOST`/`PORT` env
    /// vars but with type-safety. If unset, the address is built from the
    /// resolved [`TokioConfig::host`] and [`TokioConfig::port`].
    pub fn bind(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = Some(addr);
        self
    }

    /// Provide a custom [`Authenticator`]. Defaults to a JWT authenticator
    /// derived from environment vars.
    pub fn with_authenticator(mut self, authenticator: impl Authenticator + 'static) -> Self {
        self.authenticator = Some(Arc::new(authenticator));
        self
    }

    /// Provide a custom auth function. Stateless cases (a token map, a remote
    /// fetch) usually want this rather than implementing the trait.
    pub fn with_auth_fn<F, Fut>(self, f: F) -> Self
    where
        F: Fn(Option<String>, Option<String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<crate::auth::AuthContext, AuthError>> + Send + 'static,
    {
        self.with_authenticator(AuthenticatorFn::new(f))
    }

    /// Register an [`Observer`] for channel lifecycle events. Both built-in
    /// observers from [`Environment`] (e.g. webhook) and embedder-registered
    /// observers run on the same queue.
    pub fn with_observer(mut self, observer: impl Observer + 'static) -> Self {
        self.observers.push(Arc::new(observer));
        self
    }

    /// Register an observer from an async closure.
    pub fn with_observer_fn<F, Fut>(self, f: F) -> Self
    where
        F: Fn(ChannelEvent, ObserverContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let observer = crate::observer::observer_fn(f);
        ServerBuilder {
            observers: {
                let mut o = self.observers;
                o.push(observer);
                o
            },
            ..self
        }
    }

    /// Register a custom message handler.
    pub fn with_handler(mut self, handler: impl MessageHandler + 'static) -> Self {
        self.handlers.push(Arc::new(handler));
        self
    }

    /// Register a handler from an async closure, keyed by `message_type`.
    pub fn with_handler_fn<F, Fut>(self, message_type: &'static str, f: F) -> Self
    where
        F: Fn(MessageContext, serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<HandlerAction, HandlerError>> + Send + 'static,
    {
        self.with_handler(crate::handler::HandlerFn::new(message_type, f))
    }

    pub async fn build(self) -> Server {
        if self.config.is_none() {
            match dotenvy::dotenv() {
                Err(_e) => {
                    tracing::info!("Could not find .env file")
                }
                _ => {}
            }
        }

        let config = self
            .config
            .unwrap_or_else(|| TokioConfig::init_from_env().unwrap());

        let environment = Environment::from_config(&config).await;

        // Default observers (from env) come first; user-registered observers
        // run after them.
        let mut all_observers = environment.into_observers();
        all_observers.extend(self.observers);

        let (bus, bus_handle) = start_observer_bus(all_observers);

        let mut registry_builder = ChannelRegistryBuilder::new(config.clone()).with_observers(bus);

        for handler in self.handlers {
            registry_builder = registry_builder.with_handler(handler);
        }

        let channel_registry: Arc<ChannelRegistry> = Arc::new(registry_builder.build());

        let authenticator = self
            .authenticator
            .unwrap_or_else(|| default_authenticator(&config));

        let bind_addr = self.bind_addr;

        let app_state = TokioAppState {
            config,
            channel_registry,
            authenticator,
        };

        let router = Router::new()
            .route("/ws", any(ws_handler))
            .nest("/api", api::api_routes())
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            );

        Server {
            router,
            app_state,
            bus_handle,
            bind_addr,
        }
    }
}

impl Server {
    /// Start building a server with custom pluggable behavior.
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    /// Shorthand for `Server::builder().build().await`. Preserves the
    /// "everything from env" UX from the docker image.
    pub async fn new() -> Self {
        Self::builder().build().await
    }

    /// Bind to the configured address (or one set via [`ServerBuilder::bind`])
    /// and serve until shutdown. After the listener exits, registered
    /// [`Observer`]s receive `shutdown()` so they can flush.
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = match self.bind_addr {
            Some(addr) => addr,
            None => format!("{}:{}", self.app_state.config.host, self.app_state.config.port)
                .parse()
                .map_err(|e: std::net::AddrParseError| -> Box<dyn std::error::Error> {
                    Box::new(e)
                })?,
        };

        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("listening on {}", listener.local_addr().unwrap());

        let router = self.router.with_state(self.app_state.clone());

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

        self.bus_handle.shutdown().await;

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

    let auth_context: Option<crate::auth::authenticator::AuthContext> = if query_params
        .contains_key("token")
    {
        let token = query_params.get("token").map(String::as_str);
        match app_state
            .authenticator
            .authenticate(token, tenant_id.as_deref())
            .await
        {
            Ok(ctx) => Some(ctx),
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
        }
    } else {
        None
    };

    tracing::debug!("New WebSocket connection: {}", who,);

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

    let (participant, handle) = start_participant(
        channel,
        participant_id.clone().to_string(),
        write,
        app_state.channel_registry.message_handlers(),
        auth_context,
    )
    .await;

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
        ClientMessage::CustomMessage(msg) => {
            participant.cast(ParticipantMessage::CustomMessage { data: msg })
        }
    }
}

