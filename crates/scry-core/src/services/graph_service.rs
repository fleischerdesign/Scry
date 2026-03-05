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

        let entities = rows.into_iter().map(|(id, title_json, subtitle_json, photo_json)| {
            let display_title = title_json.and_then(|json: String| {
                serde_json::from_str::<serde_json::Value>(&json).ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            }).unwrap_or_else(|| id.clone());

            let display_subtitle = subtitle_json.and_then(|json: String| {
                serde_json::from_str::<serde_json::Value>(&json).ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            });

            let display_image = photo_json.and_then(|json: String| {
                serde_json::from_str::<serde_json::Value>(&json).ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            });

            ApiEntity {
                namespace: namespace.to_string(),
                typ: typ.to_string(),
                id: id.clone(),
                display_title,
                display_subtitle,
                display_image,
            }
        }).collect();

        Ok(entities)
    }

    pub async fn resolve_entities(&self, user_id: i64, refs: Vec<ApiEntityRef>) -> Result<Vec<ApiEntity>> {
        let repo = EntityRepository::new(&self.db, user_id);
        
        let batch_refs = refs.into_iter().map(|r| (r.namespace, r.typ, r.id)).collect();
        let rows = repo.get_entities_batch(batch_refs).await?;

        let entities = rows.into_iter().map(|(ns, typ, id, title_json, subtitle_json, photo_json)| {
            let display_title = title_json.and_then(|json: String| {
                serde_json::from_str::<serde_json::Value>(&json).ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            }).unwrap_or_else(|| id.clone());

            let display_subtitle = subtitle_json.and_then(|json: String| {
                serde_json::from_str::<serde_json::Value>(&json).ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            });

            let display_image = photo_json.and_then(|json: String| {
                serde_json::from_str::<serde_json::Value>(&json).ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            });

            ApiEntity {
                namespace: ns,
                typ: typ,
                id: id,
                display_title,
                display_subtitle,
                display_image,
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

        let mut relationships = Vec::new();
        for (sn, st, si, p, tn, tt, ti) in rel_rows {
            let direction = if sn == namespace && st == typ && si == id { "outgoing" } else { "incoming" };
            
            // Resolve display label for the predicate
            let mut display_label = p.split('/').last().unwrap_or(&p).replace('_', " ");
            for m in manifests.values() {
                if let Some(pred) = m.predicates.iter().find(|pr| pr.id == p || format!("{}/{}", sn, pr.id) == p || format!("{}/{}", tn, pr.id) == p) {
                    display_label = if direction == "outgoing" { pred.label.clone() } else { pred.inverse_label.clone() };
                    break;
                }
            }

            // Resolve display title for the TARGET entity
            let (t_ns, t_type, t_id) = if direction == "outgoing" { (&tn, &tt, &ti) } else { (&sn, &st, &si) };
            let target_display_title = repo.get_trait(t_ns, t_type, t_id, scry_plugin_sdk::schema::traits::NAME).await.ok().flatten()
                .unwrap_or_else(|| t_id.clone());

            relationships.push(json!({
                "source": { "ns": sn, "typ": st, "id": si },
                "predicate": p,
                "display_label": display_label,
                "target": { "ns": tn, "typ": tt, "id": ti, "display_title": target_display_title },
                "direction": direction
            }));
        }

        let mut result = serde_json::Map::new();
        
        // Resolve display fields using the centralized DRY repository method
        let (display_title, display_subtitle, display_image) = repo.get_display_info(namespace, typ, id).await;

        result.insert("display_title".to_string(), serde_json::Value::String(display_title));
        if let Some(sub) = display_subtitle {
            result.insert("display_subtitle".to_string(), serde_json::Value::String(sub));
        }
        if let Some(img) = display_image {
            result.insert("display_image".to_string(), serde_json::Value::String(img));
        }

        result.insert("traits".to_string(), serde_json::Value::Object(traits_map));
        result.insert("relationships".to_string(), serde_json::Value::Array(relationships));

        Ok(serde_json::Value::Object(result))
    }
}
