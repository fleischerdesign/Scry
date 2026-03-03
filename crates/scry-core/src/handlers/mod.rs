pub mod auth;
pub mod events;
pub mod dashboards;
pub mod plugins;
pub mod analytics;
pub mod entities;
pub mod system;

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use crate::state::AppState;
use crate::domain::AuthContext;

pub fn app_router(state: Arc<AppState>) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth::register_user))
        .route("/login", post(auth::login_user));

    let api_v1 = Router::new()
        .nest("/auth", auth_routes)
        .merge(Router::new()
            .nest("/discovery", Router::new()
                .route("/catalog", get(plugins::get_catalog))
                .route("/search", get(analytics::search_events))
                .route("/entities", get(entities::get_namespaces))
                .route("/entities/:namespace", get(entities::get_namespace_types))
                .route("/entities/:namespace/:typ", get(entities::get_entities))
                .route("/entities/:namespace/:typ/:id/traits", get(entities::get_entity_traits)))
            .nest("/data", Router::new()
                .route("/id/:id", get(events::get_event_by_id))
                .route("/entity/:namespace/:typ/:id", get(events::get_events_by_entity))
                .route("/*path", get(events::get_data_by_type)))
            .nest("/streams", Router::new()
                .route("/timeline", get(events::get_timeline))
                .route("/summary", get(events::get_daily_summary))
                .route("/live", get(events::stream_live_events)))
            .nest("/analytics", Router::new()
                .route("/discover", post(analytics::trigger_discovery))
                .route("/discoveries", get(analytics::get_discoveries))
                .route("/correlations", get(analytics::correlate_events))
                .route("/stats", get(analytics::get_semantic_stats))
                .route("/semantic/top", get(analytics::get_semantic_top))
                .route("/semantic/series", get(analytics::get_semantic_series))
                .route("/plugins/:id/reports/:report_id", get(plugins::run_plugin_report)))
            .nest("/system", Router::new()
                .route("/status", get(system::get_system_status))
                .route("/plugins", get(plugins::get_system_plugins))
                .route("/plugins/:id/poll", post(plugins::poll_plugin_manually))
                .route("/plugins/:id/config", get(plugins::get_plugin_config).post(plugins::update_plugin_config))
                .route("/dashboards", get(dashboards::get_dashboards).post(dashboards::create_dashboard))
                .route("/dashboards/:id/widgets", post(dashboards::add_widget))
                .route("/dashboards/:id/widgets/:widget_id", delete(dashboards::delete_widget))
                .route("/profile", get(auth::get_profile).post(auth::update_profile)))
            .route("/ingest", post(events::ingest_event))
            .layer(middleware::from_fn_with_state(state.clone(), auth_middleware)));

    Router::new()
        .route("/health", get(system::health_check))
        .nest("/api/v1", api_v1)
        .with_state(state)
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
        let auth = state.auth_service.verify_api_key(&key).await.map_err(|e| {
            tracing::error!("Auth Service Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        if let Some((user_id, scopes)) = auth {
            let ctx = AuthContext {
                user_id,
                scopes,
            };
            req.extensions_mut().insert(ctx);
            return Ok(next.run(req).await);
        } else {
            tracing::warn!("Invalid API Key provided: {}", key);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}
