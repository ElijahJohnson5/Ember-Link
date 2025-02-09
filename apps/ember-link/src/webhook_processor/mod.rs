use actor::{WebhookProcessor, WebhookProcessorArguments, WebhookProcessorMessage};
use ractor::{Actor, ActorRef};

pub mod actor;
pub mod sender;

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
