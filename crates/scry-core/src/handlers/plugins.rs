use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use std::sync::Arc;

use crate::models::*;
use crate::error::Result;

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/reports/{report_id}", responses((status = 200, body = ApiReportData)), security(("api_key" = [])))]
pub async fn run_plugin_report(
    State(state): State<Arc<AppState>>,
    Path((id, report_id)): Path<(String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<ApiReportData>> {
    let res = state.plugin_service.run_plugin_report(auth.user_id, &id, report_id).await?;
    Ok(Json(res))
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/config", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_plugin_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let res = state.plugin_service.get_plugin_config(auth.user_id, &id).await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/config", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_plugin_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Map<String, serde_json::Value>>) -> Result<impl IntoResponse> {
    state.plugin_service.update_plugin_config(auth.user_id, &id, req).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/discovery/catalog", responses((status = 200, description = "Catalog")), security(("api_key" = [])))]
pub async fn get_catalog(State(state): State<Arc<AppState>>, Extension(_auth): Extension<AuthContext>) -> impl IntoResponse {
    let catalog = state.plugin_service.get_catalog().await;
    Json(catalog)
}

#[utoipa::path(get, path = "/api/v1/system/plugins", responses((status = 200, body = [PluginStatus])), security(("api_key" = [])))]
pub async fn get_system_plugins(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<PluginStatus>>> {
    let res = state.plugin_service.get_system_plugins(auth.user_id).await?;
    Ok(Json(res))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/poll", responses((status = 200, description = "Poll")), security(("api_key" = [])))]
pub async fn poll_plugin_manually(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let count = state.plugin_service.poll_plugin_manually(auth.user_id, &id).await?;
    Ok(Json(serde_json::json!({ "plugin": id, "events_saved": count })))
}
