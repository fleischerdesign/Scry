use crate::plugins::context::MyCtx;
use crate::plugins::scry::plugin::types::{Event as PluginEvent, Manifest, ReportMetadata, ReportData, EntityRef as PluginEntityRef};
use anyhow::Result;
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, InstancePre, Linker, ResourceTable};
use wasmtime_wasi::WasiCtxBuilder;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::Client;
use scry_proto::Event as ScryEvent;

#[derive(Clone)]
pub struct LoadedPlugin {
    pub pre: InstancePre<MyCtx>,
    pub manifest: Manifest,
}

pub struct PluginManager {
    engine: Engine,
    plugins_dir: PathBuf,
    storage_dir: PathBuf,
    linker: Linker<MyCtx>,
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    db: sqlx::SqlitePool,
    http_client: Client,
}

impl PluginManager {
    pub fn new(plugins_dir: impl Into<PathBuf>, db: sqlx::SqlitePool) -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.consume_fuel(true);

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
        crate::plugins::Plugin::add_to_linker::<MyCtx, wasmtime::component::HasSelf<MyCtx>>(&mut linker, |state| state)?;

        Ok(Self {
            engine,
            plugins_dir: plugins_dir.into(),
            storage_dir: PathBuf::from("./storage"),
            linker,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            db,
            http_client: Client::builder().user_agent("Scry/0.1.0").build()?,
        })
    }

    async fn with_instance<F, Fut, R>(&self, plugin_name: &str, user_id: i64, f: F) -> Result<R>
    where
        F: FnOnce(crate::plugins::Plugin, Store<MyCtx>) -> Fut,
        Fut: std::future::Future<Output = Result<(R, Store<MyCtx>)>>,
    {
        let loaded = {
            let plugins = self.plugins.read().await;
            plugins.get(plugin_name).cloned().ok_or_else(|| anyhow::anyhow!("Plugin {} not found", plugin_name))?
        };

        let sandbox_path = self.storage_dir.join(format!("u{}", user_id)).join(format!("p{}", plugin_name));
        if !sandbox_path.exists() {
            std::fs::create_dir_all(&sandbox_path)?;
        }

        let mut wasi_builder = WasiCtxBuilder::new();
        wasi_builder.inherit_stdout().inherit_stderr();
        
        use wasmtime_wasi::{DirPerms, FilePerms};
        wasi_builder.preopened_dir(&sandbox_path, "/", DirPerms::all(), FilePerms::all())?;

        let mut store = Store::new(
            &self.engine,
            MyCtx {
                db: self.db.clone(),
                user_id,
                plugin_name: plugin_name.to_string(),
                http_client: self.http_client.clone(),
                wasi: wasi_builder.build(),
                table: ResourceTable::new(),
            },
        );

        store.set_fuel(1_000_000)?; 
        store.limiter(|s| s);

        let instance_raw = loaded.pre.instantiate_async(&mut store).await?;
        let instance = crate::plugins::Plugin::new(&mut store, &instance_raw)?;
        
        // Lifecycle: on_init
        if let Err(e) = instance.call_on_init(&mut store).await? {
            tracing::warn!(plugin = %plugin_name, user_id = %user_id, "Plugin init failed: {}", e);
        }

        let (res, _store) = f(instance, store).await?;
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
        let instance = crate::plugins::Plugin::new(&mut store, &instance_raw)?;
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
                store.set_fuel(1_000_000)?;
                store.limiter(|s| s);

                let instance_raw = pre.instantiate_async(&mut store).await?;
                let instance = crate::plugins::Plugin::new(&mut store, &instance_raw)?;
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
                if let Some(plugin) = plugins.get(&name) {
                    plugin.manifest.subscriptions.iter().any(|sub| {
                        if sub.ends_with('*') {
                            event.category.starts_with(&sub[..sub.len() - 1])
                        } else {
                            &event.category == sub
                        }
                    })
                } else {
                    false
                }
            };

            if should_run {
                let res = self.with_instance(&name, user_id, |instance, mut store| async move {
                    let ev = PluginEvent {
                        id: event.id.to_string(),
                        timestamp: event.timestamp.to_rfc3339(),
                        category: event.category.clone(),
                        source: event.source.clone(),
                        payload: serde_json::to_string(&event.payload)?,
                        metadata: event.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten(),
                        entities: event.entities.iter().map(|e| PluginEntityRef {
                            path: e.path.clone(), namespace: e.namespace.clone(), typ: e.typ.clone(), id: e.id.clone()
                        }).collect(),
                    };
                    let processed = instance.call_on_ingest(&mut store, &ev).await?.map_err(|e| anyhow::anyhow!(e))?;
                    
                    let mapped = ScryEvent {
                        id: uuid::Uuid::parse_str(&processed.id)?,
                        timestamp: chrono::DateTime::parse_from_rfc3339(&processed.timestamp)?.with_timezone(&chrono::Utc),
                        category: processed.category,
                        source: processed.source,
                        payload: serde_json::from_str(&processed.payload)?,
                        metadata: processed.metadata.and_then(|m| serde_json::from_str(&m).ok()),
                        entities: processed.entities.into_iter().map(|e| scry_proto::EntityRef {
                            path: e.path, namespace: e.namespace, typ: e.typ, id: e.id
                        }).collect(),
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
                    entities: ev.entities.into_iter().map(|e| scry_proto::EntityRef {
                        path: e.path, namespace: e.namespace, typ: e.typ, id: e.id
                    }).collect(),
                })
            }).collect::<Result<Vec<_>>>()?;
            Ok((mapped, store))
        }).await
    }

    pub async fn get_plugin_manifests(&self) -> HashMap<String, Manifest> {
        self.plugins.read().await.iter().map(|(n, p)| (n.clone(), p.manifest.clone())).collect()
    }

    pub async fn list_plugin_reports(&self, user_id: i64) -> Result<Vec<(String, Vec<ReportMetadata>)>> {
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

    pub async fn run_plugin_report(&self, user_id: i64, plugin_name: &str, report_id: String) -> Result<ReportData> {
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

    pub async fn resolve_plugin_trait(&self, user_id: i64, plugin_name: &str, namespace: String, typ: String, id: String, trait_id: String) -> Result<Option<String>> {
        self.with_instance(plugin_name, user_id, |instance, mut store| async move {
            let res = instance.call_resolve_trait(&mut store, &namespace, &typ, &id, &trait_id).await?.map_err(|e| anyhow::anyhow!(e))?;
            Ok((res, store))
        }).await
    }

    pub async fn notify_entity_discovered(&self, user_id: i64, namespace: String, typ: String, id: String) -> Result<()> {
        let names: Vec<String> = self.plugins.read().await.keys().cloned().collect();
        for name in names {
            let interested = {
                let plugins = self.plugins.read().await;
                plugins.get(&name).map(|p| p.manifest.provided_traits.iter().any(|t| t.entity_namespace == namespace && t.entity_type == typ)).unwrap_or(false)
            };

            if interested {
                let ns = namespace.clone();
                let t = typ.clone();
                let i = id.clone();
                self.with_instance(&name, user_id, |instance, mut store| async move {
                    instance.call_on_entity_discovered(&mut store, &ns, &t, &i).await?;
                    Ok(((), store))
                }).await?;
            }
        }
        Ok(())
    }
}
