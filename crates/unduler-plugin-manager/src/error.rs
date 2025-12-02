//! Error types for the plugin manager.

use std::path::PathBuf;

/// Result type for plugin manager operations.
pub type PluginManagerResult<T> = Result<T, PluginManagerError>;

/// Plugin manager error types.
#[derive(Debug, thiserror::Error)]
pub enum PluginManagerError {
    /// Failed to create storage directory.
    #[error("failed to create storage directory: {path}")]
    StorageCreation {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Plugin not found in registry.
    #[error("plugin not found: {name}")]
    PluginNotFound { name: String },

    /// Plugin already installed.
    #[error("plugin already installed: {name} v{version}")]
    AlreadyInstalled { name: String, version: String },

    /// Failed to fetch crate metadata from crates.io.
    #[error("failed to fetch crate metadata for {name}")]
    CratesIoFetch {
        name: String,
        #[source]
        source: reqwest::Error,
    },

    /// Crate not found on crates.io.
    #[error("crate not found on crates.io: {name}")]
    CrateNotFound { name: String },

    /// Invalid crate metadata (missing required fields).
    #[error("invalid crate metadata for {name}: {reason}")]
    InvalidMetadata { name: String, reason: String },

    /// Failed to download plugin from GitHub.
    #[error("failed to download plugin {name} from {url}")]
    DownloadFailed {
        name: String,
        url: String,
        #[source]
        source: reqwest::Error,
    },

    /// GitHub release not found.
    #[error("GitHub release not found for {name} v{version}")]
    ReleaseNotFound { name: String, version: String },

    /// WASM asset not found in release.
    #[error("WASM asset not found in release for {name} v{version}")]
    WasmAssetNotFound { name: String, version: String },

    /// Failed to save plugin file.
    #[error("failed to save plugin to {path}")]
    SaveFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to read registry file.
    #[error("failed to read registry file")]
    RegistryRead(#[source] std::io::Error),

    /// Failed to write registry file.
    #[error("failed to write registry file")]
    RegistryWrite(#[source] std::io::Error),

    /// Failed to parse registry file.
    #[error("failed to parse registry file")]
    RegistryParse(#[source] toml::de::Error),

    /// Failed to serialize registry file.
    #[error("failed to serialize registry")]
    RegistrySerialize(#[source] toml::ser::Error),

    /// Failed to load WASM plugin.
    #[error("failed to load WASM plugin: {name}")]
    WasmLoad {
        name: String,
        #[source]
        source: unduler_wasm_runtime::WasmError,
    },

    /// Plugin type mismatch.
    #[error("plugin type mismatch for {name}: expected {expected}, got {actual}")]
    TypeMismatch {
        name: String,
        expected: String,
        actual: String,
    },

    /// Invalid plugin name.
    #[error(
        "invalid plugin name: {name}. Must start with 'unduler-parser-', 'unduler-bumper-', 'unduler-formatter-', or 'unduler-hook-'"
    )]
    InvalidPluginName { name: String },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
