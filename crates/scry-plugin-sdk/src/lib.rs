// Scry Plugin SDK - The official toolkit for building Scry plugins.
pub use scry_proto::Event;
pub use serde_json;
pub use chrono;
pub use uuid;
pub use wit_bindgen;

pub mod prelude {
    pub use crate::{ScryPlugin, ReportMetadata, ReportData, Visualization, Manifest, DataField, scry_plugin};
    pub use crate::Event as SdkEvent;
    pub use serde_json::json;
}

pub trait ScryPlugin: Default {
    fn get_manifest(&self) -> Manifest;
    fn on_ingest(&self, event: Event) -> Result<Event, String> { Ok(event) }
    fn get_reports(&self) -> Vec<ReportMetadata> { vec![] }
    fn run_report(&self, _id: &str) -> Result<ReportData, String> { Err("Not implemented".to_string()) }
    fn on_poll(&self) -> Vec<Event> { vec![] }
    fn get_summary(&self, _start: &str, _end: &str) -> String { String::new() }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataField { pub category: String, pub path: String, pub semantic_type: String, pub description: String }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub id: String, pub name: String, pub version: String, pub description: String,
    pub subscriptions: Vec<String>, pub capabilities: Vec<String>,
    pub exports: Vec<DataField>,
    pub poll_interval: Option<u32>,
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
        });

        struct GuestImpl;

        impl Guest for GuestImpl {
            fn get_manifest() -> scry::plugin::types::Manifest {
                let m = <$plugin_type>::default().get_manifest();
                scry::plugin::types::Manifest {
                    id: m.id, name: m.name, version: m.version, description: m.description,
                    subscriptions: m.subscriptions, capabilities: m.capabilities,
                    exports: m.exports.into_iter().map(|e| scry::plugin::types::DataField {
                        category: e.category, path: e.path, semantic_type: e.semantic_type, description: e.description
                    }).collect(),
                    poll_interval: m.poll_interval,
                }
            }

            fn on_ingest(ev: scry::plugin::types::Event) -> ::std::result::Result<scry::plugin::types::Event, ::std::string::String> {
                let plugin = <$plugin_type>::default();
                let sdk_ev = $crate::Event {
                    id: $crate::uuid::Uuid::parse_str(&ev.id).map_err(|e| e.to_string())?,
                    timestamp: $crate::chrono::DateTime::parse_from_rfc3339(&ev.timestamp).map_err(|e| e.to_string())?.with_timezone(&$crate::chrono::Utc),
                    category: ev.category, source: ev.source,
                    payload: $crate::serde_json::from_str(&ev.payload).map_err(|e| e.to_string())?,
                    metadata: ev.metadata.and_then(|m| $crate::serde_json::from_str(&m).ok()),
                };
                match plugin.on_ingest(sdk_ev) {
                    Ok(res) => Ok(scry::plugin::types::Event {
                        id: res.id.to_string(), timestamp: res.timestamp.to_rfc3339(),
                        category: res.category, source: res.source,
                        payload: res.payload.to_string(), metadata: res.metadata.as_ref().map(|m| m.to_string()),
                    }),
                    Err(e) => Err(e),
                }
            }

            fn get_reports() -> Vec<scry::plugin::types::ReportMetadata> {
                let plugin = <$plugin_type>::default();
                plugin.get_reports().into_iter().map(|m| scry::plugin::types::ReportMetadata {
                    id: m.id, name: m.name, description: m.description,
                    viz: match m.viz {
                        $crate::Visualization::Table => scry::plugin::types::Visualization::Table,
                        $crate::Visualization::BarChart => scry::plugin::types::Visualization::BarChart,
                        $crate::Visualization::LineChart => scry::plugin::types::Visualization::LineChart,
                        $crate::Visualization::PieChart => scry::plugin::types::Visualization::PieChart,
                    }
                }).collect()
            }

            fn run_report(id: String) -> ::std::result::Result<scry::plugin::types::ReportData, ::std::string::String> {
                let plugin = <$plugin_type>::default();
                match plugin.run_report(&id) {
                    Ok(res) => Ok(scry::plugin::types::ReportData { columns: res.columns, data_json: res.data_json }),
                    Err(e) => Err(e),
                }
            }

            fn on_poll() -> Vec<scry::plugin::types::Event> {
                let plugin = <$plugin_type>::default();
                plugin.on_poll().into_iter().map(|result| {
                    scry::plugin::types::Event {
                        id: result.id.to_string(), timestamp: result.timestamp.to_rfc3339(),
                        category: result.category, source: result.source,
                        payload: result.payload.to_string(), metadata: result.metadata.as_ref().map(|m| m.to_string()),
                    }
                }).collect()
            }

            fn get_summary(start: String, end: String) -> String {
                let plugin = <$plugin_type>::default();
                plugin.get_summary(&start, &end)
            }
        }

        export!(GuestImpl);

        mod host {
            use $crate::serde_json::json;
            pub fn count_grouped(category: &str, payload_key: &str, limit: u32) -> Vec<$crate::serde_json::Value> {
                let q = json!({"type": "count_grouped", "category": category, "payload_key": payload_key, "limit": limit});
                match super::scry::plugin::host::query(&q.to_string()) {
                    Ok(res) => $crate::serde_json::from_str(&res).unwrap_or_default(),
                    Err(_) => vec![]
                }
            }
            pub fn count_over_time(category: &str, interval: &str, days: u32) -> Vec<$crate::serde_json::Value> {
                let q = json!({"type": "count_over_time", "category": category, "interval": interval, "days": days});
                match super::scry::plugin::host::query(&q.to_string()) {
                    Ok(res) => $crate::serde_json::from_str(&res).unwrap_or_default(),
                    _ => vec![]
                }
            }
            pub fn join_nearest(base_category: &str, join_category: &str, limit: u32) -> Vec<$crate::serde_json::Value> {
                let q = json!({"type": "join_nearest", "base_category": base_category, "join_category": join_category, "limit": limit});
                match super::scry::plugin::host::query(&q.to_string()) {
                    Ok(res) => $crate::serde_json::from_str(&res).unwrap_or_default(),
                    _ => vec![]
                }
            }
            pub fn log_info(msg: &str) { super::scry::plugin::host::log("info", msg); }
            pub fn log_warn(msg: &str) { super::scry::plugin::host::log("warn", msg); }
            pub fn log_error(msg: &str) { super::scry::plugin::host::log("error", msg); }
            pub fn get_state(key: &str) -> Option<String> { super::scry::plugin::host::get_state(key) }
            pub fn set_state(key: &str, val: &str) { super::scry::plugin::host::set_state(key, val) }
            pub fn get_config(key: &str) -> Option<String> { super::scry::plugin::host::get_config(key) }
            pub fn http_get(url: &str) -> ::std::result::Result<String, String> { 
                super::scry::plugin::host::http_get(url)
            }
        }
    };
}
