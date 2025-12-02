//! WASM formatter plugin wrapper.

use std::path::Path;

use wasmtime::Store;
use wasmtime::component::{Component, Linker};

use crate::{WasmEngine, WasmError, WasmResult};

// Generate bindings from WIT
wasmtime::component::bindgen!({
    world: "unduler-formatter",
    path: "../unduler-plugin/wit",
});

/// Store state for formatter plugins (no WASI needed).
pub struct FormatterState;

/// WASM formatter plugin wrapper.
pub struct WasmFormatter {
    store: Store<FormatterState>,
    instance: UndulerFormatter,
}

impl WasmFormatter {
    /// Creates a new WASM formatter from a component.
    ///
    /// # Errors
    ///
    /// Returns an error if the component cannot be instantiated.
    pub fn from_component(engine: &WasmEngine, component: &Component) -> WasmResult<Self> {
        let mut store = Store::new(engine.inner(), FormatterState);
        let linker = Linker::new(engine.inner());

        let instance = UndulerFormatter::instantiate(&mut store, component, &linker)
            .map_err(|e| WasmError::Instantiation(e.to_string()))?;

        Ok(Self { store, instance })
    }

    /// Creates a new WASM formatter from a file path.
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
            .unduler_plugin_formatter()
            .call_info(&mut self.store)
            .map_err(|e| WasmError::FunctionCall {
                name: "info".to_string(),
                reason: e.to_string(),
            })
    }

    /// Formats a release into a changelog string.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn format(&mut self, release: &Release, config: &FormatterConfig) -> WasmResult<String> {
        self.instance
            .unduler_plugin_formatter()
            .call_format(&mut self.store, release, config)
            .map_err(|e| WasmError::FunctionCall {
                name: "format".to_string(),
                reason: e.to_string(),
            })
    }

    /// Gets the file extension for output.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn extension(&mut self) -> WasmResult<String> {
        self.instance
            .unduler_plugin_formatter()
            .call_extension(&mut self.store)
            .map_err(|e| WasmError::FunctionCall {
                name: "extension".to_string(),
                reason: e.to_string(),
            })
    }
}

// Re-export generated types
pub use unduler::plugin::types::{FormatterConfig, PluginInfo, PluginType, Release};
