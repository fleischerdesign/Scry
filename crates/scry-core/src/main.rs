mod plugins;
mod event_service;
mod models;

use axum::{
    extract::{State, Json, Query, Path},
    http::StatusCode,
    middleware::{self, Next},
    response::{Response, IntoResponse},
    routing::{get, post},
    Router,
    Extension,
    http::Request,
};
use scry_proto::Event;
use serde::Deserialize;
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::plugins::PluginManager;
use crate::event_service::EventService;
use crate::models::*;
use tokio::time::{sleep, Duration};
use notify::{Watcher, RecursiveMode};
use serde_json::json;
use uuid::Uuid;

struct AppState {
    event_service: EventService,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        register_user, login_user,
        get_catalog, search_events, 
        get_data_by_type, 
        get_timeline, 
        get_daily_summary,
        correlate_events,
        get_system_status, get_system_plugins, poll_plugin_manually,
        ingest_event
    ),
    components(schemas(Event, User, RegisterRequest, LoginRequest, AuthResponse, ApiReportMetadata, ApiReportData, PluginReports, CorrelationResult, SemanticStats)),
    modifiers(&SecurityAddon),
    tags((name = "scry", description = "Scry Multi-Tenant Platform API"))
)]
struct ApiDoc;

struct SecurityAddon;
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-API-Key"),
                    ),
                ),
            );
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "scry_core=debug,info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:scry.db".to_string());

    let pool_options = SqliteConnectOptions::new()
        .filename(database_url.trim_start_matches("sqlite:"))
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    let db = SqlitePool::connect_with(pool_options).await?;
    sqlx::migrate!("../../migrations").run(&db).await?;

    let plugin_manager = Arc::new(PluginManager::new("./plugins", db.clone())?);
    plugin_manager.reload_plugins().await?;

    let pm_for_watcher = plugin_manager.clone();
    let rt_handle = tokio::runtime::Handle::current();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(event) => {
                if event.kind.is_modify() || event.kind.is_create() {
                    let pm = pm_for_watcher.clone();
                    rt_handle.spawn(async move {
                        tracing::info!("Plugin directory changed, reloading...");
                        let _ = pm.reload_plugins().await;
                    });
                }
            },
            Err(e) => tracing::error!("Watcher error: {}", e),
        }
    })?;
    watcher.watch(std::path::Path::new("./plugins"), RecursiveMode::NonRecursive)?;

    let event_service = EventService::new(db, plugin_manager);
    let shared_state = Arc::new(AppState { event_service });

    // Background Scheduler Task (Multi-Tenant aware)
    let scheduler_state = shared_state.clone();
    tokio::spawn(async move {
        loop {
            let manifests = scheduler_state.event_service.plugin_manager().get_plugin_manifests().await;
            for (name, _) in manifests {
                let svc = scheduler_state.event_service.clone();
                let plugin_name = name.clone();
                tokio::spawn(async move {
                    let _ = svc.poll_and_save_plugin(1, &plugin_name).await;
                });
            }
            sleep(Duration::from_secs(60)).await;
        }
    });

    let auth_routes = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user));

    let api_v1 = Router::new()
        .nest("/auth", auth_routes)
        .merge(Router::new()
            .nest("/discovery", Router::new()
                .route("/catalog", get(get_catalog))
                .route("/search", get(search_events)))
            .nest("/data", Router::new()
                .route("/*path", get(get_data_by_type)))
            .nest("/streams", Router::new()
                .route("/timeline", get(get_timeline))
                .route("/summary", get(get_daily_summary)))
            .nest("/analytics", Router::new()
                .route("/correlations", get(correlate_events))
                .route("/stats", get(get_semantic_stats)))
            .nest("/system", Router::new()
                .route("/status", get(get_system_status))
                .route("/plugins", get(get_system_plugins))
                .route("/plugins/:id/poll", post(poll_plugin_manually)))
            .route("/ingest", post(ingest_event))
            .layer(middleware::from_fn_with_state(shared_state.clone(), auth_middleware)));

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_v1)
        .route("/", get(|| async { "Scry Platform API" }))
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Scry Multi-Tenant Platform on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn auth_middleware(State(state): State<Arc<AppState>>, mut req: Request<axum::body::Body>, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req.headers().get("X-API-Key").and_then(|h| h.to_str().ok());
    if let Some(key) = auth_header {
        let db = state.event_service.db();
        let auth = sqlx::query_as::<_, (i64, String)>("SELECT user_id, scopes FROM api_keys WHERE key = ?")
            .bind(key).fetch_optional(db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        if let Some((user_id, scopes_str)) = auth {
            let ctx = AuthContext {
                user_id,
                scopes: scopes_str.split(',').map(|s| s.to_string()).collect(),
            };
            req.extensions_mut().insert(ctx);
            return Ok(next.run(req).await);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

// --- Handlers ---

#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterRequest, responses((status = 200, body = AuthResponse)))]
async fn register_user(State(state): State<Arc<AppState>>, Json(req): Json<RegisterRequest>) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let db = state.event_service.db();
    let hash = format!("hash_{}", req.password); 
    let res = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&req.username).bind(hash).execute(db).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user_id = res.last_insert_rowid();
    let api_key = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO api_keys (key, user_id, label, scopes) VALUES (?, ?, ?, ?)")
        .bind(&api_key).bind(user_id).bind("Default Key").bind("all").execute(db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(AuthResponse { api_key, user: User { id: user_id, username: req.username } }))
}

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest, responses((status = 200, body = AuthResponse)))]
async fn login_user(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let db = state.event_service.db();
    let user = sqlx::query_as::<_, (i64, String, String)>("SELECT id, username, password_hash FROM users WHERE username = ?")
        .bind(&req.username).fetch_optional(db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;
    if user.2 != format!("hash_{}", req.password) { return Err((StatusCode::UNAUTHORIZED, "Invalid password".to_string())); }
    let api_key = sqlx::query_scalar::<_, String>("SELECT key FROM api_keys WHERE user_id = ? LIMIT 1").bind(user.0).fetch_one(db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(AuthResponse { api_key, user: User { id: user.0, username: user.1 } }))
}

#[utoipa::path(get, path = "/api/v1/discovery/catalog", responses((status = 200, description = "Catalog")), security(("api_key" = [])))]
async fn get_catalog(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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
async fn search_events(State(state): State<Arc<AppState>>, Query(params): Query<SearchParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<Event>>, (StatusCode, String)> {
    let events = state.event_service.search_semantic(auth.user_id, &params.q, params.limit.unwrap_or(50)).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(events))
}

#[utoipa::path(get, path = "/api/v1/data/{path}", responses((status = 200, body = [Event])), security(("api_key" = [])))]
async fn get_data_by_type(State(state): State<Arc<AppState>>, Path(path): Path<String>, Query(params): Query<ListParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<Event>>, (StatusCode, String)> {
    let semantic_path = path.replace('/', ".");
    let events = state.event_service.search_semantic(auth.user_id, &semantic_path, params.limit.unwrap_or(100)).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(events))
}

#[utoipa::path(get, path = "/api/v1/streams/timeline", params(ListParams), responses((status = 200, body = [serde_json::Value])), security(("api_key" = [])))]
async fn get_timeline(State(state): State<Arc<AppState>>, Query(params): Query<ListParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let cat = params.category.as_deref().unwrap_or("music.scrobble");
    let timeline = state.event_service.get_enriched_timeline(auth.user_id, cat, params.limit.unwrap_or(20)).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(timeline))
}

#[utoipa::path(get, path = "/api/v1/streams/summary", params(SummaryParams), responses((status = 200, body = [String])), security(("api_key" = [])))]
async fn get_daily_summary(State(state): State<Arc<AppState>>, Query(params): Query<SummaryParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let date = params.date.as_deref().unwrap_or_else(|| "2026-02-28");
    let summary = state.event_service.generate_daily_summary(auth.user_id, date).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(summary))
}

#[utoipa::path(get, path = "/api/v1/analytics/stats", params(CorrelateParams), responses((status = 200, body = SemanticStats)), security(("api_key" = [])))]
async fn get_semantic_stats(State(state): State<Arc<AppState>>, Query(params): Query<CorrelateParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<SemanticStats>, (StatusCode, String)> {
    let bs = params.base_semantic.as_ref().ok_or((StatusCode::BAD_REQUEST, "base_semantic required".to_string()))?;
    let js = params.join_semantic.as_ref().ok_or((StatusCode::BAD_REQUEST, "join_semantic required".to_string()))?;
    let stats = state.event_service.calculate_semantic_stats(auth.user_id, bs, js, params.limit.unwrap_or(100)).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(stats))
}

#[utoipa::path(get, path = "/api/v1/analytics/correlations", params(CorrelateParams), responses((status = 200, body = [CorrelationResult])), security(("api_key" = [])))]
async fn correlate_events(State(state): State<Arc<AppState>>, Query(params): Query<CorrelateParams>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<CorrelationResult>>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(50);
    let results = if let (Some(bs), Some(js)) = (&params.base_semantic, &params.join_semantic) {
        state.event_service.correlate_semantic(auth.user_id, bs, js, limit).await
    } else if let (Some(bc), Some(jc)) = (&params.base_category, &params.join_category) {
        state.event_service.correlate_nearest(auth.user_id, bc, jc, limit).await
    } else { return Err((StatusCode::BAD_REQUEST, "Invalid params".to_string())); }.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let api_results = results.into_iter().map(|v| CorrelationResult { base: v.get("base").cloned().unwrap_or(json!({})), joined: v.get("joined").cloned().unwrap_or(json!({})), }).collect();
    Ok(Json(api_results))
}

#[utoipa::path(get, path = "/api/v1/system/status", responses((status = 200, description = "Status")))]
async fn get_system_status() -> impl IntoResponse {
    Json(json!({ "status": "online", "multi_tenant": true }))
}

#[utoipa::path(get, path = "/api/v1/system/plugins", responses((status = 200, body = [PluginReports])), security(("api_key" = [])))]
async fn get_system_plugins(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<PluginReports>>, (StatusCode, String)> {
    let reports = state.event_service.plugin_manager().list_plugin_reports(auth.user_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let api_reports = reports.into_iter().map(|(plugin, metadata_list)| {
        PluginReports { plugin, reports: metadata_list.into_iter().map(|m| ApiReportMetadata { id: m.id, name: m.name, description: m.description, viz: format!("{:?}", m.viz) }).collect() }
    }).collect();
    Ok(Json(api_reports))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/poll", responses((status = 200, description = "Poll")), security(("api_key" = [])))]
async fn poll_plugin_manually(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let count = state.event_service.poll_and_save_plugin(auth.user_id, &id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({ "plugin": id, "events_saved": count })))
}

#[utoipa::path(post, path = "/api/v1/ingest", request_body = Event, responses((status = 200, body = Event)), security(("api_key" = [])))]
async fn ingest_event(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>, Json(event): Json<Event>) -> Result<Json<Event>, (StatusCode, String)> {
    let event = state.event_service.ingest_event(auth.user_id, event).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(event))
}

#[derive(Deserialize, utoipa::IntoParams)]
struct ListParams { category: Option<String>, limit: Option<u32> }

#[derive(Deserialize, utoipa::IntoParams)]
struct SearchParams { q: String, limit: Option<u32> }

#[derive(Deserialize, utoipa::IntoParams)]
struct SummaryParams { date: Option<String> }

#[derive(Deserialize, utoipa::IntoParams)]
struct CorrelateParams { base_category: Option<String>, join_category: Option<String>, base_semantic: Option<String>, join_semantic: Option<String>, limit: Option<u32> }
