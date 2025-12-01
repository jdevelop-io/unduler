//! SemVer bump strategy plugin.

use unduler_commit::ParsedCommit;
use unduler_plugin::{BumpStrategy, BumpType, Plugin};

/// SemVer bump strategy.
///
/// Determines version bump based on conventional commit types:
/// - Breaking changes → Major
/// - `feat` → Minor
/// - `fix` → Patch
/// - Other → None (or configurable)
pub struct SemverBumper {
    /// Types that trigger a patch bump.
    patch_types: Vec<String>,
    /// Types that trigger a minor bump.
    minor_types: Vec<String>,
}

impl SemverBumper {
    /// Creates a new SemVer bumper with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            patch_types: vec!["fix".to_string(), "perf".to_string()],
            minor_types: vec!["feat".to_string()],
        }
    }

    /// Sets the types that trigger a patch bump.
    #[must_use]
    pub fn with_patch_types(mut self, types: Vec<String>) -> Self {
        self.patch_types = types;
        self
    }

    /// Sets the types that trigger a minor bump.
    #[must_use]
    pub fn with_minor_types(mut self, types: Vec<String>) -> Self {
        self.minor_types = types;
        self
    }
}

impl Default for SemverBumper {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SemverBumper {
    fn name(&self) -> &'static str {
        "semver"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        "Determines version bump using SemVer conventions"
    }
}

impl BumpStrategy for SemverBumper {
    fn determine(&self, commits: &[ParsedCommit]) -> BumpType {
        let mut bump = BumpType::None;

        for commit in commits {
            // Breaking changes always win
            if commit.breaking {
                return BumpType::Major;
            }

            // Check for minor bump
            if self.minor_types.contains(&commit.r#type) {
                bump = bump.max(BumpType::Minor);
            }

            // Check for patch bump
            if self.patch_types.contains(&commit.r#type) {
                bump = bump.max(BumpType::Patch);
            }
        }

        bump
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(commit_type: &str, breaking: bool) -> ParsedCommit {
        ParsedCommit::builder("abc123", commit_type)
            .breaking(breaking)
            .build()
    }

    #[test]
    fn test_breaking_change() {
        let bumper = SemverBumper::new();
        let commits = vec![
            make_commit("feat", false),
            make_commit("fix", true), // breaking
        ];

        assert_eq!(bumper.determine(&commits), BumpType::Major);
    }

    #[test]
    fn test_feature() {
        let bumper = SemverBumper::new();
        let commits = vec![make_commit("feat", false)];

        assert_eq!(bumper.determine(&commits), BumpType::Minor);
    }

    #[test]
    fn test_fix() {
        let bumper = SemverBumper::new();
        let commits = vec![make_commit("fix", false)];

        assert_eq!(bumper.determine(&commits), BumpType::Patch);
    }

    #[test]
    fn test_chore_only() {
        let bumper = SemverBumper::new();
        let commits = vec![make_commit("chore", false)];

        assert_eq!(bumper.determine(&commits), BumpType::None);
    }

    #[test]
    fn test_mixed_commits() {
        let bumper = SemverBumper::new();
        let commits = vec![
            make_commit("docs", false),
            make_commit("fix", false),
            make_commit("feat", false),
            make_commit("chore", false),
        ];

        // feat wins over fix
        assert_eq!(bumper.determine(&commits), BumpType::Minor);
    }

    #[test]
    fn test_default() {
        let bumper = SemverBumper::default();
        let commits = vec![make_commit("feat", false)];
        assert_eq!(bumper.determine(&commits), BumpType::Minor);
    }

    #[test]
    fn test_perf_triggers_patch() {
        let bumper = SemverBumper::new();
        let commits = vec![make_commit("perf", false)];
        assert_eq!(bumper.determine(&commits), BumpType::Patch);
    }

    #[test]
    fn test_with_custom_patch_types() {
        let bumper =
            SemverBumper::new().with_patch_types(vec!["docs".to_string(), "style".to_string()]);
        let commits = vec![make_commit("docs", false)];
        assert_eq!(bumper.determine(&commits), BumpType::Patch);
    }

    #[test]
    fn test_with_custom_minor_types() {
        let bumper = SemverBumper::new().with_minor_types(vec!["feature".to_string()]);
        let commits = vec![make_commit("feature", false)];
        assert_eq!(bumper.determine(&commits), BumpType::Minor);
    }

    #[test]
    fn test_with_custom_minor_overrides_feat() {
        let bumper = SemverBumper::new().with_minor_types(vec!["feature".to_string()]);
        // feat is no longer a minor type
        let commits = vec![make_commit("feat", false)];
        assert_eq!(bumper.determine(&commits), BumpType::None);
    }

    #[test]
    fn test_empty_commits() {
        let bumper = SemverBumper::new();
        let commits: Vec<ParsedCommit> = vec![];
        assert_eq!(bumper.determine(&commits), BumpType::None);
    }

    #[test]
    fn test_breaking_wins_even_with_custom_types() {
        let bumper = SemverBumper::new()
            .with_patch_types(vec![])
            .with_minor_types(vec![]);
        let commits = vec![make_commit("random", true)];
        assert_eq!(bumper.determine(&commits), BumpType::Major);
    }

    #[test]
    fn test_plugin_name() {
        let bumper = SemverBumper::new();
        assert_eq!(bumper.name(), "semver");
    }

    #[test]
    fn test_plugin_version() {
        let bumper = SemverBumper::new();
        assert_eq!(bumper.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_plugin_description() {
        let bumper = SemverBumper::new();
        assert!(!bumper.description().is_empty());
    }
}
