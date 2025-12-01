//! Changelog formatter trait.

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use unduler_commit::ParsedCommit;

use super::Plugin;

/// A release to be formatted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// The release version.
    pub version: Version,

    /// The release date.
    pub date: DateTime<Utc>,

    /// The commits in this release.
    pub commits: Vec<ParsedCommit>,

    /// The previous version (for comparison links).
    pub previous_version: Option<Version>,

    /// The repository URL (for links).
    pub repository_url: Option<String>,
}

impl Release {
    /// Creates a new release.
    #[must_use]
    pub fn new(version: Version, date: DateTime<Utc>, commits: Vec<ParsedCommit>) -> Self {
        Self {
            version,
            date,
            commits,
            previous_version: None,
            repository_url: None,
        }
    }

    /// Sets the previous version.
    #[must_use]
    pub fn with_previous_version(mut self, version: Version) -> Self {
        self.previous_version = Some(version);
        self
    }

    /// Sets the repository URL.
    #[must_use]
    pub fn with_repository_url(mut self, url: impl Into<String>) -> Self {
        self.repository_url = Some(url.into());
        self
    }
}

/// Configuration for the changelog formatter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct FormatterConfig {
    /// Group commits by type.
    pub group_by_type: bool,

    /// Group commits by scope.
    pub group_by_scope: bool,

    /// Include commit hashes in output.
    pub include_hashes: bool,

    /// Include commit authors in output.
    pub include_authors: bool,

    /// Custom type labels (e.g., "feat" -> "Features").
    pub type_labels: std::collections::HashMap<String, String>,
}

/// Formats changelog output.
pub trait ChangelogFormatter: Plugin {
    /// Formats a release into a changelog string.
    fn format(&self, release: &Release, config: &FormatterConfig) -> String;

    /// Returns the file extension for the output (e.g., "md").
    fn extension(&self) -> &'static str {
        "md"
    }
}
