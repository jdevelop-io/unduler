//! WASM engine setup and management.

use std::path::Path;

use wasmtime::{Config, Engine, component::Component};

use crate::{WasmError, WasmResult};

/// WASM engine for loading and executing plugin components.
pub struct WasmEngine {
    inner: Engine,
}

impl WasmEngine {
    /// Creates a new WASM engine with Component Model support.
    ///
    /// # Errors
    ///
    /// Returns an error if the engine cannot be created.
    pub fn new() -> WasmResult<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);

        let inner = Engine::new(&config).map_err(|e| WasmError::EngineCreation(e.to_string()))?;

        Ok(Self { inner })
    }

    /// Returns a reference to the inner wasmtime engine.
    #[must_use]
    pub fn inner(&self) -> &Engine {
        &self.inner
    }

    /// Loads a WASM component from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the component cannot be loaded.
    pub fn load_component(&self, path: &Path) -> WasmResult<Component> {
        let bytes = std::fs::read(path)?;

        Component::from_binary(&self.inner, &bytes).map_err(|e| WasmError::ComponentLoad {
            path: path.display().to_string(),
            reason: e.to_string(),
        })
    }

    /// Loads a WASM component from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the component cannot be loaded.
    pub fn load_component_from_bytes(&self, bytes: &[u8]) -> WasmResult<Component> {
        Component::from_binary(&self.inner, bytes)
            .map_err(|e| WasmError::Instantiation(e.to_string()))
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new().expect("failed to create default WASM engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = WasmEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_engine_default() {
        // Just verify engine was created successfully without panicking
        let _engine = WasmEngine::default();
    }

    #[test]
    fn test_load_component_not_found() {
        let engine = WasmEngine::new().unwrap();
        let result = engine.load_component(Path::new("/nonexistent/plugin.wasm"));
        assert!(result.is_err());
    }
}
