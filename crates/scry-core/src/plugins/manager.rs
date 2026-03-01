use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, ResourceTable, Linker};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};
use anyhow::Result;
use scry_proto::Event as ScryEvent;
use crate::plugins::context::MyCtx;
use crate::plugins::scry::plugin::types::{Event as PluginEvent, Manifest, ReportMetadata, ReportData, EntityRef as PluginEntityRef};

pub struct PluginInstance {
    pub name: String,
    pub component: Component,
    pub manifest: Manifest,
}

pub struct PluginManager {
    engine: Engine,
    plugins: Arc<RwLock<HashMap<String, PluginInstance>>>,
    plugin_dir: String,
    db: sqlx::SqlitePool,
}

impl PluginManager {
    pub fn new(plugin_dir: &str, db: sqlx::SqlitePool) -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        let engine = Engine::new(&config)?;

        Ok(Self {
            engine,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_dir: plugin_dir.to_string(),
            db,
        })
    }

    async fn with_instance<F, Fut, T>(&self, name: &str, user_id: i64, f: F) -> Result<T>
    where
        F: FnOnce(crate::plugins::Plugin, Store<MyCtx>) -> Fut,
        Fut: std::future::Future<Output = Result<(T, Store<MyCtx>)>>,
    {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(name).ok_or_else(|| anyhow::anyhow!("Plugin not found"))?;

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
        crate::plugins::Plugin::add_to_linker::<MyCtx, wasmtime::component::HasSelf<MyCtx>>(&mut linker, |state| state)?;

        let storage_path = format!("./storage/u{}/p{}", user_id, name);
        std::fs::create_dir_all(&storage_path)?;

        let table = ResourceTable::new();
        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .preopened_dir(&storage_path, "/", DirPerms::all(), FilePerms::all())?
            .build();

        let ctx = MyCtx {
            user_id,
            plugin_name: name.to_string(),
            db: self.db.clone(),
            http_client: reqwest::Client::new(),
            table,
            wasi,
        };

        let mut store = Store::new(&self.engine, ctx);
        let instance = crate::plugins::Plugin::instantiate_async(&mut store, &plugin.component, &linker).await?;
        
        let (res, _store) = f(instance, store).await?;
        Ok(res)
    }

    pub async fn reload_plugins(&self) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        plugins.clear();

        if !Path::new(&self.plugin_dir).exists() {
            std::fs::create_dir_all(&self.plugin_dir)?;
        }

        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                self.load_plugin_internal(&mut plugins, &path).await?;
            }
        }
        Ok(())
    }

    pub async fn reload_plugin(&self, path: &Path) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        self.load_plugin_internal(&mut plugins, path).await
    }

    async fn load_plugin_internal(&self, plugins: &mut HashMap<String, PluginInstance>, path: &Path) -> Result<()> {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let component = Component::from_file(&self.engine, path)?;

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
        crate::plugins::Plugin::add_to_linker::<MyCtx, wasmtime::component::HasSelf<MyCtx>>(&mut linker, |state| state)?;

        let table = ResourceTable::new();
        let wasi = WasiCtxBuilder::new().build();
        let ctx = MyCtx {
            user_id: 0,
            plugin_name: name.clone(),
            db: self.db.clone(),
            http_client: reqwest::Client::new(),
            table,
            wasi,
        };

        let mut store = Store::new(&self.engine, ctx);
        let instance_raw = crate::plugins::Plugin::instantiate_async(&mut store, &component, &linker).await?;
        let manifest = instance_raw.call_get_manifest(&mut store).await?;
        
        if let Err(e) = instance_raw.call_on_init(&mut store).await? {
            tracing::error!("Plugin {} on_init failed: {}", name, e);
        }

        tracing::info!("Loaded plugin: {} v{} ({})", manifest.name, manifest.version, name);
        plugins.insert(name.clone(), PluginInstance { name, component, manifest });
        Ok(())
    }

    pub async fn run_ingest_pipeline(&self, user_id: i64, mut event: ScryEvent) -> Result<ScryEvent> {
        let plugin_names: Vec<String> = {
            let plugins = self.plugins.read().await;
            plugins.keys().cloned().collect()
        };

        for name in plugin_names {
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
                        metadata: event.metadata.as_ref().and_then(|m| serde_json::to_string(m).ok()),
                        entities: event.entities.iter().map(|e| PluginEntityRef {
                            path: e.path.clone(), namespace: e.namespace.clone(), typ: e.typ.clone(), id: e.id.clone()
                        }).collect(),
                        display_title: event.display_title.clone(),
                        display_subtitle: event.display_subtitle.clone(),
                    };
                    let processed = instance.call_on_ingest(&mut store, &ev).await?.map_err(|e| anyhow::anyhow!(e))?;
                    
                    let mapped = ScryEvent {
                        id: uuid::Uuid::parse_str(&processed.id)?,
                        timestamp: chrono::DateTime::parse_from_rfc3339(&processed.timestamp)?.with_timezone(&chrono::Utc),
                        category: processed.category,
                        source: processed.source,
                        payload: serde_json::from_str(&processed.payload)?,
                        metadata: processed.metadata.as_ref().and_then(|m| serde_json::from_str(m).ok()),
                        entities: processed.entities.into_iter().map(|e| scry_proto::EntityRef {
                            path: e.path, namespace: e.namespace, typ: e.typ, id: e.id
                        }).collect(),
                        display_title: processed.display_title,
                        display_subtitle: processed.display_subtitle,
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
                    metadata: ev.metadata.as_ref().and_then(|m| serde_json::from_str(m).ok()),
                    entities: ev.entities.into_iter().map(|e| scry_proto::EntityRef {
                        path: e.path, namespace: e.namespace, typ: e.typ, id: e.id
                    }).collect(),
                    display_title: ev.display_title,
                    display_subtitle: ev.display_subtitle,
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
        let plugin_names: Vec<String> = {
            let plugins = self.plugins.read().await;
            plugins.keys().cloned().collect()
        };

        for name in plugin_names {
            let ns = namespace.clone();
            let t = typ.clone();
            let i = id.clone();
            self.with_instance(&name, user_id, |instance, mut store| async move {
                instance.call_on_entity_discovered(&mut store, &ns, &t, &i).await?;
                Ok(((), store))
            }).await?;
        }
        Ok(())
    }
}
