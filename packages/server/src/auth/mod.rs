use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use protocol::WebSocketCloseCode;
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;

#[cfg(feature = "tokio")]
use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
    RequestPartsExt,
};
#[cfg(feature = "tokio")]
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
#[cfg(feature = "tokio")]
use std::collections::HashMap;

pub mod authenticator;

pub use authenticator::{
    AllowAllAuthenticator, AuthContext, Authenticator, AuthenticatorFn, JwtAuthenticator,
};
#[cfg(feature = "tokio")]
pub(crate) use authenticator::HasAuthenticator;
#[cfg(feature = "multi-tenant")]
pub use authenticator::MultiTenantJwtAuthenticator;

pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    pub exp: usize,
}

pub type AuthData = TokenData<Claims>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AuthError {
    #[error("Could not find credentials as part of request")]
    MissingCredentials,

    #[error("Token is invalid for request")]
    InvalidToken,

    #[error("Could not deserialize query string")]
    FailedToDeserializeQueryString(#[source] BoxDynError),

    #[cfg(feature = "multi-tenant")]
    #[error("Signer endpoint could not be called for multi tenant verification")]
    InvalidSignerEndpoint,

    #[error("Signer key is invalid: {0}")]
    InvalidSignerKey(#[source] BoxDynError),

    #[error("Signer key is missing")]
    SignerKeyMissing,

    #[cfg(feature = "multi-tenant")]
    #[error("Missing tenant id for multi tenant verification")]
    MissingTenantId,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::FailedToDeserializeQueryString(e) => {
                #[cfg(feature = "tokio")]
                tracing::error!("Failed to deserialize query string: {:?}", e);
                let _ = e;
                (
                    StatusCode::BAD_REQUEST,
                    "Failed to deserialize query string",
                )
            }
            AuthError::InvalidSignerKey(e) => {
                #[cfg(feature = "tokio")]
                tracing::error!("Invalid signer key: {:?}", e);
                let _ = e;
                (StatusCode::INTERNAL_SERVER_ERROR, "Invalid signer key")
            }
            AuthError::SignerKeyMissing => {
                #[cfg(feature = "tokio")]
                tracing::error!("Signer key is missing");
                (StatusCode::INTERNAL_SERVER_ERROR, "Signer key is missing")
            }
            #[cfg(feature = "multi-tenant")]
            AuthError::InvalidSignerEndpoint => {
                #[cfg(feature = "tokio")]
                tracing::error!("Invalid signer endpoint");
                (StatusCode::INTERNAL_SERVER_ERROR, "Invalid signer endpoint")
            }
            #[cfg(feature = "multi-tenant")]
            AuthError::MissingTenantId => {
                #[cfg(feature = "tokio")]
                tracing::error!("Missing tenant id");
                (StatusCode::BAD_REQUEST, "Missing tenant id")
            }
        };
        (status, error_message).into_response()
    }
}

/// Axum extractor that authenticates an incoming HTTP request using the
/// configured [`Authenticator`].
pub struct AuthPayload(pub AuthContext);

#[cfg(feature = "tokio")]
impl<S> FromRequestParts<S> for AuthPayload
where
    S: Send + Sync + HasAuthenticator,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let query_params = parts
            .extract::<Query<HashMap<String, String>>>()
            .await
            .map(|Query(params)| params)
            .map_err(|err| AuthError::FailedToDeserializeQueryString(Box::new(err)))?;

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::MissingCredentials)?;

        let tenant_id = query_params.get("tenant_id").map(String::as_str);
        let token = bearer.token();

        let ctx = state
            .authenticator()
            .authenticate(Some(token), tenant_id)
            .await?;

        Ok(AuthPayload(ctx))
    }
}

pub(crate) fn verify_token(token: &str, signer_key: &str) -> Result<AuthData, AuthError> {
    match DecodingKey::from_rsa_pem(signer_key.as_bytes()) {
        Ok(decoding_key) => match decode::<Claims>(token, &decoding_key, &Validation::default()) {
            Ok(token_data) => Ok(token_data),
            Err(_e) => Err(AuthError::InvalidToken),
        },
        Err(e) => Err(AuthError::InvalidSignerKey(Box::new(e))),
    }
}

impl Into<WebSocketCloseCode> for AuthError {
    fn into(self) -> WebSocketCloseCode {
        match self {
            AuthError::MissingCredentials => WebSocketCloseCode::TokenNotFound,
            AuthError::InvalidToken => WebSocketCloseCode::InvalidToken,
            #[cfg(feature = "multi-tenant")]
            AuthError::InvalidSignerEndpoint => WebSocketCloseCode::InvalidSignerKey,
            AuthError::InvalidSignerKey(_) => WebSocketCloseCode::InvalidSignerKey,
            AuthError::SignerKeyMissing => WebSocketCloseCode::InvalidSignerKey,
            AuthError::FailedToDeserializeQueryString(_) => WebSocketCloseCode::InvalidSignerKey,
            #[cfg(feature = "multi-tenant")]
            AuthError::MissingTenantId => WebSocketCloseCode::InvalidSignerKey,
        }
    }
}
