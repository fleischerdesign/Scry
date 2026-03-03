use std::sync::Arc;
use std::collections::HashMap;
use crate::plugins::PluginManager;
use scry_proto::Event;
use crate::domain::*;
use crate::error::{Error, Result};
use crate::repository::{EventRepository, ConfigRepository, EntityRepository, AnalyticsRepository};
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

    pub async fn enrich_event_context(&self, user_id: i64, event: &mut Event) -> Result<()> {
        let event_repo = EventRepository::new(&self.db, user_id);
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        let mut context_categories = HashMap::new();

        for m in manifests.values() {
            for export in &m.exports {
                if event.category != export.category {
                    context_categories.insert(export.semantic_type.clone(), export.category.clone());
                }
            }
        }

        let mut info = serde_json::Map::new();
        let ts = event.timestamp.to_rfc3339();

        for (semantic_type, cat) in &context_categories {
            if let Some(p) = event_repo.get_last_payload(cat, &ts).await? {
                if let Ok(json) = serde_json::from_str::<Value>(&p) {
                    info.insert(semantic_type.clone(), json);
                }
            }
        }

        if !info.is_empty() {
            event.context_info = Some(Value::Object(info));
        }
        Ok(())
    }

    pub async fn ingest_event(&self, user_id: i64, event: Event) -> Result<Event> {
        let event_repo = EventRepository::new(&self.db, user_id);
        let config_repo = ConfigRepository::new(&self.db, user_id);
        let entity_repo = EntityRepository::new(&self.db, user_id);

        let mut processed_event: Event = self.plugin_manager.run_ingest_pipeline(user_id, event).await
            .map_err(|e| Error::Plugin(e))?;
        
        // --- Dynamic Context Resolution ---
        let hints = processed_event.context.clone();
        let processor_id = processed_event.metadata.as_ref()
            .and_then(|m| m.get("processor"))
            .and_then(|p| p.as_str())
            .unwrap_or_else(|| processed_event.source.split('+').next().unwrap_or(&processed_event.source));

        for hint in hints {
            let entity_ref = if hint.starts_with("alias:") {
                let alias_key = hint.clone();
                if let Some(target_uri) = config_repo.get(processor_id, &alias_key).await? {
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
                    let alias_name = hint.strip_prefix("alias:").unwrap();
                    let default_target = match alias_name {
                        "self" | "owner" | "subject" | "listener" => Some("scry.core/user/self"),
                        _ => None
                    };

                    if let Some(target) = default_target {
                        let _ = config_repo.set_if_not_exists(processor_id, &alias_key, target).await;
                        Some(scry_proto::EntityRef {
                            path: hint.clone(),
                            namespace: "scry.core".to_string(),
                            typ: "user".to_string(),
                            id: "self".to_string(),
                        })
                    } else { None }
                }
            } else {
                let parts: Vec<&str> = hint.split('/').collect();
                if parts.len() == 3 {
                    Some(scry_proto::EntityRef {
                        path: hint.clone(),
                        namespace: parts[0].to_string(),
                        typ: parts[1].to_string(),
                        id: parts[2].to_string(),
                    })
                } else { None }
            };

            if let Some(r) = entity_ref {
                if !processed_event.entities.iter().any(|e| e.namespace == r.namespace && e.typ == r.typ && e.id == r.id) {
                    processed_event.entities.push(r);
                }
            }
        }

        let _ = self.enrich_event_context(user_id, &mut processed_event).await;

        let mut meta = processed_event.metadata.unwrap_or_else(|| serde_json::json!({}));
        meta["user_id"] = serde_json::json!(user_id);
        processed_event.metadata = Some(meta);

        for ent in &processed_event.entities {
            entity_repo.ensure_entity(&ent.namespace, &ent.typ, &ent.id).await?;
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

        event_repo.insert(&processed_event).await?;

        if let Some(ref sender) = self.event_sender {
            let _ = sender.send(processed_event.clone());
        }

        Ok(processed_event)
    }

    pub async fn list_events(&self, user_id: i64, category: Option<String>, limit: u32, offset: u32) -> Result<Vec<Event>> {
        let event_repo = EventRepository::new(&self.db, user_id);
        event_repo.list(category, limit, offset).await
    }

    pub async fn get_event_by_id(&self, user_id: i64, id: &str) -> Result<Option<Event>> {
        let event_repo = EventRepository::new(&self.db, user_id);
        event_repo.get_by_id(id).await
    }

    pub async fn get_events_by_entity(&self, user_id: i64, namespace: &str, typ: &str, id: &str) -> Result<Vec<Event>> {
        let event_repo = EventRepository::new(&self.db, user_id);
        event_repo.get_by_entity(namespace, typ, id).await
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
            (Some(bc), Some(jc)) => {
                let repo = AnalyticsRepository::new(&self.db, user_id);
                repo.correlate_nearest(&bc, &jc, limit).await
            },
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
        let base_events = self.list_events(user_id, base_category, limit, offset).await?;
        let mut enriched_timeline = Vec::new();

        for mut ev in base_events {
            let _ = self.enrich_event_context(user_id, &mut ev).await;

            let entry = serde_json::json!({
                "id": ev.id,
                "timestamp": ev.timestamp.to_rfc3339(),
                "category": ev.category,
                "event": ev.payload,
                "metadata": ev.metadata,
                "entities": ev.entities,
                "context": ev.context,
                "context_info": ev.context_info,
                "display_title": ev.display_title,
                "display_subtitle": ev.display_subtitle,
            });
            enriched_timeline.push(entry);
        }

        Ok(enriched_timeline)
    }

    pub async fn correlate_nearest(&self, user_id: i64, base_category: &str, join_category: &str, limit: u32) -> Result<Vec<Value>> {
        let repo = AnalyticsRepository::new(&self.db, user_id);
        repo.correlate_nearest(base_category, join_category, limit).await
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

    pub async fn get_semantic_top(&self, user_id: i64, semantic_type: &str, limit: u32, days: Option<u32>) -> Result<Vec<Value>> {
        let (category, path) = self.resolve_semantic_info(semantic_type).await?;
        let repo = AnalyticsRepository::new(&self.db, user_id);
        repo.get_semantic_top(&category, &path, limit, days).await
    }

    pub async fn get_semantic_series(&self, user_id: i64, semantic_type: &str, days: u32, interval: Option<String>) -> Result<Vec<Value>> {
        let (category, path) = self.resolve_semantic_info(semantic_type).await?;
        let repo = AnalyticsRepository::new(&self.db, user_id);
        repo.get_semantic_series(&category, &path, days, interval).await
    }

    pub async fn calculate_semantic_stats(&self, user_id: i64, base_semantic: &str, join_semantic: &str, limit: u32) -> Result<SemanticStats> {
        let correlations = self.correlate_semantic(user_id, base_semantic, join_semantic, limit).await?;
        let sample_size = correlations.len();
        let mut distribution = std::collections::HashMap::new();

        for corr in &correlations {
            let base_val = corr.get("base").unwrap_or(&Value::Null).to_string();
            let join_val = corr.get("joined").unwrap_or(&Value::Null).to_string();
            let entry = distribution.entry(base_val).or_insert_with(std::collections::HashMap::new);
            *entry.entry(join_val).or_insert(0) += 1;
        }

        Ok(SemanticStats {
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
