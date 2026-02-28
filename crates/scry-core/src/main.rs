mod plugins;
mod event_service;
mod models;
mod handlers;

use axum::{
    extract::State,
    http::StatusCode,
    middleware::{self, Next},
    response::{Response},
    routing::{get, post, delete},
    Router,
    http::Request,
};
use scry_proto::Event;
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::plugins::PluginManager;
use crate::event_service::EventService;
use crate::models::*;
use crate::handlers::*;
use tokio::time::{sleep, Duration};
use notify::{Watcher, RecursiveMode};
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;
use tokio_util::sync::CancellationToken;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::register_user, handlers::login_user,
        handlers::get_catalog, handlers::search_events, 
        handlers::get_data_by_type, 
        handlers::get_timeline, 
        handlers::get_daily_summary,
        handlers::get_semantic_top,
        handlers::get_semantic_series,
        handlers::correlate_events,
        handlers::get_system_status, handlers::get_system_plugins, handlers::poll_plugin_manually,
        handlers::update_plugin_config, handlers::get_profile, handlers::update_profile,
        handlers::get_dashboards, handlers::add_widget,
        handlers::create_dashboard, handlers::delete_widget,
        handlers::ingest_event,
        handlers::run_plugin_report,
        handlers::health_check
    ),
    components(schemas(Event, User, RegisterRequest, LoginRequest, AuthResponse, ApiReportMetadata, ApiReportData, PluginReports, CorrelationResult, SemanticStats, PluginStatus, SemanticParams, Dashboard, DashboardWidget)),
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
    // Lade Umgebungsvariablen aus .env Datei
    dotenvy::dotenv().ok();

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
                        for path in event.paths {
                            if let Err(e) = pm.reload_plugin(&path).await {
                                tracing::error!("Failed to hot-reload plugin {:?}: {}", path, e);
                            }
                        }
                    });
                }
            },
            Err(e) => tracing::error!("Watcher error: {}", e),
        }
    })?;
    watcher.watch(std::path::Path::new("./plugins"), RecursiveMode::NonRecursive)?;

    let cancel_token = CancellationToken::new();
    let mut event_service = EventService::new(db, plugin_manager);
    let (event_sender, _rx) = tokio::sync::broadcast::channel(1024);
    event_service.set_event_sender(event_sender.clone());
    
    let shared_state = Arc::new(AppState { 
        event_service, 
        event_sender,
        cancel_token: cancel_token.clone()
    });

    // Background Scheduler Task (Multi-Tenant aware)
    let scheduler_state = shared_state.clone();
    let scheduler_token = cancel_token.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = scheduler_token.cancelled() => {
                    tracing::info!("Scheduler shutting down...");
                    break;
                }
                _ = async {
                    let db = scheduler_state.event_service.db();
                    let user_ids: Vec<i64> = match sqlx::query_scalar::<_, i64>("SELECT id FROM users").fetch_all(db).await {
                        Ok(ids) => ids,
                        Err(e) => {
                            tracing::error!("Failed to fetch users for scheduler: {}", e);
                            vec![]
                        }
                    };

                    let manifests = scheduler_state.event_service.plugin_manager().get_plugin_manifests().await;
                    
                    for user_id in user_ids {
                        for name in manifests.keys() {
                            let svc = scheduler_state.event_service.clone();
                            let plugin_name = name.clone();
                            tokio::spawn(async move {
                                if let Err(e) = svc.poll_and_save_plugin(user_id, &plugin_name).await {
                                    tracing::warn!(user_id = %user_id, plugin = %plugin_name, "Scheduler poll failed: {}", e);
                                }
                            });
                        }
                    }
                    sleep(Duration::from_secs(60)).await;
                } => {}
            }
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
                .route("/summary", get(get_daily_summary))
                .route("/live", get(stream_live_events)))
            .nest("/analytics", Router::new()
                .route("/correlations", get(correlate_events))
                .route("/stats", get(get_semantic_stats))
                .route("/semantic/top", get(get_semantic_top))
                .route("/semantic/series", get(get_semantic_series))
                .route("/plugins/:id/reports/:report_id", get(run_plugin_report)))
            .nest("/system", Router::new()
                .route("/status", get(get_system_status))
                .route("/plugins", get(get_system_plugins))
                .route("/plugins/:id/poll", post(poll_plugin_manually))
                .route("/plugins/:id/config", post(update_plugin_config))
                .route("/dashboards", get(get_dashboards).post(create_dashboard))
                .route("/dashboards/:id/widgets", post(add_widget))
                .route("/dashboards/:id/widgets/:widget_id", delete(delete_widget))
                .route("/profile", get(get_profile).post(update_profile)))
            .route("/ingest", post(ingest_event))
            .layer(middleware::from_fn_with_state(shared_state.clone(), auth_middleware)));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::HeaderName::from_static("x-api-key")]);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_v1)
        .fallback_service(
            tower_http::services::ServeDir::new("web/dist")
                .fallback(tower_http::services::ServeFile::new("web/dist/index.html"))
        )
        .layer(cors)
        .with_state(shared_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Scry Multi-Tenant Platform on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Graceful Shutdown Signal Handler
    let final_cancel_token = cancel_token.clone();
    let shutdown = async move {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
        tracing::info!("Shutdown signal received, triggering cancellation tokens...");
        final_cancel_token.cancel();
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    tracing::info!("Scry shutdown complete.");
    Ok(())
}

async fn auth_middleware(State(state): State<Arc<AppState>>, mut req: Request<axum::body::Body>, next: Next) -> Result<Response, StatusCode> {
    if req.method() == axum::http::Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    let auth_header = req.headers().get("X-API-Key").and_then(|h| h.to_str().ok());
    let query_key = req.uri().query()
        .and_then(|q| serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(q).ok())
        .and_then(|m| m.get("api_key").cloned());

    let key_to_check = auth_header.map(|s| s.to_string()).or(query_key);
    
    if let Some(key) = key_to_check {
        let db = state.event_service.db();
        let auth = sqlx::query_as::<_, (i64, String)>("SELECT user_id, scopes FROM api_keys WHERE key = ?")
            .bind(&key).fetch_optional(db).await.map_err(|e| {
                tracing::error!("Auth DB Error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        
        if let Some((user_id, scopes_str)) = auth {
            let ctx = AuthContext {
                user_id,
                scopes: scopes_str.split(',').map(|s| s.to_string()).collect(),
            };
            req.extensions_mut().insert(ctx);
            return Ok(next.run(req).await);
        } else {
            tracing::warn!("Invalid API Key provided: {}", key);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}
