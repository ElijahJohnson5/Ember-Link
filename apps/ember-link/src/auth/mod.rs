use std::collections::HashMap;

use axum::{
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use josekit::{
    jws,
    jwt::{self, JwtPayload},
};
use protocol::WebSocketCloseCode;
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::{AppState, BoxDynError};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AuthError {
    #[error("Could not find credentials as part of request")]
    MissingCredentials,

    #[error("Token is invalid for request")]
    InvalidToken,

    #[error("Could not deserialize query string")]
    FailedToDeserializeQueryString(#[source] BoxDynError),

    #[error("Signer endpoint could not be called for multi tenant verification")]
    InvalidSignerEndpoint,

    #[error("Signer key is invalid: {0}")]
    InvalidSignerKey(#[source] BoxDynError),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::FailedToDeserializeQueryString(e) => {
                tracing::error!("Failed to deserialize query string: {:?}", e);
                (
                    StatusCode::BAD_REQUEST,
                    "Failed to deserialize query string",
                )
            }
            AuthError::InvalidSignerKey(e) => {
                tracing::error!("Invalid signer key: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Invalid signer key")
            }
            AuthError::InvalidSignerEndpoint => {
                tracing::error!("Invalid signer endpoint");
                (StatusCode::INTERNAL_SERVER_ERROR, "Invalid signer endpoint")
            }
        };
        let body = axum::Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

impl FromRequestParts<AppState> for JwtPayload {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
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

        let payload = validate_token(&token, tenant_id, &state).await?;

        Ok(payload)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignerKeyResponse {
    public_signer_key: String,
}

pub async fn validate_token(
    token: &String,
    tenant_id: Option<String>,
    app_state: &AppState,
) -> Result<jwt::JwtPayload, AuthError> {
    if let Some(tenant_id) = tenant_id {
        {
            let jwt_signer_key_cache = app_state.jwt_signer_key_cache.lock().await;

            if let Some(signer_key) = jwt_signer_key_cache.get(&tenant_id) {
                return verify_token(&token, signer_key.clone());
            }
        }

        if let Some(jwt_signer_key_endpoint) = app_state.config.jwt_signer_key_endpoint.clone() {
            let mut url = Url::parse(&jwt_signer_key_endpoint)
                .map_err(|_e| AuthError::InvalidSignerEndpoint)?;

            url.query_pairs_mut().append_pair("tenant_id", &tenant_id);

            tracing::info!("{:?}", url);

            let response = reqwest::get(url).await.map_err(|e| {
                tracing::error!("Error: {e}");
                AuthError::InvalidSignerEndpoint
            })?;

            tracing::info!("Signing Key Endpoint status: {}", response.status());
            let response = response.json::<SignerKeyResponse>().await.map_err(|e| {
                tracing::error!("Error: {e}");
                AuthError::InvalidSignerEndpoint
            })?;

            app_state
                .jwt_signer_key_cache
                .lock()
                .await
                .insert(tenant_id.clone(), response.public_signer_key.clone());

            return verify_token(&token, response.public_signer_key);
        } else {
            Err(AuthError::InvalidSignerEndpoint)
        }
    } else if let Some(jwt_signer_key) = app_state.config.jwt_signer_key.clone() {
        // Validate signature of token
        return verify_token(&token, jwt_signer_key);
    } else {
        Err(AuthError::InvalidSignerEndpoint)
    }
}

fn verify_token(token: &String, signer_key: String) -> Result<jwt::JwtPayload, AuthError> {
    // Validate signature of token
    match jws::RS256.verifier_from_pem(signer_key) {
        Ok(jwt_verifier) => match jwt::decode_with_verifier(token, &jwt_verifier) {
            Ok((payload, _header)) => Ok(payload),
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
            AuthError::InvalidSignerEndpoint => WebSocketCloseCode::InvalidSignerKey,
            AuthError::InvalidSignerKey(_) => WebSocketCloseCode::InvalidSignerKey,
            AuthError::FailedToDeserializeQueryString(_) => WebSocketCloseCode::InvalidSignerKey,
        }
    }
}
