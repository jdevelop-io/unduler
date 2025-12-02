//! Plugin registry for tracking installed plugins.
//!
//! The registry is stored as a TOML file at `~/.unduler/registry.toml`.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::storage::{PluginStorage, PluginType};
use crate::{PluginManagerError, PluginManagerResult};

/// Information about an installed plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    /// Full crate name (e.g., "unduler-parser-conventional").
    pub crate_name: String,
    /// Plugin type.
    #[serde(with = "plugin_type_serde")]
    pub plugin_type: PluginType,
    /// Short name (e.g., "conventional").
    pub short_name: String,
    /// Installed version.
    pub version: semver::Version,
    /// Description from crates.io.
    pub description: Option<String>,
    /// Source repository URL.
    pub repository: Option<String>,
    /// Installation timestamp.
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

/// Serialization helpers for `PluginType`.
mod plugin_type_serde {
    use super::PluginType;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn serialize<S>(plugin_type: &PluginType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match *plugin_type {
            PluginType::Parser => "parser",
            PluginType::Bumper => "bumper",
            PluginType::Formatter => "formatter",
            PluginType::Hook => "hook",
        };
        s.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PluginType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "parser" => Ok(PluginType::Parser),
            "bumper" => Ok(PluginType::Bumper),
            "formatter" => Ok(PluginType::Formatter),
            "hook" => Ok(PluginType::Hook),
            _ => Err(serde::de::Error::custom(format!(
                "unknown plugin type: {s}"
            ))),
        }
    }
}

/// The registry file format.
#[derive(Debug, Default, Serialize, Deserialize)]
struct RegistryFile {
    #[serde(default)]
    plugins: HashMap<String, InstalledPlugin>,
}

/// Plugin registry for tracking installed plugins.
pub struct PluginRegistry {
    storage: PluginStorage,
    data: RegistryFile,
}

impl PluginRegistry {
    /// Creates a new registry, loading from disk if it exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry file exists but cannot be read or parsed.
    pub fn new(storage: PluginStorage) -> PluginManagerResult<Self> {
        let data = Self::load_from_path(storage.registry_path())?;
        Ok(Self { storage, data })
    }

    /// Loads registry data from a file path.
    fn load_from_path(path: impl AsRef<Path>) -> PluginManagerResult<RegistryFile> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(RegistryFile::default());
        }

        let content = std::fs::read_to_string(path).map_err(PluginManagerError::RegistryRead)?;
        toml::from_str(&content).map_err(PluginManagerError::RegistryParse)
    }

    /// Saves the registry to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry cannot be serialized or written.
    pub fn save(&self) -> PluginManagerResult<()> {
        let content =
            toml::to_string_pretty(&self.data).map_err(PluginManagerError::RegistrySerialize)?;
        std::fs::write(self.storage.registry_path(), content)
            .map_err(PluginManagerError::RegistryWrite)?;
        Ok(())
    }

    /// Returns a reference to the storage.
    #[must_use]
    pub fn storage(&self) -> &PluginStorage {
        &self.storage
    }

    /// Lists all installed plugins.
    #[must_use]
    pub fn list(&self) -> Vec<&InstalledPlugin> {
        self.data.plugins.values().collect()
    }

    /// Lists installed plugins by type.
    #[must_use]
    pub fn list_by_type(&self, plugin_type: PluginType) -> Vec<&InstalledPlugin> {
        self.data
            .plugins
            .values()
            .filter(|p| p.plugin_type == plugin_type)
            .collect()
    }

    /// Gets an installed plugin by crate name.
    #[must_use]
    pub fn get(&self, crate_name: &str) -> Option<&InstalledPlugin> {
        self.data.plugins.get(crate_name)
    }

    /// Gets an installed plugin by short name and type.
    #[must_use]
    pub fn get_by_short_name(
        &self,
        short_name: &str,
        plugin_type: PluginType,
    ) -> Option<&InstalledPlugin> {
        let crate_name = format!("{}{short_name}", plugin_type.crate_prefix());
        self.get(&crate_name)
    }

    /// Checks if a plugin is installed.
    #[must_use]
    pub fn is_installed(&self, crate_name: &str) -> bool {
        self.data.plugins.contains_key(crate_name)
    }

    /// Registers an installed plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is already installed (use `upgrade` instead).
    pub fn register(&mut self, plugin: InstalledPlugin) -> PluginManagerResult<()> {
        if self.is_installed(&plugin.crate_name) {
            return Err(PluginManagerError::AlreadyInstalled {
                name: plugin.crate_name.clone(),
                version: plugin.version.to_string(),
            });
        }

        let crate_name = plugin.crate_name.clone();
        self.data.plugins.insert(crate_name, plugin);
        self.save()?;

        Ok(())
    }

    /// Updates an installed plugin to a new version.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is not installed.
    pub fn upgrade(&mut self, plugin: InstalledPlugin) -> PluginManagerResult<()> {
        if !self.is_installed(&plugin.crate_name) {
            return Err(PluginManagerError::PluginNotFound {
                name: plugin.crate_name.clone(),
            });
        }

        let crate_name = plugin.crate_name.clone();
        self.data.plugins.insert(crate_name, plugin);
        self.save()?;

        Ok(())
    }

    /// Unregisters a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is not installed.
    pub fn unregister(&mut self, crate_name: &str) -> PluginManagerResult<InstalledPlugin> {
        let plugin = self.data.plugins.remove(crate_name).ok_or_else(|| {
            PluginManagerError::PluginNotFound {
                name: crate_name.to_string(),
            }
        })?;

        self.save()?;

        Ok(plugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_registry() -> (TempDir, PluginRegistry) {
        let temp_dir = TempDir::new().unwrap();
        let storage = PluginStorage::with_base_dir(temp_dir.path().to_path_buf()).unwrap();
        let registry = PluginRegistry::new(storage).unwrap();
        (temp_dir, registry)
    }

    fn create_test_plugin() -> InstalledPlugin {
        InstalledPlugin {
            crate_name: "unduler-parser-conventional".to_string(),
            plugin_type: PluginType::Parser,
            short_name: "conventional".to_string(),
            version: semver::Version::new(1, 0, 0),
            description: Some("Conventional commits parser".to_string()),
            repository: Some("https://github.com/example/repo".to_string()),
            installed_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_register_and_get() {
        let (_temp, mut registry) = create_test_registry();
        let plugin = create_test_plugin();

        registry.register(plugin.clone()).unwrap();

        let found = registry.get("unduler-parser-conventional").unwrap();
        assert_eq!(found.version, semver::Version::new(1, 0, 0));
    }

    #[test]
    fn test_list_by_type() {
        let (_temp, mut registry) = create_test_registry();

        registry.register(create_test_plugin()).unwrap();
        registry
            .register(InstalledPlugin {
                crate_name: "unduler-bumper-semver".to_string(),
                plugin_type: PluginType::Bumper,
                short_name: "semver".to_string(),
                version: semver::Version::new(1, 0, 0),
                description: None,
                repository: None,
                installed_at: chrono::Utc::now(),
            })
            .unwrap();

        let parsers = registry.list_by_type(PluginType::Parser);
        assert_eq!(parsers.len(), 1);
        assert_eq!(parsers[0].short_name, "conventional");
    }

    #[test]
    fn test_unregister() {
        let (_temp, mut registry) = create_test_registry();
        let plugin = create_test_plugin();

        registry.register(plugin).unwrap();
        assert!(registry.is_installed("unduler-parser-conventional"));

        registry.unregister("unduler-parser-conventional").unwrap();
        assert!(!registry.is_installed("unduler-parser-conventional"));
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Register a plugin
        {
            let storage = PluginStorage::with_base_dir(temp_dir.path().to_path_buf()).unwrap();
            let mut registry = PluginRegistry::new(storage).unwrap();
            registry.register(create_test_plugin()).unwrap();
        }

        // Load registry again and verify plugin is still there
        {
            let storage = PluginStorage::with_base_dir(temp_dir.path().to_path_buf()).unwrap();
            let registry = PluginRegistry::new(storage).unwrap();
            assert!(registry.is_installed("unduler-parser-conventional"));
        }
    }
}
