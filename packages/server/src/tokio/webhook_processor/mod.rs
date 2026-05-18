use actor::{WebhookProcessor, WebhookProcessorArguments, WebhookProcessorMessage};
use async_trait::async_trait;
use parking_lot::Mutex;
use ractor::{Actor, ActorRef};
use tokio::task::JoinHandle;

use crate::observer::{ChannelEvent, Observer, ObserverContext};

pub mod actor;
pub mod factory;

pub async fn start_webhook_processor(
    webhook_url: String,
    webhook_secret_key: String,
) -> (
    ActorRef<WebhookProcessorMessage>,
    tokio::task::JoinHandle<()>,
) {
    Actor::spawn(
        None,
        WebhookProcessor,
        WebhookProcessorArguments {
            webhook_secret_key,
            webhook_url,
        },
    )
    .await
    .expect("Could not start webhook processor")
}

/// Observer that buffers channel events and POSTs them to an HTTP webhook
/// endpoint with HMAC-SHA256 signatures. Wraps the batched [`WebhookProcessor`]
/// actor.
pub struct WebhookObserver {
    actor: ActorRef<WebhookProcessorMessage>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl WebhookObserver {
    pub async fn new(webhook_url: String, webhook_secret_key: String) -> Self {
        let (actor, handle) = start_webhook_processor(webhook_url, webhook_secret_key).await;
        Self {
            actor,
            handle: Mutex::new(Some(handle)),
        }
    }
}

#[async_trait]
impl Observer for WebhookObserver {
    async fn on_event(&self, event: ChannelEvent, ctx: ObserverContext) {
        if let Err(e) = self.actor.cast(WebhookProcessorMessage {
            msg: event,
            tenant_id: ctx.tenant_id,
        }) {
            tracing::warn!(
                error = e.to_string(),
                "WebhookObserver failed to enqueue event"
            );
        }
    }

    async fn shutdown(&self) {
        self.actor.stop(None);
        let handle = self.handle.lock().take();
        if let Some(handle) = handle {
            if let Err(e) = handle.await {
                tracing::warn!(error = e.to_string(), "WebhookObserver shutdown join failed");
            }
        }
    }
}
