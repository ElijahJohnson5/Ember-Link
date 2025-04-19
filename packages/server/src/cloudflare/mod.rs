use std::collections::HashMap;

use axum::{extract::Query, response::IntoResponse, routing::any};
#[cfg(feature = "webhook")]
use axum_extra::headers;
use envconfig::Envconfig;
#[cfg(feature = "webhook")]
use hmac::{Hmac, Mac};
use http::StatusCode;
use protocol::WebhookMessage;
use serde::{Deserialize, Serialize};
#[cfg(feature = "webhook")]
use sha2::Sha256;
use tower_http::cors::{Any, CorsLayer};
use tower_service::Service;

use crate::{
    auth::{validate_token, AuthData},
    channel::create_channel_name,
    config::{config_keys, Config},
    AppState,
};

pub mod channel;
pub mod cloudflare_websocket_upgrade;

#[derive(Clone)]
struct CloudflareAppState {
    env: worker::Env,
    #[cfg(feature = "webhook")]
    queue_name: String,
    config: Config,
}

impl AppState for CloudflareAppState {
    #[cfg(feature = "multi-tenant")]
    fn jwt_signer_key_endpoint(&self) -> Option<String> {
        self.config.jwt_signer_key_endpoint.clone()
    }

    fn jwt_signer_key(&self) -> Option<String> {
        self.config.jwt_signer_key.clone()
    }
}

fn router(app_state: CloudflareAppState) -> axum::Router {
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([http::Method::GET, http::Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    axum::Router::new()
        .route("/ws", any(ws_handler))
        .with_state(app_state)
        .layer(cors)
}

#[axum::debug_handler]
async fn ws_handler(
    axum::extract::State(state): axum::extract::State<CloudflareAppState>,
    Query(params): Query<HashMap<String, String>>,
    mut request: axum::extract::Request,
) -> impl IntoResponse {
    if !params.contains_key("channel_name") {
        return (
            StatusCode::BAD_REQUEST,
            "Missing channel_name query parameter",
        )
            .into_response();
    }

    if !state.config.allow_unauthorized && !params.contains_key("token") {
        return (StatusCode::UNAUTHORIZED, "Missing token").into_response();
    }

    let mut token_payload: Option<AuthData> = None;

    let tenant_id = {
        #[cfg(feature = "multi-tenant")]
        {
            params.get("tenant_id").cloned()
        }

        #[cfg(not(feature = "multi-tenant"))]
        {
            None
        }
    };

    if params.contains_key("token") {
        let payload = match worker::send::SendFuture::new(validate_token(
            &params["token"],
            tenant_id.clone(),
            &state,
        ))
        .await
        {
            Ok(payload) => payload,
            Err(auth_error) => {
                let auth_error_string = auth_error.to_string();
                return (StatusCode::UNAUTHORIZED, auth_error_string).into_response();
            }
        };

        token_payload.replace(payload);
    }

    // TODO actually do something with the token payload

    let namespace = match state.env.durable_object("CHANNEL") {
        Err(e) => return worker_error_into_response(e),
        Ok(namespace) => namespace,
    };

    let channel_name = params.get("channel_name").unwrap();

    let stub = {
        let name = match create_channel_name(channel_name, &tenant_id) {
            Ok(name) => name,
            Err(e) => {
                return e.into_response();
            }
        };

        let id = match namespace.id_from_name(name.as_str()) {
            Err(e) => return worker_error_into_response(e),
            Ok(id) => id,
        };

        match id.get_stub() {
            Err(e) => return worker_error_into_response(e),
            Ok(stub) => stub,
        }
    };

    #[cfg(feature = "webhook")]
    request
        .headers_mut()
        .append("x-queue-name", state.queue_name.parse().unwrap());

    let worker_request = worker::Request::try_from(request).unwrap();

    // We have to tell axum that this in fact Send
    let fut = worker::send::SendFuture::new(stub.fetch_with_request(worker_request));

    match fut.await {
        Err(e) => worker_error_into_response(e),
        Ok(response) => match http::Response::<worker::Body>::try_from(response) {
            Ok(response) => response.into_response(),
            Err(e) => return worker_error_into_response(e),
        },
    }
}

pub fn worker_error_into_response(error: worker::Error) -> axum::response::Response {
    let response = match error {
        _ => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    };

    response.into_response()
}

pub(crate) fn get_config(env: &worker::Env) -> Result<Config, envconfig::Error> {
    let mut map = HashMap::new();

    for key in config_keys() {
        let value = env.var(key).ok();

        if let Some(value) = value {
            map.insert(key.to_string(), value.to_string());
        }
    }

    return Config::init_from_hashmap(&map);
}

pub struct Options {
    #[cfg(feature = "webhook")]
    pub queue_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CloudflareQueueMessage {
    #[cfg(feature = "multi-tenant")]
    pub tenant_id: String,
    pub message: WebhookMessage,
}

pub struct Server {
    router: axum::Router,
    config: Config,
}

impl Server {
    pub fn new(env: worker::Env, options: Options) -> Result<Self, axum::response::Response> {
        let config = match get_config(&env) {
            Ok(config) => config,
            Err(e) => {
                let response = match e {
                    envconfig::Error::EnvVarMissing { name } => (
                        StatusCode::BAD_REQUEST,
                        format!("Missing env var: {}", name),
                    ),
                    envconfig::Error::ParseError { name } => (
                        StatusCode::BAD_REQUEST,
                        format!("Env variable misformatted: {}", name),
                    ),
                };

                return Err(response.into_response());
            }
        };

        Ok(Self {
            router: router(Server::create_app_state(env, options, config.clone())),
            config,
        })
    }

    pub async fn handle_fetch(
        mut self,
        req: worker::HttpRequest,
    ) -> worker::Result<axum::http::Response<axum::body::Body>> {
        let response = self.router.call(req).await?;

        Ok(response)
    }

    #[cfg(feature = "webhook")]
    pub async fn handle_queue(
        self,
        message_batch: worker::MessageBatch<CloudflareQueueMessage>,
    ) -> worker::Result<()> {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.config.webhook_secret_key.as_bytes())
            .expect("HMAC can take key of any size");

        #[cfg(feature = "multi-tenant")]
        let messages = message_batch.messages()?.into_iter().fold(
            HashMap::<String, Vec<WebhookMessage>>::new(),
            |mut acc, message| {
                let message = message.into_body();

                acc.entry(message.tenant_id)
                    .or_default()
                    .push(message.message);

                acc
            },
        );

        #[cfg(not(feature = "multi-tenant"))]
        let messages: Vec<WebhookMessage> = message_batch
            .messages()?
            .into_iter()
            .map(|message| message.into_body().message)
            .collect();

        let message_string = serde_json::to_string(&messages).unwrap();

        mac.update(message_string.as_bytes());

        let result = mac.finalize().into_bytes();

        let signature = hex::encode(result);

        let client = reqwest::Client::new();

        let future = worker::send::SendFuture::new(
            client
                .post(self.config.webhook_url)
                .header("webhook-signature", signature)
                .header("content-type", headers::ContentType::json().to_string())
                .body(message_string)
                .send(),
        );

        let response = match future.await {
            Err(_e) => {
                message_batch.retry_all();
                return Ok(());
            }
            Ok(response) => response.error_for_status(),
        };

        match response {
            Err(_e) => {
                // TODO Figure out a good way to get the attemp number for these messages
                // message_batch.retry_all_with_options(QueueRetryOptionsBuilder::new().with_delay_seconds(calculate_backoff(message_batch., random_seed)));
                message_batch.retry_all();
                return Ok(());
            }
            Ok(_response) => {
                message_batch.ack_all();
            }
        }

        Ok(())
    }

    pub fn router_mut(&mut self) -> &mut axum::Router {
        &mut self.router
    }

    #[cfg(feature = "webhook")]
    fn create_app_state(env: worker::Env, options: Options, config: Config) -> CloudflareAppState {
        CloudflareAppState {
            env,
            queue_name: options.queue_name,
            config,
        }
    }

    #[cfg(not(feature = "webhook"))]
    fn create_app_state(env: worker::Env, options: Options, config: Config) -> CloudflareAppState {
        CloudflareAppState { env, config }
    }
}
