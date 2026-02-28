use axum::{
    extract::{State, Json, Query, Path},
    http::{StatusCode, Request},
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

// --- Handlers ---

use axum::response::sse::{Event as SseEvent, Sse};
use futures::stream::{Stream, StreamExt};
use std::convert::Infallible;

pub async fn stream_live_events(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
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
                                yield Ok(SseEvent::default().data(serde_json::to_string(&event).unwrap()));
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
pub async fn register_user(State(state): State<Arc<AppState>>, Json(req): Json<RegisterRequest>) -> Result<Json<AuthResponse>, AppError> {
    req.validate()?;
    let db = state.event_service.db();
    
    // Hash password with Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| AppError::Internal)?
        .to_string();

    let res = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&req.username).bind(password_hash).execute(db).await?;
    
    let user_id = res.last_insert_rowid();
    let api_key = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO api_keys (key, user_id, label, scopes) VALUES (?, ?, ?, ?)")
        .bind(&api_key).bind(user_id).bind("Default Key").bind("all").execute(db).await?;

    Ok(Json(AuthResponse { api_key, user: User { id: user_id, username: req.username } }))
}

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest, responses((status = 200, body = AuthResponse)))]
pub async fn login_user(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> Result<Json<AuthResponse>, AppError> {
    req.validate()?;
    let db = state.event_service.db();
    let user = sqlx::query_as::<_, (i64, String, String)>("SELECT id, username, password_hash FROM users WHERE username = ?")
        .bind(&req.username).fetch_optional(db).await?
        .ok_or_else(|| AppError::Auth("User not found".to_string()))?;
    
    // Verify password with Argon2
    let parsed_hash = PasswordHash::new(&user.2)
        .map_err(|_| AppError::Internal)?;
    
    if Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash).is_err() {
        return Err(AppError::Auth("Invalid password".to_string()));
    }

    let api_key = sqlx::query_scalar::<_, String>("SELECT key FROM api_keys WHERE user_id = ? LIMIT 1").bind(user.0).fetch_one(db).await?;
    Ok(Json(AuthResponse { api_key, user: User { id: user.0, username: user.1 } }))
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/reports/{report_id}", responses((status = 200, body = ApiReportData)), security(("api_key" = [])))]
pub async fn run_plugin_report(
    State(state): State<Arc<AppState>>,
    Path((id, report_id)): Path<(String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<ApiReportData>, AppError> {
    let data = state.event_service.plugin_manager().run_plugin_report(auth.user_id, &id, report_id).await?;
    Ok(Json(ApiReportData {
        columns: data.columns,
        data_json: data.data_json,
    }))
}

#[utoipa::path(get, path = "/api/v1/system/profile", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_profile(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>, AppError> {
    let db = state.event_service.db();
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM user_profile WHERE user_id = ?").bind(auth.user_id).fetch_all(db).await?;
    let mut map = serde_json::Map::new();
    for (k, v) in rows { map.insert(k, json!(v)); }
    Ok(Json(serde_json::Value::Object(map)))
}

#[utoipa::path(post, path = "/api/v1/system/profile", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_profile(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Map<String, serde_json::Value>>) -> Result<impl IntoResponse, AppError> {
    let db = state.event_service.db();
    for (k, v) in req {
        let v_str = v.as_str().unwrap_or("").to_string();
        sqlx::query("INSERT INTO user_profile (user_id, key, value) VALUES (?, ?, ?) ON CONFLICT(user_id, key) DO UPDATE SET value = EXCLUDED.value")
            .bind(auth.user_id).bind(k).bind(v_str).execute(db).await?;
    }
    Ok(StatusCode::OK)
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/config", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_plugin_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Map<String, serde_json::Value>>) -> Result<impl IntoResponse, AppError> {
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
            catalog.entry(export.semantic_type).or_insert_with(|| json!([])).as_array_mut().unwrap().push(entry);
        }
    }
    Json(catalog)
}

#[utoipa::path(get, path = "/api/v1/discovery/search", params(SearchParams), responses((status = 200, body = [Event])), security(("api_key" = [])))]
pub async fn search_events(State(state): State<Arc<AppState>>, Query(params): Query<SearchParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<Event>>, AppError> {
    let events = state.event_service.search_semantic(auth.user_id, &params.q, params.limit.unwrap_or(50), params.offset.unwrap_or(0)).await?;
    Ok(Json(events))
}

#[utoipa::path(get, path = "/api/v1/data/{path}", responses((status = 200, body = [Event])), security(("api_key" = [])))]
pub async fn get_data_by_type(State(state): State<Arc<AppState>>, Path(path): Path<String>, Query(params): Query<ListParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<Event>>, AppError> {
    let semantic_path = path.replace('/', ".");
    let events = state.event_service.search_semantic(auth.user_id, &semantic_path, params.limit.unwrap_or(100), params.offset.unwrap_or(0)).await?;
    Ok(Json(events))
}

#[utoipa::path(get, path = "/api/v1/streams/timeline", params(ListParams), responses((status = 200, body = [serde_json::Value])), security(("api_key" = [])))]
pub async fn get_timeline(State(state): State<Arc<AppState>>, Query(params): Query<ListParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let cat = params.category.as_deref().unwrap_or("music.scrobble");
    let timeline = state.event_service.get_enriched_timeline(auth.user_id, cat, params.limit.unwrap_or(20), params.offset.unwrap_or(0)).await?;
    Ok(Json(timeline))
}

#[utoipa::path(get, path = "/api/v1/streams/summary", params(SummaryParams), responses((status = 200, body = [String])), security(("api_key" = [])))]
pub async fn get_daily_summary(State(state): State<Arc<AppState>>, Query(params): Query<SummaryParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<String>>, AppError> {
    let date = params.date.as_deref().unwrap_or("2026-02-28");
    let summary = state.event_service.generate_daily_summary(auth.user_id, date).await?;
    Ok(Json(summary))
}

#[utoipa::path(get, path = "/api/v1/analytics/stats", params(CorrelateParams), responses((status = 200, body = SemanticStats)), security(("api_key" = [])))]
pub async fn get_semantic_stats(State(state): State<Arc<AppState>>, Query(params): Query<CorrelateParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<SemanticStats>, AppError> {
    let bs = params.base_semantic.as_ref().ok_or_else(|| AppError::BadRequest("base_semantic required".to_string()))?;
    let js = params.join_semantic.as_ref().ok_or_else(|| AppError::BadRequest("join_semantic required".to_string()))?;
    let stats = state.event_service.calculate_semantic_stats(auth.user_id, bs, js, params.limit.unwrap_or(100)).await?;
    Ok(Json(stats))
}

#[utoipa::path(post, path = "/api/v1/system/dashboards", responses((status = 200)), security(("api_key" = [])))]
pub async fn create_dashboard(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Value>) -> Result<impl IntoResponse, AppError> {
    let db = state.event_service.db();
    let id = Uuid::new_v4().to_string();
    let name = req["name"].as_str().ok_or_else(|| AppError::BadRequest("Missing name".to_string()))?;
    
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
pub async fn delete_widget(State(state): State<Arc<AppState>>, Path((_id, widget_id)): Path<(String, String)>, Extension(_auth): Extension<AuthContext>) -> Result<impl IntoResponse, AppError> {
    let db = state.event_service.db();
    sqlx::query("DELETE FROM dashboard_widgets WHERE id = ?").bind(widget_id).execute(db).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/system/dashboards", responses((status = 200, body = [Dashboard])), security(("api_key" = [])))]
pub async fn get_dashboards(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<Dashboard>>, AppError> {
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
pub async fn add_widget(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(_auth): Extension<AuthContext>, Json(req): Json<serde_json::Value>) -> Result<impl IntoResponse, AppError> {
    let db = state.event_service.db();
    let widget_id = Uuid::new_v4().to_string();
    let w_type = req["type"].as_str().ok_or_else(|| AppError::BadRequest("Missing type".to_string()))?;
    let config = serde_json::to_string(&req["config"]).unwrap_or_else(|_| "{}".to_string());
    let span = req["width_span"].as_i64().unwrap_or(1) as i32;
    
    tracing::debug!("Adding widget {} to dashboard {}", w_type, id);

    sqlx::query("INSERT INTO dashboard_widgets (id, dashboard_id, type, title, config, width_span) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(widget_id).bind(id).bind(w_type).bind(req["title"].as_str()).bind(config).bind(span).execute(db).await.map_err(|e| {
            tracing::error!("Failed to insert widget: {}", e);
            AppError::Database(e)
        })?;
    
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/analytics/semantic/top", params(SemanticParams), responses((status = 200, body = [serde_json::Value])), security(("api_key" = [])))]
pub async fn get_semantic_top(State(state): State<Arc<AppState>>, Query(params): Query<SemanticParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let top = state.event_service.get_semantic_top(auth.user_id, &params.semantic_type, params.limit.unwrap_or(10), params.days).await?;
    Ok(Json(top))
}

#[utoipa::path(get, path = "/api/v1/analytics/semantic/series", params(SemanticParams), responses((status = 200, body = [serde_json::Value])), security(("api_key" = [])))]
pub async fn get_semantic_series(State(state): State<Arc<AppState>>, Query(params): Query<SemanticParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let series = state.event_service.get_semantic_series(auth.user_id, &params.semantic_type, params.days.unwrap_or(7)).await?;
    Ok(Json(series))
}

#[utoipa::path(get, path = "/api/v1/analytics/correlations", params(CorrelateParams), responses((status = 200, body = [CorrelationResult])), security(("api_key" = [])))]
pub async fn correlate_events(State(state): State<Arc<AppState>>, Query(params): Query<CorrelateParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<CorrelationResult>>, AppError> {
    let limit = params.limit.unwrap_or(50);
    let results = if let (Some(bs), Some(js)) = (&params.base_semantic, &params.join_semantic) {
        state.event_service.correlate_semantic(auth.user_id, bs, js, limit).await
    } else if let (Some(bc), Some(jc)) = (&params.base_category, &params.join_category) {
        state.event_service.correlate_nearest(auth.user_id, bc, jc, limit).await
    } else { return Err(AppError::BadRequest("Invalid params".to_string())); }?;
    let api_results = results.into_iter().map(|v| CorrelationResult { base: v.get("base").cloned().unwrap_or(json!({})), joined: v.get("joined").cloned().unwrap_or(json!({})), }).collect();
    Ok(Json(api_results))
}

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

#[utoipa::path(get, path = "/api/v1/system/plugins", responses((status = 200, body = [PluginStatus])), security(("api_key" = [])))]
pub async fn get_system_plugins(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<PluginStatus>>, AppError> {
    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
    let mut statuses = Vec::new();

    for (id, m) in manifests {
        let reports = state.event_service.plugin_manager().list_plugin_reports(auth.user_id).await?;
        let p_reports = reports.into_iter().find(|(p_id, _)| p_id == &id).map(|(_, r)| r).unwrap_or_default();
        
        statuses.push(PluginStatus {
            id: id.clone(),
            name: m.name,
            version: m.version,
            description: m.description,
            capabilities: m.capabilities,
            subscriptions: m.subscriptions,
            reports: p_reports.into_iter().map(|r| ApiReportMetadata {
                id: r.id, name: r.name, description: r.description, viz: format!("{:?}", r.viz),
            }).collect()
        });
    }
    Ok(Json(statuses))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/poll", responses((status = 200, description = "Poll")), security(("api_key" = [])))]
pub async fn poll_plugin_manually(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>, AppError> {
    let count = state.event_service.poll_and_save_plugin(auth.user_id, &id).await?;
    Ok(Json(json!({ "plugin": id, "events_saved": count })))
}

#[utoipa::path(post, path = "/api/v1/ingest", request_body = Event, responses((status = 200, body = Event)), security(("api_key" = [])))]
pub async fn ingest_event(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(event): Json<Event>) -> Result<Json<Event>, AppError> {
    let event = state.event_service.ingest_event(auth.user_id, event).await?;
    Ok(Json(event))
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
pub struct SemanticParams { pub semantic_type: String, pub limit: Option<u32>, pub days: Option<u32> }

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
