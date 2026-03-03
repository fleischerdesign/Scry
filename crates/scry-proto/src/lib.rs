use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntityRef {
    pub path: String,
    pub namespace: String,
    pub typ: String,
    pub id: String,
}

#[cfg_attr(feature = "backend", derive(utoipa::ToSchema, sqlx::FromRow))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub category: String,
    pub source: String,
    pub payload: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub entities: Vec<EntityRef>,
    #[serde(default)]
    pub context: Vec<String>,
    pub display_title: Option<String>,
    pub display_subtitle: Option<String>,
}

impl Event {
    pub fn new(category: String, source: String, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category,
            source,
            payload,
            metadata: None,
            entities: Vec::new(),
            context: Vec::new(),
            display_title: None,
            display_subtitle: None,
        }
    }
}
