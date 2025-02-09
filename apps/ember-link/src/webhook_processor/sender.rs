use hmac::{Hmac, Mac};
use protocol::WebhookMessage;
use ractor::{Actor, ActorProcessingErr, ActorRef};
use rand::Rng;
use sha2::Sha256;
use std::{collections::HashMap, time::Duration};

pub struct WebhookSender;

pub struct WebhookSenderState {
    pub messages: HashMap<String, Vec<WebhookMessage>>,
    pub webhook_url: String,
    pub signature: String,
    pub client: reqwest::Client,
    pub attempt: u64,
    pub random_seed: f32,
}

pub struct WebhookSenderArguments {
    pub messages: HashMap<String, Vec<WebhookMessage>>,
    pub webhook_url: String,
    pub webhook_secret_key: String,
}

pub enum WebhookSenderMessage {
    Send,
}

impl Actor for WebhookSender {
    type State = WebhookSenderState;
    type Msg = WebhookSenderMessage;
    type Arguments = WebhookSenderArguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("Starting webhook sender");

        let mut mac = Hmac::<Sha256>::new_from_slice(&arguments.webhook_secret_key.into_bytes())
            .expect("HMAC can take key of any size");

        if arguments.messages.contains_key("") && arguments.messages.len() == 1 {
            mac.update(
                &serde_json::to_string(&arguments.messages.get("").unwrap())
                    .unwrap()
                    .into_bytes(),
            );
        } else {
            mac.update(
                &serde_json::to_string(&arguments.messages)
                    .unwrap()
                    .into_bytes(),
            );
        }

        let result = mac.finalize().into_bytes();

        let s = hex::encode(result);

        Ok(WebhookSenderState {
            messages: arguments.messages,
            webhook_url: arguments.webhook_url,
            signature: s,
            client: reqwest::Client::new(),
            attempt: 0,
            random_seed: rand::rng().random_range(0.0..1.0),
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        _msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        tracing::info!("Sending webhook attempt number: {}", state.attempt);

        let request_future = if state.messages.contains_key("") && state.messages.len() == 1 {
            state
                .client
                .post(state.webhook_url.clone())
                .header("webhook-signature", state.signature.clone())
                .json(state.messages.get("").unwrap())
                .send()
        } else {
            state
                .client
                .post(state.webhook_url.clone())
                .header("webhook-signature", state.signature.clone())
                .json(&state.messages)
                .send()
        };

        state.attempt += 1;

        let backoff = Duration::from_millis(
            // One day is max timeout
            ((std::cmp::min(86_400_000, state.attempt * state.attempt * 1000) as f32)
                + 2000.0 * state.random_seed)
                .floor() as u64,
        );

        match request_future.await {
            Err(e) => {
                tracing::warn!(
                    "Could not send request: {}, retrying with backoff: {:?}",
                    e,
                    backoff
                );

                myself.send_after(backoff, || WebhookSenderMessage::Send);
            }

            Ok(response) => match response.error_for_status() {
                Err(e) => {
                    tracing::warn!(
                        "Error response recieved from server: {}, retrying with backoff: {:?}",
                        e,
                        backoff
                    );

                    myself.send_after(backoff, || WebhookSenderMessage::Send);
                }
                Ok(_response) => {
                    tracing::info!("Sent webhook");

                    myself.stop(Some("Success".into()));
                }
            },
        }

        Ok(())
    }
}
