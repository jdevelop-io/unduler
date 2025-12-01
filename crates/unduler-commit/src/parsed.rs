//! Parsed commit type after processing by a parser plugin.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A commit after parsing by a parser plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedCommit {
    /// The commit hash (SHA).
    pub hash: String,

    /// The commit type (feat, fix, docs, etc.).
    pub r#type: String,

    /// The optional scope.
    pub scope: Option<String>,

    /// The commit message (without type and scope prefix).
    pub message: String,

    /// Whether this is a breaking change.
    pub breaking: bool,

    /// The optional emoji (for Gitmoji).
    pub emoji: Option<String>,

    /// Additional metadata from custom parsers (e.g., ticket number).
    pub metadata: HashMap<String, String>,

    /// The commit author name.
    pub author: String,

    /// The commit date.
    pub date: DateTime<Utc>,
}

impl ParsedCommit {
    /// Creates a new parsed commit builder.
    #[must_use]
    pub fn builder(hash: impl Into<String>, r#type: impl Into<String>) -> ParsedCommitBuilder {
        ParsedCommitBuilder::new(hash, r#type)
    }

    /// Returns true if this commit represents a feature.
    #[must_use]
    pub fn is_feature(&self) -> bool {
        self.r#type == "feat"
    }

    /// Returns true if this commit represents a bug fix.
    #[must_use]
    pub fn is_fix(&self) -> bool {
        self.r#type == "fix"
    }

    /// Returns true if this commit should trigger a major version bump.
    #[must_use]
    pub fn is_major(&self) -> bool {
        self.breaking
    }

    /// Returns true if this commit should trigger a minor version bump.
    #[must_use]
    pub fn is_minor(&self) -> bool {
        !self.breaking && self.is_feature()
    }

    /// Returns true if this commit should trigger a patch version bump.
    #[must_use]
    pub fn is_patch(&self) -> bool {
        !self.breaking && self.is_fix()
    }
}

/// Builder for [`ParsedCommit`].
#[derive(Debug)]
pub struct ParsedCommitBuilder {
    hash: String,
    r#type: String,
    scope: Option<String>,
    message: String,
    breaking: bool,
    emoji: Option<String>,
    metadata: HashMap<String, String>,
    author: String,
    date: DateTime<Utc>,
}

impl ParsedCommitBuilder {
    /// Creates a new builder with required fields.
    fn new(hash: impl Into<String>, r#type: impl Into<String>) -> Self {
        Self {
            hash: hash.into(),
            r#type: r#type.into(),
            scope: None,
            message: String::new(),
            breaking: false,
            emoji: None,
            metadata: HashMap::new(),
            author: String::new(),
            date: Utc::now(),
        }
    }

    /// Sets the scope.
    #[must_use]
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Sets the message.
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Sets the breaking flag.
    #[must_use]
    pub fn breaking(mut self, breaking: bool) -> Self {
        self.breaking = breaking;
        self
    }

    /// Sets the emoji.
    #[must_use]
    pub fn emoji(mut self, emoji: impl Into<String>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }

    /// Adds metadata.
    #[must_use]
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Sets the author.
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Sets the date.
    #[must_use]
    pub fn date(mut self, date: DateTime<Utc>) -> Self {
        self.date = date;
        self
    }

    /// Builds the [`ParsedCommit`].
    #[must_use]
    pub fn build(self) -> ParsedCommit {
        ParsedCommit {
            hash: self.hash,
            r#type: self.r#type,
            scope: self.scope,
            message: self.message,
            breaking: self.breaking,
            emoji: self.emoji,
            metadata: self.metadata,
            author: self.author,
            date: self.date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let commit = ParsedCommit::builder("abc123", "feat")
            .scope("api")
            .message("add new endpoint")
            .breaking(false)
            .emoji("✨")
            .author("Test")
            .build();

        assert_eq!(commit.r#type, "feat");
        assert_eq!(commit.scope, Some("api".to_string()));
        assert_eq!(commit.message, "add new endpoint");
        assert!(!commit.breaking);
        assert_eq!(commit.emoji, Some("✨".to_string()));
    }

    #[test]
    fn test_bump_detection() {
        let breaking = ParsedCommit::builder("abc123", "feat")
            .breaking(true)
            .build();
        assert!(breaking.is_major());
        assert!(!breaking.is_minor());

        let feature = ParsedCommit::builder("abc123", "feat").build();
        assert!(feature.is_minor());
        assert!(!feature.is_major());

        let fix = ParsedCommit::builder("abc123", "fix").build();
        assert!(fix.is_patch());
        assert!(!fix.is_minor());
    }
}
