use sqlx::SqlitePool;
use std::sync::Arc;
use serde_json::json;
use crate::models::*;
use crate::error::Result;
use crate::repository::ConfigRepository;
use crate::plugins::PluginManager;
use crate::event_service::EventService;

#[derive(Clone)]
pub struct PluginService {
    db: SqlitePool,
    plugin_manager: Arc<PluginManager>,
    event_service: EventService,
}

impl PluginService {
    pub fn new(db: SqlitePool, plugin_manager: Arc<PluginManager>, event_service: EventService) -> Self {
        Self { db, plugin_manager, event_service }
    }

    pub async fn poll_plugin_manually(&self, user_id: i64, plugin_id: &str) -> Result<usize> {
        self.event_service.poll_and_save_plugin(user_id, plugin_id).await
    }

    pub async fn run_plugin_report(&self, user_id: i64, plugin_id: &str, report_id: String) -> Result<ApiReportData> {
        let data = self.plugin_manager.run_plugin_report(user_id, plugin_id, report_id).await?;
        Ok(ApiReportData {
            columns: data.columns,
            data_json: data.data_json,
        })
    }

    pub async fn get_plugin_config(&self, user_id: i64, plugin_id: &str) -> Result<serde_json::Value> {
        let repo = ConfigRepository::new(&self.db, user_id);
        let rows = repo.get_all_by_plugin(plugin_id).await?;
        
        let mut map = serde_json::Map::new();
        for (k, v) in rows {
            map.insert(k, json!(v));
        }
        Ok(serde_json::Value::Object(map))
    }

    pub async fn update_plugin_config(&self, user_id: i64, plugin_id: &str, config: serde_json::Map<String, serde_json::Value>) -> Result<()> {
        let repo = ConfigRepository::new(&self.db, user_id);
        for (k, v) in config {
            let v_str = v.as_str().unwrap_or("").to_string();
            repo.set(plugin_id, &k, &v_str).await?;
        }
        Ok(())
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
                    "category": export.category 
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

        // 1. Add Virtual Core Plugin
        statuses.push(PluginStatus {
            id: "core".to_string(),
            name: "Scry System".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Core system services and metrics.".to_string(),
            roles: vec!["system".to_string()],
            capabilities: vec!["system".to_string()],
            subscriptions: vec![],
            exports: vec![],
            provided_traits: vec![],
            reports: vec![],
            config_schema: None,
            suggested_widgets: vec![
                ApiWidgetDefinition {
                    id: "system-status".to_string(),
                    title: "System Node Status".to_string(),
                    template: ApiWidgetTemplate::Status,
                    config_json: json!({ "scope": "all" }).to_string(),
                },
                ApiWidgetDefinition {
                    id: "knowledge-growth".to_string(),
                    title: "Knowledge Growth".to_string(),
                    template: ApiWidgetTemplate::Trend,
                    config_json: json!({ "semantic_type": "system.entities", "days": 7 }).to_string(),
                }
            ],
        });

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

            statuses.push(PluginStatus {
                id: id.clone(),
                name: m.name,
                version: m.version,
                description: m.description,
                roles,
                capabilities: m.capabilities,
                subscriptions: m.subscriptions,
                exports: m.exports.into_iter().map(|e| ApiDataField {
                    category: e.category,
                    path: e.path,
                    semantic_type: e.semantic_type,
                    description: e.description,
                    icon: e.icon,
                }).collect(),
                provided_traits: m.provided_traits.into_iter().map(|t| ApiTraitCapability {
                    entity_namespace: t.entity_namespace,
                    entity_type: t.entity_type,
                    trait_id: t.trait_id,
                }).collect(),
                reports: p_reports.into_iter().map(|r| ApiReportMetadata {
                    id: r.id, name: r.name, description: r.description, viz: format!("{:?}", r.viz),
                }).collect(),
                config_schema: m.config_schema,
                suggested_widgets: m.suggested_widgets.into_iter().map(|w| ApiWidgetDefinition {
                    id: w.id, title: w.title, config_json: w.config_json,
                    template: match w.template {
                        crate::plugins::scry::plugin::types::WidgetTemplate::Metric => ApiWidgetTemplate::Metric,
                        crate::plugins::scry::plugin::types::WidgetTemplate::Trend => ApiWidgetTemplate::Trend,
                        crate::plugins::scry::plugin::types::WidgetTemplate::TopList => ApiWidgetTemplate::TopList,
                        crate::plugins::scry::plugin::types::WidgetTemplate::Status => ApiWidgetTemplate::Status,
                        crate::plugins::scry::plugin::types::WidgetTemplate::Spotlight => ApiWidgetTemplate::Spotlight,
                    }
                }).collect(),
            });
        }
        Ok(statuses)
    }
}
