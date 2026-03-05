use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, ResourceTable, Linker, InstancePre};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder};
use anyhow::Result;
use scry_plugin_sdk::{Manifest, ReportData, ReportMetadata};
use scry_proto::Event as ScryEvent;
use crate::plugins::context::MyCtx;
use crate::plugins::mapper::ConversionError;

#[derive(Clone)]
pub struct PluginConfig {
    pub max_memory_bytes: usize,
    pub max_table_entries: usize,
    pub max_fuel: u64,
    pub storage_base_path: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 256 * 1024 * 1024,
            max_table_entries: 1000,
            max_fuel: 1_000_000,
            storage_base_path: "./storage".to_string(),
        }
    }
}

pub struct PluginInstance {
    pub manifest: Manifest,
    pub instance_pre: InstancePre<MyCtx>,
}

pub struct PluginManager {
    engine: Engine,
    plugins: Arc<RwLock<HashMap<String, PluginInstance>>>,
    plugin_dir: String,
    db: sqlx::SqlitePool,
    config: PluginConfig,
}

impl PluginManager {
    pub fn new(plugin_dir: &str, db: sqlx::SqlitePool) -> Result<Self> {
        Self::with_config(plugin_dir, db, PluginConfig::default())
    }

    pub fn with_config(plugin_dir: &str, db: sqlx::SqlitePool, config: PluginConfig) -> Result<Self> {
        let mut cfg = Config::new();
        cfg.wasm_component_model(true);
        cfg.wasm_component_model_async(true);
        cfg.consume_fuel(true);
        let engine = Engine::new(&cfg)?;

        Ok(Self {
            engine,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_dir: plugin_dir.to_string(),
            db,
            config,
        })
    }

    async fn build_store(&self, user_id: i64, name: &str) -> Result<Store<MyCtx>> {
        let storage_path = format!("{}/u{}/p{}", self.config.storage_base_path, user_id, name);
        tokio::fs::create_dir_all(&storage_path).await?;

        let table = ResourceTable::new();
        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .preopened_dir(&storage_path, "/", DirPerms::all(), FilePerms::all())?
            .build();

        let ctx = MyCtx::new(
            self.db.clone(),
            user_id,
            name.to_string(),
            self.config.max_memory_bytes,
            self.config.max_table_entries,
            wasi,
            table,
        );

        let mut store = Store::new(&self.engine, ctx);
        store.limiter(|ctx| ctx);
        store.set_fuel(self.config.max_fuel)?;
        Ok(store)
    }

    fn setup_linker(&self) -> Result<Linker<MyCtx>> {
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
        crate::plugins::Plugin::add_to_linker::<MyCtx, wasmtime::component::HasSelf<MyCtx>>(&mut linker, |state| state)?;
        Ok(linker)
    }

    fn matches_subscription(subscription: &str, category: &str) -> bool {
        if subscription.ends_with('*') {
            category.starts_with(&subscription[..subscription.len() - 1])
        } else {
            category == subscription
        }
    }

    async fn get_matching_plugins(&self, category: &str) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.iter()
            .filter(|(_, p)| p.manifest.subscriptions.iter().any(|s| Self::matches_subscription(s, category)))
            .map(|(name, _)| name.clone())
            .collect()
    }

    async fn with_instance<F, Fut, T>(&self, name: &str, user_id: i64, f: F) -> Result<T>
    where
        F: FnOnce(crate::plugins::Plugin, Store<MyCtx>) -> Fut,
        Fut: std::future::Future<Output = Result<(T, Store<MyCtx>)>>,
    {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(name).ok_or_else(|| anyhow::anyhow!("Plugin not found"))?;

        let mut store = self.build_store(user_id, name).await?;
        let instance = plugin.instance_pre.instantiate_async(&mut store).await?;
        let instance_wrapper = crate::plugins::Plugin::new(&mut store, &instance)?;
        
        let (res, _store) = f(instance_wrapper, store).await?;
        Ok(res)
    }

    pub async fn reload_plugins(&self) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        plugins.clear();

        if !Path::new(&self.plugin_dir).exists() {
            tokio::fs::create_dir_all(&self.plugin_dir).await?;
        }

        let mut dir = tokio::fs::read_dir(&self.plugin_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
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

        let linker = self.setup_linker()?;
        let instance_pre = linker.instantiate_pre(&component)?;

        let mut store = self.build_store(0, &name).await?;
        
        let instance = instance_pre.instantiate_async(&mut store).await?;
        let instance_wrapper = crate::plugins::Plugin::new(&mut store, &instance)?;
        
        let wit_manifest = instance_wrapper.call_get_manifest(&mut store).await?;
        let manifest = Manifest::from(wit_manifest);
        
        if let Err(e) = instance_wrapper.call_on_init(&mut store).await? {
            tracing::error!("Plugin {} on_init failed: {}", name, e);
        }

        tracing::info!("Loaded plugin: {} v{} ({})", manifest.name, manifest.version, name);
        plugins.insert(name.clone(), PluginInstance { manifest, instance_pre });
        Ok(())
    }

    pub async fn run_ingest_pipeline(&self, user_id: i64, mut event: ScryEvent) -> Result<ScryEvent> {
        let plugin_names = self.get_matching_plugins(&event.category).await;

        for name in plugin_names {
            let res = self.with_instance(&name, user_id, |instance, mut store| async move {
                let wit_event = crate::plugins::scry::plugin::types::Event::from(&event);
                let processed = instance.call_on_ingest(&mut store, &wit_event).await?
                    .map_err(|e| anyhow::anyhow!(e))?;
                let mapped = ScryEvent::try_from(processed).map_err(|e| anyhow::anyhow!(e))?;
                Ok((mapped, store))
            }).await?;
            event = res;
        }
        Ok(event)
    }

    pub async fn poll_plugin(&self, user_id: i64, name: &str) -> Result<Vec<ScryEvent>> {
        self.with_instance(name, user_id, |instance, mut store| async move {
            let res = instance.call_on_poll(&mut store).await?;
            let mapped: Vec<ScryEvent> = res.into_iter()
                .map(|ev| ScryEvent::try_from(ev).map_err(|e| anyhow::anyhow!(e)))
                .collect::<Result<Vec<_>, _>>()?;
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
                let mapped = res.into_iter().map(ReportMetadata::from).collect();
                Ok((mapped, store))
            }).await?;
            all.push((name, reports));
        }
        Ok(all)
    }

    pub async fn run_plugin_report(&self, user_id: i64, plugin_name: &str, report_id: String) -> Result<ReportData> {
        self.with_instance(plugin_name, user_id, |instance, mut store| async move {
            let res = instance.call_run_report(&mut store, &report_id).await?
                .map_err(|e| anyhow::anyhow!(e))?;
            let mapped = ReportData::from(res);
            Ok((mapped, store))
        }).await
    }

    pub async fn get_plugin_summary(&self, user_id: i64, plugin_name: &str, start: String, end: String) -> Result<String> {
        self.with_instance(plugin_name, user_id, |instance, mut store| async move {
            let res = instance.call_get_summary(&mut store, &start, &end).await?;
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