use std::{
    cmp,
    sync::Arc,
    time::{Duration, Instant},
};

use protocol::WebhookMessage;
use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::sync::RwLock;

struct WebhookBatcher;

struct WebhookBatcherState {
    messages: Arc<RwLock<Vec<WebhookMessage>>>,
    webhook_url: String,
    client: reqwest::Client,
    last_send: Instant,
}

struct WebhookBatcherArguments {
    messages: Arc<RwLock<Vec<WebhookMessage>>>,
    webhook_url: String,
}

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
            client: reqwest::Client::new(),
            last_send: Instant::now(),
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            Self::Msg::TrySendWebhook => {
                if state.last_send.elapsed() >= Duration::from_millis(300) {
                    self.send_webhook(state).await?
                }
            }
            Self::Msg::SendWebhook => self.send_webhook(state).await?,
        }

        Ok(())
    }
}

impl WebhookBatcher {
    pub async fn send_webhook(
        &self,
        state: &mut WebhookBatcherState,
    ) -> Result<(), ActorProcessingErr> {
        let (future, end_range) = {
            let messages = state.messages.read().await;
            let end_range = cmp::min(messages.len(), 300);

            if messages.len() > 0 {
                let messages = &messages[0..end_range];
                (
                    state
                        .client
                        .post(state.webhook_url.clone())
                        .json(messages)
                        .send(),
                    end_range,
                )
            } else {
                return Ok(());
            }
        };

        match future.await {
            Err(e) => {
                println!("{}", e);
            }
            Ok(response) => {
                println!("{:?}", response);
                println!("Sent webhook");

                state.messages.write().await.drain(0..end_range);
                state.last_send = Instant::now();
            }
        }

        Ok(())
    }
}

pub struct WebhookProcessor;

pub struct WebhookProcessorState {
    messages: Arc<RwLock<Vec<WebhookMessage>>>,
    webhook_batcher: ActorRef<WebhookBatcherMessage>,
}

impl Actor for WebhookProcessor {
    type State = WebhookProcessorState;
    type Msg = WebhookMessage;
    type Arguments = String;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let messages: Arc<RwLock<Vec<WebhookMessage>>> = Arc::default();

        let (webhook_batcher, _) = Actor::spawn_linked(
            None,
            WebhookBatcher,
            WebhookBatcherArguments {
                messages: messages.clone(),
                webhook_url: arguments,
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
            messages.push(msg);

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
