use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use josekit::jwt::JwtPayload;
use protocol::{server::ServerMessage, CustomMessage};

use crate::AppState;

async fn api_fallback() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "status": "Not Found" })),
    )
}

async fn broadcast(
    Path(channel_name): Path<String>,
    State(state): State<AppState>,
    _jwt: JwtPayload,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // TODO: Probably should just put it in some redis pub sub queue so that every server is notified (once redis is used for multiple servers)
    if let Some(weak_channel) = state.channel_registry.get_channel(&channel_name).await {
        match weak_channel.upgrade() {
            Some(channel) => {
                channel.broadcast(ServerMessage::Custom(CustomMessage { data: payload }), None);
            }
            None => {
                tracing::info!("Channel has been dropped: {}", channel_name);
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "status": "Channel Not Found" })),
                );
            }
        }
    } else {
        tracing::info!("Channel not found: {}", channel_name);
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "status": "Channel Not Found" })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "Broadcasted" })),
    )
}

pub fn api_routes() -> Router<AppState> {
    let api_routes = Router::new()
        .route("/broadcast/{channel_name}", post(broadcast))
        .fallback(api_fallback);

    api_routes
}
