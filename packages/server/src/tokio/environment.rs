use std::sync::Arc;

use crate::observer::Observer;
use crate::tokio::config::TokioConfig;

/// Default observers loaded from environment (e.g. the webhook observer when
/// the `webhook` feature is enabled and `WEBHOOK_URL` is set). Embedder-supplied
/// observers from the [`crate::tokio::ServerBuilder`] are appended on top.
pub struct Environment {
    observers: Vec<Arc<dyn Observer>>,
}

impl Environment {
    pub async fn from_config(config: &TokioConfig) -> Self {
        #[cfg_attr(not(feature = "webhook"), allow(unused_mut))]
        let mut observers: Vec<Arc<dyn Observer>> = Vec::new();

        #[cfg(feature = "webhook")]
        {
            use crate::tokio::webhook_processor::WebhookObserver;

            observers.push(Arc::new(
                WebhookObserver::new(
                    config.base_config.webhook_url.clone(),
                    config.base_config.webhook_secret_key.clone(),
                )
                .await,
            ));
        }

        let _ = config;
        Self { observers }
    }

    pub(crate) fn into_observers(self) -> Vec<Arc<dyn Observer>> {
        self.observers
    }
}
