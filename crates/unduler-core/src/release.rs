//! Release orchestration.

use semver::Version;
use tracing::{debug, info};
use unduler_git::Repository;
use unduler_plugin::{FormatterConfig, Release, ReleaseContext};

use crate::{CoreError, CoreResult, Pipeline, VersionManager};

/// Manages the release process.
pub struct ReleaseManager {
    repo: Repository,
    version_manager: VersionManager,
    tag_prefix: String,
}

impl ReleaseManager {
    /// Creates a new release manager.
    #[must_use]
    pub fn new(repo: Repository, tag_prefix: impl Into<String>) -> Self {
        Self {
            repo,
            version_manager: VersionManager::new(),
            tag_prefix: tag_prefix.into(),
        }
    }

    /// Executes a release with the given pipeline.
    ///
    /// # Errors
    ///
    /// Returns an error if the release fails.
    pub fn release(&self, pipeline: &Pipeline, dry_run: bool) -> CoreResult<Version> {
        info!("starting release process");

        // Get previous version
        let previous_version = self.get_previous_version()?;
        debug!(?previous_version, "found previous version");

        // Get commits since last release
        let tag = previous_version
            .as_ref()
            .map(|v| format!("{}{v}", self.tag_prefix));
        let raw_commits = self.repo.commits_since(tag.as_deref())?;

        if raw_commits.is_empty() {
            return Err(CoreError::NoCommits);
        }

        info!(
            count = raw_commits.len(),
            "found commits since last release"
        );

        // Parse commits
        let parsed_commits = pipeline.parse_commits(&raw_commits);
        debug!(
            parsed = parsed_commits.len(),
            skipped = raw_commits.len() - parsed_commits.len(),
            "parsed commits"
        );

        // Determine bump type
        let bump_type = pipeline.determine_bump(&parsed_commits);
        info!(%bump_type, "determined bump type");

        // Calculate new version
        let base_version = previous_version.unwrap_or_else(|| Version::new(0, 0, 0));
        let next_version = self.version_manager.bump(&base_version, bump_type);
        info!(
            previous = %base_version,
            next = %next_version,
            "calculated new version"
        );

        // Create release context
        let mut ctx = ReleaseContext::new(
            self.repo.path(),
            base_version.clone(),
            next_version.clone(),
            bump_type,
            parsed_commits.clone(),
        )
        .dry_run(dry_run);

        // Run pre_bump hooks
        for hook in pipeline.hooks() {
            debug!(hook = hook.name(), "running pre_bump hook");
            hook.on_pre_bump(&mut ctx)?;
        }

        if !dry_run {
            // TODO: Update version files
        }

        // Run post_bump hooks
        for hook in pipeline.hooks() {
            debug!(hook = hook.name(), "running post_bump hook");
            hook.on_post_bump(&mut ctx)?;
        }

        // Generate changelog
        let release = Release::new(next_version.clone(), chrono::Utc::now(), parsed_commits)
            .with_previous_version(base_version);

        let changelog = pipeline
            .formatter()
            .format(&release, &FormatterConfig::default());
        ctx.changelog = Some(changelog.clone());

        debug!(changelog_len = changelog.len(), "generated changelog");

        // Run pre_commit hooks
        for hook in pipeline.hooks() {
            debug!(hook = hook.name(), "running pre_commit hook");
            hook.on_pre_commit(&mut ctx)?;
        }

        if !dry_run {
            // TODO: Commit changes
        }

        // Run pre_tag hooks
        for hook in pipeline.hooks() {
            debug!(hook = hook.name(), "running pre_tag hook");
            hook.on_pre_tag(&mut ctx)?;
        }

        if !dry_run {
            // Create tag
            let tag_name = format!("{}{next_version}", self.tag_prefix);
            self.repo
                .create_tag(&tag_name, &format!("Release {next_version}"))?;
            info!(%tag_name, "created tag");
        }

        // Run post_tag hooks
        for hook in pipeline.hooks() {
            debug!(hook = hook.name(), "running post_tag hook");
            hook.on_post_tag(&mut ctx)?;
        }

        info!(version = %next_version, "release completed");
        Ok(next_version)
    }

    /// Gets the previous version from the latest tag.
    fn get_previous_version(&self) -> CoreResult<Option<Version>> {
        let tag = self.repo.latest_version_tag(&self.tag_prefix)?;

        Ok(tag.and_then(|t| self.version_manager.from_tag(&t, &self.tag_prefix)))
    }
}
