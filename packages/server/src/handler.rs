use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::future::Future;
use std::sync::Arc;

use crate::auth::authenticator::AuthContext;

pub type BoxDynError = Box<dyn StdError + 'static + Send + Sync>;

/// Information about the connection and channel that delivered a custom
/// message to a [`MessageHandler`].
#[derive(Debug, Clone)]
pub struct MessageContext {
    pub channel_name: String,

    /// Per-connection identifier (random UUID assigned at WebSocket upgrade).
    /// Use this to address one specific socket. For *user* identity (i.e. who
    /// is on the other end), use `auth.subject` instead.
    pub participant_id: String,

    pub tenant_id: Option<String>,

    /// Authenticated identity of the sender. `None` for connections that
    /// bypassed authentication (`ALLOW_UNAUTHORIZED=true` with no token), or
    /// for authenticators (e.g. [`crate::auth::authenticator::AllowAllAuthenticator`])
    /// that don't produce a subject.
    pub auth: Option<AuthContext>,
}

/// Result of handling a custom inbound message.
#[derive(Debug, Clone)]
pub enum HandlerAction {
    /// Broadcast `value` (serialized to JSON string) to every participant in
    /// the channel except the sender. This is the same shape as the legacy
    /// `CustomMessage` fan-out, but with server-controlled payload.
    Broadcast(serde_json::Value),

    /// Like [`HandlerAction::Broadcast`] but includes the sender.
    BroadcastIncludingSender(serde_json::Value),

    /// Send `value` only to the sender (e.g. ack, error response).
    ReplyOnly(serde_json::Value),

    /// Accepted by the handler but no fan-out to do (e.g. handler already
    /// persisted the message and there's no realtime audience).
    Drop,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HandlerError {
    #[error("Could not parse handler payload: {0}")]
    InvalidPayload(#[source] BoxDynError),

    #[error("Handler rejected the message: {0}")]
    Rejected(String),

    #[error(transparent)]
    Other(#[from] BoxDynError),
}

/// Trait implemented by handlers for user-defined inbound message types.
///
/// Handlers are dispatched based on the `type` field in the JSON payload of
/// an incoming `CustomMessage`. Each handler registers a discriminator via
/// [`MessageHandler::message_type`].
///
/// # Example
///
/// ```ignore
/// pub struct ChatHandler { db: Arc<MyDb> }
///
/// #[async_trait]
/// impl MessageHandler for ChatHandler {
///     fn message_type(&self) -> &'static str { "chat:send" }
///
///     async fn handle(
///         &self,
///         ctx: MessageContext,
///         payload: serde_json::Value,
///     ) -> Result<HandlerAction, HandlerError> {
///         let body = payload["body"].as_str().unwrap_or("");
///         let id = self.db.insert_message(&ctx.channel_name, &ctx.participant_id, body).await?;
///         Ok(HandlerAction::Broadcast(serde_json::json!({
///             "type": "chat:received",
///             "id": id,
///             "sender": ctx.participant_id,
///             "body": body,
///         })))
///     }
/// }
/// ```
#[async_trait]
pub trait MessageHandler: Send + Sync + 'static {
    fn message_type(&self) -> &'static str;

    async fn handle(
        &self,
        ctx: MessageContext,
        payload: serde_json::Value,
    ) -> Result<HandlerAction, HandlerError>;
}

/// Adapter that turns an async closure + static message-type discriminator
/// into a [`MessageHandler`]. Built by
/// [`crate::tokio::ServerBuilder::with_handler_fn`].
pub struct HandlerFn<F> {
    message_type: &'static str,
    f: F,
}

impl<F, Fut> HandlerFn<F>
where
    F: Fn(MessageContext, serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<HandlerAction, HandlerError>> + Send + 'static,
{
    pub fn new(message_type: &'static str, f: F) -> Self {
        Self { message_type, f }
    }
}

#[async_trait]
impl<F, Fut> MessageHandler for HandlerFn<F>
where
    F: Fn(MessageContext, serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<HandlerAction, HandlerError>> + Send + 'static,
{
    fn message_type(&self) -> &'static str {
        self.message_type
    }

    async fn handle(
        &self,
        ctx: MessageContext,
        payload: serde_json::Value,
    ) -> Result<HandlerAction, HandlerError> {
        (self.f)(ctx, payload).await
    }
}

/// Registry that routes incoming custom messages to the handler whose
/// [`MessageHandler::message_type`] matches the payload's `type` field.
#[derive(Default)]
pub struct MessageHandlerRegistry {
    handlers: HashMap<&'static str, Arc<dyn MessageHandler>>,
}

impl MessageHandlerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, handler: Arc<dyn MessageHandler>) {
        let key = handler.message_type();
        self.handlers.insert(key, handler);
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Try to dispatch a raw payload string. Returns:
    /// - `None` if the payload didn't match any registered handler (caller
    ///   should fall back to legacy behavior, e.g. broadcast as-is)
    /// - `Some(Ok(action))` if a handler ran successfully
    /// - `Some(Err(_))` if a handler ran and failed
    pub async fn dispatch(
        &self,
        raw_payload: &str,
        ctx: MessageContext,
    ) -> Option<Result<HandlerAction, HandlerError>> {
        if self.handlers.is_empty() {
            return None;
        }

        let value: serde_json::Value = match serde_json::from_str(raw_payload) {
            Ok(v) => v,
            Err(_) => return None,
        };
        let ty = value.get("type")?.as_str()?;
        let handler = self.handlers.get(ty)?.clone();
        Some(handler.handle(ctx, value).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoHandler;

    #[async_trait]
    impl MessageHandler for EchoHandler {
        fn message_type(&self) -> &'static str {
            "echo"
        }

        async fn handle(
            &self,
            _ctx: MessageContext,
            payload: serde_json::Value,
        ) -> Result<HandlerAction, HandlerError> {
            Ok(HandlerAction::Broadcast(payload))
        }
    }

    fn make_ctx() -> MessageContext {
        MessageContext {
            channel_name: "test".into(),
            participant_id: "p1".into(),
            tenant_id: None,
            auth: None,
        }
    }

    #[tokio::test]
    async fn dispatch_returns_none_when_no_handlers_registered() {
        let registry = MessageHandlerRegistry::new();
        let out = registry
            .dispatch(r#"{"type":"echo"}"#, make_ctx())
            .await;
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn dispatch_returns_none_when_type_doesnt_match() {
        let mut registry = MessageHandlerRegistry::new();
        registry.insert(Arc::new(EchoHandler));

        let out = registry
            .dispatch(r#"{"type":"unknown"}"#, make_ctx())
            .await;
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn dispatch_returns_none_for_invalid_json() {
        let mut registry = MessageHandlerRegistry::new();
        registry.insert(Arc::new(EchoHandler));

        let out = registry.dispatch("not json", make_ctx()).await;
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn dispatch_returns_none_when_type_field_missing() {
        let mut registry = MessageHandlerRegistry::new();
        registry.insert(Arc::new(EchoHandler));

        let out = registry
            .dispatch(r#"{"body":"hi"}"#, make_ctx())
            .await;
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn dispatch_routes_matching_type_to_handler() {
        let mut registry = MessageHandlerRegistry::new();
        registry.insert(Arc::new(EchoHandler));

        let out = registry
            .dispatch(r#"{"type":"echo","body":"hi"}"#, make_ctx())
            .await;

        match out {
            Some(Ok(HandlerAction::Broadcast(v))) => {
                assert_eq!(v["body"], "hi");
            }
            _ => panic!("expected Broadcast action"),
        }
    }

    struct SubjectAssertingHandler {
        expected_subject: &'static str,
    }

    #[async_trait]
    impl MessageHandler for SubjectAssertingHandler {
        fn message_type(&self) -> &'static str {
            "auth_check"
        }

        async fn handle(
            &self,
            ctx: MessageContext,
            _payload: serde_json::Value,
        ) -> Result<HandlerAction, HandlerError> {
            let auth = ctx.auth.ok_or_else(|| {
                HandlerError::Rejected("missing auth context".to_string())
            })?;
            let subject = auth.subject.ok_or_else(|| {
                HandlerError::Rejected("missing subject".to_string())
            })?;
            if subject != self.expected_subject {
                return Err(HandlerError::Rejected(format!(
                    "wrong subject: {}",
                    subject
                )));
            }
            Ok(HandlerAction::Drop)
        }
    }

    #[tokio::test]
    async fn dispatch_passes_auth_context_to_handler() {
        let mut registry = MessageHandlerRegistry::new();
        registry.insert(Arc::new(SubjectAssertingHandler {
            expected_subject: "user_42",
        }));

        let ctx = MessageContext {
            channel_name: "test".into(),
            participant_id: "p1".into(),
            tenant_id: None,
            auth: Some(AuthContext {
                subject: Some("user_42".into()),
                tenant_id: None,
                claims: None,
            }),
        };

        let out = registry.dispatch(r#"{"type":"auth_check"}"#, ctx).await;
        match out {
            Some(Ok(HandlerAction::Drop)) => {}
            other => panic!("expected Drop, got {:?}", other),
        }
    }
}
