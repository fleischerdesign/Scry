// Scry Plugin SDK - The official toolkit for building Scry plugins.
pub use scry_proto::{Event, EntityRef};
pub use serde_json;
pub use chrono;
pub use uuid;
pub use wit_bindgen;

pub mod prelude {
    pub use crate::{ScryPlugin, ReportMetadata, ReportData, Visualization, Manifest, DataField, EntityRef, scry_plugin};
    pub use crate::Event as SdkEvent;
    pub use serde_json::json;
}

pub trait ScryPlugin: Default {
    fn get_manifest(&self) -> Manifest;
    async fn on_init(&self) -> Result<(), String> { Ok(()) }
    async fn on_ingest(&self, event: Event) -> Result<Event, String> { Ok(event) }
    async fn get_reports(&self) -> Vec<ReportMetadata> { vec![] }
    async fn run_report(&self, _id: &str) -> Result<ReportData, String> { Err("Not implemented".to_string()) }
    async fn on_poll(&self) -> Vec<Event> { vec![] }
    async fn get_summary(&self, _start: &str, _end: &str) -> String { String::new() }

    // Semantic Trait Hooks
    async fn resolve_trait(&self, _namespace: &str, _typ: &str, _id: &str, _trait_id: &str) -> Result<Option<String>, String> { Ok(None) }
    async fn on_entity_discovered(&self, _namespace: &str, _typ: &str, _id: &str) {}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataField { pub category: String, pub path: String, pub semantic_type: String, pub description: String }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraitCapability { pub entity_namespace: String, pub entity_type: String, pub trait_id: String }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum WidgetTemplate { Metric, Trend, TopList, Status, Spotlight }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WidgetDefinition {
    pub id: String,
    pub title: String,
    pub template: WidgetTemplate,
    pub config_json: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub id: String, pub name: String, pub version: String, pub description: String,
    pub subscriptions: Vec<String>, pub capabilities: Vec<String>,
    pub exports: Vec<DataField>,
    pub provided_traits: Vec<TraitCapability>,
    pub poll_interval: Option<u32>,
    pub config_schema: Option<String>,
    pub suggested_widgets: Vec<WidgetDefinition>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Visualization { Table, BarChart, LineChart, PieChart }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportMetadata {
    pub id: String, pub name: String, pub description: String, pub viz: Visualization,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportData { pub columns: Vec<String>, pub data_json: String }

#[macro_export]
macro_rules! scry_plugin {
    ($plugin_type:ty) => {
        $crate::wit_bindgen::generate!({
            world: "plugin",
            path: "../../crates/scry-proto/wit",
            additional_derives: [serde::Serialize, serde::Deserialize],
            async: true,
        });

        struct GuestImpl;

        impl Guest for GuestImpl {
            async fn get_manifest() -> scry::plugin::types::Manifest {
                let m = <$plugin_type>::default().get_manifest();
                scry::plugin::types::Manifest {
                    id: m.id, name: m.name, version: m.version, description: m.description,
                    subscriptions: m.subscriptions, capabilities: m.capabilities,
                    exports: m.exports.into_iter().map(|e| scry::plugin::types::DataField {
                        category: e.category, path: e.path, semantic_type: e.semantic_type, description: e.description
                    }).collect(),
                    provided_traits: m.provided_traits.into_iter().map(|t| scry::plugin::types::TraitCapability {
                        entity_namespace: t.entity_namespace, entity_type: t.entity_type, trait_id: t.trait_id
                    }).collect(),
                    poll_interval: m.poll_interval,
                    config_schema: m.config_schema,
                    suggested_widgets: m.suggested_widgets.into_iter().map(|w| scry::plugin::types::WidgetDefinition {
                        id: w.id, title: w.title, config_json: w.config_json,
                        template: match w.template {
                            $crate::WidgetTemplate::Metric => scry::plugin::types::WidgetTemplate::Metric,
                            $crate::WidgetTemplate::Trend => scry::plugin::types::WidgetTemplate::Trend,
                            $crate::WidgetTemplate::TopList => scry::plugin::types::WidgetTemplate::TopList,
                            $crate::WidgetTemplate::Status => scry::plugin::types::WidgetTemplate::Status,
                            $crate::WidgetTemplate::Spotlight => scry::plugin::types::WidgetTemplate::Spotlight,
                        }
                    }).collect(),
                }
            }

            async fn on_init() -> Result<(), String> {
                let plugin = <$plugin_type>::default();
                plugin.on_init().await
            }

            async fn on_ingest(ev: scry::plugin::types::Event) -> Result<scry::plugin::types::Event, String> {
                let plugin = <$plugin_type>::default();
                let sdk_ev = $crate::Event {
                    id: $crate::uuid::Uuid::parse_str(&ev.id).map_err(|e| e.to_string())?,
                    timestamp: $crate::chrono::DateTime::parse_from_rfc3339(&ev.timestamp).map_err(|e| e.to_string())?.with_timezone(&$crate::chrono::Utc),
                    category: ev.category, source: ev.source,
                    payload: $crate::serde_json::from_str(&ev.payload).map_err(|e| e.to_string())?,
                    metadata: ev.metadata.and_then(|m| $crate::serde_json::from_str(&m).ok()),
                    entities: ev.entities.into_iter().map(|e| $crate::prelude::EntityRef {
                        path: e.path, namespace: e.namespace, typ: e.typ, id: e.id
                    }).collect(),
                };
                match plugin.on_ingest(sdk_ev).await {
                    Ok(res) => Ok(scry::plugin::types::Event {
                        id: res.id.to_string(), timestamp: res.timestamp.to_rfc3339(),
                        category: res.category, source: res.source,
                        payload: res.payload.to_string(), metadata: res.metadata.as_ref().map(|m| m.to_string()),
                        entities: res.entities.into_iter().map(|e| scry::plugin::types::EntityRef {
                            path: e.path, namespace: e.namespace, typ: e.typ, id: e.id
                        }).collect(),
                    }),
                    Err(e) => Err(e),
                }
            }

            async fn get_reports() -> Vec<scry::plugin::types::ReportMetadata> {
                let plugin = <$plugin_type>::default();
                plugin.get_reports().await.into_iter().map(|m| scry::plugin::types::ReportMetadata {
                    id: m.id, name: m.name, description: m.description,
                    viz: match m.viz {
                        $crate::Visualization::Table => scry::plugin::types::Visualization::Table,
                        $crate::Visualization::BarChart => scry::plugin::types::Visualization::BarChart,
                        $crate::Visualization::LineChart => scry::plugin::types::Visualization::LineChart,
                        $crate::Visualization::PieChart => scry::plugin::types::Visualization::PieChart,
                    }
                }).collect()
            }

            async fn run_report(id: String) -> Result<scry::plugin::types::ReportData, String> {
                let plugin = <$plugin_type>::default();
                match plugin.run_report(&id).await {
                    Ok(res) => Ok(scry::plugin::types::ReportData { columns: res.columns, data_json: res.data_json }),
                    Err(e) => Err(e),
                }
            }

            async fn on_poll() -> Vec<scry::plugin::types::Event> {
                let plugin = <$plugin_type>::default();
                plugin.on_poll().await.into_iter().map(|result| {
                    scry::plugin::types::Event {
                        id: result.id.to_string(), timestamp: result.timestamp.to_rfc3339(),
                        category: result.category, source: result.source,
                        payload: result.payload.to_string(), metadata: result.metadata.as_ref().map(|m| m.to_string()),
                        entities: result.entities.into_iter().map(|e| scry::plugin::types::EntityRef {
                            path: e.path, namespace: e.namespace, typ: e.typ, id: e.id
                        }).collect(),
                    }
                }).collect()
            }

            async fn get_summary(start: String, end: String) -> String {
                let plugin = <$plugin_type>::default();
                plugin.get_summary(&start, &end).await
            }

            async fn resolve_trait(namespace: String, typ: String, id: String, trait_id: String) -> Result<Option<String>, String> {
                let plugin = <$plugin_type>::default();
                plugin.resolve_trait(&namespace, &typ, &id, &trait_id).await
            }

            async fn on_entity_discovered(namespace: String, typ: String, id: String) {
                let plugin = <$plugin_type>::default();
                plugin.on_entity_discovered(&namespace, &typ, &id).await
            }
        }

        export!(GuestImpl);

        mod host {
            use $crate::serde_json::json;
            use super::scry::plugin::host::QueryParam;

            pub async fn query(sql: &str, params: Vec<QueryParam>) -> Vec<$crate::serde_json::Value> {
                match super::scry::plugin::host::query(sql.to_string(), params).await {
                    Ok(res) => $crate::serde_json::from_str(&res).unwrap_or_default(),
                    Err(e) => {
                        super::scry::plugin::host::log("error".to_string(), format!("SDK: Query failed: {}", e)).await;
                        vec![]
                    }
                }
            }

            pub async fn count_grouped(category: &str, payload_key: &str, limit: u32) -> Vec<$crate::serde_json::Value> {
                let sql = format!("SELECT payload ->> '{}' as key, COUNT(*) as count FROM events WHERE category = ? GROUP BY key ORDER BY count DESC LIMIT ?", payload_key);
                query(&sql, vec![
                    QueryParam::S(category.to_string()),
                    QueryParam::I(limit as i64),
                ]).await
            }

            pub async fn count_over_time(category: &str, interval: &str, days: u32) -> Vec<$crate::serde_json::Value> {
                let format = match interval { "1h" => "%Y-%m-%dT%H:00:00Z", _ => "%Y-%m-%d" };
                let sql = "SELECT strftime(?, timestamp) as label, COUNT(*) as count FROM events WHERE category = ? AND timestamp > date('now', ?) GROUP BY label ORDER BY label ASC";
                query(sql, vec![
                    QueryParam::S(format.to_string()),
                    QueryParam::S(category.to_string()),
                    QueryParam::S(format!("-{} days", days)),
                ]).await
            }

            pub async fn join_nearest(base_category: &str, join_category: &str, limit: u32) -> Vec<$crate::serde_json::Value> {
                let sql = r#"
                    SELECT 
                        CAST(b.payload AS TEXT) as base,
                        CAST(j.payload AS TEXT) as joined
                    FROM events b
                    JOIN events j ON j.category = ?
                    WHERE b.category = ?
                    GROUP BY b.id
                    HAVING MIN(ABS(julianday(substr(b.timestamp, 1, 19)) - julianday(substr(j.timestamp, 1, 19))))
                    ORDER BY b.timestamp DESC
                    LIMIT ?
                "#;
                query(sql, vec![
                    QueryParam::S(join_category.to_string()),
                    QueryParam::S(base_category.to_string()),
                    QueryParam::I(limit as i64),
                ]).await
            }
            pub async fn log_info(msg: &str) { super::scry::plugin::host::log("info".to_string(), msg.to_string()).await; }
            pub async fn log_warn(msg: &str) { super::scry::plugin::host::log("warn".to_string(), msg.to_string()).await; }
            pub async fn log_error(msg: &str) { super::scry::plugin::host::log("error".to_string(), msg.to_string()).await; }
            pub async fn get_state(key: &str) -> Option<String> { super::scry::plugin::host::get_state(key.to_string()).await }
            pub async fn set_state(key: &str, val: &str) { super::scry::plugin::host::set_state(key.to_string(), val.to_string()).await }
            pub async fn get_config(key: &str) -> Option<String> { super::scry::plugin::host::get_config(key.to_string()).await }
            pub async fn get_profile_value(key: &str) -> Option<String> { super::scry::plugin::host::get_profile_value(key.to_string()).await }
            pub async fn http_get(url: &str) -> ::std::result::Result<String, String> { 
                match super::scry::plugin::host::http_get(url.to_string()).await {
                    Ok(res) => Ok(res),
                    Err(e) => {
                        super::scry::plugin::host::log("error".to_string(), format!("SDK: HTTP GET failed: {}", e)).await;
                        Err(e)
                    }
                }
            }
            pub async fn set_entity_trait(namespace: &str, typ: &str, id: &str, trait_id: &str, value_json: &str) {
                super::scry::plugin::host::set_entity_trait(namespace.to_string(), typ.to_string(), id.to_string(), trait_id.to_string(), value_json.to_string()).await;
            }
            pub async fn get_entity_trait(namespace: &str, typ: &str, id: &str, trait_id: &str) -> Option<String> {
                super::scry::plugin::host::get_entity_trait(namespace.to_string(), typ.to_string(), id.to_string(), trait_id.to_string()).await
            }
        }
    };
}
