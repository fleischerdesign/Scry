use sqlx::SqlitePool;
use scry_proto::Event;
use crate::error::{Error, Result};

#[derive(sqlx::FromRow, Debug)]
pub struct DbEvent {
    pub id: String,
    pub timestamp: String,
    pub category: String,
    pub source: String,
    pub payload: String,
    pub metadata: Option<String>,
    pub entities: Option<String>,
    pub context: Option<String>,
    pub display_title: Option<String>,
    pub display_subtitle: Option<String>,
    pub display_image: Option<String>,
    pub display_icon: Option<String>,
    pub display_value: Option<String>,
    pub confidence: Option<f64>,
}

impl TryFrom<DbEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(db_ev: DbEvent) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: uuid::Uuid::parse_str(&db_ev.id)?,
            timestamp: chrono::DateTime::parse_from_rfc3339(&db_ev.timestamp)?.with_timezone(&chrono::Utc),
            category: db_ev.category,
            source: db_ev.source,
            payload: serde_json::from_str(&db_ev.payload)?,
            metadata: db_ev.metadata.and_then(|m| serde_json::from_str(&m).ok()),
            entities: db_ev.entities.and_then(|e| serde_json::from_str(&e).ok()).unwrap_or_default(),
            context: db_ev.context.and_then(|c| serde_json::from_str(&c).ok()).unwrap_or_default(),
            context_info: None, // Will be populated by EventService::enrich_event_context
            display_title: db_ev.display_title,
            display_subtitle: db_ev.display_subtitle,
            display_image: db_ev.display_image,
            display_icon: db_ev.display_icon,
            display_value: db_ev.display_value,
            confidence: db_ev.confidence,
        })
    }
}

pub struct EventRepository<'a> {
    pool: &'a SqlitePool,
    user_id: i64,
}

impl<'a> EventRepository<'a> {
    pub fn new(pool: &'a SqlitePool, user_id: i64) -> Self {
        Self { pool, user_id }
    }

    pub async fn list(&self, category: Option<String>, limit: u32, offset: u32) -> Result<Vec<Event>> {
        let db_events = if let Some(cat) = category {
            sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata, entities, context, display_title, display_subtitle, display_image, display_icon, display_value, confidence FROM events WHERE user_id = ? AND category = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?")
                .bind(self.user_id).bind(cat).bind(limit).bind(offset)
        } else {
            sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata, entities, context, display_title, display_subtitle, display_image, display_icon, display_value, confidence FROM events WHERE user_id = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?")
                .bind(self.user_id).bind(limit).bind(offset)
        }.fetch_all(self.pool).await?;

        Ok(db_events.into_iter().filter_map(|e| Event::try_from(e).ok()).collect())
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Event>> {
        let db_event = sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata, entities, context, display_title, display_subtitle, display_image, display_icon, display_value, confidence FROM events WHERE user_id = ? AND id = ?")
            .bind(self.user_id).bind(id).fetch_optional(self.pool).await?;

        Ok(db_event.and_then(|e| Event::try_from(e).ok()))
    }

    pub async fn get_by_entity(&self, namespace: &str, typ: &str, id: &str) -> Result<Vec<Event>> {
        let rows = sqlx::query_as::<_, DbEvent>(
            "SELECT id, timestamp, category, source, payload, metadata, entities, context, display_title, display_subtitle, display_image, display_icon, display_value, confidence FROM events 
             WHERE user_id = ? AND EXISTS (
                SELECT 1 FROM json_each(entities) WHERE json_extract(value, '$.namespace') = ? AND json_extract(value, '$.typ') = ? AND json_extract(value, '$.id') = ?
             ) ORDER BY timestamp DESC LIMIT 100"
        )
        .bind(self.user_id).bind(namespace).bind(typ).bind(id).fetch_all(self.pool).await?;

        Ok(rows.into_iter().filter_map(|r| Event::try_from(r).ok()).collect())
    }

    pub async fn insert(&self, event: &Event) -> Result<()> {
        sqlx::query("INSERT INTO events (id, user_id, timestamp, category, source, payload, metadata, entities, context, display_title, display_subtitle, display_image, display_icon, display_value, confidence) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(event.id.to_string())
            .bind(self.user_id)
            .bind(event.timestamp.to_rfc3339())
            .bind(&event.category)
            .bind(&event.source)
            .bind(serde_json::to_string(&event.payload).map_err(|e| Error::BadRequest(e.to_string()))?)
            .bind(event.metadata.as_ref().and_then(|m| serde_json::to_string(m).ok()).unwrap_or_else(|| "{}".to_string()))
            .bind(serde_json::to_string(&event.entities).unwrap_or_else(|_| "[]".to_string()))
            .bind(serde_json::to_string(&event.context).unwrap_or_else(|_| "[]".to_string()))
            .bind(&event.display_title)
            .bind(&event.display_subtitle)
            .bind(&event.display_image)
            .bind(&event.display_icon)
            .bind(event.display_value.clone())
            .bind(event.confidence)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_last_event(&self, category: &str, timestamp: &str) -> Result<Option<Event>> {
        let db_event = sqlx::query_as::<_, DbEvent>(
            "SELECT id, timestamp, category, source, payload, metadata, entities, context, display_title, display_subtitle, display_image, display_icon, display_value, confidence 
             FROM events WHERE user_id = ? AND category = ? AND timestamp <= ? ORDER BY timestamp DESC LIMIT 1"
        )
        .bind(self.user_id).bind(category).bind(timestamp).fetch_optional(self.pool).await?;
        
        match db_event {
            Some(e) => Ok(Some(Event::try_from(e).map_err(|_err| Error::Internal)?)),
            None => Ok(None)
        }
    }
}
