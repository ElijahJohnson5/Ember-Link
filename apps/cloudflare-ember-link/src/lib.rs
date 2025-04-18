use server::cloudflare::{fetch, queue, CloudflareQueueMessage, Options};

use worker::*;

#[event(fetch)]
async fn main(
    req: HttpRequest,
    env: Env,
    ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    fetch(
        req,
        env,
        Options {
            queue_name: "WEBHOOK_EVENTS".to_string(),
        },
        ctx,
    )
    .await
}

#[event(queue)]
async fn queue_handler(
    message_batch: MessageBatch<CloudflareQueueMessage>,
    env: Env,
    ctx: Context,
) -> Result<()> {
    queue(message_batch, env, ctx).await
}
