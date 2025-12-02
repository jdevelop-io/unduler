//! Error types for the WASM runtime.

use thiserror::Error;

/// Errors that can occur in the WASM runtime.
#[derive(Debug, Error)]
pub enum WasmError {
    /// Failed to create the WASM engine.
    #[error("failed to create WASM engine: {0}")]
    EngineCreation(String),

    /// Failed to load a WASM component.
    #[error("failed to load WASM component from {path}: {reason}")]
    ComponentLoad { path: String, reason: String },

    /// Failed to instantiate a WASM component.
    #[error("failed to instantiate WASM component: {0}")]
    Instantiation(String),

    /// Failed to call a WASM function.
    #[error("failed to call WASM function '{name}': {reason}")]
    FunctionCall { name: String, reason: String },

    /// Plugin type mismatch.
    #[error("plugin type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Wasmtime error.
    #[error("wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
}

/// Result type for WASM runtime operations.
pub type WasmResult<T> = Result<T, WasmError>;
