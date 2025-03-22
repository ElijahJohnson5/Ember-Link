use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use josekit::jwt::JwtPayload;

use crate::AppState;

async fn api_fallback() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "status": "Not Found" })),
    )
}

async fn publish(
    Path(channel_name): Path<String>,
    State(state): State<AppState>,
    test: JwtPayload,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!(
        "Received publish request for channel: {}, payload: {:?}, jwt: {:?}",
        channel_name,
        payload,
        test
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "Published" })),
    )
}

pub fn api_routes() -> Router<AppState> {
    let api_routes = Router::new()
        .route("/publish/{channel_name}", post(publish))
        .fallback(api_fallback);

    api_routes
}
