use serde::{Deserialize, Serialize};
use scry_proto::Event;
use utoipa::ToSchema;
use crate::event_service::EventService;
use crate::analytics_service::AnalyticsService;
use validator::Validate;

#[derive(Clone)]
pub struct AppState {
    pub event_service: EventService,
    pub analytics_service: AnalyticsService,
    pub event_sender: tokio::sync::broadcast::Sender<Event>,
    pub cancel_token: tokio_util::sync::CancellationToken,
}

#[derive(sqlx::FromRow, Debug)]
pub struct DbEvent {
    pub id: String,
    pub timestamp: String,
    pub category: String,
    pub source: String,
    pub payload: String,
    pub metadata: Option<String>,
    pub entities: Option<String>,
    pub display_title: Option<String>,
    pub display_subtitle: Option<String>,
}

impl TryFrom<DbEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(db_ev: DbEvent) -> Result<Self, Self::Error> {
        Ok(Self {
            id: uuid::Uuid::parse_str(&db_ev.id)?,
            timestamp: chrono::DateTime::parse_from_rfc3339(&db_ev.timestamp)?.with_timezone(&chrono::Utc),
            category: db_ev.category,
            source: db_ev.source,
            payload: serde_json::from_str(&db_ev.payload)?,
            metadata: db_ev.metadata.and_then(|m| serde_json::from_str(&m).ok()),
            entities: db_ev.entities.and_then(|e| serde_json::from_str(&e).ok()).unwrap_or_default(),
            context: vec![], // Resolved during ingestion, not stored in DB
            context_info: None, // Will be populated by EventService::enrich_event_context
            display_title: db_ev.display_title,
            display_subtitle: db_ev.display_subtitle,
        })
    }
}

#[derive(Serialize, ToSchema)]
pub struct ApiReportMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub viz: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApiReportData {
    pub columns: Vec<String>,
    pub data_json: String,
}

#[derive(Serialize, ToSchema)]
pub enum ApiWidgetTemplate { Metric, Trend, TopList, Status, Spotlight }

#[derive(Serialize, ToSchema)]
pub struct ApiWidgetDefinition {
    pub id: String,
    pub title: String,
    pub template: ApiWidgetTemplate,
    pub config_json: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApiTraitCapability {
    pub entity_namespace: String,
    pub entity_type: String,
    pub trait_id: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApiNamespace {
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ApiEntity {
    pub namespace: String,
    pub typ: String,
    pub id: String,
    pub title: String,
    pub photo_url: Option<String>,
    pub link: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApiDataField {
    pub category: String,
    pub path: String,
    pub semantic_type: String,
    pub description: String,
    pub icon: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct PluginStatus {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub roles: Vec<String>,
    pub capabilities: Vec<String>,
    pub subscriptions: Vec<String>,
    pub exports: Vec<ApiDataField>,
    pub provided_traits: Vec<ApiTraitCapability>,
    pub reports: Vec<ApiReportMetadata>,
    pub config_schema: Option<String>,
    pub suggested_widgets: Vec<ApiWidgetDefinition>,
}

#[derive(Serialize, ToSchema)]
pub struct PluginReports {
    pub plugin: String,
    pub reports: Vec<ApiReportMetadata>,
}

#[derive(Serialize, ToSchema)]
pub struct CorrelationResult {
    pub base: serde_json::Value,
    pub joined: serde_json::Value,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 8, max = 100))]
    pub password: String,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub api_key: String,
    pub user: User,
}

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: i64,
    #[allow(dead_code)]
    pub scopes: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_default: bool,
    pub widgets: Vec<DashboardWidget>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct DashboardWidget {
    pub id: String,
    pub dashboard_id: String,
    pub r#type: String,
    pub title: Option<String>,
    pub config: serde_json::Value,
    pub width_span: i32,
    pub sort_order: i32,
}

#[derive(Serialize, ToSchema)]
pub struct SemanticStats {
    pub base_type: String,
    pub join_type: String,
    pub sample_size: usize,
    pub correlations: serde_json::Value,
}
