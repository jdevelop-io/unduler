//! Plugin error types.

use thiserror::Error;

/// Plugin-related errors.
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin not found.
    #[error("plugin not found: {0}")]
    NotFound(String),

    /// Plugin initialization failed.
    #[error("plugin initialization failed: {0}")]
    InitFailed(String),

    /// Plugin execution failed.
    #[error("plugin execution failed: {0}")]
    ExecutionFailed(String),

    /// Configuration error.
    #[error("plugin configuration error: {0}")]
    ConfigError(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_display() {
        let err = PluginError::NotFound("my-plugin".to_string());
        assert_eq!(err.to_string(), "plugin not found: my-plugin");
    }

    #[test]
    fn test_init_failed_display() {
        let err = PluginError::InitFailed("failed to load".to_string());
        assert_eq!(
            err.to_string(),
            "plugin initialization failed: failed to load"
        );
    }

    #[test]
    fn test_execution_failed_display() {
        let err = PluginError::ExecutionFailed("crash".to_string());
        assert_eq!(err.to_string(), "plugin execution failed: crash");
    }

    #[test]
    fn test_config_error_display() {
        let err = PluginError::ConfigError("invalid value".to_string());
        assert_eq!(err.to_string(), "plugin configuration error: invalid value");
    }

    #[test]
    fn test_error_is_debug() {
        let err = PluginError::NotFound("test".to_string());
        let debug = format!("{err:?}");
        assert!(debug.contains("NotFound"));
    }

    #[test]
    fn test_plugin_result_ok() {
        let result: PluginResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(42));
    }

    #[test]
    fn test_plugin_result_err() {
        let result: PluginResult<i32> = Err(PluginError::NotFound("test".to_string()));
        assert!(result.is_err());
    }
}
