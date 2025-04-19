use server::cloudflare::{self, CloudflareQueueMessage, Options};

use worker::*;

#[event(fetch)]
async fn main(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    let server = match cloudflare::Server::new(
        env,
        Options {
            queue_name: "WEBHOOK_EVENTS".to_string(),
        },
    ) {
        Err(response) => return Ok(response),
        Ok(server) => server,
    };

    server.handle_fetch(req).await
}

#[event(queue)]
async fn queue_handler(
    message_batch: MessageBatch<CloudflareQueueMessage>,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    let server = match cloudflare::Server::new(
        env,
        Options {
            queue_name: "WEBHOOK_EVENTS".to_string(),
        },
    ) {
        Err(_response) => {
            return Err(worker::Error::RustError(
                "Could not parse config from env".into(),
            ))
        }
        Ok(server) => server,
    };

    server.handle_queue(message_batch).await
}
