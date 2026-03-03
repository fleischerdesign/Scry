use sqlx::SqlitePool;
use std::sync::Arc;
use serde_json::json;
use crate::domain::*;
use crate::error::Result;
use crate::repository::EntityRepository;
use crate::plugins::PluginManager;

#[derive(Clone)]
pub struct GraphService {
    db: SqlitePool,
    plugin_manager: Arc<PluginManager>,
}

impl GraphService {
    pub fn new(db: SqlitePool, plugin_manager: Arc<PluginManager>) -> Self {
        Self { db, plugin_manager }
    }

    pub async fn get_namespaces(&self, user_id: i64) -> Result<Vec<ApiNamespace>> {
        let repo = EntityRepository::new(&self.db, user_id);
        let names = repo.get_namespaces().await?;
        let manifests = self.plugin_manager.get_plugin_manifests().await;
        
        let namespaces = names.into_iter().map(|name| {
            let mut icon = None;
            for m in manifests.values() {
                if let Some(domain) = m.domain_info.iter().find(|d| d.ns == name) {
                    if domain.icon.is_some() {
                        icon = domain.icon.clone();
                        break;
                    }
                }
            }
            if icon.is_none() && name == "scry.core" {
                icon = Some("lucide:shield-check".to_string());
            }
            ApiNamespace { name, icon }
        }).collect();

        Ok(namespaces)
    }

    pub async fn get_namespace_types(&self, user_id: i64, namespace: &str) -> Result<Vec<String>> {
        let repo = EntityRepository::new(&self.db, user_id);
        repo.get_types_by_namespace(namespace).await
    }

    pub async fn get_entities(&self, user_id: i64, namespace: &str, typ: &str) -> Result<Vec<ApiEntity>> {
        let repo = EntityRepository::new(&self.db, user_id);
        let rows = repo.get_entities_by_type(namespace, typ).await?;

        let entities = rows.into_iter().map(|(id, title_json, photo_json)| {
            let title = title_json.and_then(|json| {
                serde_json::from_str::<serde_json::Value>(&json).ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            }).unwrap_or_else(|| id.clone());

            let photo_url = photo_json.and_then(|json| {
                serde_json::from_str::<serde_json::Value>(&json).ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            });

            ApiEntity {
                namespace: namespace.to_string(),
                typ: typ.to_string(),
                id: id.clone(),
                title,
                photo_url,
                link: format!("/entity/{}/{}/{}", namespace, typ, id),
            }
        }).collect();

        Ok(entities)
    }

    pub async fn get_entity_details(&self, user_id: i64, namespace: &str, typ: &str, id: &str) -> Result<serde_json::Value> {
        let repo = EntityRepository::new(&self.db, user_id);
        
        let trait_rows = repo.get_traits(namespace, typ, id).await?;
        let mut traits_map = serde_json::Map::new();
        for (_plugin_id, trait_id, value_json) in trait_rows {
            let val: serde_json::Value = serde_json::from_str(&value_json).unwrap_or(serde_json::Value::Null);
            traits_map.insert(trait_id, val);
        }

        let rel_rows = repo.get_relationships(namespace, typ, id).await?;
        let manifests = self.plugin_manager.get_plugin_manifests().await;

        let relationships: Vec<serde_json::Value> = rel_rows.into_iter().map(|(sn, st, si, p, tn, tt, ti)| {
            let direction = if sn == namespace && st == typ && si == id { "outgoing" } else { "incoming" };
            let mut display_label = p.split('/').last().unwrap_or(&p).replace('_', " ");
            for m in manifests.values() {
                if let Some(pred) = m.predicates.iter().find(|pr| pr.id == p || format!("{}/{}", sn, pr.id) == p || format!("{}/{}", tn, pr.id) == p) {
                    display_label = if direction == "outgoing" { pred.label.clone() } else { pred.inverse_label.clone() };
                    break;
                }
            }

            json!({
                "source": { "ns": sn, "typ": st, "id": si },
                "predicate": p,
                "display_label": display_label,
                "target": { "ns": tn, "typ": tt, "id": ti },
                "direction": direction
            })
        }).collect();

        let mut result = serde_json::Map::new();
        result.insert("traits".to_string(), serde_json::Value::Object(traits_map));
        result.insert("relationships".to_string(), serde_json::Value::Array(relationships));

        Ok(serde_json::Value::Object(result))
    }
}
