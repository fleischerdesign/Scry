use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxView};
use wasmtime::ResourceLimiter;
use reqwest::Client;

pub struct MyCtx {
    pub db: sqlx::SqlitePool,
    pub user_id: i64,
    pub plugin_name: String,
    pub http_client: Client,
    pub wasi: WasiCtx,
    pub table: ResourceTable,
}

impl ResourceLimiter for MyCtx {
    fn memory_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>) -> wasmtime::Result<bool> {
        // Limit auf 256 MB setzen
        const MAX_MEMORY: usize = 256 * 1024 * 1024;
        Ok(desired <= MAX_MEMORY)
    }

    fn table_growing(&mut self, _current: usize, desired: usize, _maximum: Option<usize>) -> wasmtime::Result<bool> {
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
