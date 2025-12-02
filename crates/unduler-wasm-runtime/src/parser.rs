//! WASM parser plugin wrapper.

use std::path::Path;

use wasmtime::Store;
use wasmtime::component::{Component, Linker};

use crate::{WasmEngine, WasmError, WasmResult};

// Generate bindings from WIT
wasmtime::component::bindgen!({
    world: "unduler-parser",
    path: "../unduler-plugin/wit",
});

/// Store state for parser plugins (no WASI needed).
pub struct ParserState;

/// WASM parser plugin wrapper.
pub struct WasmParser {
    store: Store<ParserState>,
    instance: UndulerParser,
}

impl WasmParser {
    /// Creates a new WASM parser from a component.
    ///
    /// # Errors
    ///
    /// Returns an error if the component cannot be instantiated.
    pub fn from_component(engine: &WasmEngine, component: &Component) -> WasmResult<Self> {
        let mut store = Store::new(engine.inner(), ParserState);
        let linker = Linker::new(engine.inner());

        let instance = UndulerParser::instantiate(&mut store, component, &linker)
            .map_err(|e| WasmError::Instantiation(e.to_string()))?;

        Ok(Self { store, instance })
    }

    /// Creates a new WASM parser from a file path.
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
            .unduler_plugin_parser()
            .call_info(&mut self.store)
            .map_err(|e| WasmError::FunctionCall {
                name: "info".to_string(),
                reason: e.to_string(),
            })
    }

    /// Parses a raw commit.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn parse(&mut self, commit: &RawCommit) -> WasmResult<Option<ParsedCommit>> {
        self.instance
            .unduler_plugin_parser()
            .call_parse(&mut self.store, commit)
            .map_err(|e| WasmError::FunctionCall {
                name: "parse".to_string(),
                reason: e.to_string(),
            })
    }

    /// Checks if this parser can handle the commit.
    ///
    /// # Errors
    ///
    /// Returns an error if the WASM function call fails.
    pub fn can_parse(&mut self, commit: &RawCommit) -> WasmResult<bool> {
        self.instance
            .unduler_plugin_parser()
            .call_can_parse(&mut self.store, commit)
            .map_err(|e| WasmError::FunctionCall {
                name: "can_parse".to_string(),
                reason: e.to_string(),
            })
    }
}

// Re-export generated types for convenience
pub use unduler::plugin::types::{ParsedCommit, PluginInfo, PluginType, RawCommit};
