use actor::WebhookProcessor;
use protocol::WebhookMessage;
use ractor::{Actor, ActorRef};

pub mod actor;

pub async fn start_webhook_processor(
    webhook_url: String,
) -> (ActorRef<WebhookMessage>, tokio::task::JoinHandle<()>) {
    Actor::spawn(None, WebhookProcessor, webhook_url)
        .await
        .expect("Could not start webhook processor")
}
