use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;

/// Lifecycle event emitted by channels and delivered to every registered
/// [`Observer`].
///
/// Today this is the protocol-level `WebhookMessage` enum, so an observer
/// receives the same shape that webhooks deliver out-of-process.
pub use protocol::WebhookMessage as ChannelEvent;

/// Context that accompanies every channel event. Reserved for cross-cutting
/// data that's not encoded in the [`ChannelEvent`] variants themselves
/// (e.g. tenant id for multi-tenant deployments).
#[derive(Debug, Clone, Default)]
pub struct ObserverContext {
    pub tenant_id: Option<String>,
}

/// Single pluggable receiver for channel lifecycle events.
///
/// Events are queued in-process and dispatched to every registered observer
/// on a background task, so [`Observer::on_event`] runs off the channel hot
/// path. If you want fire-and-forget side effects (metrics, audit log,
/// webhook) you don't need to spawn your own task.
///
/// Use [`observer_fn`] for stateless closures; implement the trait directly
/// when you need to hold state (a DB pool, a metrics handle, etc.).
#[async_trait]
pub trait Observer: Send + Sync + 'static {
    async fn on_event(&self, event: ChannelEvent, ctx: ObserverContext);

    /// Called once during server shutdown after the event queue has drained.
    /// Use this to flush buffered work (HTTP retries, queue producers).
    async fn shutdown(&self) {}
}

/// Build an [`Observer`] from an async closure.
///
/// ```ignore
/// Server::builder()
///     .with_observer_fn(|event, ctx| async move {
///         tracing::info!(?event, tenant = ?ctx.tenant_id, "channel event");
///     })
///     .build()
///     .await;
/// ```
pub fn observer_fn<F, Fut>(f: F) -> Arc<dyn Observer>
where
    F: Fn(ChannelEvent, ObserverContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    struct FnObserver<F>(F);

    #[async_trait]
    impl<F, Fut> Observer for FnObserver<F>
    where
        F: Fn(ChannelEvent, ObserverContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        async fn on_event(&self, event: ChannelEvent, ctx: ObserverContext) {
            (self.0)(event, ctx).await
        }
    }

    Arc::new(FnObserver(f))
}
