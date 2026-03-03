use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::models::*;

#[utoipa::path(get, path = "/api/v1/system/status", responses((status = 200, description = "Status")))]
pub async fn get_system_status() -> impl IntoResponse {
    Json(json!({ "status": "online", "multi_tenant": true }))
}

#[utoipa::path(get, path = "/health", responses((status = 200, description = "Health Check")))]
pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.event_service.db().acquire().await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "healthy", "db": "connected" }))),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "status": "unhealthy", "db": e.to_string() }))),
    }
}
