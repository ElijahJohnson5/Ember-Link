use async_trait::async_trait;
use std::future::Future;
use std::sync::Arc;

#[cfg(feature = "multi-tenant")]
use tokio::sync::Mutex;
#[cfg(feature = "multi-tenant")]
use std::collections::HashMap;

use super::{verify_token, AuthError, Claims};

/// Result returned by an [`Authenticator`] after validating a connection.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Subject identifier (e.g. user id). `None` if the authenticator does not
    /// produce one (e.g. [`AllowAllAuthenticator`]).
    pub subject: Option<String>,

    /// Tenant id resolved at auth time. Present whenever the connection
    /// provided one, regardless of whether the authenticator inspected it.
    pub tenant_id: Option<String>,

    /// Raw claims for embedder inspection.
    pub claims: Option<serde_json::Value>,
}

#[async_trait]
pub trait Authenticator: Send + Sync + 'static {
    async fn authenticate(
        &self,
        token: Option<&str>,
        tenant_id: Option<&str>,
    ) -> Result<AuthContext, AuthError>;
}

/// Trait implemented by app-state types that expose an [`Authenticator`] to
/// axum extractors. Internal plumbing — not part of the public API surface.
pub(crate) trait HasAuthenticator {
    fn authenticator(&self) -> &Arc<dyn Authenticator>;
}

/// Accepts any connection. Useful for development and for the
/// `ALLOW_UNAUTHORIZED` mode.
pub struct AllowAllAuthenticator;

#[async_trait]
impl Authenticator for AllowAllAuthenticator {
    async fn authenticate(
        &self,
        _token: Option<&str>,
        tenant_id: Option<&str>,
    ) -> Result<AuthContext, AuthError> {
        Ok(AuthContext {
            subject: None,
            tenant_id: tenant_id.map(str::to_owned),
            claims: None,
        })
    }
}

/// Validates JWTs signed with a single static RSA public key.
pub struct JwtAuthenticator {
    signer_key: String,
}

impl JwtAuthenticator {
    pub fn new(signer_key: impl Into<String>) -> Self {
        Self {
            signer_key: signer_key.into(),
        }
    }
}

#[async_trait]
impl Authenticator for JwtAuthenticator {
    async fn authenticate(
        &self,
        token: Option<&str>,
        tenant_id: Option<&str>,
    ) -> Result<AuthContext, AuthError> {
        let token = token.ok_or(AuthError::MissingCredentials)?;
        let data = verify_token(token, &self.signer_key)?;
        Ok(claims_to_context(data.claims, tenant_id))
    }
}

/// Resolves per-tenant signer keys from an HTTP endpoint, with an in-memory
/// cache. Falls back to no static key.
#[cfg(feature = "multi-tenant")]
pub struct MultiTenantJwtAuthenticator {
    signer_key_endpoint: String,
    cache: Mutex<HashMap<String, String>>,
}

#[cfg(feature = "multi-tenant")]
impl MultiTenantJwtAuthenticator {
    pub fn new(signer_key_endpoint: impl Into<String>) -> Self {
        Self {
            signer_key_endpoint: signer_key_endpoint.into(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    async fn fetch_key(&self, tenant_id: &str) -> Result<String, AuthError> {
        use url::Url;

        let mut url = Url::parse(&self.signer_key_endpoint)
            .map_err(|_| AuthError::InvalidSignerEndpoint)?;
        url.query_pairs_mut().append_pair("tenant_id", tenant_id);

        let response = reqwest::get(url)
            .await
            .map_err(|_| AuthError::InvalidSignerEndpoint)?;
        let response = response
            .json::<protocol::SignerKeyResponse>()
            .await
            .map_err(|_| AuthError::InvalidSignerEndpoint)?;

        Ok(response.public_signer_key)
    }
}

#[cfg(feature = "multi-tenant")]
#[async_trait]
impl Authenticator for MultiTenantJwtAuthenticator {
    async fn authenticate(
        &self,
        token: Option<&str>,
        tenant_id: Option<&str>,
    ) -> Result<AuthContext, AuthError> {
        let token = token.ok_or(AuthError::MissingCredentials)?;
        let tenant_id = tenant_id.ok_or(AuthError::MissingTenantId)?;

        if let Some(cached) = self.cache.lock().await.get(tenant_id).cloned() {
            let data = verify_token(token, &cached)?;
            return Ok(claims_to_context(data.claims, Some(tenant_id)));
        }

        let key = self.fetch_key(tenant_id).await?;
        self.cache
            .lock()
            .await
            .insert(tenant_id.to_owned(), key.clone());

        let data = verify_token(token, &key)?;
        Ok(claims_to_context(data.claims, Some(tenant_id)))
    }
}

/// Adapter that turns an async closure into an [`Authenticator`]. Built by
/// [`crate::tokio::ServerBuilder::with_auth_fn`]; embedders rarely need to
/// construct it directly.
pub struct AuthenticatorFn<F>(F);

impl<F, Fut> AuthenticatorFn<F>
where
    F: Fn(Option<String>, Option<String>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<AuthContext, AuthError>> + Send + 'static,
{
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

#[async_trait]
impl<F, Fut> Authenticator for AuthenticatorFn<F>
where
    F: Fn(Option<String>, Option<String>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<AuthContext, AuthError>> + Send + 'static,
{
    async fn authenticate(
        &self,
        token: Option<&str>,
        tenant_id: Option<&str>,
    ) -> Result<AuthContext, AuthError> {
        (self.0)(token.map(String::from), tenant_id.map(String::from)).await
    }
}

fn claims_to_context(claims: Claims, tenant_id: Option<&str>) -> AuthContext {
    let subject = Some(claims.sub.clone());
    let json = serde_json::to_value(&claims).ok();
    AuthContext {
        subject,
        tenant_id: tenant_id.map(str::to_owned),
        claims: json,
    }
}
