use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::{Config, Engine, Store, ResourceLimiter};
use wasmtime::component::{Component, InstancePre, Linker, ResourceTable};
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

impl ResourceLimiter for MyCtx {
    fn memory_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>) -> Result<bool> {
        // Limit auf 32 MB setzen
        const MAX_MEMORY: usize = 32 * 1024 * 1024;
        Ok(desired <= MAX_MEMORY)
    }

    fn table_growing(&mut self, _current: u32, desired: u32, _maximum: Option<u32>) -> Result<bool> {
        Ok(desired <= 1000)
    }
}

impl WasiView for MyCtx {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.wasi }
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

#[async_trait]
impl scry::plugin::host::Host for MyCtx {
    async fn query(&mut self, sql: String, params: Vec<scry::plugin::host::QueryParam>) -> Result<Result<String, String>> {
        // Sicherstellen, dass das SQL nur lesend ist (einfacher Check)
        let sql_trimmed = sql.trim().to_lowercase();
        if !sql_trimmed.starts_with("select") {
            return Ok(Err("Only SELECT queries are allowed".to_string()));
        }

        // Wir nutzen eine CTE (Common Table Expression), um das 'events' Table für das Plugin
        // transparent vorzufiltern. So kann das Plugin einfach 'SELECT * FROM events' schreiben,
        // sieht aber nur seine eigenen Daten. Das ist sicherer und performanter.
        let safe_sql = format!(
            "WITH events AS (SELECT * FROM events WHERE user_id = ?) {}",
            sql
        );
        let mut query = sqlx::query(&safe_sql);

        // Zuerst binden wir die user_id für die CTE
        query = query.bind(self.user_id);

        // Danach die Parameter vom Plugin
        for param in params {
            query = match param {
                scry::plugin::host::QueryParam::S(s) => query.bind(s),
                scry::plugin::host::QueryParam::I(i) => query.bind(i),
                scry::plugin::host::QueryParam::F(f) => query.bind(f),
            };
        }

        match query.fetch_all(&self.db).await {
            Ok(rows) => {
                let mut results = Vec::new();
                for row in rows {
                    let mut map = serde_json::Map::new();
                    use sqlx::{Column, TypeInfo, Row, ValueRef};
                    for col in row.columns() {
                        let name = col.name();
                        let val = match row.try_get_raw(col.ordinal()) {
                            Ok(raw) if !raw.is_null() => {
                                match col.type_info().name() {
                                    "INTEGER" | "INT64" => serde_json::to_value(row.get::<i64, _>(col.ordinal())).unwrap_or(serde_json::Value::Null),
                                    "REAL" | "FLOAT" => serde_json::to_value(row.get::<f64, _>(col.ordinal())).unwrap_or(serde_json::Value::Null),
                                    _ => serde_json::Value::String(row.get::<String, _>(col.ordinal())),
                                }
                            },
                            _ => serde_json::Value::Null,
                        };
                        map.insert(name.to_string(), val);
                    }
                    results.push(serde_json::Value::Object(map));
                }
                Ok(Ok(serde_json::to_string(&results).unwrap()))
            },
            Err(e) => Ok(Err(e.to_string())),
        }
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

#[derive(Clone)]
pub struct LoadedPlugin {
    pub pre: InstancePre<MyCtx>,
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
            config.consume_fuel(true); // Aktiviert die Rechenzeitbegrenzung
    
            let engine = Engine::new(&config)?;        let mut linker = Linker::new(&engine);
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

        // RESOURCE LIMITS: CPU (Fuel)
        // 1.000.000 "Fuel" entspricht ca. 1-10ms CPU-Zeit (je nach Befehlen)
        store.set_fuel(1_000_000)?; 
        
        // RESOURCE LIMITS: Memory
        store.limiter(|s| s);

        // MONITORING: Startzeit messen
        let start_time = std::time::Instant::now();

        let instance_raw = loaded.pre.instantiate_async(&mut store).await?;
        let instance = Plugin::new(&mut store, &instance_raw)?;
        let (res, _store) = f(instance, store).await?;

        // MONITORING: Dauer berechnen und loggen
        let duration = start_time.elapsed();
        
        // Wir nutzen strukturierte Felder im Log für einfachere Auswertung (z.B. mit Vector/Loki)
        tracing::info!(
            target: "scry::plugin_metrics",
            plugin = %plugin_name,
            user_id = %user_id,
            duration_ms = %duration.as_millis(),
            "Plugin execution completed"
        );

        // Optional: Warnen, wenn ein Plugin zu lange braucht (> 500ms)
        if duration.as_millis() > 500 {
            tracing::warn!(
                plugin = %plugin_name,
                user_id = %user_id,
                duration_ms = %duration.as_millis(),
                "Slow plugin detected!"
            );
        }

        Ok(res)
    }

    pub async fn reload_plugin(&self, path: &std::path::Path) -> Result<()> {
        if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
            return Ok(());
        }

        let Some(name) = path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()) else {
            tracing::warn!("Skipping plugin with invalid filename: {:?}", path);
            return Ok(());
        };

        tracing::info!("Hot-reloading plugin: {}", name);
        let component = Component::from_file(&self.engine, path)?;
        let pre = self.linker.instantiate_pre(&component)?;
        
        let mut store = Store::new(&self.engine, MyCtx {
            db: self.db.clone(),
            user_id: 0,
            plugin_name: name.clone(),
            http_client: self.http_client.clone(),
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
        });
        store.set_fuel(1_000_000)?;
        store.limiter(|s| s);

        let instance_raw = pre.instantiate_async(&mut store).await?;
        let instance = Plugin::new(&mut store, &instance_raw)?;
        let manifest = instance.call_get_manifest(&mut store).await?;
        
        let mut plugins = self.plugins.write().await;
        plugins.insert(name, LoadedPlugin { pre, manifest });
        
        Ok(())
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
                let Some(name) = path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()) else {
                    tracing::warn!("Skipping plugin with invalid filename: {:?}", path);
                    continue;
                };

                let component = Component::from_file(&self.engine, &path)?;
                let pre = self.linker.instantiate_pre(&component)?;
                
                let mut store = Store::new(&self.engine, MyCtx {
                    db: self.db.clone(),
                    user_id: 0,
                    plugin_name: name.clone(),
                    http_client: self.http_client.clone(),
                    wasi: WasiCtxBuilder::new().build(),
                    table: ResourceTable::new(),
                });
                store.set_fuel(1_000_000)?; // 1 Million "Anweisungen" für Manifest-Laden
                store.limiter(|s| s);

                let instance_raw = pre.instantiate_async(&mut store).await?;
                let instance = Plugin::new(&mut store, &instance_raw)?;
                let manifest = instance.call_get_manifest(&mut store).await?;
                
                plugins.insert(name, LoadedPlugin { pre, manifest });
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

    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::ResourceLimiter;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_resource_limits() {
        let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let mut ctx = MyCtx {
            db,
            user_id: 1,
            plugin_name: "test".to_string(),
            http_client: Client::new(),
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
        };

        // 1. Memory Limit (32 MB)
        // 10 MB ist okay
        assert!(ctx.memory_growing(0, 10 * 1024 * 1024, None).unwrap());
        // 40 MB ist zu viel
        assert!(!ctx.memory_growing(0, 40 * 1024 * 1024, None).unwrap());

        // 2. Table Limit (1000)
        assert!(ctx.table_growing(0, 500, None).unwrap());
        assert!(!ctx.table_growing(0, 1500, None).unwrap());
    }
}
