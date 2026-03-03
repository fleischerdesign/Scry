use axum::{
    extract::{State, Json, Query, Path},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use scry_proto::Event;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::{PasswordHash, SaltString, rand_core::OsRng};
use serde_json::json;
use crate::models::*;
use validator::Validate;
use crate::error::{Error, Result};

// --- Handlers ---

use axum::response::sse::{Event as SseEvent, Sse};
use futures::stream::Stream;
use std::convert::Infallible;

pub async fn stream_live_events(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Sse<impl Stream<Item = std::result::Result<SseEvent, Infallible>>> {
    let mut rx = state.event_sender.subscribe();
    let cancel_token = state.cancel_token.clone();

    let stream = async_stream::stream! {
        loop {
            tokio::select! {
                // Beende den Stream, wenn der Server herunterfährt
                _ = cancel_token.cancelled() => {
                    break;
                }
                // Warte auf neue Events
                res = rx.recv() => {
                    match res {
                        Ok(event) => {
                            let is_user_event = event.metadata.as_ref()
                                .and_then(|m| m.get("user_id"))
                                .and_then(|u| u.as_i64()) == Some(auth.user_id);
                            
                            if is_user_event {
                                if let Ok(data) = serde_json::to_string(&event) {
                                    yield Ok(SseEvent::default().data(data));
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }
            }
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterRequest, responses((status = 200, body = AuthResponse)))]
pub async fn register_user(State(state): State<Arc<AppState>>, Json(req): Json<RegisterRequest>) -> Result<Json<AuthResponse>> {
    req.validate()?;
    let db = state.event_service.db();
    
    // Hash password with Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| Error::Internal)?
        .to_string();

    let res = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&req.username).bind(password_hash).execute(db).await?;
    
    let user_id = res.last_insert_rowid();
    
    // Ensure the 'self' user entity exists in the graph
    sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
        .bind(user_id).bind("scry.core").bind("user").bind("self").execute(db).await?;

    let api_key = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO api_keys (key, user_id, label, scopes) VALUES (?, ?, ?, ?)")
        .bind(&api_key).bind(user_id).bind("Default Key").bind("all").execute(db).await?;

    Ok(Json(AuthResponse { api_key, user: User { id: user_id, username: req.username } }))
}

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest, responses((status = 200, body = AuthResponse)))]
pub async fn login_user(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> Result<Json<AuthResponse>> {
    req.validate()?;
    let db = state.event_service.db();
    let user = sqlx::query_as::<_, (i64, String, String)>("SELECT id, username, password_hash FROM users WHERE username = ?")
        .bind(&req.username).fetch_optional(db).await?
        .ok_or_else(|| Error::Auth("User not found".to_string()))?;
    
    // Verify password with Argon2
    let parsed_hash = PasswordHash::new(&user.2)
        .map_err(|_| Error::Internal)?;
    
    if Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_err() {
        return Err(Error::Auth("Invalid password".to_string()));
    }

    // Ensure the 'self' user entity exists in the graph (migration for old users)
    sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
        .bind(user.0).bind("scry.core").bind("user").bind("self").execute(db).await?;

    let api_key = sqlx::query_scalar::<_, String>("SELECT key FROM api_keys WHERE user_id = ? LIMIT 1").bind(user.0).fetch_one(db).await?;
    Ok(Json(AuthResponse { api_key, user: User { id: user.0, username: user.1 } }))
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/reports/{report_id}", responses((status = 200, body = ApiReportData)), security(("api_key" = [])))]
pub async fn run_plugin_report(
    State(state): State<Arc<AppState>>,
    Path((id, report_id)): Path<(String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<ApiReportData>> {
    let data = state.event_service.plugin_manager().run_plugin_report(auth.user_id, &id, report_id).await?;
    Ok(Json(ApiReportData {
        columns: data.columns,
        data_json: data.data_json,
    }))
}

#[utoipa::path(get, path = "/api/v1/system/profile", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_profile(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let db = state.event_service.db();

    // 1. Self-Healing: Ensure 'self' user entity exists
    sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
        .bind(auth.user_id).bind("scry.core").bind("user").bind("self").execute(db).await?;

    // 2. Load legacy profile rows
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM user_profile WHERE user_id = ?")
        .bind(auth.user_id).fetch_all(db).await?;
    
    let mut map = serde_json::Map::new();
    for (k, v) in rows {
        map.insert(k.clone(), json!(v));
        
        // 3. Auto-Sync to Knowledge Graph if not already there
        let trait_id = format!("scry.core/{}", k);
        let value_json = json!(v).to_string();
        
        sqlx::query("INSERT INTO entity_traits (user_id, namespace, entity_type, entity_id, plugin_id, trait_id, value_json) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO NOTHING")
            .bind(auth.user_id).bind("scry.core").bind("user").bind("self").bind("core").bind(trait_id).bind(value_json).execute(db).await?;
    }
    
    Ok(Json(serde_json::Value::Object(map)))
}

#[utoipa::path(post, path = "/api/v1/system/profile", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_profile(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Map<String, serde_json::Value>>) -> Result<impl IntoResponse> {
    let db = state.event_service.db();
    for (k, v) in req {
        let v_str = v.as_str().unwrap_or("").to_string();
        
        // 1. Update legacy table
        sqlx::query("INSERT INTO user_profile (user_id, key, value) VALUES (?, ?, ?) ON CONFLICT(user_id, key) DO UPDATE SET value = EXCLUDED.value")
            .bind(auth.user_id).bind(&k).bind(&v_str).execute(db).await?;

        // 2. Update semantic graph (Trait)
        // We use 'scry.core' as the trait namespace for profile values
        let trait_id = format!("scry.core/{}", k);
        let value_json = serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string());
        
        sqlx::query("INSERT INTO entity_traits (user_id, namespace, entity_type, entity_id, plugin_id, trait_id, value_json) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(user_id, namespace, entity_type, entity_id, plugin_id, trait_id) DO UPDATE SET value_json = EXCLUDED.value_json")
            .bind(auth.user_id).bind("scry.core").bind("user").bind("self").bind("core").bind(trait_id).bind(value_json).execute(db).await?;
    }
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/config", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_plugin_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let db = state.event_service.db();
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM plugin_config WHERE user_id = ? AND plugin_id = ?")
        .bind(auth.user_id).bind(&id).fetch_all(db).await?;
    
    let mut map = serde_json::Map::new();
    for (k, v) in rows {
        map.insert(k, json!(v));
    }
    Ok(Json(serde_json::Value::Object(map)))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/config", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_plugin_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Map<String, serde_json::Value>>) -> Result<impl IntoResponse> {
    let db = state.event_service.db();
    for (k, v) in req {
        let v_str = v.as_str().unwrap_or("").to_string();
        sqlx::query("INSERT INTO plugin_config (user_id, plugin_id, key, value) VALUES (?, ?, ?, ?) ON CONFLICT(user_id, plugin_id, key) DO UPDATE SET value = EXCLUDED.value")
            .bind(auth.user_id).bind(&id).bind(k).bind(v_str).execute(db).await?;
    }
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/discovery/catalog", responses((status = 200, description = "Catalog")), security(("api_key" = [])))]
pub async fn get_catalog(State(state): State<Arc<AppState>>, Extension(_auth): Extension<AuthContext>) -> impl IntoResponse {
    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
    let mut catalog = serde_json::Map::new();
    for (plugin_name, manifest) in manifests {
        for export in manifest.exports {
            let entry = json!({ 
                "plugin": plugin_name, 
                "path": export.path, 
                "description": export.description, 
                "category": export.category 
            });
            let array = catalog.entry(export.semantic_type).or_insert_with(|| json!([]));
            if let Some(arr) = array.as_array_mut() {
                arr.push(entry);
            }
        }
    }
    Json(catalog)
}

#[utoipa::path(get, path = "/api/v1/discovery/search", params(SearchParams), responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn search_events(State(state): State<Arc<AppState>>, Query(params): Query<SearchParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let q = params.q.trim();
    if q.is_empty() { return Ok(Json(serde_json::Value::Array(vec![]))); }

    let db = state.event_service.db();
    let search_term = format!("{}*", q);
    
    let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT item_id, type, content, subtext, link FROM universal_search 
         WHERE user_id = ? AND universal_search MATCH ? 
         ORDER BY rank LIMIT ?"
    )
    .bind(auth.user_id).bind(search_term).bind(params.limit.unwrap_or(20)).fetch_all(db).await?;

    let results: Vec<serde_json::Value> = rows.into_iter().map(|(id, typ, content, subtext, link)| {
        // Wir versuchen den display_title aus dem content zu extrahieren (der am Anfang steht)
        // Einfacher: Wir geben Title und Content getrennt zurück
        serde_json::json!({
            "id": id,
            "type": typ,
            "title": if typ == "event" { subtext.clone() } else { id.clone() },
            "label": content.split('{').next().unwrap_or(&content).trim(), // Extrahiert den Text vor dem JSON
            "content": content,
            "link": link
        })
    }).collect();

    Ok(Json(serde_json::Value::Array(results)))
}

#[utoipa::path(get, path = "/api/v1/data/{path}", responses((status = 200, body = [Event])), security(("api_key" = [])))]
pub async fn get_data_by_type(State(state): State<Arc<AppState>>, Path(path): Path<String>, Query(params): Query<ListParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<Event>>> {
    let semantic_path = path.replace('/', ".");
    let events = state.event_service.search_semantic(auth.user_id, &semantic_path, params.limit.unwrap_or(100), params.offset.unwrap_or(0)).await?;
    Ok(Json(events))
}

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

#[utoipa::path(get, path = "/api/v1/streams/timeline", params(ListParams), responses((status = 200, body = [serde_json::Value])), security(("api_key" = [])))]
pub async fn get_timeline(State(state): State<Arc<AppState>>, Query(params): Query<ListParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<serde_json::Value>>> {
    let timeline = state.event_service.get_enriched_timeline(auth.user_id, params.category, params.limit.unwrap_or(20), params.offset.unwrap_or(0)).await?;
    Ok(Json(timeline))
}

#[utoipa::path(get, path = "/api/v1/streams/summary", params(SummaryParams), responses((status = 200, body = [String])), security(("api_key" = [])))]
pub async fn get_daily_summary(State(state): State<Arc<AppState>>, Query(params): Query<SummaryParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<String>>> {
    let date = params.date.as_deref().unwrap_or("2026-02-28");
    let summary = state.event_service.generate_daily_summary(auth.user_id, date).await?;
    Ok(Json(summary))
}

#[utoipa::path(get, path = "/api/v1/analytics/stats", params(CorrelateParams), responses((status = 200, body = SemanticStats)), security(("api_key" = [])))]
pub async fn get_semantic_stats(State(state): State<Arc<AppState>>, Query(params): Query<CorrelateParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<SemanticStats>> {
    let bs = params.base_semantic.as_ref().ok_or_else(|| Error::BadRequest("base_semantic required".to_string()))?;
    let js = params.join_semantic.as_ref().ok_or_else(|| Error::BadRequest("join_semantic required".to_string()))?;
    let stats = state.event_service.calculate_semantic_stats(auth.user_id, bs, js, params.limit.unwrap_or(100)).await?;
    Ok(Json(stats))
}

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
    let results = if let (Some(bs), Some(js)) = (&params.base_semantic, &params.join_semantic) {
        state.event_service.correlate_semantic(auth.user_id, bs, js, limit).await
    } else if let (Some(bc), Some(jc)) = (&params.base_category, &params.join_category) {
        state.event_service.correlate_nearest(auth.user_id, bc, jc, limit).await
    } else { return Err(Error::BadRequest("Invalid params".to_string())); }?;
    let api_results = results.into_iter().map(|v| CorrelationResult { base: v.get("base").cloned().unwrap_or(json!({})), joined: v.get("joined").cloned().unwrap_or(json!({})), }).collect();
    Ok(Json(api_results))
}

#[utoipa::path(get, path = "/api/v1/system/status", responses((status = 200, description = "Status")))]
pub async fn get_system_status() -> impl IntoResponse {
    Json(json!({ "status": "online", "multi_tenant": true }))
}

#[utoipa::path(get, path = "/api/v1/data/id/{id}", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_event_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<serde_json::Value>> {
    let db = state.event_service.db();
    let row = sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle FROM events WHERE user_id = ? AND id = ?")
        .bind(auth.user_id).bind(id).fetch_one(db).await?;
    
    let ev = Event::try_from(row).map_err(|e| Error::Plugin(e))?;
    Ok(Json(serde_json::to_value(ev).unwrap()))
}

#[utoipa::path(get, path = "/api/v1/data/entity/{namespace}/{typ}/{id}", responses((status = 200, body = Vec<serde_json::Value>)), security(("api_key" = [])))]
pub async fn get_events_by_entity(
    State(state): State<Arc<AppState>>,
    Path((namespace, typ, id)): Path<(String, String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<serde_json::Value>> {
    let db = state.event_service.db();
    let rows = sqlx::query_as::<_, DbEvent>(
        "SELECT id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle FROM events 
         WHERE user_id = ? AND EXISTS (
            SELECT 1 FROM json_each(entities) WHERE json_extract(value, '$.namespace') = ? AND json_extract(value, '$.typ') = ? AND json_extract(value, '$.id') = ?
         ) ORDER BY timestamp DESC LIMIT 100"
    )
    .bind(auth.user_id).bind(namespace).bind(typ).bind(id).fetch_all(db).await?;
    
    let events: Vec<serde_json::Value> = rows.into_iter()
        .filter_map(|r| Event::try_from(r).ok())
        .map(|e| serde_json::to_value(e).unwrap())
        .collect();

    Ok(Json(serde_json::Value::Array(events)))
}

#[utoipa::path(get, path = "/health", responses((status = 200, description = "Health Check")))]
pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.event_service.db().acquire().await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "healthy", "db": "connected" }))),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "status": "unhealthy", "db": e.to_string() }))),
    }
}

#[utoipa::path(get, path = "/api/v1/system/plugins", responses((status = 200, body = [PluginStatus])), security(("api_key" = [])))]
pub async fn get_system_plugins(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<PluginStatus>>> {
    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
    let mut statuses = Vec::new();

    // 1. Add Virtual Core Plugin for system widgets
    statuses.push(PluginStatus {
        id: "core".to_string(),
        name: "Scry System".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Core system services and metrics.".to_string(),
        roles: vec!["system".to_string()],
        capabilities: vec!["system".to_string()],
        subscriptions: vec![],
        exports: vec![],
        provided_traits: vec![],
        reports: vec![],
        config_schema: None,
        suggested_widgets: vec![
            ApiWidgetDefinition {
                id: "system-status".to_string(),
                title: "System Node Status".to_string(),
                template: ApiWidgetTemplate::Status,
                config_json: json!({ "scope": "all" }).to_string(),
            },
            ApiWidgetDefinition {
                id: "knowledge-growth".to_string(),
                title: "Knowledge Growth".to_string(),
                template: ApiWidgetTemplate::Trend,
                config_json: json!({ "semantic_type": "system.entities", "days": 7 }).to_string(),
            }
        ],
    });

    // 2. Add real plugins
    for (id, m) in manifests {
        let reports_list: Vec<(String, Vec<crate::plugins::scry::plugin::types::ReportMetadata>)> = state.event_service.plugin_manager().list_plugin_reports(auth.user_id).await?;
        let p_reports = reports_list.into_iter().find(|(p_id, _)| p_id == &id).map(|(_, r)| r).unwrap_or_default();
        
        // Auto-classify roles based on manifest structure
        let mut roles = Vec::new();
        if m.poll_interval.is_some() {
            roles.push("SOURCE".to_string());
        }
        if !m.subscriptions.is_empty() {
            roles.push("ENRICHER".to_string());
        }
        if !m.provided_traits.is_empty() {
            roles.push("RESOLVER".to_string());
        }
        if !m.suggested_widgets.is_empty() {
            roles.push("VISUALIZER".to_string());
        }
        if !p_reports.is_empty() {
            roles.push("ANALYZER".to_string());
        }

        statuses.push(PluginStatus {
            id: id.clone(),
            name: m.name,
            version: m.version,
            description: m.description,
            roles,
            capabilities: m.capabilities,
            subscriptions: m.subscriptions,
            exports: m.exports.into_iter().map(|e| ApiDataField {
                category: e.category,
                path: e.path,
                semantic_type: e.semantic_type,
                description: e.description,
                icon: e.icon,
            }).collect(),
            provided_traits: m.provided_traits.into_iter().map(|t| ApiTraitCapability {
                entity_namespace: t.entity_namespace,
                entity_type: t.entity_type,
                trait_id: t.trait_id,
            }).collect(),
            reports: p_reports.into_iter().map(|r| ApiReportMetadata {
                id: r.id, name: r.name, description: r.description, viz: format!("{:?}", r.viz),
            }).collect(),
            config_schema: m.config_schema,
            suggested_widgets: m.suggested_widgets.into_iter().map(|w| ApiWidgetDefinition {
                id: w.id, title: w.title, config_json: w.config_json,
                template: match w.template {
                    crate::plugins::scry::plugin::types::WidgetTemplate::Metric => ApiWidgetTemplate::Metric,
                    crate::plugins::scry::plugin::types::WidgetTemplate::Trend => ApiWidgetTemplate::Trend,
                    crate::plugins::scry::plugin::types::WidgetTemplate::TopList => ApiWidgetTemplate::TopList,
                    crate::plugins::scry::plugin::types::WidgetTemplate::Status => ApiWidgetTemplate::Status,
                    crate::plugins::scry::plugin::types::WidgetTemplate::Spotlight => ApiWidgetTemplate::Spotlight,
                }
            }).collect(),
        });
    }
    Ok(Json(statuses))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/poll", responses((status = 200, description = "Poll")), security(("api_key" = [])))]
pub async fn poll_plugin_manually(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let count = state.event_service.poll_and_save_plugin(auth.user_id, &id).await?;
    Ok(Json(json!({ "plugin": id, "events_saved": count })))
}

#[utoipa::path(post, path = "/api/v1/ingest", request_body = Event, responses((status = 200, body = Event)), security(("api_key" = [])))]
pub async fn ingest_event(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(event): Json<Event>) -> Result<Json<Event>> {
    let event = state.event_service.ingest_event(auth.user_id, event).await?;
    Ok(Json(event))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities", responses((status = 200, body = [ApiNamespace])), security(("api_key" = [])))]
pub async fn get_namespaces(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<Vec<crate::models::ApiNamespace>>> {
    let db = state.event_service.db();
    let names = sqlx::query_scalar::<_, String>("SELECT DISTINCT namespace FROM entities WHERE user_id = ?")
        .bind(auth.user_id).fetch_all(db).await?;
    
    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
    
    let namespaces = names.into_iter().map(|name| {
        let mut icon = None;

        // Deterministic Ownership: Check domain_info directly
        for m in manifests.values() {
            if let Some(domain) = m.domain_info.iter().find(|d| d.ns == name) {
                if domain.icon.is_some() {
                    icon = domain.icon.clone();
                    break;
                }
            }
        }

        // System Core Fallback (since core is not a plugin yet)
        if icon.is_none() && name == "scry.core" {
            icon = Some("lucide:shield-check".to_string());
        }

        crate::models::ApiNamespace { name, icon }
    }).collect();

    Ok(Json(namespaces))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities/{namespace}", responses((status = 200, body = [String])), security(("api_key" = [])))]
pub async fn get_namespace_types(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<Vec<String>>> {
    let db = state.event_service.db();
    let types = sqlx::query_scalar::<_, String>("SELECT DISTINCT typ FROM entities WHERE user_id = ? AND namespace = ?")
        .bind(auth.user_id).bind(&namespace).fetch_all(db).await?;
    Ok(Json(types))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities/{namespace}/{typ}", responses((status = 200, body = [ApiEntity])), security(("api_key" = [])))]
pub async fn get_entities(
    State(state): State<Arc<AppState>>,
    Path((namespace, typ)): Path<(String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<Vec<crate::models::ApiEntity>>> {
    let db = state.event_service.db();
    
    // Wir holen alle Entitäten des Typs und versuchen Titel und Bild aus den Traits zu finden
    let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>)>("
        SELECT e.id, (
            SELECT value_json FROM entity_traits t 
            WHERE t.user_id = e.user_id AND t.namespace = e.namespace AND t.entity_type = e.typ AND t.entity_id = e.id 
            AND (t.trait_id LIKE '%name' OR t.trait_id LIKE '%title')
            LIMIT 1
        ) as title,
        (
            SELECT value_json FROM entity_traits t 
            WHERE t.user_id = e.user_id AND t.namespace = e.namespace AND t.entity_type = e.typ AND t.entity_id = e.id 
            AND (t.trait_id LIKE '%photo' OR t.trait_id LIKE '%avatar' OR t.trait_id LIKE '%image')
            LIMIT 1
        ) as photo
        FROM entities e
        WHERE e.user_id = ? AND e.namespace = ? AND e.typ = ?
    ")
    .bind(auth.user_id).bind(&namespace).bind(&typ).fetch_all(db).await?;

    let entities = rows.into_iter().map(|(id, title_json, photo_json)| {
        let title = title_json.and_then(|json| {
            serde_json::from_str::<serde_json::Value>(&json).ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        }).unwrap_or_else(|| id.clone());

        let photo_url = photo_json.and_then(|json| {
            serde_json::from_str::<serde_json::Value>(&json).ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        });

        crate::models::ApiEntity {
            namespace: namespace.clone(),
            typ: typ.clone(),
            id: id.clone(),
            title,
            photo_url,
            link: format!("/entity/{}/{}/{}", namespace, typ, id),
        }
    }).collect();

    Ok(Json(entities))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities/{namespace}/{typ}/{id}/traits", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_entity_traits(
    State(state): State<Arc<AppState>>,
    Path((namespace, typ, id)): Path<(String, String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<serde_json::Value>> {
    let db = state.event_service.db();
    let rows = sqlx::query_as::<_, (String, String, String)>("SELECT plugin_id, trait_id, value_json FROM entity_traits WHERE user_id = ? AND namespace = ? AND entity_type = ? AND entity_id = ?")
        .bind(auth.user_id).bind(&namespace).bind(&typ).bind(&id).fetch_all(db).await?;
    
    let mut map = serde_json::Map::new();
    for (_plugin_id, trait_id, value_json) in rows {
        let val: serde_json::Value = serde_json::from_str(&value_json).unwrap_or(serde_json::Value::Null);
        map.insert(trait_id, val);
    }

    // Beziehungen laden
    let rel_rows = sqlx::query_as::<_, (String, String, String, String, String, String, String)>("SELECT source_ns, source_type, source_id, predicate, target_ns, target_type, target_id FROM entity_relationships WHERE user_id = ? AND (source_ns = ? AND source_type = ? AND source_id = ? OR target_ns = ? AND target_type = ? AND target_id = ?)")
        .bind(auth.user_id)
        .bind(&namespace).bind(&typ).bind(&id)
        .bind(&namespace).bind(&typ).bind(&id)
        .fetch_all(db).await?;

    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;

    let relationships: Vec<serde_json::Value> = rel_rows.into_iter().map(|(sn, st, si, p, tn, tt, ti)| {
        let direction = if sn == namespace && st == typ && si == id { "outgoing" } else { "incoming" };
        
        // Find a human-friendly label from manifest predicates
        let mut display_label = p.split('/').last().unwrap_or(&p).replace('_', " ");
        for m in manifests.values() {
            if let Some(pred) = m.predicates.iter().find(|pr| pr.id == p || format!("{}/{}", sn, pr.id) == p || format!("{}/{}", tn, pr.id) == p) {
                display_label = if direction == "outgoing" { pred.label.clone() } else { pred.inverse_label.clone() };
                break;
            }
        }

        serde_json::json!({
            "source": { "ns": sn, "typ": st, "id": si },
            "predicate": p,
            "display_label": display_label,
            "target": { "ns": tn, "typ": tt, "id": ti },
            "direction": direction
        })
    }).collect();

    let mut result = serde_json::Map::new();
    result.insert("traits".to_string(), serde_json::Value::Object(map));
    result.insert("relationships".to_string(), serde_json::Value::Array(relationships));

    Ok(Json(serde_json::Value::Object(result)))
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ListParams { pub category: Option<String>, pub limit: Option<u32>, pub offset: Option<u32> }

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SearchParams { pub q: String, pub limit: Option<u32>, pub offset: Option<u32> }

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SummaryParams { pub date: Option<String> }

#[derive(Deserialize, utoipa::IntoParams)]
pub struct CorrelateParams { pub base_category: Option<String>, pub join_category: Option<String>, pub base_semantic: Option<String>, pub join_semantic: Option<String>, pub limit: Option<u32> }

#[derive(Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct SemanticParams { 
    pub semantic_type: String, 
    pub limit: Option<u32>, 
    pub days: Option<u32>,
    pub interval: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use crate::event_service::EventService;
    use crate::plugins::PluginManager;

    async fn setup_test_state() -> Arc<AppState> {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
        let pm = Arc::new(PluginManager::new("./non_existent", pool.clone()).unwrap());
        let svc = EventService::new(pool, pm);
        Arc::new(AppState { event_service: svc })
    }

    #[tokio::test]
    async fn test_auth_flow() {
        let state = setup_test_state().await;
        
        // 1. Zu kurzes Passwort (Validierung)
        let reg_fail = RegisterRequest { username: "alice".to_string(), password: "123".to_string() };
        let res = register_user(State(state.clone()), Json(reg_fail)).await;
        assert!(res.is_err()); // Sollte wegen Passwort-Länge (< 8) fehlschlagen

        // 2. Korrekte Registrierung
        let reg_ok = RegisterRequest { username: "alice".to_string(), password: "password123".to_string() };
        let res = register_user(State(state.clone()), Json(reg_ok)).await.unwrap();
        assert_eq!(res.user.username, "alice");
        let key = res.api_key.clone();

        // 3. Login
        let login = LoginRequest { username: "alice".to_string(), password: "password123".to_string() };
        let res_login = login_user(State(state.clone()), Json(login)).await.unwrap();
        assert_eq!(res_login.api_key, key);
        
        // 4. Falsches Passwort
        let login_wrong = LoginRequest { username: "alice".to_string(), password: "wrong_password".to_string() };
        let res_wrong = login_user(State(state.clone()), Json(login_wrong)).await;
        assert!(res_wrong.is_err());
    }
}
