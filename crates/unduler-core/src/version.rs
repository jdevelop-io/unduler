//! Version management.

use semver::Version;
use unduler_plugin::BumpType;

/// Manages version operations.
pub struct VersionManager;

impl VersionManager {
    /// Creates a new version manager.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Bumps a version according to the bump type.
    #[must_use]
    pub fn bump(&self, version: &Version, bump_type: BumpType) -> Version {
        let mut new_version = version.clone();

        match bump_type {
            BumpType::Major => {
                new_version.major += 1;
                new_version.minor = 0;
                new_version.patch = 0;
                new_version.pre = semver::Prerelease::EMPTY;
            }
            BumpType::Minor => {
                new_version.minor += 1;
                new_version.patch = 0;
                new_version.pre = semver::Prerelease::EMPTY;
            }
            BumpType::Patch => {
                new_version.patch += 1;
                new_version.pre = semver::Prerelease::EMPTY;
            }
            BumpType::None => {}
        }

        new_version
    }

    /// Parses a version string.
    ///
    /// # Errors
    ///
    /// Returns an error if the version string is invalid.
    pub fn parse(&self, version: &str) -> Result<Version, semver::Error> {
        Version::parse(version)
    }

    /// Extracts version from a tag string.
    #[must_use]
    pub fn from_tag(&self, tag: &str, prefix: &str) -> Option<Version> {
        let version_str = tag.strip_prefix(prefix)?;
        Version::parse(version_str).ok()
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_major() {
        let vm = VersionManager::new();
        let version = Version::new(1, 2, 3);
        let bumped = vm.bump(&version, BumpType::Major);
        assert_eq!(bumped, Version::new(2, 0, 0));
    }

    #[test]
    fn test_bump_minor() {
        let vm = VersionManager::new();
        let version = Version::new(1, 2, 3);
        let bumped = vm.bump(&version, BumpType::Minor);
        assert_eq!(bumped, Version::new(1, 3, 0));
    }

    #[test]
    fn test_bump_patch() {
        let vm = VersionManager::new();
        let version = Version::new(1, 2, 3);
        let bumped = vm.bump(&version, BumpType::Patch);
        assert_eq!(bumped, Version::new(1, 2, 4));
    }

    #[test]
    fn test_from_tag() {
        let vm = VersionManager::new();
        let version = vm.from_tag("v1.2.3", "v");
        assert_eq!(version, Some(Version::new(1, 2, 3)));
    }

    #[test]
    fn test_bump_none() {
        let vm = VersionManager::new();
        let version = Version::new(1, 2, 3);
        let bumped = vm.bump(&version, BumpType::None);
        assert_eq!(bumped, Version::new(1, 2, 3));
    }

    #[test]
    fn test_bump_clears_prerelease() {
        let vm = VersionManager::new();
        let version = Version::parse("1.2.3-alpha.1").unwrap();
        let bumped = vm.bump(&version, BumpType::Patch);
        assert_eq!(bumped, Version::new(1, 2, 4));
        assert!(bumped.pre.is_empty());
    }

    #[test]
    fn test_default() {
        let vm = VersionManager;
        let version = Version::new(1, 0, 0);
        let bumped = vm.bump(&version, BumpType::Minor);
        assert_eq!(bumped, Version::new(1, 1, 0));
    }

    #[test]
    fn test_parse_valid() {
        let vm = VersionManager::new();
        let version = vm.parse("2.0.0").unwrap();
        assert_eq!(version, Version::new(2, 0, 0));
    }

    #[test]
    fn test_parse_invalid() {
        let vm = VersionManager::new();
        let result = vm.parse("not-a-version");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_tag_invalid_version() {
        let vm = VersionManager::new();
        let version = vm.from_tag("vnotaversion", "v");
        assert!(version.is_none());
    }

    #[test]
    fn test_from_tag_wrong_prefix() {
        let vm = VersionManager::new();
        let version = vm.from_tag("release-1.0.0", "v");
        assert!(version.is_none());
    }

    #[test]
    fn test_from_tag_different_prefix() {
        let vm = VersionManager::new();
        let version = vm.from_tag("release-1.0.0", "release-");
        assert_eq!(version, Some(Version::new(1, 0, 0)));
    }
}
