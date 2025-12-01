//! Release context shared between hooks.

use std::collections::HashMap;

use semver::Version;
use serde_json::Value;
use unduler_commit::ParsedCommit;

use crate::BumpType;

/// Shared state passed to all hooks during the release process.
#[derive(Debug)]
pub struct ReleaseContext {
    /// Path to the repository root.
    pub repo_path: std::path::PathBuf,

    /// The previous version (before bump).
    pub previous_version: Version,

    /// The next version (after bump).
    pub next_version: Version,

    /// The determined bump type.
    pub bump_type: BumpType,

    /// All parsed commits since last release.
    pub commits: Vec<ParsedCommit>,

    /// The generated changelog (populated after formatter runs).
    pub changelog: Option<String>,

    /// Whether this is a dry run (no actual changes).
    pub dry_run: bool,

    /// Arbitrary metadata for inter-hook communication.
    pub metadata: HashMap<String, Value>,
}

impl ReleaseContext {
    /// Creates a new release context.
    #[must_use]
    pub fn new(
        repo_path: impl Into<std::path::PathBuf>,
        previous_version: Version,
        next_version: Version,
        bump_type: BumpType,
        commits: Vec<ParsedCommit>,
    ) -> Self {
        Self {
            repo_path: repo_path.into(),
            previous_version,
            next_version,
            bump_type,
            commits,
            changelog: None,
            dry_run: false,
            metadata: HashMap::new(),
        }
    }

    /// Sets the dry run flag.
    #[must_use]
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Gets a metadata value.
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }

    /// Sets a metadata value.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Returns the version tag string (e.g., "v1.2.3").
    #[must_use]
    pub fn tag(&self, prefix: &str) -> String {
        format!("{prefix}{}", self.next_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_context() -> ReleaseContext {
        ReleaseContext::new(
            "/tmp/test-repo",
            Version::new(1, 0, 0),
            Version::new(1, 1, 0),
            BumpType::Minor,
            vec![],
        )
    }

    #[test]
    fn test_new() {
        let ctx = create_context();
        assert_eq!(ctx.repo_path.to_string_lossy(), "/tmp/test-repo");
        assert_eq!(ctx.previous_version, Version::new(1, 0, 0));
        assert_eq!(ctx.next_version, Version::new(1, 1, 0));
        assert_eq!(ctx.bump_type, BumpType::Minor);
        assert!(ctx.commits.is_empty());
        assert!(ctx.changelog.is_none());
        assert!(!ctx.dry_run);
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_new_with_string_path() {
        let ctx = ReleaseContext::new(
            String::from("/path/to/repo"),
            Version::new(0, 0, 1),
            Version::new(0, 1, 0),
            BumpType::Minor,
            vec![],
        );
        assert_eq!(ctx.repo_path.to_string_lossy(), "/path/to/repo");
    }

    #[test]
    fn test_dry_run_builder() {
        let ctx = create_context().dry_run(true);
        assert!(ctx.dry_run);
    }

    #[test]
    fn test_dry_run_false() {
        let ctx = create_context().dry_run(false);
        assert!(!ctx.dry_run);
    }

    #[test]
    fn test_get_metadata_none() {
        let ctx = create_context();
        assert!(ctx.get_metadata("key").is_none());
    }

    #[test]
    fn test_set_and_get_metadata() {
        let mut ctx = create_context();
        ctx.set_metadata("key", json!("value"));
        assert_eq!(ctx.get_metadata("key"), Some(&json!("value")));
    }

    #[test]
    fn test_set_metadata_with_string_key() {
        let mut ctx = create_context();
        ctx.set_metadata(String::from("test"), json!(123));
        assert_eq!(ctx.get_metadata("test"), Some(&json!(123)));
    }

    #[test]
    fn test_set_metadata_complex_value() {
        let mut ctx = create_context();
        ctx.set_metadata("data", json!({"foo": "bar", "num": 42}));
        let value = ctx.get_metadata("data").unwrap();
        assert_eq!(value["foo"], "bar");
        assert_eq!(value["num"], 42);
    }

    #[test]
    fn test_set_metadata_overwrite() {
        let mut ctx = create_context();
        ctx.set_metadata("key", json!("first"));
        ctx.set_metadata("key", json!("second"));
        assert_eq!(ctx.get_metadata("key"), Some(&json!("second")));
    }

    #[test]
    fn test_tag_with_v_prefix() {
        let ctx = create_context();
        assert_eq!(ctx.tag("v"), "v1.1.0");
    }

    #[test]
    fn test_tag_with_release_prefix() {
        let ctx = create_context();
        assert_eq!(ctx.tag("release-"), "release-1.1.0");
    }

    #[test]
    fn test_tag_with_empty_prefix() {
        let ctx = create_context();
        assert_eq!(ctx.tag(""), "1.1.0");
    }

    #[test]
    fn test_debug() {
        let ctx = create_context();
        let debug = format!("{ctx:?}");
        assert!(debug.contains("ReleaseContext"));
        assert!(debug.contains("previous_version"));
    }

    #[test]
    fn test_with_commits() {
        let commit = ParsedCommit::builder("abc123", "feat")
            .message("test")
            .build();
        let ctx = ReleaseContext::new(
            "/tmp",
            Version::new(1, 0, 0),
            Version::new(1, 1, 0),
            BumpType::Minor,
            vec![commit],
        );
        assert_eq!(ctx.commits.len(), 1);
    }

    #[test]
    fn test_changelog_default_none() {
        let ctx = create_context();
        assert!(ctx.changelog.is_none());
    }

    #[test]
    fn test_changelog_set() {
        let mut ctx = create_context();
        ctx.changelog = Some("# Changelog".to_string());
        assert_eq!(ctx.changelog, Some("# Changelog".to_string()));
    }
}
