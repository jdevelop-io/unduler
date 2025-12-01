//! Core error types.

use thiserror::Error;

/// Core-related errors.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Git error.
    #[error("git error: {0}")]
    Git(#[from] unduler_git::GitError),

    /// Plugin error.
    #[error("plugin error: {0}")]
    Plugin(#[from] unduler_plugin::PluginError),

    /// Configuration error.
    #[error("config error: {0}")]
    Config(#[from] unduler_config::ConfigError),

    /// Version parsing error.
    #[error("version error: {0}")]
    Version(#[from] semver::Error),

    /// No commits found for release.
    #[error("no commits found since last release")]
    NoCommits,

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for core operations.
pub type CoreResult<T> = Result<T, CoreError>;
