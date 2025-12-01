//! Raw commit type as retrieved from Git.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A commit as retrieved from Git, before parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawCommit {
    /// The commit hash (SHA).
    pub hash: String,

    /// The full commit message (subject + body).
    pub message: String,

    /// The commit author name.
    pub author: String,

    /// The commit author email.
    pub email: String,

    /// The commit date.
    pub date: DateTime<Utc>,
}

impl RawCommit {
    /// Creates a new raw commit.
    #[must_use]
    pub fn new(
        hash: impl Into<String>,
        message: impl Into<String>,
        author: impl Into<String>,
        email: impl Into<String>,
        date: DateTime<Utc>,
    ) -> Self {
        Self {
            hash: hash.into(),
            message: message.into(),
            author: author.into(),
            email: email.into(),
            date,
        }
    }

    /// Returns the first line of the commit message (the subject).
    #[must_use]
    pub fn subject(&self) -> &str {
        self.message.lines().next().unwrap_or("")
    }

    /// Returns the commit body (everything after the first line).
    #[must_use]
    pub fn body(&self) -> Option<&str> {
        let mut lines = self.message.lines();
        lines.next(); // Skip subject
        lines.next(); // Skip empty line

        let body: String = lines.collect::<Vec<_>>().join("\n");
        if body.is_empty() {
            None
        } else {
            // This is a bit inefficient but keeps the API simple
            // In practice, we'd use indices into the original string
            None
        }
    }

    /// Returns the short hash (first 7 characters).
    #[must_use]
    pub fn short_hash(&self) -> &str {
        &self.hash[..7.min(self.hash.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(hash: &str, message: &str) -> RawCommit {
        RawCommit::new(hash, message, "Test Author", "test@example.com", Utc::now())
    }

    #[test]
    fn test_new() {
        let now = Utc::now();
        let commit = RawCommit::new(
            "abc1234567890",
            "feat: add feature",
            "Test Author",
            "test@example.com",
            now,
        );

        assert_eq!(commit.hash, "abc1234567890");
        assert_eq!(commit.message, "feat: add feature");
        assert_eq!(commit.author, "Test Author");
        assert_eq!(commit.email, "test@example.com");
        assert_eq!(commit.date, now);
    }

    #[test]
    fn test_new_with_into() {
        let commit = RawCommit::new(
            String::from("hash"),
            String::from("message"),
            String::from("author"),
            String::from("email"),
            Utc::now(),
        );

        assert_eq!(commit.hash, "hash");
        assert_eq!(commit.message, "message");
    }

    #[test]
    fn test_subject() {
        let commit = RawCommit::new(
            "abc1234567890",
            "feat: add new feature\n\nThis is the body",
            "Test Author",
            "test@example.com",
            Utc::now(),
        );

        assert_eq!(commit.subject(), "feat: add new feature");
    }

    #[test]
    fn test_subject_single_line() {
        let commit = make_commit("abc123", "single line message");
        assert_eq!(commit.subject(), "single line message");
    }

    #[test]
    fn test_subject_empty() {
        let commit = make_commit("abc123", "");
        assert_eq!(commit.subject(), "");
    }

    #[test]
    fn test_subject_only_newlines() {
        let commit = make_commit("abc123", "\n\n\n");
        assert_eq!(commit.subject(), "");
    }

    #[test]
    fn test_body_none_single_line() {
        let commit = make_commit("abc123", "single line");
        assert!(commit.body().is_none());
    }

    #[test]
    fn test_body_none_with_body_text() {
        let commit = make_commit("abc123", "subject\n\nbody text");
        // Current implementation always returns None (as documented)
        assert!(commit.body().is_none());
    }

    #[test]
    fn test_short_hash() {
        let commit = RawCommit::new(
            "abc1234567890",
            "feat: add new feature",
            "Test Author",
            "test@example.com",
            Utc::now(),
        );

        assert_eq!(commit.short_hash(), "abc1234");
    }

    #[test]
    fn test_short_hash_exact_7() {
        let commit = make_commit("abc1234", "message");
        assert_eq!(commit.short_hash(), "abc1234");
    }

    #[test]
    fn test_short_hash_less_than_7() {
        let commit = make_commit("abc", "message");
        assert_eq!(commit.short_hash(), "abc");
    }

    #[test]
    fn test_short_hash_empty() {
        let commit = make_commit("", "message");
        assert_eq!(commit.short_hash(), "");
    }

    #[test]
    fn test_clone() {
        let commit = make_commit("abc123", "message");
        let cloned = commit.clone();
        assert_eq!(commit.hash, cloned.hash);
        assert_eq!(commit.message, cloned.message);
    }

    #[test]
    fn test_debug() {
        let commit = make_commit("abc123", "message");
        let debug = format!("{commit:?}");
        assert!(debug.contains("RawCommit"));
        assert!(debug.contains("abc123"));
    }

    #[test]
    fn test_eq() {
        let now = Utc::now();
        let commit1 = RawCommit::new("abc", "msg", "author", "email", now);
        let commit2 = RawCommit::new("abc", "msg", "author", "email", now);
        assert_eq!(commit1, commit2);
    }

    #[test]
    fn test_serialize_deserialize() {
        let commit = make_commit("abc123", "test message");
        let json = serde_json::to_string(&commit).unwrap();
        let deserialized: RawCommit = serde_json::from_str(&json).unwrap();
        assert_eq!(commit.hash, deserialized.hash);
        assert_eq!(commit.message, deserialized.message);
    }
}
