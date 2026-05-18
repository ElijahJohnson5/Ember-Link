#![recursion_limit = "256"]

use axum::response::IntoResponse;
use protocol::WebSocketCloseCode;

#[cfg(feature = "multi-tenant")]
use http::StatusCode;

#[cfg(feature = "cloudflare")]
pub mod cloudflare;

#[cfg(feature = "tokio")]
pub mod tokio;

pub mod auth;
pub mod channel;
mod config;
pub mod handler;
pub mod observer;
mod storage;
mod webhook;

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
