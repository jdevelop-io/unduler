//! Configuration loader.

use std::path::Path;

use tracing::debug;

use crate::{Config, ConfigError, ConfigResult};

/// Default configuration file name.
pub const CONFIG_FILE_NAME: &str = "unduler.toml";

/// Loads configuration from the given path.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn load_config(path: impl AsRef<Path>) -> ConfigResult<Config> {
    let path = path.as_ref();
    debug!(?path, "loading configuration");

    if !path.exists() {
        return Err(ConfigError::NotFound(path.to_path_buf()));
    }

    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

/// Finds and loads configuration from the current directory or parents.
///
/// # Errors
///
/// Returns an error if no configuration file is found or it cannot be parsed.
pub fn find_and_load_config() -> ConfigResult<Config> {
    let current_dir = std::env::current_dir()?;
    find_and_load_config_from(&current_dir)
}

/// Finds and loads configuration starting from the given directory.
///
/// Walks up the directory tree until a configuration file is found.
///
/// # Errors
///
/// Returns an error if no configuration file is found or it cannot be parsed.
pub fn find_and_load_config_from(start_dir: impl AsRef<Path>) -> ConfigResult<Config> {
    let start_dir = start_dir.as_ref();
    let mut dir = start_dir;

    loop {
        let config_path = dir.join(CONFIG_FILE_NAME);
        if config_path.exists() {
            return load_config(config_path);
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    Err(ConfigError::NotFound(start_dir.join(CONFIG_FILE_NAME)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_file_name() {
        assert_eq!(CONFIG_FILE_NAME, "unduler.toml");
    }

    #[test]
    fn test_load_config_not_found() {
        let result = load_config("/nonexistent/path/unduler.toml");
        assert!(result.is_err());
        match result {
            Err(ConfigError::NotFound(path)) => {
                assert!(path.to_string_lossy().contains("unduler.toml"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_load_config_valid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("unduler.toml");
        fs::write(
            &config_path,
            r#"
            [parser]
            name = "conventional"
        "#,
        )
        .unwrap();

        let result = load_config(&config_path);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.parser.name, "conventional");
    }

    #[test]
    fn test_load_config_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("unduler.toml");
        fs::write(&config_path, "").unwrap();

        let result = load_config(&config_path);
        assert!(result.is_ok());
        // Default values should be used
        let config = result.unwrap();
        assert_eq!(config.parser.name, "conventional");
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("unduler.toml");
        fs::write(&config_path, "this is not valid toml [[[").unwrap();

        let result = load_config(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_custom_values() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("unduler.toml");
        fs::write(
            &config_path,
            r#"
            [parser]
            name = "gitmoji"

            [version]
            tag_prefix = "release-"

            [changelog]
            output = "HISTORY.md"
        "#,
        )
        .unwrap();

        let config = load_config(&config_path).unwrap();
        assert_eq!(config.parser.name, "gitmoji");
        assert_eq!(config.version.tag_prefix, "release-");
        assert_eq!(config.changelog.output, "HISTORY.md");
    }

    #[test]
    fn test_find_and_load_config_in_temp_dir() {
        // Create a temp directory with a config file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("unduler.toml");
        fs::write(
            &config_path,
            r#"
            [parser]
            name = "test-parser"
        "#,
        )
        .unwrap();

        let result = find_and_load_config_from(temp_dir.path());

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.parser.name, "test-parser");
    }

    #[test]
    fn test_find_and_load_config_in_parent() {
        // Create parent dir with config
        let parent_dir = TempDir::new().unwrap();
        let config_path = parent_dir.path().join("unduler.toml");
        fs::write(
            &config_path,
            r#"
            [parser]
            name = "parent-parser"
        "#,
        )
        .unwrap();

        // Create child dir
        let child_dir = parent_dir.path().join("subdir");
        fs::create_dir(&child_dir).unwrap();

        let result = find_and_load_config_from(&child_dir);

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.parser.name, "parent-parser");
    }
}
