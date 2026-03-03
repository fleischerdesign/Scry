use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use std::sync::Arc;

use crate::domain::*;
use crate::error::{Error, Result};
use crate::state::AppState;

#[utoipa::path(post, path = "/api/v1/system/dashboards", responses((status = 200)), security(("api_key" = [])))]
pub async fn create_dashboard(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Value>) -> Result<impl IntoResponse> {
    let name = req["name"].as_str().ok_or_else(|| Error::BadRequest("Missing name".to_string()))?;
    state.dashboard_service.create_dashboard(auth.user_id, name).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(delete, path = "/api/v1/system/dashboards/{id}/widgets/{widget_id}", responses((status = 200)), security(("api_key" = [])))]
pub async fn delete_widget(State(state): State<Arc<AppState>>, Path((_id, widget_id)): Path<(String, String)>, Extension(auth): Extension<AuthContext>) -> Result<impl IntoResponse> {
    state.dashboard_service.delete_widget(auth.user_id, &widget_id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/system/dashboards", responses((status = 200, body = [Dashboard])), security(("api_key" = [])))]
pub async fn get_dashboards(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<Dashboard>>> {
    let res = state.dashboard_service.get_dashboards(auth.user_id).await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/system/dashboards/{id}/widgets", responses((status = 200)), security(("api_key" = [])))]
pub async fn add_widget(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Value>) -> Result<impl IntoResponse> {
    let w_type = req["type"].as_str().ok_or_else(|| Error::BadRequest("Missing type".to_string()))?;
    let title = req["title"].as_str();
    
    state.dashboard_service.add_widget(auth.user_id, &id, w_type, title, req["config"].clone()).await?;
    
    Ok(StatusCode::OK)
}
