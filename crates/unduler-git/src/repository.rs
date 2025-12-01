//! Git repository wrapper.

use std::path::Path;

use chrono::{TimeZone, Utc};
use git2::Repository as Git2Repo;
use unduler_commit::RawCommit;

use crate::{GitError, GitResult};

/// A Git repository wrapper.
pub struct Repository {
    inner: Git2Repo,
}

impl Repository {
    /// Opens a repository at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not a valid Git repository.
    pub fn open(path: impl AsRef<Path>) -> GitResult<Self> {
        let path = path.as_ref();
        let inner = Git2Repo::open(path).map_err(|_| GitError::NotARepo(path.to_path_buf()))?;
        Ok(Self { inner })
    }

    /// Discovers the repository from the current directory.
    ///
    /// # Errors
    ///
    /// Returns an error if no repository is found.
    pub fn discover() -> GitResult<Self> {
        let inner = Git2Repo::discover(".")?;
        Ok(Self { inner })
    }

    /// Returns the repository root path.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.inner.workdir().unwrap_or_else(|| self.inner.path())
    }

    /// Returns all tags in the repository.
    ///
    /// # Errors
    ///
    /// Returns an error if tags cannot be read.
    pub fn tags(&self) -> GitResult<Vec<String>> {
        let tags = self.inner.tag_names(None)?;
        Ok(tags.iter().flatten().map(String::from).collect())
    }

    /// Returns commits since the given tag.
    ///
    /// If tag is `None`, returns all commits.
    ///
    /// # Errors
    ///
    /// Returns an error if commits cannot be read.
    pub fn commits_since(&self, tag: Option<&str>) -> GitResult<Vec<RawCommit>> {
        let mut revwalk = self.inner.revwalk()?;
        revwalk.push_head()?;

        // If we have a tag, stop at it
        if let Some(tag_name) = tag {
            let tag_ref = self
                .inner
                .resolve_reference_from_short_name(tag_name)
                .map_err(|_| GitError::TagNotFound(tag_name.to_string()))?;
            let tag_oid = tag_ref
                .target()
                .ok_or_else(|| GitError::TagNotFound(tag_name.to_string()))?;
            revwalk.hide(tag_oid)?;
        }

        let mut commits = Vec::new();
        for oid in revwalk {
            let oid = oid?;
            let commit = self.inner.find_commit(oid)?;

            let message = commit.message().unwrap_or("").to_string();
            let author = commit.author();
            let time = commit.time();

            let raw = RawCommit::new(
                oid.to_string(),
                message,
                author.name().unwrap_or("Unknown"),
                author.email().unwrap_or(""),
                Utc.timestamp_opt(time.seconds(), 0)
                    .single()
                    .unwrap_or_else(Utc::now),
            );

            commits.push(raw);
        }

        Ok(commits)
    }

    /// Returns the latest tag matching a version pattern.
    ///
    /// # Errors
    ///
    /// Returns an error if tags cannot be read.
    pub fn latest_version_tag(&self, prefix: &str) -> GitResult<Option<String>> {
        let tags = self.tags()?;

        // Find tags matching the prefix and parse as semver
        let mut version_tags: Vec<_> = tags
            .into_iter()
            .filter(|t| t.starts_with(prefix))
            .filter_map(|t| {
                let version_str = t.strip_prefix(prefix)?;
                semver::Version::parse(version_str).ok().map(|v| (t, v))
            })
            .collect();

        // Sort by version descending
        version_tags.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(version_tags.into_iter().next().map(|(tag, _)| tag))
    }

    /// Creates a new tag.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag cannot be created.
    pub fn create_tag(&self, name: &str, message: &str) -> GitResult<()> {
        let head = self.inner.head()?;
        let commit = head.peel_to_commit()?;
        let sig = self.inner.signature()?;

        self.inner
            .tag(name, commit.as_object(), &sig, message, false)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository as Git2Repository, Signature};
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let git2_repo = Git2Repository::init(temp_dir.path()).unwrap();

        // Configure user for commits
        let mut config = git2_repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        let repo = Repository { inner: git2_repo };
        (temp_dir, repo)
    }

    fn create_commit(repo: &Repository, message: &str) -> git2::Oid {
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.inner.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.inner.find_tree(tree_id).unwrap();

        let parent = repo.inner.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit<'_>> = parent.iter().collect();

        repo.inner
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
            .unwrap()
    }

    #[test]
    fn test_open_valid_repo() {
        let (temp_dir, _repo) = create_test_repo();
        let result = Repository::open(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_invalid_path() {
        let result = Repository::open("/nonexistent/path/to/repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_not_a_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = Repository::open(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_from_unduler_repo() {
        // This test runs from within the unduler repo
        let result = Repository::discover();
        assert!(result.is_ok());
    }

    #[test]
    fn test_path() {
        let (temp_dir, repo) = create_test_repo();
        let path = repo.path();
        // Use canonicalize to resolve symlinks (macOS /var -> /private/var)
        let expected = temp_dir.path().canonicalize().unwrap();
        let actual = path.canonicalize().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tags_empty() {
        let (_temp_dir, repo) = create_test_repo();
        let tags = repo.tags().unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_tags_with_tags() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "Initial commit");

        repo.create_tag("v1.0.0", "First release").unwrap();
        repo.create_tag("v1.1.0", "Second release").unwrap();

        let tags = repo.tags().unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"v1.0.0".to_string()));
        assert!(tags.contains(&"v1.1.0".to_string()));
    }

    #[test]
    fn test_commits_since_none() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "First commit");
        create_commit(&repo, "Second commit");
        create_commit(&repo, "Third commit");

        let commits = repo.commits_since(None).unwrap();
        assert_eq!(commits.len(), 3);
    }

    #[test]
    fn test_commits_since_tag() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "First commit");
        repo.create_tag("v1.0.0", "Release 1.0.0").unwrap();
        create_commit(&repo, "Second commit");
        create_commit(&repo, "Third commit");

        let commits = repo.commits_since(Some("v1.0.0")).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].subject(), "Third commit");
        assert_eq!(commits[1].subject(), "Second commit");
    }

    #[test]
    fn test_commits_since_invalid_tag() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "First commit");

        let result = repo.commits_since(Some("nonexistent-tag"));
        assert!(result.is_err());
    }

    #[test]
    fn test_commits_contain_correct_data() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "feat: add feature");

        let commits = repo.commits_since(None).unwrap();
        assert_eq!(commits.len(), 1);

        let commit = &commits[0];
        assert_eq!(commit.subject(), "feat: add feature");
        assert_eq!(commit.author, "Test User");
        assert!(!commit.hash.is_empty());
    }

    #[test]
    fn test_latest_version_tag_none() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "Initial commit");

        let result = repo.latest_version_tag("v").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_latest_version_tag_single() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "Initial commit");
        repo.create_tag("v1.0.0", "Release").unwrap();

        let result = repo.latest_version_tag("v").unwrap();
        assert_eq!(result, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_latest_version_tag_multiple() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "Initial commit");
        repo.create_tag("v1.0.0", "Release 1.0.0").unwrap();
        create_commit(&repo, "Another commit");
        repo.create_tag("v1.1.0", "Release 1.1.0").unwrap();
        create_commit(&repo, "Yet another commit");
        repo.create_tag("v2.0.0", "Release 2.0.0").unwrap();

        let result = repo.latest_version_tag("v").unwrap();
        assert_eq!(result, Some("v2.0.0".to_string()));
    }

    #[test]
    fn test_latest_version_tag_prerelease() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "Initial commit");
        repo.create_tag("v1.0.0", "Release").unwrap();
        create_commit(&repo, "Another commit");
        repo.create_tag("v1.0.1-beta.1", "Beta").unwrap();

        let result = repo.latest_version_tag("v").unwrap();
        // 1.0.1-beta.1 < 1.0.1 but > 1.0.0
        assert_eq!(result, Some("v1.0.1-beta.1".to_string()));
    }

    #[test]
    fn test_latest_version_tag_different_prefix() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "Initial commit");
        repo.create_tag("v1.0.0", "Release").unwrap();
        repo.create_tag("release-2.0.0", "Release").unwrap();

        let result_v = repo.latest_version_tag("v").unwrap();
        assert_eq!(result_v, Some("v1.0.0".to_string()));

        let result_release = repo.latest_version_tag("release-").unwrap();
        assert_eq!(result_release, Some("release-2.0.0".to_string()));
    }

    #[test]
    fn test_latest_version_tag_ignores_non_semver() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "Initial commit");
        repo.create_tag("v1.0.0", "Release").unwrap();
        create_commit(&repo, "Another commit");
        repo.create_tag("vnot-semver", "Not semver").unwrap();

        let result = repo.latest_version_tag("v").unwrap();
        assert_eq!(result, Some("v1.0.0".to_string()));
    }

    #[test]
    fn test_create_tag() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "Initial commit");

        let result = repo.create_tag("v1.0.0", "First release");
        assert!(result.is_ok());

        let tags = repo.tags().unwrap();
        assert!(tags.contains(&"v1.0.0".to_string()));
    }

    #[test]
    fn test_create_tag_duplicate() {
        let (_temp_dir, repo) = create_test_repo();
        create_commit(&repo, "Initial commit");

        repo.create_tag("v1.0.0", "First release").unwrap();
        let result = repo.create_tag("v1.0.0", "Duplicate");
        assert!(result.is_err());
    }
}
