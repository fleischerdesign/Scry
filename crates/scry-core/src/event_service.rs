use anyhow::Result;
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::plugins::PluginManager;
use scry_proto::Event;
use crate::models::DbEvent;
use serde_json::Value;

#[derive(Clone)]
pub struct EventService {
    db: SqlitePool,
    plugin_manager: Arc<PluginManager>,
}

impl EventService {
    pub fn new(db: SqlitePool, plugin_manager: Arc<PluginManager>) -> Self {
        Self { db, plugin_manager }
    }

    pub fn db(&self) -> &SqlitePool { &self.db }
    pub fn plugin_manager(&self) -> &PluginManager { &self.plugin_manager }

    pub async fn ingest_event(&self, user_id: i64, event: Event) -> Result<Event> {
        let processed_event = self.plugin_manager.run_ingest_pipeline(user_id, event).await?;
        
        sqlx::query("INSERT INTO events (id, user_id, timestamp, category, source, payload, metadata) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(processed_event.id.to_string())
            .bind(user_id)
            .bind(processed_event.timestamp.to_rfc3339())
            .bind(&processed_event.category)
            .bind(&processed_event.source)
            .bind(serde_json::to_string(&processed_event.payload)?)
            .bind(processed_event.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap()))
            .execute(&self.db)
            .await?;

        Ok(processed_event)
    }

    pub async fn list_events(&self, user_id: i64, category: Option<String>, limit: u32) -> Result<Vec<Event>> {
        let db_events = if let Some(cat) = category {
            sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata FROM events WHERE user_id = ? AND category = ? ORDER BY timestamp DESC LIMIT ?")
                .bind(user_id).bind(cat).bind(limit)
        } else {
            sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata FROM events WHERE user_id = ? ORDER BY timestamp DESC LIMIT ?")
                .bind(user_id).bind(limit)
        }.fetch_all(&self.db).await?;

        Ok(db_events.into_iter().filter_map(|e| Event::try_from(e).ok()).collect())
    }

    pub async fn poll_and_save_plugin(&self, user_id: i64, name: &str) -> Result<usize> {
        let events = self.plugin_manager.poll_plugin(user_id, name).await?;
        let count = events.len();
        for event in events {
            let _ = self.ingest_event(user_id, event).await?;
        }
        Ok(count)
    }

    pub async fn correlate_semantic(&self, user_id: i64, base_semantic: &str, join_semantic: &str, limit: u32) -> Result<Vec<Value>> {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        let mut base_cat = None;
        let mut join_cat = None;

        for m in manifests.values() {
            for export in &m.exports {
                if export.semantic_type == base_semantic { base_cat = Some(export.category.clone()); }
                if export.semantic_type == join_semantic { join_cat = Some(export.category.clone()); }
            }
        }

        match (base_cat, join_cat) {
            (Some(bc), Some(jc)) => self.correlate_nearest(user_id, &bc, &jc, limit).await,
            _ => Err(anyhow::anyhow!("Semantic types not found")),
        }
    }

    pub async fn search_semantic(&self, user_id: i64, semantic_query: &str, limit: u32) -> Result<Vec<Event>> {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        let mut target_categories = std::collections::HashSet::new();

        for m in manifests.values() {
            for export in &m.exports {
                if export.semantic_type.contains(semantic_query) {
                    target_categories.insert(export.category.clone());
                }
            }
        }

        if target_categories.is_empty() { return Ok(vec![]); }

        let mut all_events = Vec::new();
        for cat in target_categories {
            all_events.extend(self.list_events(user_id, Some(cat), limit).await?);
        }

        all_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        all_events.truncate(limit as usize);
        Ok(all_events)
    }

    pub async fn get_enriched_timeline(&self, user_id: i64, base_category: &str, limit: u32) -> Result<Vec<Value>> {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        let mut context_categories = std::collections::HashMap::new();

        for m in manifests.values() {
            for export in &m.exports {
                if export.category != base_category {
                    context_categories.insert(export.semantic_type.clone(), export.category.clone());
                }
            }
        }

        let base_events = self.list_events(user_id, Some(base_category.to_string()), limit).await?;
        let mut enriched_timeline = Vec::new();

        for ev in base_events {
            let ts = ev.timestamp.to_rfc3339();
            let mut entry = serde_json::json!({
                "id": ev.id,
                "timestamp": ts,
                "event": ev.payload,
                "context": {}
            });

            for (semantic_type, cat) in &context_categories {
                let context_payload = sqlx::query_scalar::<_, String>(
                    "SELECT payload FROM events WHERE user_id = ? AND category = ? ORDER BY ABS(julianday(substr(timestamp, 1, 19)) - julianday(substr(?, 1, 19))) ASC LIMIT 1"
                )
                .bind(user_id).bind(cat).bind(&ts).fetch_optional(&self.db).await?;

                if let Some(p) = context_payload {
                    entry["context"][semantic_type] = serde_json::from_str::<Value>(&p).unwrap_or_default();
                }
            }
            enriched_timeline.push(entry);
        }

        Ok(enriched_timeline)
    }

    pub async fn correlate_nearest(&self, user_id: i64, base_category: &str, join_category: &str, limit: u32) -> Result<Vec<Value>> {
        let sql = r#"
            SELECT 
                CAST(b.payload AS TEXT),
                CAST(j.payload AS TEXT)
            FROM events b
            JOIN events j ON j.category = ? AND j.user_id = ?
            WHERE b.category = ? AND b.user_id = ?
            GROUP BY b.id
            HAVING MIN(ABS(julianday(substr(b.timestamp, 1, 19)) - julianday(substr(j.timestamp, 1, 19))))
            ORDER BY b.timestamp DESC
            LIMIT ?
        "#;

        let rows = sqlx::query_as::<_, (String, Option<String>)>(sql)
            .bind(join_category).bind(user_id)
            .bind(base_category).bind(user_id)
            .bind(limit)
            .fetch_all(&self.db).await?;

        Ok(rows.into_iter().map(|(b, j)| {
            serde_json::json!({
                "base": serde_json::from_str::<Value>(&b).unwrap_or_default(),
                "joined": j.and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or_default(),
            })
        }).collect())
    }

    pub async fn calculate_semantic_stats(&self, user_id: i64, base_semantic: &str, join_semantic: &str, limit: u32) -> Result<crate::models::SemanticStats> {
        let correlations = self.correlate_semantic(user_id, base_semantic, join_semantic, limit).await?;
        let sample_size = correlations.len();
        let mut distribution = std::collections::HashMap::new();

        for corr in &correlations {
            let base_val = corr.get("base").unwrap_or(&Value::Null).to_string();
            let join_val = corr.get("joined").unwrap_or(&Value::Null).to_string();
            let entry = distribution.entry(base_val).or_insert_with(std::collections::HashMap::new);
            *entry.entry(join_val).or_insert(0) += 1;
        }

        Ok(crate::models::SemanticStats {
            base_type: base_semantic.to_string(), join_type: join_semantic.to_string(),
            sample_size, correlations: serde_json::to_value(distribution)?,
        })
    }

    pub async fn generate_daily_summary(&self, user_id: i64, date: &str) -> Result<Vec<String>> {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        let mut full_summary = Vec::new();
        let start = format!("{}T00:00:00Z", date);
        let end = format!("{}T23:59:59Z", date);

        for plugin_name in manifests.keys() {
            if let Ok(p_summary) = self.plugin_manager.get_plugin_summary(user_id, plugin_name, start.clone(), end.clone()).await {
                if !p_summary.is_empty() { full_summary.push(p_summary); }
            }
        }
        Ok(full_summary)
    }
}
