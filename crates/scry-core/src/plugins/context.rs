use crate::repository::PluginStateRepository;
use reqwest::Client;
use std::cell::OnceCell;
use wasmtime::component::ResourceTable;
use wasmtime::ResourceLimiter;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

pub struct MyCtx {
    pub db: sqlx::SqlitePool,
    pub user_id: i64,
    pub plugin_name: String,
    pub http_client: Client,
    pub wasi: WasiCtx,
    pub table: ResourceTable,
    state_repo: OnceCell<PluginStateRepository>,
}

impl MyCtx {
    pub fn new(db: sqlx::SqlitePool, user_id: i64, plugin_name: String) -> Self {
        Self {
            db,
            user_id,
            plugin_name,
            http_client: Client::new(),
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
            state_repo: OnceCell::new(),
        }
    }

    pub fn state_repo(&self) -> &PluginStateRepository {
        self.state_repo
            .get_or_init(|| PluginStateRepository::new(&self.db, self.user_id, &self.plugin_name))
    }
}

impl ResourceLimiter for MyCtx {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        const MAX_MEMORY: usize = 256 * 1024 * 1024;
        Ok(desired <= MAX_MEMORY)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(desired <= 1000)
    }
}

impl WasiView for MyCtx {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}
