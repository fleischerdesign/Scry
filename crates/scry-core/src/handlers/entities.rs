use axum::{
    extract::{State, Json, Path},
    Extension,
};
use std::sync::Arc;

use crate::models::*;
use crate::error::Result;

#[utoipa::path(get, path = "/api/v1/discovery/entities", responses((status = 200, body = [ApiNamespace])), security(("api_key" = [])))]
pub async fn get_namespaces(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<Vec<crate::models::ApiNamespace>>> {
    let db = state.event_service.db();
    let names = sqlx::query_scalar::<_, String>("SELECT DISTINCT namespace FROM entities WHERE user_id = ?")
        .bind(auth.user_id).fetch_all(db).await?;
    
    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;
    
    let namespaces = names.into_iter().map(|name| {
        let mut icon = None;

        // Deterministic Ownership: Check domain_info directly
        for m in manifests.values() {
            if let Some(domain) = m.domain_info.iter().find(|d| d.ns == name) {
                if domain.icon.is_some() {
                    icon = domain.icon.clone();
                    break;
                }
            }
        }

        // System Core Fallback (since core is not a plugin yet)
        if icon.is_none() && name == "scry.core" {
            icon = Some("lucide:shield-check".to_string());
        }

        crate::models::ApiNamespace { name, icon }
    }).collect();

    Ok(Json(namespaces))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities/{namespace}", responses((status = 200, body = [String])), security(("api_key" = [])))]
pub async fn get_namespace_types(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<Vec<String>>> {
    let db = state.event_service.db();
    let types = sqlx::query_scalar::<_, String>("SELECT DISTINCT typ FROM entities WHERE user_id = ? AND namespace = ?")
        .bind(auth.user_id).bind(&namespace).fetch_all(db).await?;
    Ok(Json(types))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities/{namespace}/{typ}", responses((status = 200, body = [ApiEntity])), security(("api_key" = [])))]
pub async fn get_entities(
    State(state): State<Arc<AppState>>,
    Path((namespace, typ)): Path<(String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<Vec<crate::models::ApiEntity>>> {
    let db = state.event_service.db();
    
    // Wir holen alle Entitäten des Typs und versuchen Titel und Bild aus den Traits zu finden
    let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>)>("
        SELECT e.id, (
            SELECT value_json FROM entity_traits t 
            WHERE t.user_id = e.user_id AND t.namespace = e.namespace AND t.entity_type = e.typ AND t.entity_id = e.id 
            AND (t.trait_id LIKE '%name' OR t.trait_id LIKE '%title')
            LIMIT 1
        ) as title,
        (
            SELECT value_json FROM entity_traits t 
            WHERE t.user_id = e.user_id AND t.namespace = e.namespace AND t.entity_type = e.typ AND t.entity_id = e.id 
            AND (t.trait_id LIKE '%photo' OR t.trait_id LIKE '%avatar' OR t.trait_id LIKE '%image')
            LIMIT 1
        ) as photo
        FROM entities e
        WHERE e.user_id = ? AND e.namespace = ? AND e.typ = ?
    ")
    .bind(auth.user_id).bind(&namespace).bind(&typ).fetch_all(db).await?;

    let entities = rows.into_iter().map(|(id, title_json, photo_json)| {
        let title = title_json.and_then(|json| {
            serde_json::from_str::<serde_json::Value>(&json).ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        }).unwrap_or_else(|| id.clone());

        let photo_url = photo_json.and_then(|json| {
            serde_json::from_str::<serde_json::Value>(&json).ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        });

        crate::models::ApiEntity {
            namespace: namespace.clone(),
            typ: typ.clone(),
            id: id.clone(),
            title,
            photo_url,
            link: format!("/entity/{}/{}/{}", namespace, typ, id),
        }
    }).collect();

    Ok(Json(entities))
}

#[utoipa::path(get, path = "/api/v1/discovery/entities/{namespace}/{typ}/{id}/traits", responses((status = 200, body = serde_json::Value)), security(("api_key" = [])))]
pub async fn get_entity_traits(
    State(state): State<Arc<AppState>>,
    Path((namespace, typ, id)): Path<(String, String, String)>,
    Extension(auth): Extension<AuthContext>
) -> Result<Json<serde_json::Value>> {
    let db = state.event_service.db();
    let rows = sqlx::query_as::<_, (String, String, String)>("SELECT plugin_id, trait_id, value_json FROM entity_traits WHERE user_id = ? AND namespace = ? AND entity_type = ? AND entity_id = ?")
        .bind(auth.user_id).bind(&namespace).bind(&typ).bind(&id).fetch_all(db).await?;
    
    let mut map = serde_json::Map::new();
    for (_plugin_id, trait_id, value_json) in rows {
        let val: serde_json::Value = serde_json::from_str(&value_json).unwrap_or(serde_json::Value::Null);
        map.insert(trait_id, val);
    }

    // Beziehungen laden
    let rel_rows = sqlx::query_as::<_, (String, String, String, String, String, String, String)>("SELECT source_ns, source_type, source_id, predicate, target_ns, target_type, target_id FROM entity_relationships WHERE user_id = ? AND (source_ns = ? AND source_type = ? AND source_id = ? OR target_ns = ? AND target_type = ? AND target_id = ?)")
        .bind(auth.user_id)
        .bind(&namespace).bind(&typ).bind(&id)
        .bind(&namespace).bind(&typ).bind(&id)
        .fetch_all(db).await?;

    let manifests = state.event_service.plugin_manager().get_plugin_manifests().await;

    let relationships: Vec<serde_json::Value> = rel_rows.into_iter().map(|(sn, st, si, p, tn, tt, ti)| {
        let direction = if sn == namespace && st == typ && si == id { "outgoing" } else { "incoming" };
        
        // Find a human-friendly label from manifest predicates
        let mut display_label = p.split('/').last().unwrap_or(&p).replace('_', " ");
        for m in manifests.values() {
            if let Some(pred) = m.predicates.iter().find(|pr| pr.id == p || format!("{}/{}", sn, pr.id) == p || format!("{}/{}", tn, pr.id) == p) {
                display_label = if direction == "outgoing" { pred.label.clone() } else { pred.inverse_label.clone() };
                break;
            }
        }

        serde_json::json!({
            "source": { "ns": sn, "typ": st, "id": si },
            "predicate": p,
            "display_label": display_label,
            "target": { "ns": tn, "typ": tt, "id": ti },
            "direction": direction
        })
    }).collect();

    let mut result = serde_json::Map::new();
    result.insert("traits".to_string(), serde_json::Value::Object(map));
    result.insert("relationships".to_string(), serde_json::Value::Array(relationships));

    Ok(Json(serde_json::Value::Object(result)))
}
