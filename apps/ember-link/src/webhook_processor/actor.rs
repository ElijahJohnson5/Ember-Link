use std::{
    cmp,
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use protocol::WebhookMessage;
use ractor::{Actor, ActorProcessingErr, ActorRef, SupervisionEvent};
use tokio::sync::RwLock;
use tracing::instrument;

use super::sender::{
    WebhookSender, WebhookSenderArguments, WebhookSenderMessage, WebhookSenderState,
};

struct WebhookBatcher;

struct WebhookBatcherState {
    messages: WebhookMessageContainer,
    webhook_url: String,
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

        Ok(WebhookBatcherState {
            messages: arguments.messages,
            webhook_url: arguments.webhook_url,
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

    async fn handle_supervisor_evt(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorTerminated(_actor, child_state, reason) => {
                if let Some(mut child_state) = child_state {
                    let child_state = child_state.take::<WebhookSenderState>().unwrap();
                    if reason.is_some_and(|reason| reason == "Success") {
                        tracing::info!("Webhook sent after {} attempts", child_state.attempt);

                        state.last_send = Instant::now();
                    }
                }
            }
            SupervisionEvent::ActorFailed(_actor, error) => {
                return Err(error);
            }
            _ => {}
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

                let (webhook_sender, _) = Actor::spawn_linked(
                    None,
                    WebhookSender,
                    WebhookSenderArguments {
                        messages: messages.clone(),
                        webhook_url: state.webhook_url.clone(),
                        webhook_secret_key: state.webhook_secret_key.clone(),
                    },
                    myself.get_cell(),
                )
                .await
                .expect("Could not spawn webhook sender");

                webhook_sender
                    .cast(WebhookSenderMessage::Send)
                    .expect("Could not send message to webhook sender");
            } else {
                return Ok(());
            }

            end_range
        };

        state.messages.write().await.drain(0..end_range);

        Ok(())
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
