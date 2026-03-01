pub mod context;
pub mod host_impl;
pub mod manager;

#[allow(unused_imports)]
pub use manager::{PluginManager, LoadedPlugin};
#[allow(unused_imports)]
pub use context::MyCtx;

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
