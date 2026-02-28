use serde::{Deserialize, Serialize};
use scry_proto::Event;
use utoipa::ToSchema;

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

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
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
    pub scopes: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct SemanticStats {
    pub base_type: String,
    pub join_type: String,
    pub sample_size: usize,
    pub correlations: serde_json::Value,
}
