//! Configuration management for Unduler.
//!
//! This crate handles loading and validating the `unduler.toml` configuration file.

mod error;
mod loader;
mod schema;

pub use error::{ConfigError, ConfigResult};
pub use loader::{CONFIG_FILE_NAME, find_and_load_config, load_config};
pub use schema::{
    ChangelogConfig, Config, FormatterPluginConfig, HooksConfig, ParserConfig, PluginsConfig,
    VersionConfig,
};
