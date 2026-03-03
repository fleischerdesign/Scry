use scry_proto::Event;
use crate::services::{AuthService, DashboardService, GraphService, PluginService, AnalyticsService, SystemService, EventService};

use crate::handlers::middleware::rate_limit::RateLimitState;

#[derive(Clone)]
pub struct AppState {
    pub event_service: EventService,
    pub analytics_service: AnalyticsService,
    pub auth_service: AuthService,
    pub dashboard_service: DashboardService,
    pub graph_service: GraphService,
    pub plugin_service: PluginService,
    pub system_service: SystemService,
    pub rate_limiter: std::sync::Arc<RateLimitState>,
    pub event_sender: tokio::sync::broadcast::Sender<Event>,
    pub cancel_token: tokio_util::sync::CancellationToken,
}
