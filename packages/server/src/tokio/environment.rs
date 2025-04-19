use crate::tokio::config::TokioConfig;

#[cfg(feature = "webhook")]
use crate::tokio::webhook_processor::actor::WebhookProcessorMessage;
#[cfg(feature = "webhook")]
use ractor::ActorRef;
#[cfg(feature = "webhook")]
use tokio::task::JoinHandle;

pub struct Environment {
    #[cfg(feature = "webhook")]
    webhook_processor_actor: WebhookProcessorActor,
}

#[cfg(feature = "webhook")]
pub struct WebhookProcessorActor {
    webhook_processor: ActorRef<WebhookProcessorMessage>,
    handle: JoinHandle<()>,
}

impl Environment {
    pub async fn from_config(config: &TokioConfig) -> Self {
        #[cfg(feature = "webhook")]
        let webhook_processor_actor = webhook_processor_actor(config).await;

        Self {
            #[cfg(feature = "webhook")]
            webhook_processor_actor,
        }
    }

    #[cfg(feature = "webhook")]
    pub fn webhook_processor(&self) -> ActorRef<WebhookProcessorMessage> {
        self.webhook_processor_actor.webhook_processor.clone()
    }

    pub async fn cleanup(self) {
        #[cfg(feature = "webhook")]
        {
            self.webhook_processor_actor.webhook_processor.stop(None);
            self.webhook_processor_actor.handle.await.unwrap();
        }
    }
}

#[cfg(feature = "webhook")]
async fn webhook_processor_actor(config: &TokioConfig) -> WebhookProcessorActor {
    use crate::tokio::webhook_processor::start_webhook_processor;

    let (webhook_processor, webhook_processor_handle) = start_webhook_processor(
        config.base_config.webhook_url.clone(),
        config.base_config.webhook_secret_key.clone(),
    )
    .await;

    WebhookProcessorActor {
        webhook_processor,
        handle: webhook_processor_handle,
    }
}
