use ractor::{Actor, ActorProcessingErr, ActorRef};

pub struct WebhookProcessor;

#[derive(Debug)]
pub enum WebhookMessage {
    PrintHelloWorld,
}

pub struct WebhookProcessorState {
    client: reqwest::Client,
    webhook_url: String,
    messages: Vec<WebhookMessage>,
}

impl Actor for WebhookProcessor {
    type State = WebhookProcessorState;
    type Msg = WebhookMessage;
    type Arguments = String;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        arguments: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(WebhookProcessorState {
            client: reqwest::Client::new(),
            webhook_url: arguments,
            messages: Vec::default(),
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            Self::Msg::PrintHelloWorld => {
                // Batch 300 at a time
                if state.messages.len() >= 300 {
                    state
                        .client
                        .post(state.webhook_url.clone())
                        .body(format!("Sending: {} messages", state.messages.len()))
                        .send()
                        .await?;

                    state.messages.clear();
                }

                state.messages.push(msg);
            }
            _ => {}
        }

        Ok(())
    }
}
