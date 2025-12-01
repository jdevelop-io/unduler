//! Conventional Commits parser plugin.

use regex::Regex;
use std::sync::LazyLock;
use unduler_commit::{ParsedCommit, RawCommit};
use unduler_plugin::{CommitParser, Plugin};

static CONVENTIONAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<type>\w+)(?:\((?P<scope>[^)]+)\))?(?P<breaking>!)?: (?P<message>.+)$")
        .expect("invalid regex")
});

/// Conventional Commits parser.
pub struct ConventionalParser;

impl ConventionalParser {
    /// Creates a new conventional parser.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConventionalParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ConventionalParser {
    fn name(&self) -> &'static str {
        "conventional"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        "Parses Conventional Commits format"
    }
}

impl CommitParser for ConventionalParser {
    fn parse(&self, raw: &RawCommit) -> Option<ParsedCommit> {
        let subject = raw.subject();
        let captures = CONVENTIONAL_RE.captures(subject)?;

        let commit_type = captures.name("type")?.as_str().to_string();
        let scope = captures.name("scope").map(|m| m.as_str().to_string());
        let breaking = captures.name("breaking").is_some();
        let message = captures.name("message")?.as_str().to_string();

        Some(
            ParsedCommit::builder(&raw.hash, commit_type)
                .scope(scope.unwrap_or_default())
                .message(message)
                .breaking(breaking)
                .author(&raw.author)
                .date(raw.date)
                .build(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_raw(message: &str) -> RawCommit {
        RawCommit::new("abc123", message, "Test", "test@test.com", Utc::now())
    }

    #[test]
    fn test_simple_commit() {
        let parser = ConventionalParser::new();
        let raw = make_raw("feat: add new feature");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert!(parsed.scope.is_none() || parsed.scope.as_deref() == Some(""));
        assert_eq!(parsed.message, "add new feature");
        assert!(!parsed.breaking);
    }

    #[test]
    fn test_with_scope() {
        let parser = ConventionalParser::new();
        let raw = make_raw("fix(parser): handle edge case");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "fix");
        assert_eq!(parsed.scope.as_deref(), Some("parser"));
        assert_eq!(parsed.message, "handle edge case");
    }

    #[test]
    fn test_breaking_change() {
        let parser = ConventionalParser::new();
        let raw = make_raw("feat(api)!: redesign endpoints");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert!(parsed.breaking);
    }

    #[test]
    fn test_invalid_commit() {
        let parser = ConventionalParser::new();
        let raw = make_raw("random commit message");
        assert!(parser.parse(&raw).is_none());
    }

    #[test]
    fn test_default() {
        let parser = ConventionalParser;
        let raw = make_raw("fix: bug");
        let parsed = parser.parse(&raw).unwrap();
        assert_eq!(parsed.r#type, "fix");
    }

    #[test]
    fn test_plugin_name() {
        let parser = ConventionalParser::new();
        assert_eq!(parser.name(), "conventional");
    }

    #[test]
    fn test_plugin_version() {
        let parser = ConventionalParser::new();
        assert_eq!(parser.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_plugin_description() {
        let parser = ConventionalParser::new();
        assert!(!parser.description().is_empty());
    }

    #[test]
    fn test_breaking_without_scope() {
        let parser = ConventionalParser::new();
        let raw = make_raw("feat!: breaking feature");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert!(parsed.breaking);
        assert_eq!(parsed.message, "breaking feature");
    }

    #[test]
    fn test_can_parse_valid() {
        let parser = ConventionalParser::new();
        let raw = make_raw("feat: something");
        assert!(parser.can_parse(&raw));
    }

    #[test]
    fn test_can_parse_invalid() {
        let parser = ConventionalParser::new();
        let raw = make_raw("invalid");
        assert!(!parser.can_parse(&raw));
    }

    #[test]
    fn test_preserves_author() {
        let parser = ConventionalParser::new();
        let raw = RawCommit::new(
            "hash123",
            "feat: test",
            "John Doe",
            "john@test.com",
            Utc::now(),
        );
        let parsed = parser.parse(&raw).unwrap();
        assert_eq!(parsed.author, "John Doe");
    }

    #[test]
    fn test_preserves_hash() {
        let parser = ConventionalParser::new();
        let raw = RawCommit::new("abc123def", "feat: test", "Author", "a@b.com", Utc::now());
        let parsed = parser.parse(&raw).unwrap();
        assert_eq!(parsed.hash, "abc123def");
    }

    #[test]
    fn test_various_commit_types() {
        let parser = ConventionalParser::new();

        for commit_type in [
            "docs", "style", "refactor", "perf", "test", "ci", "chore", "revert",
        ] {
            let raw = make_raw(&format!("{commit_type}: test message"));
            let parsed = parser.parse(&raw).unwrap();
            assert_eq!(parsed.r#type, commit_type);
        }
    }

    #[test]
    fn test_missing_colon() {
        let parser = ConventionalParser::new();
        let raw = make_raw("feat add feature");
        assert!(parser.parse(&raw).is_none());
    }

    #[test]
    fn test_missing_space_after_colon() {
        let parser = ConventionalParser::new();
        let raw = make_raw("feat:add feature");
        assert!(parser.parse(&raw).is_none());
    }

    #[test]
    fn test_empty_message() {
        let parser = ConventionalParser::new();
        let raw = make_raw("");
        assert!(parser.parse(&raw).is_none());
    }
}
