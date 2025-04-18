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

use crate::AppState;

pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    company: String,
    exp: usize,
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
                (
                    StatusCode::BAD_REQUEST,
                    "Failed to deserialize query string",
                )
            }
            AuthError::InvalidSignerKey(e) => {
                #[cfg(feature = "tokio")]
                tracing::error!("Invalid signer key: {:?}", e);
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

// Define a local wrapper type for JwtPayload
pub struct AuthPayload(pub TokenData<Claims>);

#[cfg(feature = "tokio")]
impl<S> FromRequestParts<S> for AuthPayload
where
    S: Send + Sync,
    S: AppState,
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

        let tenant_id = query_params.get("tenant_id").cloned();
        let token = bearer.token().to_string();

        let payload = validate_token(&token, tenant_id, state).await?;

        Ok(AuthPayload(payload))
    }
}

#[cfg(not(feature = "multi-tenant"))]
pub async fn validate_token<S>(
    token: &String,
    _tenant_id: Option<String>,
    app_state: &S,
) -> Result<AuthData, AuthError>
where
    S: Send + Sync,
    S: AppState,
{
    if let Some(jwt_signer_key) = app_state.jwt_signer_key() {
        // Validate signature of token
        return verify_token(&token, jwt_signer_key);
    } else {
        Err(AuthError::SignerKeyMissing)
    }
}

#[cfg(feature = "multi-tenant")]
pub async fn validate_token<S>(
    token: &String,
    tenant_id: Option<String>,
    app_state: &S,
) -> Result<AuthData, AuthError>
where
    S: Send + Sync,
    S: AppState,
{
    use url::Url;

    if let None = tenant_id {
        return Err(AuthError::MissingTenantId);
    }

    // We guard against this unwrap above
    let tenant_id = tenant_id.unwrap();

    {
        if let Some(signer_key) = app_state.get_cached_key(&tenant_id).await {
            return verify_token(&token, signer_key.clone());
        }
    }

    if let Some(jwt_signer_key_endpoint) = app_state.jwt_signer_key_endpoint() {
        let mut url = Url::parse(jwt_signer_key_endpoint.as_str())
            .map_err(|_e| AuthError::InvalidSignerEndpoint)?;

        url.query_pairs_mut().append_pair("tenant_id", &tenant_id);

        #[cfg(feature = "tokio")]
        tracing::info!("{:?}", url);

        let response = reqwest::get(url).await.map_err(|e| {
            #[cfg(feature = "tokio")]
            tracing::error!("Error: {e}");
            AuthError::InvalidSignerEndpoint
        })?;

        #[cfg(feature = "tokio")]
        tracing::info!("Signing Key Endpoint status: {}", response.status());
        let response = response
            .json::<protocol::SignerKeyResponse>()
            .await
            .map_err(|e| {
                #[cfg(feature = "tokio")]
                tracing::error!("Error: {e}");
                AuthError::InvalidSignerEndpoint
            })?;

        app_state
            .cache_key(tenant_id.clone(), response.public_signer_key.clone())
            .await;

        return verify_token(&token, response.public_signer_key);
    } else {
        Err(AuthError::InvalidSignerEndpoint)
    }
}

fn verify_token(token: &String, signer_key: String) -> Result<AuthData, AuthError> {
    // Validate signature of token
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
