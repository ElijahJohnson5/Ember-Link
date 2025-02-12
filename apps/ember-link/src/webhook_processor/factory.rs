use std::{collections::HashMap, time::Duration};

use protocol::WebhookMessage;
use ractor::{
    factory::{
        queues, routing, Factory, FactoryArguments, FactoryMessage, Job, JobOptions, Worker,
        WorkerBuilder, WorkerId,
    },
    Actor, ActorProcessingErr, ActorRef,
};

pub struct WebhookSenderMessage {
    pub messages: HashMap<String, Vec<WebhookMessage>>,
    pub signature: String,
    pub attempt: u64,
    pub random_seed: f32,
}

pub struct WebhookSenderState {
    pub webhook_url: String,
    pub client: reqwest::Client,
}

pub struct WebhookSenderArguments {
    pub webhook_url: String,
}

struct WebhookSender;
impl Worker for WebhookSender {
    type Key = ();
    type Message = WebhookSenderMessage;
    type State = WebhookSenderState;
    type Arguments = WebhookSenderArguments;
    async fn pre_start(
        &self,
        _wid: WorkerId,
        _factory: &ActorRef<FactoryMessage<(), WebhookSenderMessage>>,
        startup_context: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(WebhookSenderState {
            client: reqwest::Client::new(),
            webhook_url: startup_context.webhook_url,
        })
    }

    async fn handle(
        &self,
        _wid: WorkerId,
        factory: &ActorRef<FactoryMessage<(), WebhookSenderMessage>>,
        Job { msg, key, .. }: Job<(), WebhookSenderMessage>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        tracing::info!("Sending webhook attempt number: {}", msg.attempt);

        let request_future = if msg.messages.contains_key("") && msg.messages.len() == 1 {
            state
                .client
                .post(state.webhook_url.clone())
                .header("webhook-signature", msg.signature.clone())
                .json(msg.messages.get("").unwrap())
                .send()
        } else {
            state
                .client
                .post(state.webhook_url.clone())
                .header("webhook-signature", msg.signature.clone())
                .json(&msg.messages)
                .send()
        };

        let attempt = msg.attempt + 1;

        let backoff = Duration::from_millis(
            // One day is max timeout
            ((std::cmp::min(86_400_000, attempt * attempt * 1000) as f32)
                + 2000.0 * msg.random_seed)
                .floor() as u64,
        );

        match request_future.await {
            Err(e) => {
                tracing::warn!(
                    "Could not send request: {}, retrying with backoff: {:?}",
                    e,
                    backoff
                );

                factory.send_after(backoff, move || {
                    FactoryMessage::Dispatch(Job {
                        key: (),
                        msg: WebhookSenderMessage {
                            attempt,
                            messages: msg.messages,
                            random_seed: msg.random_seed,
                            signature: msg.signature,
                        },
                        accepted: None,
                        options: JobOptions::default(),
                    })
                });
            }

            Ok(response) => match response.error_for_status() {
                Err(e) => {
                    tracing::warn!(
                        "Error response recieved from server: {}, retrying with backoff: {:?}",
                        e,
                        backoff
                    );
                    factory.send_after(backoff, move || {
                        FactoryMessage::Dispatch(Job {
                            key: (),
                            msg: WebhookSenderMessage {
                                attempt,
                                messages: msg.messages,
                                random_seed: msg.random_seed,
                                signature: msg.signature,
                            },
                            accepted: None,
                            options: JobOptions::default(),
                        })
                    });
                }
                Ok(_response) => {
                    tracing::info!("Sent webhook");
                }
            },
        }

        Ok(key)
    }
}

struct WebhookSenderBuilder {
    pub webhook_url: String,
}

impl WorkerBuilder<WebhookSender, WebhookSenderArguments> for WebhookSenderBuilder {
    fn build(&mut self, _wid: usize) -> (WebhookSender, WebhookSenderArguments) {
        (
            WebhookSender,
            WebhookSenderArguments {
                webhook_url: self.webhook_url.clone(),
            },
        )
    }
}

pub async fn create_webhook_sender_factory(
    webhook_url: &String,
) -> (
    ActorRef<FactoryMessage<(), WebhookSenderMessage>>,
    tokio::task::JoinHandle<()>,
) {
    let factory_def = Factory::<
        (),
        WebhookSenderMessage,
        WebhookSenderArguments,
        WebhookSender,
        routing::QueuerRouting<(), WebhookSenderMessage>,
        queues::DefaultQueue<(), WebhookSenderMessage>,
    >::default();

    let factory_args = FactoryArguments::builder()
        .worker_builder(Box::new(WebhookSenderBuilder {
            webhook_url: webhook_url.clone(),
        }))
        .queue(Default::default())
        .router(Default::default())
        .num_initial_workers(2)
        .build();

    Actor::spawn(None, factory_def, factory_args)
        .await
        .expect("Failed to startup factory")
}
