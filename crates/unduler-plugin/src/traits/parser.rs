//! Commit parser trait.

use unduler_commit::{ParsedCommit, RawCommit};

use super::Plugin;

/// Parses raw commit messages into structured data.
pub trait CommitParser: Plugin {
    /// Parses a raw commit into a parsed commit.
    ///
    /// Returns `None` if the commit message doesn't match the expected format.
    fn parse(&self, raw: &RawCommit) -> Option<ParsedCommit>;

    /// Returns whether this parser can handle the given commit.
    ///
    /// This is a quick check that can be used before attempting to parse.
    /// Default implementation just tries to parse and checks if it succeeds.
    fn can_parse(&self, raw: &RawCommit) -> bool {
        self.parse(raw).is_some()
    }
}
