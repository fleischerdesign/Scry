use sqlx::SqlitePool;
use std::sync::Arc;
use serde_json::json;
use crate::domain::*;
use crate::error::Result;
use crate::repository::ConfigRepository;
use crate::plugins::PluginManager;
use super::EventService;
use super::SecretService;

#[derive(Clone)]
pub struct PluginService {
    db: SqlitePool,
    plugin_manager: Arc<PluginManager>,
    event_service: EventService,
    secret_service: SecretService,
}

impl PluginService {
    pub fn new(db: SqlitePool, plugin_manager: Arc<PluginManager>, event_service: EventService, secret_service: SecretService) -> Self {
        Self { db, plugin_manager, event_service, secret_service }
    }

    pub async fn poll_plugin_manually(&self, user_id: i64, plugin_id: &str) -> Result<usize> {
        self.event_service.poll_and_save_plugin(user_id, plugin_id).await
    }

    pub async fn run_plugin_report(&self, user_id: i64, plugin_id: &str, report_id: String) -> Result<ApiReportData> {
        let data = self.plugin_manager.run_plugin_report(user_id, plugin_id, report_id).await?;
        Ok(data.into())
    }

    pub async fn get_plugin_config(&self, user_id: i64, plugin_id: &str) -> Result<serde_json::Value> {
        let repo = ConfigRepository::new(&self.db, user_id, &self.secret_service);
        let rows = repo.get_all_by_plugin(plugin_id).await?;
        
        let mut map = serde_json::Map::new();
        for (k, v, is_secret) in rows {
            if !is_secret {
                map.insert(k, json!(v));
            }
        }
        Ok(serde_json::Value::Object(map))
    }

    pub async fn get_plugin_secrets(&self, user_id: i64, plugin_id: &str) -> Result<serde_json::Value> {
        let repo = ConfigRepository::new(&self.db, user_id, &self.secret_service);
        let rows = repo.get_secrets_by_plugin(plugin_id).await?;
        
        let mut map = serde_json::Map::new();
        for (k, v) in rows {
            map.insert(k, json!(v));
        }
        Ok(serde_json::Value::Object(map))
    }

    pub async fn update_plugin_config(&self, user_id: i64, plugin_id: &str, config: serde_json::Map<String, serde_json::Value>) -> Result<()> {
        let schema = self.get_config_schema(plugin_id).await;
        let secret_keys = self.extract_secret_keys(&schema);
        
        let repo = ConfigRepository::new(&self.db, user_id, &self.secret_service);
        for (k, v) in config {
            let v_str = v.as_str().unwrap_or("").to_string();
            let is_secret = secret_keys.contains(&k);
            repo.set(plugin_id, &k, &v_str, is_secret).await?;
        }
        Ok(())
    }

    async fn get_config_schema(&self, plugin_id: &str) -> Option<serde_json::Value> {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        manifests.get(plugin_id).and_then(|m| m.config_schema.as_ref()).and_then(|s| serde_json::from_str(s).ok())
    }

    fn extract_secret_keys(&self, schema: &Option<serde_json::Value>) -> std::collections::HashSet<String> {
        let mut keys = std::collections::HashSet::new();
        if let Some(s) = schema
            && let Some(props) = s.get("properties").and_then(|p| p.as_object()) {
                for (k, prop) in props {
                    if prop.get("secret").and_then(|v| v.as_bool()).unwrap_or(false) {
                        keys.insert(k.clone());
                    }
                }
            }
        keys
    }

    pub async fn get_catalog(&self) -> serde_json::Value {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        let mut catalog = serde_json::Map::new();
        for (plugin_name, manifest) in manifests {
            for export in manifest.exports {
                let entry = json!({ 
                    "plugin": plugin_name, 
                    "path": export.path, 
                    "description": export.description, 
                    "category": export.category,
                    "icon": export.icon,
                    "unit": export.unit,
                    "privacy": export.privacy,
                    "confidence": export.confidence,
                    "temporal": export.temporal,
                });
                let array = catalog.entry(export.semantic_type).or_insert_with(|| json!([]));
                if let Some(arr) = array.as_array_mut() {
                    arr.push(entry);
                }
            }
        }
        serde_json::Value::Object(catalog)
    }

    pub async fn get_system_plugins(&self, user_id: i64) -> Result<Vec<PluginStatus>> {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        let mut statuses = Vec::new();

        // 1. Add Virtual Core Plugin (Using SDK structure for consistency)
        let core_manifest = scry_plugin_sdk::Manifest {
            id: "core".to_string(),
            name: "Scry System".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Core system services and metrics.".to_string(),
            subscriptions: vec![],
            capabilities: vec!["system".to_string()],
            exports: vec![],
            domain_info: vec![],
            predicates: vec![],
            provided_traits: vec![],
            poll_interval: None,
            config_schema: None,
            suggested_widgets: vec![
                scry_plugin_sdk::WidgetDefinition {
                    id: "system-status".to_string(),
                    title: "System Node Status".to_string(),
                    template: scry_plugin_sdk::WidgetTemplate::Status,
                    config_json: json!({ "scope": "all" }).to_string(),
                },
                scry_plugin_sdk::WidgetDefinition {
                    id: "knowledge-growth".to_string(),
                    title: "Knowledge Growth".to_string(),
                    template: scry_plugin_sdk::WidgetTemplate::Trend,
                    config_json: json!({ "semantic_type": "system.entities", "days": 7 }).to_string(),
                }
            ],
            oauth_config: None,
        };

        let mut core_status = PluginStatus::from_sdk("core".to_string(), core_manifest, vec![]);
        core_status.roles = vec!["system".to_string()];
        statuses.push(core_status);

        // 2. Add real plugins
        for (id, m) in manifests {
            let reports_list = self.plugin_manager.list_plugin_reports(user_id).await?;
            let p_reports = reports_list.into_iter().find(|(p_id, _)| p_id == &id).map(|(_, r)| r).unwrap_or_default();
            
            let mut roles = Vec::new();
            if m.poll_interval.is_some() { roles.push("SOURCE".to_string()); }
            if !m.subscriptions.is_empty() { roles.push("ENRICHER".to_string()); }
            if !m.provided_traits.is_empty() { roles.push("RESOLVER".to_string()); }
            if !m.suggested_widgets.is_empty() { roles.push("VISUALIZER".to_string()); }
            if !p_reports.is_empty() { roles.push("ANALYZER".to_string()); }

            let mut status = PluginStatus::from_sdk(id.clone(), m, p_reports);
            status.roles = roles;
            statuses.push(status);
        }
        Ok(statuses)
    }

    pub async fn get_oauth_config(&self, plugin_id: &str) -> Result<scry_plugin_sdk::OAuthConfig> {
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        let manifest = manifests.get(plugin_id).ok_or_else(|| crate::error::Error::NotFound("Plugin not found".into()))?;
        
        manifest.oauth_config.clone().ok_or_else(|| crate::error::Error::NotFound("OAuth not configured".into()))
    }

    pub async fn get_oauth_credentials(&self, user_id: i64, plugin_id: &str) -> Result<(String, String)> {
        let repo = ConfigRepository::new(&self.db, user_id, &self.secret_service);
        
        let client_id = repo.get_secret(plugin_id, "client_id").await?
            .ok_or_else(|| crate::error::Error::NotFound("client_id not set".into()))?;
        
        let client_secret = repo.get_secret(plugin_id, "client_secret").await?
            .ok_or_else(|| crate::error::Error::NotFound("client_secret not set".into()))?;
        
        Ok((client_id, client_secret))
    }
}
