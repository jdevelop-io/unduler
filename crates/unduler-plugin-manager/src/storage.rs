//! Local plugin storage management.
//!
//! Plugins are stored in `~/.unduler/plugins/` with the following structure:
//! ```text
//! ~/.unduler/
//! ├── plugins/
//! │   ├── parser-conventional/
//! │   │   └── 0.1.0.wasm
//! │   ├── bumper-semver/
//! │   │   └── 0.1.0.wasm
//! │   └── ...
//! └── registry.toml
//! ```

use std::path::{Path, PathBuf};

use crate::{PluginManagerError, PluginManagerResult};

/// Plugin type derived from plugin name prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    Parser,
    Bumper,
    Formatter,
    Hook,
}

impl PluginType {
    /// Returns the prefix for this plugin type.
    #[must_use]
    pub const fn prefix(&self) -> &'static str {
        match self {
            Self::Parser => "parser-",
            Self::Bumper => "bumper-",
            Self::Formatter => "formatter-",
            Self::Hook => "hook-",
        }
    }

    /// Returns the full crate name prefix.
    #[must_use]
    pub const fn crate_prefix(&self) -> &'static str {
        match self {
            Self::Parser => "unduler-parser-",
            Self::Bumper => "unduler-bumper-",
            Self::Formatter => "unduler-formatter-",
            Self::Hook => "unduler-hook-",
        }
    }
}

/// Manages local plugin storage.
pub struct PluginStorage {
    base_dir: PathBuf,
}

impl PluginStorage {
    /// Creates a new plugin storage instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the storage directory cannot be created.
    pub fn new() -> PluginManagerResult<Self> {
        let base_dir = Self::default_base_dir()?;
        Self::with_base_dir(base_dir)
    }

    /// Creates a new plugin storage instance with a custom base directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the storage directory cannot be created.
    pub fn with_base_dir(base_dir: PathBuf) -> PluginManagerResult<Self> {
        let plugins_dir = base_dir.join("plugins");

        std::fs::create_dir_all(&plugins_dir).map_err(|source| {
            PluginManagerError::StorageCreation {
                path: plugins_dir,
                source,
            }
        })?;

        Ok(Self { base_dir })
    }

    /// Returns the default base directory (`~/.unduler`).
    fn default_base_dir() -> PluginManagerResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| PluginManagerError::StorageCreation {
            path: PathBuf::from("~/.unduler"),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "could not determine home directory",
            ),
        })?;

        Ok(home.join(".unduler"))
    }

    /// Returns the base directory path.
    #[must_use]
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Returns the plugins directory path.
    #[must_use]
    pub fn plugins_dir(&self) -> PathBuf {
        self.base_dir.join("plugins")
    }

    /// Returns the registry file path.
    #[must_use]
    pub fn registry_path(&self) -> PathBuf {
        self.base_dir.join("registry.toml")
    }

    /// Returns the path for a specific plugin version.
    ///
    /// # Arguments
    ///
    /// * `short_name` - The short plugin name (e.g., "conventional" for parser-conventional)
    /// * `plugin_type` - The type of plugin
    /// * `version` - The plugin version
    #[must_use]
    pub fn plugin_path(
        &self,
        short_name: &str,
        plugin_type: PluginType,
        version: &semver::Version,
    ) -> PathBuf {
        let dir_name = format!("{}{short_name}", plugin_type.prefix());
        self.plugins_dir()
            .join(dir_name)
            .join(format!("{version}.wasm"))
    }

    /// Saves plugin WASM bytes to storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save_plugin(
        &self,
        short_name: &str,
        plugin_type: PluginType,
        version: &semver::Version,
        wasm_bytes: &[u8],
    ) -> PluginManagerResult<PathBuf> {
        let path = self.plugin_path(short_name, plugin_type, version);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| {
                PluginManagerError::StorageCreation {
                    path: parent.to_path_buf(),
                    source,
                }
            })?;
        }

        std::fs::write(&path, wasm_bytes).map_err(|source| PluginManagerError::SaveFailed {
            path: path.clone(),
            source,
        })?;

        tracing::info!("Saved plugin to {}", path.display());

        Ok(path)
    }

    /// Removes a plugin from storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be deleted.
    pub fn remove_plugin(
        &self,
        short_name: &str,
        plugin_type: PluginType,
        version: &semver::Version,
    ) -> PluginManagerResult<()> {
        let path = self.plugin_path(short_name, plugin_type, version);

        if path.exists() {
            std::fs::remove_file(&path)?;
            tracing::info!("Removed plugin from {}", path.display());

            // Remove parent directory if empty
            if let Some(parent) = path.parent()
                && parent.read_dir()?.next().is_none()
            {
                std::fs::remove_dir(parent)?;
            }
        }

        Ok(())
    }

    /// Checks if a plugin exists in storage.
    #[must_use]
    pub fn plugin_exists(
        &self,
        short_name: &str,
        plugin_type: PluginType,
        version: &semver::Version,
    ) -> bool {
        self.plugin_path(short_name, plugin_type, version).exists()
    }

    /// Parses a full crate name into plugin type and short name.
    ///
    /// # Errors
    ///
    /// Returns an error if the crate name doesn't match any known plugin type prefix.
    pub fn parse_crate_name(crate_name: &str) -> PluginManagerResult<(PluginType, String)> {
        for plugin_type in [
            PluginType::Parser,
            PluginType::Bumper,
            PluginType::Formatter,
            PluginType::Hook,
        ] {
            if let Some(short_name) = crate_name.strip_prefix(plugin_type.crate_prefix()) {
                return Ok((plugin_type, short_name.to_string()));
            }
        }

        Err(PluginManagerError::InvalidPluginName {
            name: crate_name.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_crate_name() {
        let (plugin_type, short_name) =
            PluginStorage::parse_crate_name("unduler-parser-conventional").unwrap();
        assert_eq!(plugin_type, PluginType::Parser);
        assert_eq!(short_name, "conventional");

        let (plugin_type, short_name) =
            PluginStorage::parse_crate_name("unduler-bumper-semver").unwrap();
        assert_eq!(plugin_type, PluginType::Bumper);
        assert_eq!(short_name, "semver");

        let (plugin_type, short_name) =
            PluginStorage::parse_crate_name("unduler-formatter-keepachangelog").unwrap();
        assert_eq!(plugin_type, PluginType::Formatter);
        assert_eq!(short_name, "keepachangelog");

        let (plugin_type, short_name) =
            PluginStorage::parse_crate_name("unduler-hook-cargo").unwrap();
        assert_eq!(plugin_type, PluginType::Hook);
        assert_eq!(short_name, "cargo");
    }

    #[test]
    fn test_parse_invalid_crate_name() {
        let result = PluginStorage::parse_crate_name("some-random-crate");
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_path() {
        let storage = PluginStorage::with_base_dir(PathBuf::from("/tmp/unduler-test")).unwrap();
        let version = semver::Version::new(1, 0, 0);

        let path = storage.plugin_path("conventional", PluginType::Parser, &version);
        assert_eq!(
            path,
            PathBuf::from("/tmp/unduler-test/plugins/parser-conventional/1.0.0.wasm")
        );
    }
}
