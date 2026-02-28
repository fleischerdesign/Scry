use serde::{Deserialize, Serialize};
use scry_proto::Event;
use utoipa::ToSchema;
use crate::event_service::EventService;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use validator::Validate;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Plugin error: {0}")]
    Plugin(#[from] anyhow::Error),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Invalid request: {0}")]
    BadRequest(String),
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Plugin(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub event_service: EventService,
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
pub struct PluginStatus {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub subscriptions: Vec<String>,
    pub reports: Vec<ApiReportMetadata>,
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
