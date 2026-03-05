mod plugins;
mod domain;
mod state;
mod handlers;
mod error;
mod repository;
mod services;

use services::{AuthService, DashboardService, GraphService, AnalyticsService, PluginService, SystemService, EventService, SecretService};
use state::AppState;

use axum::{
    http::HeaderName,
    Router,
};
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::plugins::PluginManager;
use notify::{Watcher, RecursiveMode};
use tower_http::cors::{Any, CorsLayer};
use tokio_util::sync::CancellationToken;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::register_user, handlers::auth::login_user,
        handlers::plugins::get_catalog, handlers::analytics::search_events, 
        handlers::events::get_data_by_type, 
        handlers::events::get_timeline, 
        handlers::events::get_daily_summary,
        handlers::analytics::get_semantic_top,
        handlers::analytics::get_semantic_series,
        handlers::analytics::correlate_events,
        handlers::system::get_system_status, handlers::plugins::get_system_plugins, handlers::plugins::poll_plugin_manually,
        handlers::plugins::update_plugin_config, handlers::plugins::get_plugin_config, handlers::auth::get_profile, handlers::auth::update_profile,
        handlers::dashboards::get_dashboards, handlers::dashboards::add_widget,
        handlers::dashboards::create_dashboard, handlers::dashboards::delete_widget,
        handlers::events::ingest_event,
        handlers::entities::get_namespaces,
        handlers::entities::get_namespace_types,
        handlers::entities::get_entities,
        handlers::entities::get_entity_traits,
        handlers::events::get_event_by_id,
        handlers::events::get_events_by_entity,
        handlers::plugins::run_plugin_report,
        handlers::system::health_check
    ),
    components(schemas(
        scry_proto::Event, domain::User, domain::RegisterRequest, domain::LoginRequest, domain::AuthResponse, 
        domain::ApiReportMetadata, domain::ApiReportData, domain::PluginReports, domain::CorrelationResult, 
        domain::SemanticStats, domain::PluginStatus, domain::SemanticParams, domain::Dashboard, 
        domain::DashboardWidget, domain::ApiEntity, domain::ApiNamespace
    )),
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
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "scry_core=debug,info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:scry.db".to_string());
    let db_filename = database_url.trim_start_matches("sqlite:").split('?').next().unwrap_or("scry.db");

    let pool_options = SqliteConnectOptions::new()
        .filename(db_filename)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(5));

    let db = SqlitePool::connect_with(pool_options).await?;
    sqlx::migrate!("../../migrations").run(&db).await?;

    let plugin_manager = Arc::new(PluginManager::new("./plugins", db.clone())?);
    plugin_manager.reload_plugins().await?;

    let pm_for_watcher = plugin_manager.clone();
    let rt_handle = tokio::runtime::Handle::current();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
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
        }
    })?;
    watcher.watch(std::path::Path::new("./plugins"), RecursiveMode::NonRecursive)?;

    let cancel_token = CancellationToken::new();
    let mut event_service = EventService::new(db.clone(), plugin_manager.clone());
    let (event_sender, _rx) = tokio::sync::broadcast::channel(1024);
    event_service.set_event_sender(event_sender.clone());

    let rate_limiter = Arc::new(crate::handlers::middleware::rate_limit::RateLimitState::new());
    
    // Background task for rate limiter cleanup
    let rl_for_cleanup = rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            rl_for_cleanup.cleanup();
        }
    });
    
    let shared_state = Arc::new(AppState { 
        event_service: event_service.clone(), 
        analytics_service: AnalyticsService::new(db.clone(), plugin_manager.clone()),
        auth_service: AuthService::new(db.clone()),
        dashboard_service: DashboardService::new(db.clone()),
        graph_service: GraphService::new(db.clone(), plugin_manager.clone()),
        plugin_service: PluginService::new(db.clone(), plugin_manager.clone(), event_service, SecretService::new()),
        system_service: SystemService::new(db.clone()),
        rate_limiter,
        event_sender,
        cancel_token: cancel_token.clone(),
        db: db.clone(),
    });

    // Start Background Tasks via SystemService
    let background_state = shared_state.clone();
    let background_token = cancel_token.clone();
    tokio::spawn(async move {
        background_state.system_service.run_background_tasks(background_state.clone(), background_token).await;
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            HeaderName::from_static("x-api-key"),
        ]);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(handlers::app_router(shared_state.clone()))
        .fallback_service(
            tower_http::services::ServeDir::new("web/dist")
                .fallback(tower_http::services::ServeFile::new("web/dist/index.html"))
        )
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Scry Multi-Tenant Platform on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    let final_cancel_token = cancel_token.clone();
    let shutdown = async move {
        let ctrl_c = async { tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler"); };
        #[cfg(unix)]
        let terminate = async { tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("failed to install signal handler").recv().await; };
        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
        tracing::info!("Shutdown signal received, triggering cancellation tokens...");
        final_cancel_token.cancel();
    };

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown)
        .await?;
    tracing::info!("Scry shutdown complete.");
    Ok(())
}
