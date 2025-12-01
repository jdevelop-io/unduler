//! Git error types.

use thiserror::Error;

/// Git-related errors.
#[derive(Debug, Error)]
pub enum GitError {
    /// Repository not found.
    #[error("repository not found at {0}")]
    RepoNotFound(std::path::PathBuf),

    /// Not a git repository.
    #[error("not a git repository: {0}")]
    NotARepo(std::path::PathBuf),

    /// Tag not found.
    #[error("tag not found: {0}")]
    TagNotFound(String),

    /// No commits found.
    #[error("no commits found")]
    NoCommits,

    /// Git2 error.
    #[error("git error: {0}")]
    Git2(#[from] git2::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for git operations.
pub type GitResult<T> = Result<T, GitError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_repo_not_found_display() {
        let err = GitError::RepoNotFound(PathBuf::from("/tmp/repo"));
        assert_eq!(err.to_string(), "repository not found at /tmp/repo");
    }

    #[test]
    fn test_not_a_repo_display() {
        let err = GitError::NotARepo(PathBuf::from("/tmp/not-git"));
        assert_eq!(err.to_string(), "not a git repository: /tmp/not-git");
    }

    #[test]
    fn test_tag_not_found_display() {
        let err = GitError::TagNotFound("v1.0.0".to_string());
        assert_eq!(err.to_string(), "tag not found: v1.0.0");
    }

    #[test]
    fn test_no_commits_display() {
        let err = GitError::NoCommits;
        assert_eq!(err.to_string(), "no commits found");
    }

    #[test]
    fn test_error_is_debug() {
        let err = GitError::NoCommits;
        let debug = format!("{err:?}");
        assert!(debug.contains("NoCommits"));
    }
}
