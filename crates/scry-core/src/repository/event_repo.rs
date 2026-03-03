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
    pub display_title: Option<String>,
    pub display_subtitle: Option<String>,
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
            context: vec![], // Resolved during ingestion, not stored in DB
            context_info: None, // Will be populated by EventService::enrich_event_context
            display_title: db_ev.display_title,
            display_subtitle: db_ev.display_subtitle,
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
            sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle FROM events WHERE user_id = ? AND category = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?")
                .bind(self.user_id).bind(cat).bind(limit).bind(offset)
        } else {
            sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle FROM events WHERE user_id = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?")
                .bind(self.user_id).bind(limit).bind(offset)
        }.fetch_all(self.pool).await?;

        Ok(db_events.into_iter().filter_map(|e| Event::try_from(e).ok()).collect())
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Event>> {
        let db_event = sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle FROM events WHERE user_id = ? AND id = ?")
            .bind(self.user_id).bind(id).fetch_optional(self.pool).await?;

        Ok(db_event.and_then(|e| Event::try_from(e).ok()))
    }

    pub async fn get_by_entity(&self, namespace: &str, typ: &str, id: &str) -> Result<Vec<Event>> {
        let rows = sqlx::query_as::<_, DbEvent>(
            "SELECT id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle FROM events 
             WHERE user_id = ? AND EXISTS (
                SELECT 1 FROM json_each(entities) WHERE json_extract(value, '$.namespace') = ? AND json_extract(value, '$.typ') = ? AND json_extract(value, '$.id') = ?
             ) ORDER BY timestamp DESC LIMIT 100"
        )
        .bind(self.user_id).bind(namespace).bind(typ).bind(id).fetch_all(self.pool).await?;

        Ok(rows.into_iter().filter_map(|r| Event::try_from(r).ok()).collect())
    }

    pub async fn insert(&self, event: &Event) -> Result<()> {
        sqlx::query("INSERT INTO events (id, user_id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(event.id.to_string())
            .bind(self.user_id)
            .bind(event.timestamp.to_rfc3339())
            .bind(&event.category)
            .bind(&event.source)
            .bind(serde_json::to_string(&event.payload).map_err(|e| Error::BadRequest(e.to_string()))?)
            .bind(serde_json::to_string(event.metadata.as_ref().unwrap()).unwrap())
            .bind(serde_json::to_string(&event.entities).unwrap_or_else(|_| "[]".to_string()))
            .bind(&event.display_title)
            .bind(&event.display_subtitle)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_last_payload(&self, category: &str, timestamp: &str) -> Result<Option<String>> {
        let payload = sqlx::query_scalar::<_, String>(
            "SELECT payload FROM events WHERE user_id = ? AND category = ? AND timestamp <= ? ORDER BY timestamp DESC LIMIT 1"
        )
        .bind(self.user_id).bind(category).bind(timestamp).fetch_optional(self.pool).await?;
        Ok(payload)
    }
}
