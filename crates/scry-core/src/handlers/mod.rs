pub mod analytics;
pub mod auth;
pub mod dashboards;
pub mod entities;
pub mod events;
pub mod middleware;
pub mod plugins;
pub mod semantic;
pub mod system;

use self::middleware::{auth_middleware, identity_resolver, rate_limit_middleware};
use crate::state::AppState;
use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
    routing::{delete, get, post},
};
use std::sync::Arc;

pub fn app_router(state: Arc<AppState>) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth::register_user))
        .route("/login", post(auth::login_user));

    let api_v1 = Router::new()
        .nest("/auth", auth_routes)
        .merge(
            Router::new()
                .nest(
                    "/semantic",
                    Router::new().route("/resolve", get(semantic::resolve_semantic_type)),
                )
                .nest(
                    "/discovery",
                    Router::new()
                        .route("/catalog", get(plugins::get_catalog))
                        .route("/search", get(analytics::search_events))
                        .route("/entities", get(entities::get_namespaces))
                        .route("/entities/:namespace", get(entities::get_namespace_types))
                        .route("/entities/:namespace/:typ", get(entities::get_entities))
                        .route(
                            "/entities/:namespace/:typ/:id/traits",
                            get(entities::get_entity_traits),
                        )
                        .route("/resolve", post(entities::resolve_entities)),
                )
                .nest(
                    "/data",
                    Router::new()
                        .route("/id/:id", get(events::get_event_by_id))
                        .route(
                            "/entity/:namespace/:typ/:id",
                            get(events::get_events_by_entity),
                        )
                        .route("/*path", get(events::get_data_by_type)),
                )
                .nest(
                    "/streams",
                    Router::new()
                        .route("/timeline", get(events::get_timeline))
                        .route("/summary", get(events::get_daily_summary))
                        .route("/live", get(events::stream_live_events)),
                )
                .nest(
                    "/analytics",
                    Router::new()
                        .route("/discover", post(analytics::trigger_discovery))
                        .route("/discoveries", get(analytics::get_discoveries))
                        .route("/correlations", get(analytics::correlate_events))
                        .route("/stats", get(analytics::get_semantic_stats))
                        .route("/semantic/top", get(analytics::get_semantic_top))
                        .route("/semantic/series", get(analytics::get_semantic_series))
                        .route(
                            "/plugins/:id/reports/:report_id",
                            get(plugins::run_plugin_report),
                        ),
                )
                .nest(
                    "/system",
                    Router::new()
                        .route("/status", get(system::get_system_status))
                        .route("/plugins", get(plugins::get_system_plugins))
                        .route("/plugins/:id/poll", post(plugins::poll_plugin_manually))
                        .route(
                            "/plugins/:id/config",
                            get(plugins::get_plugin_config).post(plugins::update_plugin_config),
                        )
                        .route("/plugins/:id/secrets", get(plugins::get_plugin_secrets))
                        .route("/plugins/:id/auth", get(plugins::plugin_oauth_start))
                        .route(
                            "/dashboards",
                            get(dashboards::get_dashboards).post(dashboards::create_dashboard),
                        )
                        .route("/dashboards/:id/widgets", post(dashboards::add_widget))
                        .route(
                            "/dashboards/:id/widgets/:widget_id",
                            delete(dashboards::delete_widget),
                        )
                        .route(
                            "/profile",
                            get(auth::get_profile).post(auth::update_profile),
                        ),
                )
                .route("/ingest", post(events::ingest_event))
                .layer(from_fn_with_state(state.clone(), auth_middleware)),
        )
        .route(
            "/system/plugins/:id/auth/callback",
            get(plugins::plugin_oauth_callback),
        )
        // Pipeline: 1. Identity Resolver -> 2. Rate Limiting (protects DB) -> 3. Authentication (uses DB)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn(identity_resolver));

    Router::new()
        .route("/health", get(system::health_check))
        .nest("/api/v1", api_v1)
        .with_state(state)
}
