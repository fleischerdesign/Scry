use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::*;
use crate::error::{Error, Result};

#[utoipa::path(post, path = "/api/v1/system/dashboards", responses((status = 200)), security(("api_key" = [])))]
pub async fn create_dashboard(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Value>) -> Result<impl IntoResponse> {
    let db = state.event_service.db();
    let id = Uuid::new_v4().to_string();
    let name = req["name"].as_str().ok_or_else(|| Error::BadRequest("Missing name".to_string()))?;
    
    let slug = name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .replace("--", "-");
    
    sqlx::query("INSERT INTO dashboards (id, user_id, name, slug) VALUES (?, ?, ?, ?)")
        .bind(id).bind(auth.user_id).bind(name).bind(slug).execute(db).await?;
    
    Ok(StatusCode::OK)
}

#[utoipa::path(delete, path = "/api/v1/system/dashboards/{id}/widgets/{widget_id}", responses((status = 200)), security(("api_key" = [])))]
pub async fn delete_widget(State(state): State<Arc<AppState>>, Path((_id, widget_id)): Path<(String, String)>, Extension(_auth): Extension<AuthContext>) -> Result<impl IntoResponse> {
    let db = state.event_service.db();
    sqlx::query("DELETE FROM dashboard_widgets WHERE id = ?").bind(widget_id).execute(db).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/system/dashboards", responses((status = 200, body = [Dashboard])), security(("api_key" = [])))]
pub async fn get_dashboards(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<Dashboard>>> {
    let db = state.event_service.db();
    let dashboards = sqlx::query_as::<_, (String, String, String, bool)>("SELECT id, name, COALESCE(slug, id) as slug, is_default FROM dashboards WHERE user_id = ?")
        .bind(auth.user_id).fetch_all(db).await?;
    
    let mut results = Vec::new();
    for (id, name, slug, is_default) in dashboards {
        let widgets_rows = sqlx::query_as::<_, (String, String, String, Option<String>, String, i32, i32)>(
            "SELECT id, dashboard_id, type, title, config, width_span, sort_order FROM dashboard_widgets WHERE dashboard_id = ? ORDER BY sort_order ASC"
        ).bind(&id).fetch_all(db).await?;
        
        let widgets = widgets_rows.into_iter().map(|w| DashboardWidget {
            id: w.0, dashboard_id: w.1, r#type: w.2, title: w.3,
            config: serde_json::from_str(&w.4).unwrap_or(json!({})),
            width_span: w.5, sort_order: w.6
        }).collect();
        
        results.push(Dashboard { id, name, slug, is_default, widgets });
    }
    Ok(Json(results))
}

#[utoipa::path(post, path = "/api/v1/system/dashboards/{id}/widgets", responses((status = 200)), security(("api_key" = [])))]
pub async fn add_widget(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(_auth): Extension<AuthContext>, Json(req): Json<serde_json::Value>) -> Result<impl IntoResponse> {
    let db = state.event_service.db();
    let widget_id = Uuid::new_v4().to_string();
    let w_type = req["type"].as_str().ok_or_else(|| Error::BadRequest("Missing type".to_string()))?;
    let config = serde_json::to_string(&req["config"]).unwrap_or_else(|_| "{}".to_string());
    let span = req["width_span"].as_i64().unwrap_or(1) as i32;
    
    tracing::debug!("Adding widget {} to dashboard {}", w_type, id);

    sqlx::query("INSERT INTO dashboard_widgets (id, dashboard_id, type, title, config, width_span) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(widget_id).bind(id).bind(w_type).bind(req["title"].as_str()).bind(config).bind(span).execute(db).await.map_err(|e| {
            tracing::error!("Failed to insert widget: {}", e);
            Error::Database(e)
        })?;
    
    Ok(StatusCode::OK)
}
