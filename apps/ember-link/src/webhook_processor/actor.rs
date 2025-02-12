use std::{
    cmp,
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use hmac::{Hmac, Mac};
use protocol::WebhookMessage;
use ractor::{
    factory::{FactoryMessage, Job, JobOptions},
    Actor, ActorProcessingErr, ActorRef,
};
use rand::Rng;
use sha2::Sha256;
use tokio::sync::RwLock;
use tracing::instrument;

use super::factory::{create_webhook_sender_factory, WebhookSenderMessage};

struct WebhookBatcher;

struct WebhookBatcherState {
    messages: WebhookMessageContainer,
    factory: ActorRef<FactoryMessage<(), WebhookSenderMessage>>,
    webhook_secret_key: String,
    last_send: Instant,
}

struct WebhookBatcherArguments {
    messages: WebhookMessageContainer,
    webhook_url: String,
    webhook_secret_key: String,
}

type WebhookMessageContainer = Arc<RwLock<Vec<(Option<String>, WebhookMessage)>>>;

enum WebhookBatcherMessage {
    TrySendWebhook,
    SendWebhook,
}

impl Actor for WebhookBatcher {
    type State = WebhookBatcherState;
    type Msg = WebhookBatcherMessage;
    type Arguments = WebhookBatcherArguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        myself.send_interval(Duration::from_millis(300), || Self::Msg::TrySendWebhook);

        let (factory, _join_handle) = create_webhook_sender_factory(&arguments.webhook_url).await;

        factory.link(myself.get_cell());

        Ok(WebhookBatcherState {
            messages: arguments.messages,
            factory,
            webhook_secret_key: arguments.webhook_secret_key,
            last_send: Instant::now(),
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            Self::Msg::TrySendWebhook => {
                if state.last_send.elapsed() >= Duration::from_millis(300) {
                    self.send_webhook(myself, state).await?
                }
            }
            Self::Msg::SendWebhook => self.send_webhook(myself, state).await?,
        }

        Ok(())
    }
}

impl WebhookBatcher {
    #[instrument(skip(self, state))]
    pub async fn send_webhook(
        &self,
        myself: ActorRef<WebhookBatcherMessage>,
        state: &mut WebhookBatcherState,
    ) -> Result<(), ActorProcessingErr> {
        let end_range = {
            let messages = state.messages.read().await;
            let end_range = cmp::min(messages.len(), 300);

            if messages.len() > 0 {
                let messages = &messages[0..end_range].iter().fold(
                    HashMap::<String, Vec<WebhookMessage>>::new(),
                    |mut acc, (tenant_id, message)| {
                        acc.entry(tenant_id.clone().unwrap_or("".into()))
                            .or_default()
                            .push(message.clone());

                        acc
                    },
                );

                state
                    .factory
                    .cast(FactoryMessage::Dispatch(Job {
                        accepted: None,
                        key: (),
                        options: JobOptions::default(),
                        msg: self.create_webhook_sender_message(
                            &state.webhook_secret_key,
                            messages.clone(),
                        ),
                    }))
                    .expect("Failed to send webhook message job to factory");
            } else {
                return Ok(());
            }

            end_range
        };

        state.messages.write().await.drain(0..end_range);

        Ok(())
    }

    pub fn create_webhook_sender_message(
        &self,
        webhook_secret_key: &String,
        messages: HashMap<String, Vec<WebhookMessage>>,
    ) -> WebhookSenderMessage {
        let mut mac = Hmac::<Sha256>::new_from_slice(webhook_secret_key.as_bytes())
            .expect("HMAC can take key of any size");

        if messages.contains_key("") && messages.len() == 1 {
            mac.update(
                &serde_json::to_string(&messages.get("").unwrap())
                    .unwrap()
                    .into_bytes(),
            );
        } else {
            mac.update(&serde_json::to_string(&messages).unwrap().into_bytes());
        }

        let result = mac.finalize().into_bytes();

        let s = hex::encode(result);

        WebhookSenderMessage {
            attempt: 0,
            random_seed: rand::rng().random_range(0.0..1.0),
            signature: s,
            messages,
        }
    }
}

pub struct WebhookProcessor;

pub struct WebhookProcessorState {
    messages: WebhookMessageContainer,
    webhook_batcher: ActorRef<WebhookBatcherMessage>,
}

pub struct WebhookProcessorMessage {
    pub msg: WebhookMessage,
    pub tenant_id: Option<String>,
}

pub struct WebhookProcessorArguments {
    pub webhook_url: String,
    pub webhook_secret_key: String,
}

impl Actor for WebhookProcessor {
    type State = WebhookProcessorState;
    type Msg = WebhookProcessorMessage;
    type Arguments = WebhookProcessorArguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let messages: WebhookMessageContainer = Arc::default();

        let (webhook_batcher, _) = Actor::spawn_linked(
            None,
            WebhookBatcher,
            WebhookBatcherArguments {
                messages: messages.clone(),
                webhook_url: arguments.webhook_url,
                webhook_secret_key: arguments.webhook_secret_key,
            },
            myself.get_cell(),
        )
        .await
        .expect("Could not start webhook batcher");

        Ok(WebhookProcessorState {
            messages: messages,
            webhook_batcher,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let len = {
            let mut messages = state.messages.write().await;

            let tenant_id = msg.tenant_id;

            messages.push((tenant_id, msg.msg));
            messages.len()
        };

        if len >= 300 {
            state
                .webhook_batcher
                .cast(WebhookBatcherMessage::SendWebhook)
                .expect("Could not send message to webhook batcher");
        }

        Ok(())
    }
}
