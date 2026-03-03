use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::state::AppState;

#[utoipa::path(get, path = "/api/v1/system/status", responses((status = 200, description = "Status")))]
pub async fn get_system_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let status = state.system_service.get_status().await;
    Json(status)
}

#[utoipa::path(get, path = "/health", responses((status = 200, description = "Health Check")))]
pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.system_service.health_check().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "healthy", "db": "connected" }))),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({ "status": "unhealthy", "db": e.to_string() }))),
    }
}
