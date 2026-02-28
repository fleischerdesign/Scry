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

// --- Handlers ---

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

#[utoipa::path(get, path = "/api/v1/discovery/catalog", responses((status = 200, description = "Catalog")), security(("api_key" = [])))]
pub async fn get_catalog(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
    let mut catalog = serde_json::Map::new();
    for (plugin_name, manifest) in manifests {
        for export in manifest.exports {
            let entry = json!({ "plugin": plugin_name, "path": export.path, "description": export.description, "category": export.category });
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

#[utoipa::path(get, path = "/api/v1/system/plugins", responses((status = 200, body = [PluginReports])), security(("api_key" = [])))]
pub async fn get_system_plugins(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<PluginReports>>, AppError> {
    let reports = state.event_service.plugin_manager().list_plugin_reports(auth.user_id).await?;
    let api_reports = reports.into_iter().map(|(plugin, metadata_list)| {
        PluginReports { plugin, reports: metadata_list.into_iter().map(|m| ApiReportMetadata { id: m.id, name: m.name, description: m.description, viz: format!("{:?}", m.viz) }).collect() }
    }).collect();
    Ok(Json(api_reports))
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
