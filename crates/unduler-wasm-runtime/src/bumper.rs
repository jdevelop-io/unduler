//! WASM bumper plugin wrapper.

use std::path::Path;

use wasmtime::Store;
use wasmtime::component::{Component, Linker};

use crate::{WasmEngine, WasmError, WasmResult};

// Generate bindings from WIT
wasmtime::component::bindgen!({
    world: "unduler-bumper",
    path: "../unduler-plugin/wit",
});

/// Store state for bumper plugins (no WASI needed).
pub struct BumperState;

/// WASM bumper plugin wrapper.
pub struct WasmBumper {
    store: Store<BumperState>,
    instance: UndulerBumper,
}

impl WasmBumper {
    /// Creates a new WASM bumper from a component.
    ///
    /// # Errors
    ///
    /// Returns an error if the component cannot be instantiated.
    pub fn from_component(engine: &WasmEngine, component: &Component) -> WasmResult<Self> {
        let mut store = Store::new(engine.inner(), BumperState);
        let linker = Linker::new(engine.inner());

        let instance = UndulerBumper::instantiate(&mut store, component, &linker)
            .map_err(|e| WasmError::Instantiation(e.to_string()))?;

        Ok(Self { store, instance })
    }

    /// Creates a new WASM bumper from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the component cannot be loaded or instantiated.
    pub fn from_file(engine: &WasmEngine, path: &Path) -> WasmResult<Self> {
        let component = engine.load_component(path)?;
        Self::from_component(engine, &component)
    }

    /// Gets plugin information.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn info(&mut self) -> WasmResult<PluginInfo> {
        self.instance
            .unduler_plugin_bumper()
            .call_info(&mut self.store)
            .map_err(|e| WasmError::FunctionCall {
                name: "info".to_string(),
                reason: e.to_string(),
            })
    }

    /// Determines bump type from parsed commits.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn determine(&mut self, commits: &[ParsedCommit]) -> WasmResult<BumpType> {
        self.instance
            .unduler_plugin_bumper()
            .call_determine(&mut self.store, commits)
            .map_err(|e| WasmError::FunctionCall {
                name: "determine".to_string(),
                reason: e.to_string(),
            })
    }
}

// Re-export generated types
pub use unduler::plugin::types::{BumpType, ParsedCommit, PluginInfo, PluginType};
