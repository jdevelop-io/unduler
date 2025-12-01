//! Plugin pipeline execution.

use tracing::info;
use unduler_commit::{ParsedCommit, RawCommit};
use unduler_plugin::{BumpStrategy, BumpType, ChangelogFormatter, CommitParser, ReleaseHook};

/// Orchestrates plugin execution.
pub struct Pipeline {
    parser: Box<dyn CommitParser>,
    bumper: Box<dyn BumpStrategy>,
    formatter: Box<dyn ChangelogFormatter>,
    hooks: Vec<Box<dyn ReleaseHook>>,
}

impl Pipeline {
    /// Creates a new pipeline with the given plugins.
    #[must_use]
    pub fn new(
        parser: Box<dyn CommitParser>,
        bumper: Box<dyn BumpStrategy>,
        formatter: Box<dyn ChangelogFormatter>,
    ) -> Self {
        Self {
            parser,
            bumper,
            formatter,
            hooks: Vec::new(),
        }
    }

    /// Adds a release hook.
    #[must_use]
    pub fn with_hook(mut self, hook: Box<dyn ReleaseHook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Parses raw commits using the configured parser.
    pub fn parse_commits(&self, raw_commits: &[RawCommit]) -> Vec<ParsedCommit> {
        raw_commits
            .iter()
            .filter_map(|raw| {
                let parsed = self.parser.parse(raw);
                if parsed.is_none() {
                    info!(
                        hash = %raw.short_hash(),
                        subject = %raw.subject(),
                        "skipping unparseable commit"
                    );
                }
                parsed
            })
            .collect()
    }

    /// Determines the bump type using the configured bumper.
    pub fn determine_bump(&self, commits: &[ParsedCommit]) -> BumpType {
        self.bumper.determine(commits)
    }

    /// Returns a reference to the formatter.
    pub fn formatter(&self) -> &dyn ChangelogFormatter {
        self.formatter.as_ref()
    }

    /// Returns a reference to the hooks.
    pub fn hooks(&self) -> &[Box<dyn ReleaseHook>] {
        &self.hooks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use unduler_plugin::{FormatterConfig, Plugin, PluginResult, Release, ReleaseContext};

    fn make_raw(hash: &str, message: &str) -> RawCommit {
        RawCommit::new(hash, message, "Test Author", "test@example.com", Utc::now())
    }

    // Mock parser that parses commits starting with "feat:" or "fix:"
    struct MockParser;

    impl Plugin for MockParser {
        fn name(&self) -> &'static str {
            "mock-parser"
        }
        fn version(&self) -> &'static str {
            "1.0.0"
        }
    }

    impl CommitParser for MockParser {
        fn parse(&self, raw: &RawCommit) -> Option<ParsedCommit> {
            let message = &raw.message;
            let (commit_type, rest) = message
                .strip_prefix("feat:")
                .map(|r| ("feat", r))
                .or_else(|| message.strip_prefix("fix:").map(|r| ("fix", r)))?;

            Some(
                ParsedCommit::builder(&raw.hash, commit_type)
                    .message(rest.trim())
                    .build(),
            )
        }
    }

    // Mock bumper that returns Minor for feat, Patch for fix
    struct MockBumper;

    impl Plugin for MockBumper {
        fn name(&self) -> &'static str {
            "mock-bumper"
        }
        fn version(&self) -> &'static str {
            "1.0.0"
        }
    }

    impl BumpStrategy for MockBumper {
        fn determine(&self, commits: &[ParsedCommit]) -> BumpType {
            let mut bump = BumpType::None;
            for commit in commits {
                let commit_bump = match commit.r#type.as_str() {
                    "feat" => BumpType::Minor,
                    "fix" => BumpType::Patch,
                    _ => BumpType::None,
                };
                bump = bump.max(commit_bump);
            }
            bump
        }
    }

    // Mock formatter
    struct MockFormatter;

    impl Plugin for MockFormatter {
        fn name(&self) -> &'static str {
            "mock-formatter"
        }
        fn version(&self) -> &'static str {
            "1.0.0"
        }
    }

    impl ChangelogFormatter for MockFormatter {
        fn format(&self, release: &Release, _config: &FormatterConfig) -> String {
            format!("# {}\n", release.version)
        }

        fn extension(&self) -> &'static str {
            "md"
        }
    }

    // Mock hook for testing
    struct MockHook {
        name: &'static str,
    }

    impl Plugin for MockHook {
        fn name(&self) -> &'static str {
            self.name
        }
        fn version(&self) -> &'static str {
            "1.0.0"
        }
    }

    impl ReleaseHook for MockHook {
        fn on_pre_bump(&self, _ctx: &mut ReleaseContext) -> PluginResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_new() {
        let pipeline = Pipeline::new(
            Box::new(MockParser),
            Box::new(MockBumper),
            Box::new(MockFormatter),
        );
        assert!(pipeline.hooks.is_empty());
    }

    #[test]
    fn test_with_hook() {
        let pipeline = Pipeline::new(
            Box::new(MockParser),
            Box::new(MockBumper),
            Box::new(MockFormatter),
        )
        .with_hook(Box::new(MockHook { name: "hook1" }))
        .with_hook(Box::new(MockHook { name: "hook2" }));

        assert_eq!(pipeline.hooks().len(), 2);
        assert_eq!(pipeline.hooks()[0].name(), "hook1");
        assert_eq!(pipeline.hooks()[1].name(), "hook2");
    }

    #[test]
    fn test_parse_commits_all_valid() {
        let pipeline = Pipeline::new(
            Box::new(MockParser),
            Box::new(MockBumper),
            Box::new(MockFormatter),
        );

        let raw_commits = vec![
            make_raw("abc123", "feat: add feature"),
            make_raw("def456", "fix: fix bug"),
        ];

        let parsed = pipeline.parse_commits(&raw_commits);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].r#type, "feat");
        assert_eq!(parsed[1].r#type, "fix");
    }

    #[test]
    fn test_parse_commits_skips_invalid() {
        let pipeline = Pipeline::new(
            Box::new(MockParser),
            Box::new(MockBumper),
            Box::new(MockFormatter),
        );

        let raw_commits = vec![
            make_raw("abc123", "feat: add feature"),
            make_raw("def456", "invalid commit message"),
            make_raw("ghi789", "fix: fix bug"),
        ];

        let parsed = pipeline.parse_commits(&raw_commits);
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_parse_commits_empty() {
        let pipeline = Pipeline::new(
            Box::new(MockParser),
            Box::new(MockBumper),
            Box::new(MockFormatter),
        );

        let raw_commits: Vec<RawCommit> = vec![];
        let parsed = pipeline.parse_commits(&raw_commits);
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_determine_bump() {
        let pipeline = Pipeline::new(
            Box::new(MockParser),
            Box::new(MockBumper),
            Box::new(MockFormatter),
        );

        let commits = vec![
            ParsedCommit::builder("abc123", "feat")
                .message("new feature")
                .build(),
            ParsedCommit::builder("def456", "fix")
                .message("bug fix")
                .build(),
        ];

        let bump = pipeline.determine_bump(&commits);
        assert_eq!(bump, BumpType::Minor);
    }

    #[test]
    fn test_determine_bump_fix_only() {
        let pipeline = Pipeline::new(
            Box::new(MockParser),
            Box::new(MockBumper),
            Box::new(MockFormatter),
        );

        let commits = vec![
            ParsedCommit::builder("abc123", "fix")
                .message("bug fix")
                .build(),
        ];

        let bump = pipeline.determine_bump(&commits);
        assert_eq!(bump, BumpType::Patch);
    }

    #[test]
    fn test_formatter() {
        let pipeline = Pipeline::new(
            Box::new(MockParser),
            Box::new(MockBumper),
            Box::new(MockFormatter),
        );

        assert_eq!(pipeline.formatter().name(), "mock-formatter");
        assert_eq!(pipeline.formatter().extension(), "md");
    }

    #[test]
    fn test_hooks_empty() {
        let pipeline = Pipeline::new(
            Box::new(MockParser),
            Box::new(MockBumper),
            Box::new(MockFormatter),
        );

        assert!(pipeline.hooks().is_empty());
    }
}
