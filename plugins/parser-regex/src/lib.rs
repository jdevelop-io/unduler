//! Custom regex parser plugin.

use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};
use unduler_commit::{ParsedCommit, RawCommit};
use unduler_plugin::{CommitParser, Plugin};

/// Configuration for the regex parser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexParserConfig {
    /// The regex pattern with named capture groups.
    pub pattern: String,
    /// Mapping of capture group names to commit fields.
    pub mapping: FieldMapping,
    /// Optional validation rules.
    #[serde(default)]
    pub validation: HashMap<String, Vec<String>>,
}

/// Mapping of capture group names to commit fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Capture group for commit type.
    #[serde(default = "default_type")]
    pub r#type: String,
    /// Capture group for scope.
    pub scope: Option<String>,
    /// Capture group for message.
    #[serde(default = "default_message")]
    pub message: String,
    /// Additional fields to capture into metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_type() -> String {
    "type".to_string()
}

fn default_message() -> String {
    "message".to_string()
}

/// Custom regex parser.
pub struct RegexParser {
    regex: Regex,
    config: RegexParserConfig,
}

impl RegexParser {
    /// Creates a new regex parser with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the regex pattern is invalid.
    pub fn new(config: RegexParserConfig) -> Result<Self, regex::Error> {
        let regex = Regex::new(&config.pattern)?;
        Ok(Self { regex, config })
    }

    /// Validates a captured value against validation rules.
    fn validate(&self, field: &str, value: &str) -> bool {
        if let Some(allowed) = self.config.validation.get(field) {
            allowed.iter().any(|v| v == value)
        } else {
            true
        }
    }
}

impl Plugin for RegexParser {
    fn name(&self) -> &'static str {
        "regex"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        "Parses commits using custom regex patterns"
    }
}

impl CommitParser for RegexParser {
    fn parse(&self, raw: &RawCommit) -> Option<ParsedCommit> {
        let subject = raw.subject();
        let captures = self.regex.captures(subject)?;

        // Extract type
        let commit_type = captures
            .name(&self.config.mapping.r#type)
            .map(|m| m.as_str().to_string())?;

        // Validate type
        if !self.validate("type", &commit_type) {
            return None;
        }

        // Extract scope
        let scope = self
            .config
            .mapping
            .scope
            .as_ref()
            .and_then(|name| captures.name(name))
            .map(|m| m.as_str().to_string());

        // Extract message
        let message = captures
            .name(&self.config.mapping.message)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        // Build commit
        let mut builder = ParsedCommit::builder(&raw.hash, commit_type)
            .message(message)
            .author(&raw.author)
            .date(raw.date);

        if let Some(s) = scope {
            builder = builder.scope(s);
        }

        // Extract metadata
        for (field, group_name) in &self.config.mapping.metadata {
            if let Some(m) = captures.name(group_name) {
                builder = builder.metadata(field, m.as_str());
            }
        }

        Some(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_raw(message: &str) -> RawCommit {
        RawCommit::new("abc123", message, "Test", "test@test.com", Utc::now())
    }

    fn simple_config() -> RegexParserConfig {
        RegexParserConfig {
            pattern: r"^(?P<type>\w+):\s+(?P<message>.+)$".to_string(),
            mapping: FieldMapping {
                r#type: "type".to_string(),
                scope: None,
                message: "message".to_string(),
                metadata: HashMap::new(),
            },
            validation: HashMap::new(),
        }
    }

    fn conventional_config() -> RegexParserConfig {
        RegexParserConfig {
            pattern: r"^(?P<type>\w+)(?:\((?P<scope>\w+)\))?:\s+(?P<message>.+)$".to_string(),
            mapping: FieldMapping {
                r#type: "type".to_string(),
                scope: Some("scope".to_string()),
                message: "message".to_string(),
                metadata: HashMap::new(),
            },
            validation: HashMap::new(),
        }
    }

    #[test]
    fn test_new_valid_regex() {
        let config = simple_config();
        let parser = RegexParser::new(config);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_new_invalid_regex() {
        let config = RegexParserConfig {
            pattern: r"^(?P<type".to_string(), // Invalid regex
            mapping: FieldMapping::default(),
            validation: HashMap::new(),
        };
        let parser = RegexParser::new(config);
        assert!(parser.is_err());
    }

    #[test]
    fn test_plugin_name() {
        let parser = RegexParser::new(simple_config()).unwrap();
        assert_eq!(parser.name(), "regex");
    }

    #[test]
    fn test_plugin_version() {
        let parser = RegexParser::new(simple_config()).unwrap();
        assert_eq!(parser.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_plugin_description() {
        let parser = RegexParser::new(simple_config()).unwrap();
        assert_eq!(
            parser.description(),
            "Parses commits using custom regex patterns"
        );
    }

    #[test]
    fn test_parse_simple() {
        let parser = RegexParser::new(simple_config()).unwrap();
        let raw = make_raw("feat: add new feature");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert_eq!(parsed.message, "add new feature");
        assert!(parsed.scope.is_none());
    }

    #[test]
    fn test_parse_with_scope() {
        let parser = RegexParser::new(conventional_config()).unwrap();
        let raw = make_raw("feat(api): add endpoint");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert_eq!(parsed.scope.as_deref(), Some("api"));
        assert_eq!(parsed.message, "add endpoint");
    }

    #[test]
    fn test_parse_without_scope() {
        let parser = RegexParser::new(conventional_config()).unwrap();
        let raw = make_raw("fix: resolve bug");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "fix");
        assert!(parsed.scope.is_none());
        assert_eq!(parsed.message, "resolve bug");
    }

    #[test]
    fn test_parse_no_match() {
        let parser = RegexParser::new(simple_config()).unwrap();
        let raw = make_raw("invalid commit message");
        let parsed = parser.parse(&raw);

        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_empty_message() {
        let parser = RegexParser::new(simple_config()).unwrap();
        let raw = make_raw("");
        let parsed = parser.parse(&raw);

        assert!(parsed.is_none());
    }

    #[test]
    fn test_jira_style() {
        let config = RegexParserConfig {
            pattern: r"^(?P<ticket>[A-Z]+-\d+)\s+(?P<type>\w+)(?:\((?P<scope>\w+)\))?:\s+(?P<message>.+)$".to_string(),
            mapping: FieldMapping {
                r#type: "type".to_string(),
                scope: Some("scope".to_string()),
                message: "message".to_string(),
                metadata: [("ticket".to_string(), "ticket".to_string())].into(),
            },
            validation: HashMap::new(),
        };

        let parser = RegexParser::new(config).unwrap();
        let raw = make_raw("PROJ-123 feat(api): add endpoint");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert_eq!(parsed.scope.as_deref(), Some("api"));
        assert_eq!(parsed.message, "add endpoint");
        assert_eq!(
            parsed.metadata.get("ticket").map(String::as_str),
            Some("PROJ-123")
        );
    }

    #[test]
    fn test_validation_allowed_types() {
        let config = RegexParserConfig {
            pattern: r"^(?P<type>\w+):\s+(?P<message>.+)$".to_string(),
            mapping: FieldMapping {
                r#type: "type".to_string(),
                scope: None,
                message: "message".to_string(),
                metadata: HashMap::new(),
            },
            validation: [(
                "type".to_string(),
                vec!["feat".to_string(), "fix".to_string()],
            )]
            .into(),
        };

        let parser = RegexParser::new(config).unwrap();

        // Valid type
        let raw = make_raw("feat: add feature");
        assert!(parser.parse(&raw).is_some());

        // Invalid type
        let raw = make_raw("chore: cleanup");
        assert!(parser.parse(&raw).is_none());
    }

    #[test]
    fn test_validation_empty_allowed() {
        let config = RegexParserConfig {
            pattern: r"^(?P<type>\w+):\s+(?P<message>.+)$".to_string(),
            mapping: FieldMapping::default(),
            validation: [("type".to_string(), vec![])].into(),
        };

        let parser = RegexParser::new(config).unwrap();
        let raw = make_raw("feat: add feature");

        assert!(parser.parse(&raw).is_none());
    }

    #[test]
    fn test_validate_no_rules() {
        let parser = RegexParser::new(simple_config()).unwrap();
        assert!(parser.validate("type", "any_value"));
    }

    #[test]
    fn test_validate_with_rules_pass() {
        let config = RegexParserConfig {
            pattern: r"^(?P<type>\w+):\s+(?P<message>.+)$".to_string(),
            mapping: FieldMapping::default(),
            validation: [(
                "type".to_string(),
                vec!["feat".to_string(), "fix".to_string()],
            )]
            .into(),
        };

        let parser = RegexParser::new(config).unwrap();
        assert!(parser.validate("type", "feat"));
        assert!(parser.validate("type", "fix"));
    }

    #[test]
    fn test_validate_with_rules_fail() {
        let config = RegexParserConfig {
            pattern: r"^(?P<type>\w+):\s+(?P<message>.+)$".to_string(),
            mapping: FieldMapping::default(),
            validation: [(
                "type".to_string(),
                vec!["feat".to_string(), "fix".to_string()],
            )]
            .into(),
        };

        let parser = RegexParser::new(config).unwrap();
        assert!(!parser.validate("type", "chore"));
    }

    #[test]
    fn test_multiple_metadata_fields() {
        let config = RegexParserConfig {
            pattern:
                r"^(?P<ticket>[A-Z]+-\d+)\s+(?P<priority>P\d)\s+(?P<type>\w+):\s+(?P<message>.+)$"
                    .to_string(),
            mapping: FieldMapping {
                r#type: "type".to_string(),
                scope: None,
                message: "message".to_string(),
                metadata: [
                    ("ticket".to_string(), "ticket".to_string()),
                    ("priority".to_string(), "priority".to_string()),
                ]
                .into(),
            },
            validation: HashMap::new(),
        };

        let parser = RegexParser::new(config).unwrap();
        let raw = make_raw("PROJ-123 P1 feat: add feature");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(
            parsed.metadata.get("ticket").map(String::as_str),
            Some("PROJ-123")
        );
        assert_eq!(
            parsed.metadata.get("priority").map(String::as_str),
            Some("P1")
        );
    }

    #[test]
    fn test_preserves_author_and_date() {
        let parser = RegexParser::new(simple_config()).unwrap();
        let now = Utc::now();
        let raw = RawCommit::new("hash123", "feat: test", "John Doe", "john@example.com", now);
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.author, "John Doe");
        assert_eq!(parsed.date, now);
        assert_eq!(parsed.hash, "hash123");
    }

    #[test]
    fn test_default_field_mapping() {
        let mapping = FieldMapping::default();
        // Default uses empty strings, serde default functions are for deserialization only
        assert_eq!(mapping.r#type, "");
        assert_eq!(mapping.message, "");
        assert!(mapping.scope.is_none());
        assert!(mapping.metadata.is_empty());
    }

    #[test]
    fn test_missing_type_capture_group() {
        let config = RegexParserConfig {
            pattern: r"^(?P<message>.+)$".to_string(),
            mapping: FieldMapping {
                r#type: "type".to_string(),
                scope: None,
                message: "message".to_string(),
                metadata: HashMap::new(),
            },
            validation: HashMap::new(),
        };

        let parser = RegexParser::new(config).unwrap();
        let raw = make_raw("some message");
        let parsed = parser.parse(&raw);

        assert!(parsed.is_none());
    }

    #[test]
    fn test_missing_message_capture_group() {
        let config = RegexParserConfig {
            pattern: r"^(?P<type>\w+):$".to_string(),
            mapping: FieldMapping {
                r#type: "type".to_string(),
                scope: None,
                message: "message".to_string(),
                metadata: HashMap::new(),
            },
            validation: HashMap::new(),
        };

        let parser = RegexParser::new(config).unwrap();
        let raw = make_raw("feat:");
        let parsed = parser.parse(&raw);

        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap().message, "");
    }

    #[test]
    fn test_optional_metadata_not_captured() {
        let config = RegexParserConfig {
            pattern: r"^(?P<type>\w+):\s+(?P<message>.+)$".to_string(),
            mapping: FieldMapping {
                r#type: "type".to_string(),
                scope: None,
                message: "message".to_string(),
                metadata: [("ticket".to_string(), "ticket".to_string())].into(),
            },
            validation: HashMap::new(),
        };

        let parser = RegexParser::new(config).unwrap();
        let raw = make_raw("feat: no ticket");
        let parsed = parser.parse(&raw).unwrap();

        assert!(!parsed.metadata.contains_key("ticket"));
    }
}
