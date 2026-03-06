use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EntityRef {
    pub path: String,
    pub namespace: String,
    pub typ: String,
    pub id: String,
}

#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[cfg_attr(feature = "backend", derive(utoipa::ToSchema, sqlx::FromRow))]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
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
    #[serde(default)]
    pub context_info: Option<serde_json::Value>,
    pub display_title: Option<String>,
    pub display_subtitle: Option<String>,
    pub display_image: Option<String>,
    pub display_icon: Option<String>,
    pub display_value: Option<String>,
    pub confidence: Option<f64>,
}

impl Event {
    pub fn new(
        category: impl Into<String>,
        source: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category: category.into(),
            source: source.into(),
            payload,
            ..Default::default()
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.display_title = Some(title.into());
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.display_subtitle = Some(subtitle.into());
        self
    }

    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.display_image = Some(image.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.display_icon = Some(icon.into());
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.display_value = Some(value.into());
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_entity(mut self, entity: EntityRef) -> Self {
        self.entities.push(entity);
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context.push(context.into());
        self
    }
}
