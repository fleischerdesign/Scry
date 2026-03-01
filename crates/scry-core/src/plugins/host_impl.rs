use crate::plugins::scry::plugin::host::{Host, QueryParam, Relationship};
use crate::plugins::context::MyCtx;
use anyhow::Result;
use sqlx::{Column, Row, ValueRef};

impl crate::plugins::scry::plugin::types::Host for MyCtx {}

impl Host for MyCtx {
    async fn query(&mut self, sql: String, params: Vec<QueryParam>) -> Result<Result<String, String>> {
        let sql_trimmed = sql.trim().to_lowercase();
        if !sql_trimmed.starts_with("select") {
            return Ok(Err("Only SELECT queries are allowed".to_string()));
        }

        let safe_sql = format!(
            "WITH events AS (SELECT id, user_id, timestamp, category, source, json(payload) as payload, metadata FROM main.events WHERE user_id = ?) {}",
            sql
        );
        let mut query = sqlx::query(&safe_sql);
        query = query.bind(self.user_id);

        for param in params {
            query = match param {
                QueryParam::S(s) => query.bind(s),
                QueryParam::I(i) => query.bind(i),
                QueryParam::F(f) => query.bind(f),
            };
        }

        match query.fetch_all(&self.db).await {
            Ok(rows) => {
                let mut results = Vec::new();
                for row in rows {
                    let mut map = serde_json::Map::new();
                    for col in row.columns() {
                        let name = col.name();
                        let val = match row.try_get_raw(col.ordinal()) {
                            Ok(raw) if !raw.is_null() => {
                                if let Ok(v) = row.try_get::<i64, _>(col.ordinal()) {
                                    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
                                } else if let Ok(v) = row.try_get::<f64, _>(col.ordinal()) {
                                    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
                                } else if let Ok(v) = row.try_get::<String, _>(col.ordinal()) {
                                    serde_json::Value::String(v)
                                } else {
                                    tracing::warn!("Failed to map column '{}' with type {:?}", col.name(), col.type_info());
                                    serde_json::Value::Null
                                }
                            },
                            _ => serde_json::Value::Null,
                        };
                        map.insert(name.to_string(), val);
                    }
                    results.push(serde_json::Value::Object(map));
                }
                match serde_json::to_string(&results) {
                    Ok(json) => Ok(Ok(json)),
                    Err(e) => Ok(Err(format!("JSON Serialization Error: {}", e))),
                }
            },
            Err(e) => Ok(Err(e.to_string())),
        }
    }

    async fn http_get(&mut self, url: String) -> Result<Result<String, String>> {
        let res = match self.http_client.get(&url)
            .header("User-Agent", "Scry/1.0")
            .send().await {
            Ok(resp) => resp.text().await.map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        Ok(res)
    }

    async fn set_state(&mut self, key: String, value: String) -> Result<()> {
        sqlx::query("INSERT INTO plugin_state (user_id, plugin_name, key, value) VALUES (?, ?, ?, ?) ON CONFLICT(user_id, plugin_name, key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP")
            .bind(self.user_id).bind(&self.plugin_name).bind(key).bind(value).execute(&self.db).await?;
        Ok(())
    }

    async fn get_state(&mut self, key: String) -> Result<Option<String>> {
        let row = sqlx::query_as::<_, (String,)>("SELECT value FROM plugin_state WHERE user_id = ? AND plugin_name = ? AND key = ?")
            .bind(self.user_id).bind(&self.plugin_name).bind(key).fetch_optional(&self.db).await?;
        Ok(row.map(|r| r.0))
    }

    async fn get_config(&mut self, key: String) -> Result<Option<String>> {
        tracing::debug!(plugin = %self.plugin_name, key = %key, "Host: get_config called");
        let res = sqlx::query_as::<_, (String,)>("SELECT value FROM plugin_config WHERE user_id = ? AND plugin_id = ? AND key = ?")
            .bind(self.user_id).bind(&self.plugin_name).bind(&key).fetch_optional(&self.db).await;
        
        match res {
            Ok(row) => Ok(row.map(|r| r.0)),
            Err(e) => {
                tracing::error!(plugin = %self.plugin_name, key = %key, "Host: get_config DB error: {}", e);
                Ok(None) // Return None instead of Err to prevent WASM trap
            }
        }
    }

    async fn get_profile_value(&mut self, key: String) -> Result<Option<String>> {
        let row = sqlx::query_as::<_, (String,)>("SELECT value FROM user_profile WHERE user_id = ? AND key = ?")
            .bind(self.user_id).bind(key).fetch_optional(&self.db).await?;
        Ok(row.map(|r| r.0))
    }

    async fn log(&mut self, level: String, message: String) -> Result<()> {
        match level.to_lowercase().as_str() {
            "error" => tracing::error!(plugin_id = %self.plugin_name, user_id = %self.user_id, "{}", message),
            "warn" => tracing::warn!(plugin_id = %self.plugin_name, user_id = %self.user_id, "{}", message),
            _ => tracing::info!(plugin_id = %self.plugin_name, user_id = %self.user_id, "{}", message),
        }
        Ok(())
    }

    async fn set_entity_trait(&mut self, namespace: String, typ: String, id: String, trait_id: String, value_json: String) -> Result<()> {
        // Erst Entität sicherstellen (Upsert)
        sqlx::query("INSERT INTO entities (user_id, namespace, typ, id) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
            .bind(self.user_id).bind(&namespace).bind(&typ).bind(&id).execute(&self.db).await?;

        // Dann Trait setzen (inkl. plugin_id)
        sqlx::query("INSERT INTO entity_traits (user_id, plugin_id, namespace, entity_type, entity_id, trait_id, value_json) VALUES (?, ?, ?, ?, ?, ?, ?) ON CONFLICT(user_id, plugin_id, namespace, entity_type, entity_id, trait_id) DO UPDATE SET value_json = EXCLUDED.value_json, updated_at = CURRENT_TIMESTAMP")
            .bind(self.user_id).bind(&self.plugin_name).bind(namespace).bind(typ).bind(id).bind(trait_id).bind(value_json).execute(&self.db).await?;
        
        Ok(())
    }

    async fn get_entity_trait(&mut self, namespace: String, typ: String, id: String, trait_id: String) -> Result<Option<String>> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT value_json FROM entity_traits WHERE user_id = ? AND namespace = ? AND entity_type = ? AND entity_id = ? AND trait_id = ? ORDER BY updated_at DESC LIMIT 1"
        )
        .bind(self.user_id).bind(namespace).bind(typ).bind(id).bind(trait_id)
        .fetch_optional(&self.db).await?;
        Ok(row)
    }

    async fn set_relationship(&mut self, rel: Relationship) -> Result<()> {
        sqlx::query("INSERT INTO entity_relationships (user_id, plugin_id, source_ns, source_type, source_id, predicate, target_ns, target_type, target_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT DO UPDATE SET updated_at = CURRENT_TIMESTAMP")
            .bind(self.user_id)
            .bind(&self.plugin_name)
            .bind(rel.source_namespace)
            .bind(rel.source_type)
            .bind(rel.source_id)
            .bind(rel.predicate)
            .bind(rel.target_namespace)
            .bind(rel.target_type)
            .bind(rel.target_id)
            .execute(&self.db).await?;
        Ok(())
    }

    async fn get_relationships(&mut self, namespace: String, typ: String, id: String, direction: String) -> Result<Vec<Relationship>> {
        let sql = if direction == "in" {
            "SELECT source_ns, source_type, source_id, predicate, target_ns, target_type, target_id FROM entity_relationships WHERE user_id = ? AND target_ns = ? AND target_type = ? AND target_id = ?"
        } else {
            "SELECT source_ns, source_type, source_id, predicate, target_ns, target_type, target_id FROM entity_relationships WHERE user_id = ? AND source_ns = ? AND source_type = ? AND source_id = ?"
        };

        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, String)>(sql)
            .bind(self.user_id).bind(namespace).bind(typ).bind(id)
            .fetch_all(&self.db).await?;

        Ok(rows.into_iter().map(|(sn, st, si, p, tn, tt, ti)| Relationship {
            source_namespace: sn, source_type: st, source_id: si,
            predicate: p,
            target_namespace: tn, target_type: tt, target_id: ti,
        }).collect())
    }
}
