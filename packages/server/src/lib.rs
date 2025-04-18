use axum::response::IntoResponse;
use mutually_exclusive_features::exactly_one_of;
use protocol::WebSocketCloseCode;

#[cfg(feature = "multi-tenant")]
use http::StatusCode;

exactly_one_of!("cloudflare", "tokio");

#[cfg(feature = "cloudflare")]
pub mod cloudflare;

#[cfg(feature = "tokio")]
pub mod tokio;

mod auth;
pub mod channel;
mod config;
mod storage;
mod webhook;

#[async_trait::async_trait]
#[allow(unused_variables)]
trait AppState: Send + Sync {
    #[cfg(feature = "multi-tenant")]
    fn jwt_signer_key_endpoint(&self) -> Option<String>;
    fn jwt_signer_key(&self) -> Option<String>;

    #[cfg(feature = "multi-tenant")]
    async fn get_cached_key(&self, tenant_id: &String) -> Option<String> {
        None
    }
    #[cfg(feature = "multi-tenant")]
    async fn cache_key(&self, tenant_id: String, key: String) {}
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EmberLinkError {
    #[cfg(feature = "multi-tenant")]
    #[error("Missing tenant id during multi tenant operation")]
    MissingTenantId,
}

impl IntoResponse for EmberLinkError {
    fn into_response(self) -> axum::response::Response {
        match self {
            #[cfg(feature = "multi-tenant")]
            EmberLinkError::MissingTenantId => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
        }
    }
}

impl Into<WebSocketCloseCode> for EmberLinkError {
    fn into(self) -> WebSocketCloseCode {
        match self {
            #[cfg(feature = "multi-tenant")]
            EmberLinkError::MissingTenantId => WebSocketCloseCode::MissingTenantId,
        }
    }
}
