//! Plugin system for Unduler.
//!
//! This crate provides the plugin traits and infrastructure:
//! - [`Plugin`]: Base trait for all plugins
//! - [`CommitParser`]: Parses raw commits into structured data
//! - [`BumpStrategy`]: Determines version bump type
//! - [`ChangelogFormatter`]: Formats changelog output
//! - [`ReleaseHook`]: Lifecycle hooks during release

mod context;
mod error;
mod traits;

pub use context::ReleaseContext;
pub use error::{PluginError, PluginResult};
pub use traits::Plugin;
pub use traits::bumper::{BumpStrategy, BumpType};
pub use traits::formatter::{ChangelogFormatter, FormatterConfig, Release};
pub use traits::hook::ReleaseHook;
pub use traits::parser::CommitParser;
