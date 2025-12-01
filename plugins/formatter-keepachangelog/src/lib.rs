//! Keep a Changelog formatter plugin.

use std::collections::HashMap;
use std::fmt::Write;

use unduler_commit::ParsedCommit;
use unduler_plugin::{ChangelogFormatter, FormatterConfig, Plugin, Release};

/// Keep a Changelog formatter.
///
/// Formats changelog following the [Keep a Changelog](https://keepachangelog.com/) convention.
pub struct KeepAChangelogFormatter;

impl KeepAChangelogFormatter {
    /// Creates a new formatter.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Groups commits by type.
    fn group_by_type(commits: &[ParsedCommit]) -> HashMap<String, Vec<&ParsedCommit>> {
        let mut groups: HashMap<String, Vec<&ParsedCommit>> = HashMap::new();

        for commit in commits {
            groups
                .entry(commit.r#type.clone())
                .or_default()
                .push(commit);
        }

        groups
    }

    /// Returns the display label for a commit type.
    fn type_label(commit_type: &str, config: &FormatterConfig) -> String {
        config
            .type_labels
            .get(commit_type)
            .cloned()
            .unwrap_or_else(|| Self::default_label(commit_type))
    }

    /// Returns the default label for a commit type.
    fn default_label(commit_type: &str) -> String {
        match commit_type {
            "feat" => "Added".to_string(),
            "fix" => "Fixed".to_string(),
            "docs" => "Documentation".to_string(),
            "style" => "Styling".to_string(),
            "refactor" => "Changed".to_string(),
            "perf" => "Performance".to_string(),
            "test" => "Testing".to_string(),
            "build" | "ci" => "Build".to_string(),
            "chore" => "Maintenance".to_string(),
            "revert" => "Reverted".to_string(),
            "deps" => "Dependencies".to_string(),
            "security" => "Security".to_string(),
            "breaking" => "Breaking Changes".to_string(),
            _ => commit_type.to_string(),
        }
    }

    /// Order for displaying sections.
    fn section_order() -> Vec<&'static str> {
        vec![
            "breaking", "security", "feat", "fix", "perf", "refactor", "docs", "style", "test",
            "build", "ci", "deps", "chore", "revert",
        ]
    }
}

impl Default for KeepAChangelogFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for KeepAChangelogFormatter {
    fn name(&self) -> &'static str {
        "keepachangelog"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        "Formats changelog following Keep a Changelog convention"
    }
}

impl ChangelogFormatter for KeepAChangelogFormatter {
    fn format(&self, release: &Release, config: &FormatterConfig) -> String {
        let mut output = String::new();

        // Header
        let date = release.date.format("%Y-%m-%d");
        _ = writeln!(output, "## [{}] - {}\n", release.version, date);

        // Group commits
        let groups = Self::group_by_type(&release.commits);

        // Output in order
        for commit_type in Self::section_order() {
            if let Some(commits) = groups.get(commit_type) {
                let label = Self::type_label(commit_type, config);
                _ = writeln!(output, "### {label}\n");

                for commit in commits {
                    let scope = commit
                        .scope
                        .as_ref()
                        .map(|s| format!("**{s}:** "))
                        .unwrap_or_default();

                    let hash = if config.include_hashes {
                        format!(" ({})", &commit.hash[..7.min(commit.hash.len())])
                    } else {
                        String::new()
                    };

                    let author = if config.include_authors {
                        format!(" - @{}", commit.author)
                    } else {
                        String::new()
                    };

                    _ = writeln!(output, "- {scope}{}{hash}{author}", commit.message);
                }

                output.push('\n');
            }
        }

        // Handle unknown types
        for (commit_type, commits) in &groups {
            if !Self::section_order().contains(&commit_type.as_str()) {
                let label = Self::type_label(commit_type, config);
                _ = writeln!(output, "### {label}\n");

                for commit in commits {
                    _ = writeln!(output, "- {}", commit.message);
                }

                output.push('\n');
            }
        }

        // Comparison link
        if let (Some(prev), Some(repo_url)) = (&release.previous_version, &release.repository_url) {
            _ = writeln!(
                output,
                "[{}]: {}/compare/v{}...v{}",
                release.version, repo_url, prev, release.version
            );
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use semver::Version;

    fn make_commit(commit_type: &str, message: &str) -> ParsedCommit {
        ParsedCommit::builder("abc1234567890", commit_type)
            .message(message)
            .author("testuser")
            .build()
    }

    fn make_commit_with_scope(commit_type: &str, scope: &str, message: &str) -> ParsedCommit {
        ParsedCommit::builder("abc1234567890", commit_type)
            .scope(scope)
            .message(message)
            .author("testuser")
            .build()
    }

    #[test]
    fn test_new() {
        let formatter = KeepAChangelogFormatter::new();
        assert_eq!(formatter.name(), "keepachangelog");
    }

    #[test]
    fn test_default() {
        let formatter = KeepAChangelogFormatter;
        assert_eq!(formatter.name(), "keepachangelog");
    }

    #[test]
    fn test_plugin_name() {
        let formatter = KeepAChangelogFormatter::new();
        assert_eq!(formatter.name(), "keepachangelog");
    }

    #[test]
    fn test_plugin_version() {
        let formatter = KeepAChangelogFormatter::new();
        assert_eq!(formatter.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_plugin_description() {
        let formatter = KeepAChangelogFormatter::new();
        assert_eq!(
            formatter.description(),
            "Formats changelog following Keep a Changelog convention"
        );
    }

    #[test]
    fn test_extension() {
        let formatter = KeepAChangelogFormatter::new();
        assert_eq!(formatter.extension(), "md");
    }

    #[test]
    fn test_basic_format() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![
            make_commit("feat", "add new feature"),
            make_commit("fix", "resolve bug"),
        ];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(output.contains("## [1.0.0]"));
        assert!(output.contains("### Added"));
        assert!(output.contains("- add new feature"));
        assert!(output.contains("### Fixed"));
        assert!(output.contains("- resolve bug"));
    }

    #[test]
    fn test_format_with_scope() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![make_commit_with_scope("feat", "api", "add endpoint")];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(output.contains("**api:** add endpoint"));
    }

    #[test]
    fn test_format_with_hashes() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![make_commit("feat", "add feature")];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let config = FormatterConfig {
            include_hashes: true,
            ..Default::default()
        };
        let output = formatter.format(&release, &config);

        assert!(output.contains("(abc1234)"));
    }

    #[test]
    fn test_format_with_authors() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![make_commit("feat", "add feature")];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let config = FormatterConfig {
            include_authors: true,
            ..Default::default()
        };
        let output = formatter.format(&release, &config);

        assert!(output.contains("- @testuser"));
    }

    #[test]
    fn test_format_with_comparison_link() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![make_commit("feat", "add feature")];

        let release = Release::new(Version::new(1, 1, 0), Utc::now(), commits)
            .with_previous_version(Version::new(1, 0, 0))
            .with_repository_url("https://github.com/user/repo");

        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(output.contains("[1.1.0]: https://github.com/user/repo/compare/v1.0.0...v1.1.0"));
    }

    #[test]
    fn test_format_no_comparison_without_previous() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![make_commit("feat", "add feature")];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits)
            .with_repository_url("https://github.com/user/repo");

        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(!output.contains("compare"));
    }

    #[test]
    fn test_format_no_comparison_without_repo_url() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![make_commit("feat", "add feature")];

        let release = Release::new(Version::new(1, 1, 0), Utc::now(), commits)
            .with_previous_version(Version::new(1, 0, 0));

        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(!output.contains("compare"));
    }

    #[test]
    fn test_format_with_custom_type_labels() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![make_commit("feat", "add feature")];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let mut type_labels = HashMap::new();
        type_labels.insert("feat".to_string(), "New Features".to_string());
        let config = FormatterConfig {
            type_labels,
            ..Default::default()
        };
        let output = formatter.format(&release, &config);

        assert!(output.contains("### New Features"));
        assert!(!output.contains("### Added"));
    }

    #[test]
    fn test_default_labels_all_types() {
        assert_eq!(KeepAChangelogFormatter::default_label("feat"), "Added");
        assert_eq!(KeepAChangelogFormatter::default_label("fix"), "Fixed");
        assert_eq!(
            KeepAChangelogFormatter::default_label("docs"),
            "Documentation"
        );
        assert_eq!(KeepAChangelogFormatter::default_label("style"), "Styling");
        assert_eq!(
            KeepAChangelogFormatter::default_label("refactor"),
            "Changed"
        );
        assert_eq!(
            KeepAChangelogFormatter::default_label("perf"),
            "Performance"
        );
        assert_eq!(KeepAChangelogFormatter::default_label("test"), "Testing");
        assert_eq!(KeepAChangelogFormatter::default_label("build"), "Build");
        assert_eq!(KeepAChangelogFormatter::default_label("ci"), "Build");
        assert_eq!(
            KeepAChangelogFormatter::default_label("chore"),
            "Maintenance"
        );
        assert_eq!(KeepAChangelogFormatter::default_label("revert"), "Reverted");
        assert_eq!(
            KeepAChangelogFormatter::default_label("deps"),
            "Dependencies"
        );
        assert_eq!(
            KeepAChangelogFormatter::default_label("security"),
            "Security"
        );
        assert_eq!(
            KeepAChangelogFormatter::default_label("breaking"),
            "Breaking Changes"
        );
        assert_eq!(KeepAChangelogFormatter::default_label("unknown"), "unknown");
    }

    #[test]
    fn test_format_all_known_types() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![
            make_commit("breaking", "break api"),
            make_commit("security", "fix vuln"),
            make_commit("feat", "add feature"),
            make_commit("fix", "fix bug"),
            make_commit("perf", "improve speed"),
            make_commit("refactor", "refactor code"),
            make_commit("docs", "update docs"),
            make_commit("style", "format code"),
            make_commit("test", "add tests"),
            make_commit("build", "update build"),
            make_commit("ci", "update ci"),
            make_commit("deps", "update deps"),
            make_commit("chore", "cleanup"),
            make_commit("revert", "revert change"),
        ];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(output.contains("### Breaking Changes"));
        assert!(output.contains("### Security"));
        assert!(output.contains("### Added"));
        assert!(output.contains("### Fixed"));
        assert!(output.contains("### Performance"));
        assert!(output.contains("### Changed"));
        assert!(output.contains("### Documentation"));
        assert!(output.contains("### Styling"));
        assert!(output.contains("### Testing"));
        assert!(output.contains("### Build"));
        assert!(output.contains("### Dependencies"));
        assert!(output.contains("### Maintenance"));
        assert!(output.contains("### Reverted"));
    }

    #[test]
    fn test_format_unknown_type() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![make_commit("custom", "custom change")];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(output.contains("### custom"));
        assert!(output.contains("- custom change"));
    }

    #[test]
    fn test_format_multiple_unknown_types() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![
            make_commit("custom1", "first custom"),
            make_commit("custom2", "second custom"),
        ];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(output.contains("### custom1"));
        assert!(output.contains("- first custom"));
        assert!(output.contains("### custom2"));
        assert!(output.contains("- second custom"));
    }

    #[test]
    fn test_format_empty_commits() {
        let formatter = KeepAChangelogFormatter::new();
        let commits: Vec<ParsedCommit> = vec![];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(output.contains("## [1.0.0]"));
        assert!(!output.contains("###"));
    }

    #[test]
    fn test_format_multiple_commits_same_type() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![
            make_commit("feat", "first feature"),
            make_commit("feat", "second feature"),
            make_commit("feat", "third feature"),
        ];

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), commits);
        let output = formatter.format(&release, &FormatterConfig::default());

        assert!(output.contains("- first feature"));
        assert!(output.contains("- second feature"));
        assert!(output.contains("- third feature"));
    }

    #[test]
    fn test_group_by_type() {
        let commits = vec![
            make_commit("feat", "feature 1"),
            make_commit("fix", "fix 1"),
            make_commit("feat", "feature 2"),
        ];

        let groups = KeepAChangelogFormatter::group_by_type(&commits);

        assert_eq!(groups.get("feat").map(Vec::len), Some(2));
        assert_eq!(groups.get("fix").map(Vec::len), Some(1));
    }

    #[test]
    fn test_section_order() {
        let order = KeepAChangelogFormatter::section_order();

        assert_eq!(order[0], "breaking");
        assert_eq!(order[1], "security");
        assert_eq!(order[2], "feat");
        assert!(order.contains(&"fix"));
        assert!(order.contains(&"perf"));
    }

    #[test]
    fn test_format_with_short_hash() {
        let formatter = KeepAChangelogFormatter::new();
        let commit = ParsedCommit::builder("abc", "feat")
            .message("short hash")
            .author("user")
            .build();

        let release = Release::new(Version::new(1, 0, 0), Utc::now(), vec![commit]);
        let config = FormatterConfig {
            include_hashes: true,
            ..Default::default()
        };
        let output = formatter.format(&release, &config);

        assert!(output.contains("(abc)"));
    }

    #[test]
    fn test_format_full_options() {
        let formatter = KeepAChangelogFormatter::new();
        let commits = vec![make_commit_with_scope("feat", "api", "add endpoint")];

        let release = Release::new(Version::new(1, 1, 0), Utc::now(), commits)
            .with_previous_version(Version::new(1, 0, 0))
            .with_repository_url("https://github.com/user/repo");

        let config = FormatterConfig {
            include_hashes: true,
            include_authors: true,
            ..Default::default()
        };
        let output = formatter.format(&release, &config);

        assert!(output.contains("**api:**"));
        assert!(output.contains("(abc1234)"));
        assert!(output.contains("- @testuser"));
        assert!(output.contains("compare/v1.0.0...v1.1.0"));
    }
}
