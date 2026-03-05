use crate::plugins::scry::plugin::host::{Host, QueryParam, Relationship, HttpRequestData, HttpResponse};
use crate::plugins::context::MyCtx;
use crate::repository::ProfileRepository;
use anyhow::Result;
use sqlx::{Column, Row, ValueRef};

impl crate::plugins::scry::plugin::types::Host for MyCtx {}

impl Host for MyCtx {
    async fn query(&mut self, sql: String, params: Vec<QueryParam>) -> Result<std::result::Result<String, String>> {
        let sql_trimmed = sql.trim().to_lowercase();
        if !sql_trimmed.starts_with("select") {
            return Ok(Err("Only SELECT queries are allowed".to_string()));
        }

        let repo = self.state_repo();
        let rows = match repo.raw_query(&sql, self.user_id, params).await {
            Ok(r) => r,
            Err(e) => return Ok(Err(e.to_string())),
        };

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
    }

    async fn http_request(&mut self, req: HttpRequestData) -> Result<std::result::Result<HttpResponse, String>> {
        let method = match req.method.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            _ => return Ok(Err(format!("Unsupported HTTP method: {}", req.method))),
        };

        let mut builder = self.http_client.request(method, &req.url)
            .header("User-Agent", "Scry/1.0");

        for (k, v) in req.headers {
            builder = builder.header(k, v);
        }

        if let Some(body) = req.body {
            builder = builder.body(body);
        }

        match builder.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let headers = resp.headers().iter()
                    .map(|(k, v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).to_string()))
                    .collect();
                let body = match resp.text().await {
                    Ok(t) => t,
                    Err(e) => return Ok(Err(format!("Failed to read response body: {}", e))),
                };
                Ok(Ok(HttpResponse { status, headers, body }))
            }
            Err(e) => Ok(Err(e.to_string())),
        }
    }

    async fn set_state(&mut self, key: String, value: String) -> Result<()> {
        self.state_repo().set_state(&key, &value).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_state(&mut self, key: String) -> Result<Option<String>> {
        self.state_repo().get_state(&key).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_config(&mut self, key: String) -> Result<Option<String>> {
        self.state_repo().get_config(&key).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_profile_value(&mut self, key: String) -> Result<Option<String>> {
        let repo = ProfileRepository::new(&self.db, self.user_id);
        let rows = repo.get_all().await.map_err(|e| anyhow::anyhow!(e))?;
        let val = rows.into_iter().find(|(k, _)| k == &key).map(|(_, v)| v);
        Ok(val)
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
        self.state_repo().set_trait(&namespace, &typ, &id, &trait_id, &value_json).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_entity_trait(&mut self, namespace: String, typ: String, id: String, trait_id: String) -> Result<Option<String>> {
        self.state_repo().get_trait(&namespace, &typ, &id, &trait_id).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn set_relationship(&mut self, rel: Relationship) -> Result<()> {
        self.state_repo().set_relationship(rel).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_relationships(&mut self, namespace: String, typ: String, id: String, direction: String) -> Result<Vec<Relationship>> {
        self.state_repo().get_relationships(&namespace, &typ, &id, &direction).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn get_secret(&mut self, key: String) -> Result<Option<String>> {
        self.state_repo().get_secret(&key).await.map_err(|e| anyhow::anyhow!(e))
    }
}