use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use serde_json::json;
use std::sync::Arc;

use crate::models::*;
use crate::error::Result;

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/reports/{report_id}", responses((status = 200, body = ApiReportData)), security(("api_key" = [])))]
pub async fn run_plugin_report(
    State(state): State<Arc<AppState>>,
    Path((id, report_id)): Path<(String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<ApiReportData>> {
    let data = state.event_service.plugin_manager().run_plugin_report(auth.user_id, &id, report_id).await?;
    Ok(Json(ApiReportData {
        columns: data.columns,
        data_json: data.data_json,
    }))
}

#[utoipa::path(get, path = "/api/v1/system/plugins/{id}/config", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_plugin_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let db = state.event_service.db();
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM plugin_config WHERE user_id = ? AND plugin_id = ?")
        .bind(auth.user_id).bind(&id).fetch_all(db).await?;
    
    let mut map = serde_json::Map::new();
    for (k, v) in rows {
        map.insert(k, json!(v));
    }
    Ok(Json(serde_json::Value::Object(map)))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/config", responses((status = 200)), security(("api_key" = [])))]
pub async fn update_plugin_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>, Json(req): Json<serde_json::Map<String, serde_json::Value>>) -> Result<impl IntoResponse> {
    let db = state.event_service.db();
    for (k, v) in req {
        let v_str = v.as_str().unwrap_or("").to_string();
        sqlx::query("INSERT INTO plugin_config (user_id, plugin_id, key, value) VALUES (?, ?, ?, ?) ON CONFLICT(user_id, plugin_id, key) DO UPDATE SET value = EXCLUDED.value")
            .bind(auth.user_id).bind(&id).bind(k).bind(v_str).execute(db).await?;
    }
    Ok(StatusCode::OK)
}

#[utoipa::path(get, path = "/api/v1/discovery/catalog", responses((status = 200, description = "Catalog")), security(("api_key" = [])))]
pub async fn get_catalog(State(state): State<Arc<AppState>>, Extension(_auth): Extension<AuthContext>) -> impl IntoResponse {
    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
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
    Json(catalog)
}

#[utoipa::path(get, path = "/api/v1/system/plugins", responses((status = 200, body = [PluginStatus])), security(("api_key" = [])))]
pub async fn get_system_plugins(State(state): State<Arc<AppState>>, Extension(auth): Extension<AuthContext>) -> Result<Json<Vec<PluginStatus>>> {
    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
    let mut statuses = Vec::new();

    // 1. Add Virtual Core Plugin for system widgets
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
        let reports_list: Vec<(String, Vec<crate::plugins::scry::plugin::types::ReportMetadata>)> = state.event_service.plugin_manager().list_plugin_reports(auth.user_id).await?;
        let p_reports = reports_list.into_iter().find(|(p_id, _)| p_id == &id).map(|(_, r)| r).unwrap_or_default();
        
        // Auto-classify roles based on manifest structure
        let mut roles = Vec::new();
        if m.poll_interval.is_some() {
            roles.push("SOURCE".to_string());
        }
        if !m.subscriptions.is_empty() {
            roles.push("ENRICHER".to_string());
        }
        if !m.provided_traits.is_empty() {
            roles.push("RESOLVER".to_string());
        }
        if !m.suggested_widgets.is_empty() {
            roles.push("VISUALIZER".to_string());
        }
        if !p_reports.is_empty() {
            roles.push("ANALYZER".to_string());
        }

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
    Ok(Json(statuses))
}

#[utoipa::path(post, path = "/api/v1/system/plugins/{id}/poll", responses((status = 200, description = "Poll")), security(("api_key" = [])))]
pub async fn poll_plugin_manually(State(state): State<Arc<AppState>>, Path(id): Path<String>, Extension(auth): Extension<AuthContext>) -> Result<Json<serde_json::Value>> {
    let count = state.event_service.poll_and_save_plugin(auth.user_id, &id).await?;
    Ok(Json(json!({ "plugin": id, "events_saved": count })))
}
