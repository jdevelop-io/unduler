//! Configuration error types.

use thiserror::Error;

/// Configuration-related errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file not found.
    #[error("configuration file not found: {0}")]
    NotFound(std::path::PathBuf),

    /// Invalid TOML syntax.
    #[error("invalid TOML: {0}")]
    InvalidToml(#[from] toml::de::Error),

    /// Invalid configuration value.
    #[error("invalid configuration: {0}")]
    Invalid(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for configuration operations.
pub type ConfigResult<T> = Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_not_found_display() {
        let err = ConfigError::NotFound(PathBuf::from("/path/to/config.toml"));
        assert_eq!(
            err.to_string(),
            "configuration file not found: /path/to/config.toml"
        );
    }

    #[test]
    fn test_invalid_display() {
        let err = ConfigError::Invalid("missing required field".to_string());
        assert_eq!(
            err.to_string(),
            "invalid configuration: missing required field"
        );
    }

    #[test]
    fn test_error_is_debug() {
        let err = ConfigError::Invalid("test".to_string());
        let debug = format!("{err:?}");
        assert!(debug.contains("Invalid"));
    }
}
