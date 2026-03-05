pub mod context;
pub mod host_impl;
pub mod manager;
pub mod mapper;

#[allow(unused_imports)]
pub use manager::PluginManager;
#[allow(unused_imports)]
pub use context::MyCtx;
#[allow(unused_imports)]
pub use mapper::ConversionError;

wasmtime::component::bindgen!({
    world: "plugin",
    path: "../../crates/scry-proto/wit",
    anyhow: true,
    imports: {
        default: async | trappable,
    },
    exports: {
        default: async,
    },
});
