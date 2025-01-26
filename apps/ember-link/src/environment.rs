use protocol::WebhookMessage;
use ractor::ActorRef;
use tokio::task::JoinHandle;

use crate::{config::Config, webhook_processor::start_webhook_processor};

pub struct Environment {
    webhook_processor_actor: Option<WebhookProcessorActor>,
}

pub struct WebhookProcessorActor {
    webhook_processor: ActorRef<WebhookMessage>,
    handle: JoinHandle<()>,
}

impl Environment {
    pub async fn from_config(config: &Config) -> Self {
        let mut webhook_processor_actor = None;

        if let Some(webhook_url) = config.webhook_url.clone() {
            let (webhook_processor, webhook_processor_handle) =
                start_webhook_processor(webhook_url).await;

            webhook_processor_actor.replace(WebhookProcessorActor {
                webhook_processor,
                handle: webhook_processor_handle,
            });
        }

        Self {
            webhook_processor_actor,
        }
    }

    pub fn webhook_processor(&self) -> Option<ActorRef<WebhookMessage>> {
        self.webhook_processor_actor
            .as_ref()
            .map(|webhook_processor_actor| webhook_processor_actor.webhook_processor.clone())
    }

    pub async fn cleanup(self) {
        if let Some(webhook_processor_actor) = self.webhook_processor_actor {
            webhook_processor_actor.webhook_processor.stop(None);
            webhook_processor_actor.handle.await.unwrap();
        }
    }
}
