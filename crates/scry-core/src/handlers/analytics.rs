use axum::{
    extract::{State, Json, Query},
    Extension,
};
use serde_json::json;
use std::sync::Arc;

use crate::domain::*;
use crate::error::{Error, Result};
use crate::state::AppState;

#[utoipa::path(post, path = "/api/v1/analytics/discover", responses((status = 200)), security(("api_key" = [])))]
pub async fn trigger_discovery(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let count = state.analytics_service.run_correlation_discovery(auth.user_id).await?;
    Ok(Json(json!({ "status": "success", "new_discoveries": count })))
}

#[utoipa::path(get, path = "/api/v1/analytics/discoveries", responses((status = 200, body = [serde_json::Value])), security(("api_key" = [])))]
pub async fn get_discoveries(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<serde_json::Value>>> {
    let discoveries = state.analytics_service.get_discoveries(auth.user_id).await?;
    Ok(Json(discoveries))
}

#[utoipa::path(get, path = "/api/v1/analytics/stats", params(CorrelateParams), responses((status = 200, body = SemanticStats)), security(("api_key" = [])))]
pub async fn get_semantic_stats(State(state): State<Arc<AppState>>, Query(params): Query<CorrelateParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<SemanticStats>> {
    let bs = params.base_semantic.as_ref().ok_or_else(|| Error::BadRequest("base_semantic required".to_string()))?;
    let js = params.join_semantic.as_ref().ok_or_else(|| Error::BadRequest("join_semantic required".to_string()))?;
    let stats = state.event_service.calculate_semantic_stats(auth.user_id, bs, js, params.limit.unwrap_or(100)).await?;
    Ok(Json(stats))
}

#[utoipa::path(get, path = "/api/v1/analytics/semantic/top", params(SemanticParams), responses((status = 200, body = [serde_json::Value])), security(("api_key" = [])))]
pub async fn get_semantic_top(State(state): State<Arc<AppState>>, Query(params): Query<SemanticParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<serde_json::Value>>> {
    let top = state.event_service.get_semantic_top(auth.user_id, &params.semantic_type, params.limit.unwrap_or(10), params.days).await?;
    Ok(Json(top))
}

#[utoipa::path(get, path = "/api/v1/analytics/semantic/series", params(SemanticParams), responses((status = 200, body = [serde_json::Value])), security(("api_key" = [])))]
pub async fn get_semantic_series(State(state): State<Arc<AppState>>, Query(params): Query<SemanticParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<serde_json::Value>>> {
    let series = state.event_service.get_semantic_series(auth.user_id, &params.semantic_type, params.days.unwrap_or(7), params.interval).await?;
    Ok(Json(series))
}

#[utoipa::path(get, path = "/api/v1/analytics/correlations", params(CorrelateParams), responses((status = 200, body = [CorrelationResult])), security(("api_key" = [])))]
pub async fn correlate_events(State(state): State<Arc<AppState>>, Query(params): Query<CorrelateParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<CorrelationResult>>> {
    let limit = params.limit.unwrap_or(50);
    let results: Vec<serde_json::Value> = if let (Some(bs), Some(js)) = (&params.base_semantic, &params.join_semantic) {
        state.event_service.correlate_semantic(auth.user_id, bs, js, limit).await
    } else if let (Some(bc), Some(jc)) = (&params.base_category, &params.join_category) {
        state.event_service.correlate_nearest(auth.user_id, bc, jc, limit).await
    } else { return Err(Error::BadRequest("Invalid params".to_string())); }?;
    let api_results = results.into_iter().map(|v| CorrelationResult { 
        base: v.get("base").cloned().unwrap_or(serde_json::json!({})), 
        joined: v.get("joined").cloned().unwrap_or(serde_json::json!({})), 
    }).collect();
    Ok(Json(api_results))
}

#[utoipa::path(get, path = "/api/v1/discovery/search", params(SearchParams), responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn search_events(State(state): State<Arc<AppState>>, Query(params): Query<SearchParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let q = params.q.trim();
    if q.is_empty() { return Ok(Json(serde_json::Value::Array(vec![]))); }

    let results = state.analytics_service.search(auth.user_id, q, params.limit.unwrap_or(20)).await?;
    Ok(Json(serde_json::Value::Array(results)))
}
