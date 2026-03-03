use std::sync::Arc;
use std::collections::HashMap;
use crate::plugins::PluginManager;
use scry_proto::Event;
use crate::models::{DbEvent};
use crate::error::{Error, Result};
use serde_json::Value;

#[derive(Clone)]
pub struct EventService {
    db: sqlx::SqlitePool,
    plugin_manager: Arc<PluginManager>,
    event_sender: Option<tokio::sync::broadcast::Sender<Event>>,
}

impl EventService {
    pub fn new(db: sqlx::SqlitePool, plugin_manager: Arc<PluginManager>) -> Self {
        Self { db, plugin_manager, event_sender: None }
    }

    pub fn set_event_sender(&mut self, sender: tokio::sync::broadcast::Sender<Event>) {
        self.event_sender = Some(sender);
    }

    pub fn db(&self) -> &sqlx::SqlitePool { &self.db }
    pub fn plugin_manager(&self) -> &PluginManager { &self.plugin_manager }

    pub async fn ingest_event(&self, user_id: i64, event: Event) -> Result<Event> {
        let mut processed_event: Event = self.plugin_manager.run_ingest_pipeline(user_id, event).await
            .map_err(|e| Error::Plugin(e))?;
        
        // --- Dynamic Context Resolution ---
        // Resolve semantic hints (aliases or full URIs) into real EntityRefs
        let hints = processed_event.context.clone();
        
        // Use the processor ID from metadata if available, otherwise fallback to source (base ID)
        let processor_id = processed_event.metadata.as_ref()
            .and_then(|m| m.get("processor"))
            .and_then(|p| p.as_str())
            .unwrap_or_else(|| processed_event.source.split('+').next().unwrap_or(&processed_event.source));

        for hint in hints {
            let entity_ref = if hint.starts_with("alias:") {
                let alias_key = hint.clone();
                
                // 1. Try to load from plugin configuration
                let config_val = sqlx::query_scalar::<_, String>(
                    "SELECT value FROM plugin_config WHERE user_id = ? AND plugin_id = ? AND key = ?"
                )
                .bind(user_id).bind(processor_id).bind(&alias_key).fetch_optional(&self.db).await?;

                if let Some(target_uri) = config_val {
                    // Resolve the URI found in config (ns/typ/id)
                    let parts: Vec<&str> = target_uri.split('/').collect();
                    if parts.len() == 3 {
                        Some(scry_proto::EntityRef {
                            path: hint.clone(),
                            namespace: parts[0].to_string(),
                            typ: parts[1].to_string(),
                            id: parts[2].to_string(),
                        })
                    } else { None }
                } else {
                    // 2. Default Fallbacks & Dynamic Discovery
                    let alias_name = hint.strip_prefix("alias:").unwrap();
                    let default_target = match alias_name {
                        "self" | "owner" | "subject" | "listener" => Some("scry.core/user/self"),
                        _ => None
                    };

                    if let Some(target) = default_target {
                        // Store default in config so it becomes visible in UI
                        let _ = sqlx::query("INSERT INTO plugin_config (user_id, plugin_id, key, value) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
                            .bind(user_id).bind(processor_id).bind(&alias_key).bind(target).execute(&self.db).await;
                        
                        Some(scry_proto::EntityRef {
                            path: hint.clone(),
                            namespace: "scry.core".to_string(),
                            typ: "user".to_string(),
                            id: "self".to_string(),
                        })
                    } else { None }
                }
            } else {
                // Expected format: namespace/type/id
                let parts: Vec<&str> = hint.split('/').collect();
                if parts.len() == 3 {
                    Some(scry_proto::EntityRef {
                        path: hint.clone(),
                        namespace: parts[0].to_string(),
                        typ: parts[1].to_string(),
                        id: parts[2].to_string(),
                    })
                } else {
                    None
                }
            };

            if let Some(r) = entity_ref {
                // Avoid duplicates
                if !processed_event.entities.iter().any(|e| e.namespace == r.namespace && e.typ == r.typ && e.id == r.id) {
                    processed_event.entities.push(r);
                }
            }
        }

        // Metadaten vervollständigen (für DB und Broadcast)
        let mut meta = processed_event.metadata.unwrap_or_else(|| serde_json::json!({}));
        meta["user_id"] = serde_json::json!(user_id);
        processed_event.metadata = Some(meta);

        // Entitäten persistieren (Global Knowledge Graph) und benachrichtigen
        for ent in &processed_event.entities {
            // 1. In globale Entitäten-Tabelle eintragen (für Suche und Traits)
            sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
                .bind(user_id)
                .bind(&ent.namespace)
                .bind(&ent.typ)
                .bind(&ent.id)
                .execute(&self.db)
                .await?;

            // 2. Enricher benachrichtigen (Background)
            let pm = self.plugin_manager.clone();
            let ns = ent.namespace.clone();
            let typ = ent.typ.clone();
            let id = ent.id.clone();
            tokio::spawn(async move {
                if let Err(e) = pm.notify_entity_discovered(user_id, ns, typ, id).await {
                    tracing::warn!("Entity discovery notification failed: {}", e);
                }
            });
        }

        sqlx::query("INSERT INTO events (id, user_id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(processed_event.id.to_string())
            .bind(user_id)
            .bind(processed_event.timestamp.to_rfc3339())
            .bind(&processed_event.category)
            .bind(&processed_event.source)
            .bind(serde_json::to_string(&processed_event.payload).map_err(|e| Error::BadRequest(e.to_string()))?)
            .bind(serde_json::to_string(processed_event.metadata.as_ref().unwrap()).unwrap())
            .bind(serde_json::to_string(&processed_event.entities).unwrap_or_else(|_| "[]".to_string()))
            .bind(&processed_event.display_title)
            .bind(&processed_event.display_subtitle)
            .execute(&self.db)
            .await?;

        // BROADCAST
        if let Some(ref sender) = self.event_sender {
            let _ = sender.send(processed_event.clone());
        }

        Ok(processed_event)
    }

    pub async fn list_events(&self, user_id: i64, category: Option<String>, limit: u32, offset: u32) -> Result<Vec<Event>> {
        let db_events = if let Some(cat) = category {
            sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle FROM events WHERE user_id = ? AND category = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?")
                .bind(user_id).bind(cat).bind(limit).bind(offset)
        } else {
            sqlx::query_as::<_, DbEvent>("SELECT id, timestamp, category, source, payload, metadata, entities, display_title, display_subtitle FROM events WHERE user_id = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?")
                .bind(user_id).bind(limit).bind(offset)
        }.fetch_all(&self.db).await?;

        Ok(db_events.into_iter().filter_map(|e| {
            match Event::try_from(e) {
                Ok(ev) => Some(ev),
                Err(err) => {
                    tracing::error!("Failed to convert DB event: {}", err);
                    None
                }
            }
        }).collect())
    }

    pub async fn poll_and_save_plugin(&self, user_id: i64, name: &str) -> Result<usize> {
        let events: Vec<Event> = self.plugin_manager.poll_plugin(user_id, name).await
            .map_err(|e| Error::Plugin(e))?;
        let count = events.len();
        
        for event in events {
            self.ingest_event(user_id, event).await?;
        }
        
        Ok(count)
    }

    pub async fn correlate_semantic(&self, user_id: i64, base_semantic: &str, join_semantic: &str, limit: u32) -> Result<Vec<Value>> {
        let manifests: HashMap<String, crate::plugins::scry::plugin::types::Manifest> = self.plugin_manager.get_plugin_manifests().await;
        let mut base_cat: Option<String> = None;
        let mut join_cat: Option<String> = None;

        for m in manifests.values() {
            for export in &m.exports {
                if export.semantic_type == base_semantic { base_cat = Some(export.category.clone()); }
                if export.semantic_type == join_semantic { join_cat = Some(export.category.clone()); }
            }
        }

        match (base_cat, join_cat) {
            (Some(bc), Some(jc)) => self.correlate_nearest(user_id, &bc, &jc, limit).await,
            _ => Err(Error::NotFound(format!("Semantic types {} or {} not found", base_semantic, join_semantic))),
        }
    }

    pub async fn search_semantic(&self, user_id: i64, semantic_query: &str, limit: u32, offset: u32) -> Result<Vec<Event>> {
        let manifests: HashMap<String, crate::plugins::scry::plugin::types::Manifest> = self.plugin_manager.get_plugin_manifests().await;
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
            all_events.extend(self.list_events(user_id, Some(cat), limit + offset, 0).await?);
        }

        all_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        let paged_events = all_events.into_iter().skip(offset as usize).take(limit as usize).collect::<Vec<_>>();
        Ok(paged_events)
    }

    pub async fn get_enriched_timeline(&self, user_id: i64, base_category: Option<String>, limit: u32, offset: u32) -> Result<Vec<Value>> {
        let manifests: HashMap<String, crate::plugins::scry::plugin::types::Manifest> = self.plugin_manager.get_plugin_manifests().await;
        let mut context_categories = std::collections::HashMap::new();

        for m in manifests.values() {
            for export in &m.exports {
                if base_category.as_ref() != Some(&export.category) {
                    context_categories.insert(export.semantic_type.clone(), export.category.clone());
                }
            }
        }

        let base_events = self.list_events(user_id, base_category, limit, offset).await?;
        let mut enriched_timeline = Vec::new();

        for ev in base_events {
            let ts = ev.timestamp.to_rfc3339();
            let mut entry = serde_json::json!({
                "id": ev.id,
                "timestamp": ts,
                "category": ev.category,
                "event": ev.payload,
                "metadata": ev.metadata,
                "entities": ev.entities,
                "display_title": ev.display_title,
                "display_subtitle": ev.display_subtitle,
                "context": {}
            });

            for (semantic_type, cat) in &context_categories {
                // Don't enrich an event with its own category context
                if &ev.category == cat { continue; }

                let context_payload = sqlx::query_scalar::<_, String>(
                    "SELECT payload FROM events WHERE user_id = ? AND category = ? AND timestamp <= ? ORDER BY timestamp DESC LIMIT 1"
                )
                .bind(user_id).bind(cat).bind(&ts).fetch_optional(&self.db).await?;

                if let Some(p) = context_payload {
                    match serde_json::from_str::<Value>(&p) {
                        Ok(json) => { entry["context"][semantic_type] = json; },
                        Err(e) => { tracing::warn!("Failed to parse context JSON for {}: {}", semantic_type, e); }
                    }
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
                CAST(j.payload AS TEXT),
                b.entities,
                b.display_title,
                b.display_subtitle
            FROM events b
            JOIN events j ON j.category = ? AND j.user_id = ?
            WHERE b.category = ? AND b.user_id = ?
            GROUP BY b.id
            HAVING MIN(ABS(julianday(substr(b.timestamp, 1, 19)) - julianday(substr(j.timestamp, 1, 19))))
            ORDER BY b.timestamp DESC
            LIMIT ?
        "#;

        let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>, Option<String>)>(sql)
            .bind(join_category).bind(user_id)
            .bind(base_category).bind(user_id)
            .bind(limit)
            .fetch_all(&self.db).await?;

        Ok(rows.into_iter().map(|(b, j, e, dt, ds)| {
            serde_json::json!({
                "base": serde_json::from_str::<Value>(&b).unwrap_or_default(),
                "joined": j.and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or_default(),
                "entities": e.and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or_default(),
                "display_title": dt,
                "display_subtitle": ds,
            })
        }).collect())
    }

    pub async fn resolve_semantic_info(&self, semantic_type: &str) -> Result<(String, String)> {
        let manifests: HashMap<String, crate::plugins::scry::plugin::types::Manifest> = self.plugin_manager.get_plugin_manifests().await;
        for m in manifests.values() {
            for export in &m.exports {
                if export.semantic_type == semantic_type {
                    let path = export.path.strip_prefix("payload.").unwrap_or(&export.path).to_string();
                    return Ok((export.category.clone(), path));
                }
            }
        }
        Err(Error::NotFound(format!("Semantic type {} not found in catalog", semantic_type)))
    }

    pub async fn get_semantic_top(&self, _user_id: i64, semantic_type: &str, limit: u32, days: Option<u32>) -> Result<Vec<Value>> {
        let (category, path) = self.resolve_semantic_info(semantic_type).await?;
        
        let mut sql = format!(
            "SELECT payload ->> '{}' as key, COUNT(*) as count FROM events WHERE category = ?",
            path
        );

        if days.is_some() {
            sql.push_str(" AND timestamp > date('now', ?)");
        }

        sql.push_str(" GROUP BY key ORDER BY count DESC LIMIT ?");

        let mut query = sqlx::query_as::<_, (Option<String>, i64)>(&sql)
            .bind(category);

        if let Some(d) = days {
            query = query.bind(format!("-{} days", d));
        }

        let rows = query.bind(limit).fetch_all(&self.db).await?;

        Ok(rows.into_iter().map(|(k, c)| {
            serde_json::json!({ "key": k.unwrap_or_else(|| "Unknown".to_string()), "count": c })
        }).collect())
    }

    pub async fn get_semantic_series(&self, _user_id: i64, semantic_type: &str, days: u32, interval: Option<String>) -> Result<Vec<Value>> {
        let (category, path) = self.resolve_semantic_info(semantic_type).await?;
        
        let format_str = match interval.as_deref() {
            Some("1h") => "%Y-%m-%dT%H:00:00Z",
            _ => "%Y-%m-%d",
        };

        let sql = format!(
            "SELECT strftime('{}', timestamp) as label, AVG(CAST(payload ->> '{}' as REAL)) as value FROM events WHERE category = ? AND timestamp > date('now', ?) GROUP BY label ORDER BY label ASC",
            format_str, path
        );

        let rows = sqlx::query_as::<_, (String, f64)>(&sql)
            .bind(category)
            .bind(format!("-{} days", days))
            .fetch_all(&self.db).await?;

        Ok(rows.into_iter().map(|(l, v)| {
            serde_json::json!({ "label": l, "value": v })
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
            sample_size, correlations: serde_json::to_value(distribution).map_err(|_e| Error::Internal)?,
        })
    }

    pub async fn generate_daily_summary(&self, user_id: i64, date: &str) -> Result<Vec<String>> {
        let manifests: HashMap<String, crate::plugins::scry::plugin::types::Manifest> = self.plugin_manager.get_plugin_manifests().await;
        let mut full_summary = Vec::new();
        let start = format!("{}T00:00:00Z", date);
        let end = format!("{}T23:59:59Z", date);

        for plugin_name in manifests.keys() {
            if let Ok(p_summary) = self.plugin_manager.get_plugin_summary(user_id, plugin_name, start.clone(), end.clone()).await
                && !p_summary.is_empty() { full_summary.push(p_summary); }
        }
        Ok(full_summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use scry_proto::Event;
    use std::sync::Arc;
    use crate::plugins::PluginManager;
    use chrono::Utc;
    use uuid::Uuid;
    use serde_json::json;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(r#"
            CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT, password_hash TEXT);
            CREATE TABLE events (
                id TEXT PRIMARY KEY, user_id INTEGER, timestamp TEXT, 
                category TEXT, source TEXT, payload TEXT, metadata TEXT
            );
            INSERT INTO users (id, username, password_hash) VALUES (1, 'alice', 'hash'), (2, 'bob', 'hash');
        "#).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_ingest_and_list_events() {
        let db = setup_test_db().await;
        // Wir nutzen einen leeren PluginManager für den Test
        let pm = Arc::new(PluginManager::new("./non_existent_plugins", db.clone()).unwrap());
        let service = EventService::new(db, pm);

        let event = Event {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category: "test.event".to_string(),
            source: "test-suite".to_string(),
            payload: json!({"temp": 22.5}),
            metadata: None,
        };

        // Ingest
        service.ingest_event(1, event.clone()).await.unwrap();

        // List
        let events = service.list_events(1, Some("test.event".to_string()), 10, 0).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].category, "test.event");
        assert_eq!(events[0].payload["temp"], 22.5);
    }

    #[tokio::test]
    async fn test_multi_tenancy_isolation() {
        let db = setup_test_db().await;
        let pm = Arc::new(PluginManager::new("./non_existent_plugins", db.clone()).unwrap());
        let service = EventService::new(db, pm);

        let event_alice = Event {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category: "private".to_string(),
            source: "alice-phone".to_string(),
            payload: json!({"secret": "alice_data"}),
            metadata: None,
        };

        service.ingest_event(1, event_alice).await.unwrap();

        // Bob (User 2) sollte Alices Event nicht sehen
        let bob_events = service.list_events(2, None, 10, 0).await.unwrap();
        assert_eq!(bob_events.len(), 0);

        // Alice (User 1) sollte ihr Event sehen
        let alice_events = service.list_events(1, None, 10, 0).await.unwrap();
        assert_eq!(alice_events.len(), 1);
    }
}
