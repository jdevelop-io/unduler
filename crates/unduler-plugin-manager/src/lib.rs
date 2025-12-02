//! Plugin manager for Unduler.
//!
//! This crate handles:
//! - Plugin discovery via crates.io
//! - Plugin installation from GitHub Releases
//! - Local plugin storage and registry
//! - Plugin loading through the WASM runtime

pub mod discovery;
pub mod error;
pub mod registry;
pub mod storage;

pub use discovery::PluginDiscovery;
pub use error::{PluginManagerError, PluginManagerResult};
pub use registry::{InstalledPlugin, PluginRegistry};
pub use storage::PluginStorage;
