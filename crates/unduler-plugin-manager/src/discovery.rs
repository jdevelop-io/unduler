//! Plugin discovery via crates.io and download from GitHub Releases.
//!
//! Plugins are distributed as:
//! 1. Rust crates on crates.io (for source and metadata)
//! 2. Pre-compiled WASM on GitHub Releases (for runtime)

use serde::Deserialize;

use crate::storage::{PluginStorage, PluginType};

/// Search response from crates.io API.
#[derive(Deserialize)]
struct SearchResponse {
    crates: Vec<SearchCrate>,
}

/// Crate in search results.
#[derive(Deserialize)]
struct SearchCrate {
    name: String,
    description: Option<String>,
    max_stable_version: Option<String>,
    downloads: u64,
}
use crate::{InstalledPlugin, PluginManagerError, PluginManagerResult, PluginRegistry};

/// Crates.io API response for crate metadata.
#[derive(Debug, Deserialize)]
struct CratesIoResponse {
    #[serde(rename = "crate")]
    krate: CrateInfo,
    versions: Vec<VersionInfo>,
}

/// Crate metadata from crates.io.
#[derive(Debug, Deserialize)]
struct CrateInfo {
    #[allow(dead_code)]
    name: String,
    description: Option<String>,
    repository: Option<String>,
    max_stable_version: Option<String>,
}

/// Version metadata from crates.io.
#[derive(Debug, Deserialize)]
struct VersionInfo {
    num: String,
    yanked: bool,
}

/// GitHub release response.
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    #[allow(dead_code)]
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

/// GitHub release asset.
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Plugin metadata discovered from crates.io.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Full crate name.
    pub crate_name: String,
    /// Plugin type.
    pub plugin_type: PluginType,
    /// Short name.
    pub short_name: String,
    /// Latest stable version.
    pub latest_version: semver::Version,
    /// All available versions.
    pub versions: Vec<semver::Version>,
    /// Description.
    pub description: Option<String>,
    /// Repository URL.
    pub repository: Option<String>,
}

/// Plugin discovery and download service.
pub struct PluginDiscovery {
    client: reqwest::Client,
}

impl PluginDiscovery {
    /// Creates a new plugin discovery instance.
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be built.
    #[must_use]
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .expect("failed to build HTTP client");

        Self { client }
    }

    /// Fetches plugin metadata from crates.io.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The crate name is not a valid plugin name
    /// - The crate cannot be found on crates.io
    /// - The network request fails
    pub async fn fetch_metadata(&self, crate_name: &str) -> PluginManagerResult<PluginMetadata> {
        let (plugin_type, short_name) = PluginStorage::parse_crate_name(crate_name)?;

        let url = format!("https://crates.io/api/v1/crates/{crate_name}");

        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| PluginManagerError::CratesIoFetch {
                    name: crate_name.to_string(),
                    source: e,
                })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PluginManagerError::CrateNotFound {
                name: crate_name.to_string(),
            });
        }

        let data: CratesIoResponse =
            response
                .json()
                .await
                .map_err(|e| PluginManagerError::CratesIoFetch {
                    name: crate_name.to_string(),
                    source: e,
                })?;

        let latest_version = data
            .krate
            .max_stable_version
            .as_deref()
            .or_else(|| data.versions.first().map(|v| v.num.as_str()))
            .ok_or_else(|| PluginManagerError::InvalidMetadata {
                name: crate_name.to_string(),
                reason: "no versions available".to_string(),
            })?
            .parse()
            .map_err(|_| PluginManagerError::InvalidMetadata {
                name: crate_name.to_string(),
                reason: "invalid version format".to_string(),
            })?;

        let versions = data
            .versions
            .iter()
            .filter(|v| !v.yanked)
            .filter_map(|v| v.num.parse().ok())
            .collect();

        Ok(PluginMetadata {
            crate_name: crate_name.to_string(),
            plugin_type,
            short_name,
            latest_version,
            versions,
            description: data.krate.description,
            repository: data.krate.repository,
        })
    }

    /// Downloads a plugin WASM from GitHub Releases.
    ///
    /// Expects the release to have a `<crate-name>.wasm` asset.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The repository URL is missing or invalid
    /// - The release cannot be found
    /// - The WASM asset is missing
    /// - The download fails
    pub async fn download_wasm(
        &self,
        metadata: &PluginMetadata,
        version: &semver::Version,
    ) -> PluginManagerResult<Vec<u8>> {
        let repo_url =
            metadata
                .repository
                .as_ref()
                .ok_or_else(|| PluginManagerError::InvalidMetadata {
                    name: metadata.crate_name.clone(),
                    reason: "missing repository URL".to_string(),
                })?;

        let (owner, repo) =
            parse_github_url(repo_url).ok_or_else(|| PluginManagerError::InvalidMetadata {
                name: metadata.crate_name.clone(),
                reason: format!("invalid GitHub repository URL: {repo_url}"),
            })?;

        // Try different tag formats: v0.1.0, 0.1.0, crate-name-v0.1.0
        let tag_formats = [
            format!("v{version}"),
            version.to_string(),
            format!("{}-v{version}", metadata.crate_name),
        ];

        let mut last_error = None;

        for tag in &tag_formats {
            match self
                .try_download_release(&owner, &repo, tag, metadata)
                .await
            {
                Ok(bytes) => return Ok(bytes),
                Err(e) => last_error = Some(e),
            }
        }

        Err(
            last_error.unwrap_or_else(|| PluginManagerError::ReleaseNotFound {
                name: metadata.crate_name.clone(),
                version: version.to_string(),
            }),
        )
    }

    /// Attempts to download a WASM asset from a specific release.
    async fn try_download_release(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
        metadata: &PluginMetadata,
    ) -> PluginManagerResult<Vec<u8>> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| PluginManagerError::DownloadFailed {
                name: metadata.crate_name.clone(),
                url: url.clone(),
                source: e,
            })?;

        if !response.status().is_success() {
            return Err(PluginManagerError::ReleaseNotFound {
                name: metadata.crate_name.clone(),
                version: tag.to_string(),
            });
        }

        let release: GitHubRelease =
            response
                .json()
                .await
                .map_err(|e| PluginManagerError::DownloadFailed {
                    name: metadata.crate_name.clone(),
                    url: url.clone(),
                    source: e,
                })?;

        // Look for WASM asset
        let wasm_name = format!("{}.wasm", metadata.crate_name);
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == wasm_name)
            .ok_or_else(|| PluginManagerError::WasmAssetNotFound {
                name: metadata.crate_name.clone(),
                version: tag.to_string(),
            })?;

        // Download the WASM file
        let bytes = self
            .client
            .get(&asset.browser_download_url)
            .send()
            .await
            .map_err(|e| PluginManagerError::DownloadFailed {
                name: metadata.crate_name.clone(),
                url: asset.browser_download_url.clone(),
                source: e,
            })?
            .bytes()
            .await
            .map_err(|e| PluginManagerError::DownloadFailed {
                name: metadata.crate_name.clone(),
                url: asset.browser_download_url.clone(),
                source: e,
            })?;

        Ok(bytes.to_vec())
    }

    /// Installs a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin cannot be fetched or installed.
    pub async fn install(
        &self,
        registry: &mut PluginRegistry,
        crate_name: &str,
        version: Option<&semver::Version>,
    ) -> PluginManagerResult<InstalledPlugin> {
        let metadata = self.fetch_metadata(crate_name).await?;

        let version = version
            .cloned()
            .unwrap_or_else(|| metadata.latest_version.clone());

        // Check if already installed
        if let Some(existing) = registry.get(crate_name)
            && existing.version == version
        {
            return Err(PluginManagerError::AlreadyInstalled {
                name: crate_name.to_string(),
                version: version.to_string(),
            });
        }

        tracing::info!("Downloading {} v{}", crate_name, version);

        let wasm_bytes = self.download_wasm(&metadata, &version).await?;

        // Save to storage
        let storage = registry.storage();
        storage.save_plugin(
            &metadata.short_name,
            metadata.plugin_type,
            &version,
            &wasm_bytes,
        )?;

        // Register in registry
        let plugin = InstalledPlugin {
            crate_name: crate_name.to_string(),
            plugin_type: metadata.plugin_type,
            short_name: metadata.short_name.clone(),
            version,
            description: metadata.description,
            repository: metadata.repository,
            installed_at: chrono::Utc::now(),
        };

        if registry.is_installed(crate_name) {
            registry.upgrade(plugin.clone())?;
        } else {
            registry.register(plugin.clone())?;
        }

        tracing::info!("Installed {} v{}", plugin.crate_name, plugin.version);

        Ok(plugin)
    }

    /// Uninstalls a plugin.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin is not installed or cannot be removed.
    pub fn uninstall(
        &self,
        registry: &mut PluginRegistry,
        crate_name: &str,
    ) -> PluginManagerResult<()> {
        let plugin = registry.unregister(crate_name)?;

        registry.storage().remove_plugin(
            &plugin.short_name,
            plugin.plugin_type,
            &plugin.version,
        )?;

        tracing::info!("Uninstalled {} v{}", plugin.crate_name, plugin.version);

        Ok(())
    }

    /// Searches for plugins on crates.io.
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails.
    pub async fn search(&self, query: &str) -> PluginManagerResult<Vec<SearchResult>> {
        let url = format!("https://crates.io/api/v1/crates?q=unduler-{query}&per_page=20");

        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| PluginManagerError::CratesIoFetch {
                    name: query.to_string(),
                    source: e,
                })?;

        let data: SearchResponse =
            response
                .json()
                .await
                .map_err(|e| PluginManagerError::CratesIoFetch {
                    name: query.to_string(),
                    source: e,
                })?;

        // Filter to only valid unduler plugins
        let results = data
            .crates
            .into_iter()
            .filter_map(|c| {
                let (plugin_type, short_name) = PluginStorage::parse_crate_name(&c.name).ok()?;
                Some(SearchResult {
                    crate_name: c.name,
                    plugin_type,
                    short_name,
                    description: c.description,
                    latest_version: c.max_stable_version,
                    downloads: c.downloads,
                })
            })
            .collect();

        Ok(results)
    }
}

impl Default for PluginDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Search result from crates.io.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Full crate name.
    pub crate_name: String,
    /// Plugin type.
    pub plugin_type: PluginType,
    /// Short name.
    pub short_name: String,
    /// Description.
    pub description: Option<String>,
    /// Latest stable version.
    pub latest_version: Option<String>,
    /// Total downloads.
    pub downloads: u64,
}

/// Parses a GitHub URL into (owner, repo).
fn parse_github_url(url: &str) -> Option<(String, String)> {
    let url = url.trim_end_matches('/');

    // Handle both https://github.com/owner/repo and git@github.com:owner/repo
    let path = if url.starts_with("https://github.com/") {
        url.strip_prefix("https://github.com/")?
    } else if url.starts_with("git@github.com:") {
        url.strip_prefix("git@github.com:")?
    } else {
        return None;
    };

    let path = path.trim_end_matches(".git");
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_https() {
        let (owner, repo) = parse_github_url("https://github.com/jdevelop-io/unduler").unwrap();
        assert_eq!(owner, "jdevelop-io");
        assert_eq!(repo, "unduler");
    }

    #[test]
    fn test_parse_github_url_with_git_extension() {
        let (owner, repo) = parse_github_url("https://github.com/jdevelop-io/unduler.git").unwrap();
        assert_eq!(owner, "jdevelop-io");
        assert_eq!(repo, "unduler");
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let (owner, repo) = parse_github_url("git@github.com:jdevelop-io/unduler").unwrap();
        assert_eq!(owner, "jdevelop-io");
        assert_eq!(repo, "unduler");
    }

    #[test]
    fn test_parse_github_url_invalid() {
        assert!(parse_github_url("https://gitlab.com/foo/bar").is_none());
        assert!(parse_github_url("not-a-url").is_none());
    }
}
