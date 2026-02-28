use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, Linker, ResourceTable};
use reqwest::Client;
use scry_proto::Event as ScryEvent;
use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxBuilder};

wasmtime::component::bindgen!({
    world: "plugin",
    path: "../../crates/scry-proto/wit",
    async: true,
});

pub struct MyCtx {
    pub db: sqlx::SqlitePool,
    pub user_id: i64,
    pub plugin_name: String,
    pub http_client: Client,
    pub wasi: WasiCtx,
    pub table: ResourceTable,
}

impl WasiView for MyCtx {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.wasi }
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

#[async_trait]
impl scry::plugin::host::Host for MyCtx {
    async fn query(&mut self, query_json: String) -> Result<Result<String, String>> {
        #[derive(serde::Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        enum GenericQuery {
            CountGrouped { category: String, payload_key: String, limit: u32 },
            CountOverTime { category: String, interval: String, days: u32 },
            JoinNearest { base_category: String, join_category: String, limit: u32 },
        }

        let q: GenericQuery = serde_json::from_str(&query_json)?;
        let res = match q {
            GenericQuery::CountGrouped { category, payload_key, limit } => {
                let sql = format!("SELECT json_extract(payload, '$.' || ?) as key, COUNT(*) as count FROM events WHERE category = ? AND user_id = ? GROUP BY key ORDER BY count DESC LIMIT ?");
                match sqlx::query_as::<_, (Option<String>, i64)>(&sql)
                    .bind(payload_key).bind(category).bind(self.user_id).bind(limit).fetch_all(&self.db).await {
                    Ok(rows) => {
                        let mapped: Vec<_> = rows.into_iter().map(|(k, c)| serde_json::json!({"key": k.unwrap_or_default(), "count": c})).collect();
                        serde_json::to_string(&mapped).unwrap()
                    },
                    Err(e) => return Ok(Err(e.to_string())),
                }
            },
            GenericQuery::CountOverTime { category, interval, days } => {
                let format = match interval.as_str() { "1h" => "%Y-%m-%dT%H:00:00Z", _ => "%Y-%m-%d" };
                let sql = "SELECT strftime(?, timestamp) as label, COUNT(*) as count FROM events WHERE category = ? AND user_id = ? AND timestamp > date('now', ?) GROUP BY label ORDER BY label ASC";
                match sqlx::query_as::<_, (Option<String>, i64)>(sql)
                    .bind(format).bind(category).bind(self.user_id).bind(format!("-{} days", days)).fetch_all(&self.db).await {
                    Ok(rows) => {
                        let mapped: Vec<_> = rows.into_iter().map(|(l, c)| serde_json::json!({"label": l.unwrap_or_default(), "count": c})).collect();
                        serde_json::to_string(&mapped).unwrap()
                    },
                    Err(e) => return Ok(Err(e.to_string())),
                }
            },
            GenericQuery::JoinNearest { base_category, join_category, limit } => {
                let sql = r#"
                    SELECT 
                        CAST(b.payload AS TEXT),
                        CAST(j.payload AS TEXT)
                    FROM events b
                    JOIN events j ON j.category = ? AND j.user_id = ?
                    WHERE b.category = ? AND b.user_id = ?
                    GROUP BY b.id
                    HAVING MIN(ABS(julianday(substr(b.timestamp, 1, 19)) - julianday(substr(j.timestamp, 1, 19))))
                    ORDER BY b.timestamp DESC
                    LIMIT ?
                "#;

                match sqlx::query_as::<_, (String, Option<String>)>(sql)
                    .bind(&join_category).bind(self.user_id)
                    .bind(&base_category).bind(self.user_id)
                    .bind(limit).fetch_all(&self.db).await {
                    Ok(rows) => {
                        let results: Vec<serde_json::Value> = rows.into_iter().map(|(b, j)| {
                            serde_json::json!({
                                "base": serde_json::from_str::<serde_json::Value>(&b).unwrap_or_default(),
                                "joined": j.and_then(|j_str| serde_json::from_str::<serde_json::Value>(&j_str).ok()).unwrap_or_default()
                            })
                        }).collect();
                        serde_json::to_string(&results).unwrap()
                    },
                    Err(e) => return Ok(Err(e.to_string())),
                }
            }
        };
        Ok(Ok(res))
    }

    async fn http_get(&mut self, url: String) -> Result<Result<String, String>> {
        let res = match self.http_client.get(&url).send().await {
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
        self.get_state(key).await
    }

    async fn log(&mut self, level: String, message: String) -> Result<()> {
        match level.to_lowercase().as_str() {
            "error" => tracing::error!(plugin_id = %self.plugin_name, user_id = %self.user_id, "{}", message),
            "warn" => tracing::warn!(plugin_id = %self.plugin_name, user_id = %self.user_id, "{}", message),
            _ => tracing::info!(plugin_id = %self.plugin_name, user_id = %self.user_id, "{}", message),
        }
        Ok(())
    }
}

pub struct LoadedPlugin {
    pub component: Component,
    pub manifest: scry::plugin::types::Manifest,
}

pub struct PluginManager {
    engine: Engine,
    plugins_dir: PathBuf,
    linker: Linker<MyCtx>,
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    db: sqlx::SqlitePool,
    http_client: Client,
}

impl PluginManager {
    pub fn new(plugins_dir: impl Into<PathBuf>, db: sqlx::SqlitePool) -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        scry::plugin::host::add_to_linker(&mut linker, |state: &mut MyCtx| state)?;

        Ok(Self {
            engine,
            plugins_dir: plugins_dir.into(),
            linker,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            db,
            http_client: Client::builder().user_agent("Scry/0.1.0").build()?,
        })
    }

    async fn with_instance<F, Fut, R>(&self, plugin_name: &str, user_id: i64, f: F) -> Result<R>
    where
        F: FnOnce(Plugin, Store<MyCtx>) -> Fut,
        Fut: std::future::Future<Output = Result<(R, Store<MyCtx>)>>,
    {
        let loaded = {
            let plugins = self.plugins.read().await;
            plugins.get(plugin_name).cloned().ok_or_else(|| anyhow::anyhow!("Plugin {} not found", plugin_name))?
        };

        let mut store = Store::new(
            &self.engine,
            MyCtx {
                db: self.db.clone(),
                user_id,
                plugin_name: plugin_name.to_string(),
                http_client: self.http_client.clone(),
                wasi: WasiCtxBuilder::new().inherit_stdout().build(),
                table: ResourceTable::new(),
            },
        );

        let (instance, _) = Plugin::instantiate_async(&mut store, &loaded.component, &self.linker).await?;
        let (res, _) = f(instance, store).await?;
        Ok(res)
    }

    pub async fn reload_plugins(&self) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        plugins.clear();

        if !self.plugins_dir.exists() {
            std::fs::create_dir_all(&self.plugins_dir)?;
        }

        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                let component = Component::from_file(&self.engine, &path)?;
                
                let mut store = Store::new(&self.engine, MyCtx {
                    db: self.db.clone(),
                    user_id: 0,
                    plugin_name: name.clone(),
                    http_client: self.http_client.clone(),
                    wasi: WasiCtxBuilder::new().build(),
                    table: ResourceTable::new(),
                });
                let (instance, _) = Plugin::instantiate_async(&mut store, &component, &self.linker).await?;
                let manifest = instance.call_get_manifest(&mut store).await?;
                
                plugins.insert(name, LoadedPlugin { component, manifest });
            }
        }
        Ok(())
    }

    pub async fn run_ingest_pipeline(&self, user_id: i64, mut event: ScryEvent) -> Result<ScryEvent> {
        let names: Vec<String> = self.plugins.read().await.keys().cloned().collect();
        for name in names {
            let should_run = {
                let plugins = self.plugins.read().await;
                let manifest = &plugins.get(&name).unwrap().manifest;
                manifest.subscriptions.iter().any(|sub| {
                    if sub.ends_with('*') {
                        event.category.starts_with(&sub[..sub.len() - 1])
                    } else {
                        &event.category == sub
                    }
                })
            };

            if should_run {
                let res = self.with_instance(&name, user_id, |instance, mut store| async move {
                    let ev = scry::plugin::types::Event {
                        id: event.id.to_string(),
                        timestamp: event.timestamp.to_rfc3339(),
                        category: event.category.clone(),
                        source: event.source.clone(),
                        payload: serde_json::to_string(&event.payload)?,
                        metadata: event.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap()),
                    };
                    let processed = instance.call_on_ingest(&mut store, &ev).await?.map_err(|e| anyhow::anyhow!(e))?;
                    let mapped = ScryEvent {
                        id: uuid::Uuid::parse_str(&processed.id)?,
                        timestamp: chrono::DateTime::parse_from_rfc3339(&processed.timestamp)?.with_timezone(&chrono::Utc),
                        category: processed.category,
                        source: processed.source,
                        payload: serde_json::from_str(&processed.payload)?,
                        metadata: processed.metadata.and_then(|m| serde_json::from_str(&m).ok()),
                    };
                    Ok((mapped, store))
                }).await?;
                event = res;
            }
        }
        Ok(event)
    }

    pub async fn poll_plugin(&self, user_id: i64, name: &str) -> Result<Vec<ScryEvent>> {
        self.with_instance(name, user_id, |instance, mut store| async move {
            let res = instance.call_on_poll(&mut store).await?;
            let mapped = res.into_iter().map(|ev| {
                Ok(ScryEvent {
                    id: uuid::Uuid::parse_str(&ev.id)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&ev.timestamp)?.with_timezone(&chrono::Utc),
                    category: ev.category, source: ev.source, payload: serde_json::from_str(&ev.payload)?,
                    metadata: ev.metadata.and_then(|m| serde_json::from_str(&m).ok()),
                })
            }).collect::<Result<Vec<_>>>()?;
            Ok((mapped, store))
        }).await
    }

    pub async fn get_plugin_manifests(&self) -> HashMap<String, scry::plugin::types::Manifest> {
        self.plugins.read().await.iter().map(|(n, p)| (n.clone(), p.manifest.clone())).collect()
    }

    pub async fn list_plugin_reports(&self, user_id: i64) -> Result<Vec<(String, Vec<scry::plugin::types::ReportMetadata>)>> {
        let mut all = Vec::new();
        let names: Vec<String> = self.plugins.read().await.keys().cloned().collect();
        for name in names {
            let reports = self.with_instance(&name, user_id, |instance, mut store| async move {
                let res = instance.call_get_reports(&mut store).await?;
                Ok((res, store))
            }).await?;
            all.push((name, reports));
        }
        Ok(all)
    }

    pub async fn run_plugin_report(&self, user_id: i64, plugin_name: &str, report_id: String) -> Result<scry::plugin::types::ReportData> {
        self.with_instance(plugin_name, user_id, |instance, mut store| async move {
            let res = instance.call_run_report(&mut store, &report_id).await?.map_err(|e| anyhow::anyhow!(e))?;
            Ok((res, store))
        }).await
    }

    pub async fn get_plugin_summary(&self, user_id: i64, plugin_name: &str, start: String, end: String) -> Result<String> {
        self.with_instance(plugin_name, user_id, |instance, mut store| async move {
            let res = instance.call_get_summary(&mut store, &start, &end).await?;
            Ok((res, store))
        }).await
    }
}

impl Clone for LoadedPlugin {
    fn clone(&self) -> Self {
        Self {
            component: self.component.clone(),
            manifest: self.manifest.clone(),
        }
    }
}
