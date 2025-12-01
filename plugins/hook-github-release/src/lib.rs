//! GitHub Release hook plugin.

use unduler_plugin::{Plugin, PluginResult, ReleaseContext, ReleaseHook};

/// GitHub Release hook.
pub struct GithubReleaseHook {
    /// Create release as draft.
    draft: bool,
    /// Mark release as prerelease.
    prerelease: bool,
    /// Assets to upload.
    assets: Vec<String>,
}

impl GithubReleaseHook {
    /// Creates a new GitHub Release hook.
    #[must_use]
    pub fn new() -> Self {
        Self {
            draft: false,
            prerelease: false,
            assets: Vec::new(),
        }
    }

    /// Creates release as draft.
    #[must_use]
    pub fn with_draft(mut self, draft: bool) -> Self {
        self.draft = draft;
        self
    }

    /// Marks release as prerelease.
    #[must_use]
    pub fn with_prerelease(mut self, prerelease: bool) -> Self {
        self.prerelease = prerelease;
        self
    }

    /// Sets assets to upload.
    #[must_use]
    pub fn with_assets(mut self, assets: Vec<String>) -> Self {
        self.assets = assets;
        self
    }

    /// Returns whether this is a draft release.
    #[must_use]
    pub fn is_draft(&self) -> bool {
        self.draft
    }

    /// Returns whether this is a prerelease.
    #[must_use]
    pub fn is_prerelease(&self) -> bool {
        self.prerelease
    }

    /// Returns the assets to upload.
    #[must_use]
    pub fn assets(&self) -> &[String] {
        &self.assets
    }
}

impl Default for GithubReleaseHook {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for GithubReleaseHook {
    fn name(&self) -> &'static str {
        "github-release"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        "Creates GitHub Releases and uploads assets"
    }
}

impl ReleaseHook for GithubReleaseHook {
    fn on_post_tag(&self, ctx: &mut ReleaseContext) -> PluginResult<()> {
        if ctx.dry_run {
            return Ok(());
        }

        // TODO: Create GitHub Release via API
        // TODO: Upload assets

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use unduler_plugin::BumpType;

    use super::*;

    fn create_test_context(dry_run: bool) -> ReleaseContext {
        ReleaseContext::new(
            "/tmp/test",
            Version::new(1, 0, 0),
            Version::new(1, 1, 0),
            BumpType::Minor,
            vec![],
        )
        .dry_run(dry_run)
    }

    #[test]
    fn test_new() {
        let hook = GithubReleaseHook::new();
        assert!(!hook.is_draft());
        assert!(!hook.is_prerelease());
        assert!(hook.assets().is_empty());
    }

    #[test]
    fn test_default() {
        let hook = GithubReleaseHook::default();
        assert!(!hook.is_draft());
        assert!(!hook.is_prerelease());
        assert!(hook.assets().is_empty());
    }

    #[test]
    fn test_with_draft() {
        let hook = GithubReleaseHook::new().with_draft(true);
        assert!(hook.is_draft());
    }

    #[test]
    fn test_with_draft_false() {
        let hook = GithubReleaseHook::new().with_draft(true).with_draft(false);
        assert!(!hook.is_draft());
    }

    #[test]
    fn test_with_prerelease() {
        let hook = GithubReleaseHook::new().with_prerelease(true);
        assert!(hook.is_prerelease());
    }

    #[test]
    fn test_with_prerelease_false() {
        let hook = GithubReleaseHook::new()
            .with_prerelease(true)
            .with_prerelease(false);
        assert!(!hook.is_prerelease());
    }

    #[test]
    fn test_with_assets() {
        let assets = vec!["dist/app.zip".to_string(), "dist/app.tar.gz".to_string()];
        let hook = GithubReleaseHook::new().with_assets(assets.clone());
        assert_eq!(hook.assets(), &assets);
    }

    #[test]
    fn test_with_assets_empty() {
        let hook = GithubReleaseHook::new().with_assets(vec![]);
        assert!(hook.assets().is_empty());
    }

    #[test]
    fn test_builder_chain() {
        let assets = vec!["binary.exe".to_string()];
        let hook = GithubReleaseHook::new()
            .with_draft(true)
            .with_prerelease(true)
            .with_assets(assets.clone());
        assert!(hook.is_draft());
        assert!(hook.is_prerelease());
        assert_eq!(hook.assets(), &assets);
    }

    #[test]
    fn test_plugin_name() {
        let hook = GithubReleaseHook::new();
        assert_eq!(hook.name(), "github-release");
    }

    #[test]
    fn test_plugin_version() {
        let hook = GithubReleaseHook::new();
        assert_eq!(hook.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_plugin_description() {
        let hook = GithubReleaseHook::new();
        assert_eq!(
            hook.description(),
            "Creates GitHub Releases and uploads assets"
        );
    }

    #[test]
    fn test_on_post_tag_dry_run() {
        let hook = GithubReleaseHook::new();
        let mut ctx = create_test_context(true);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_not_dry_run() {
        let hook = GithubReleaseHook::new();
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_with_draft() {
        let hook = GithubReleaseHook::new().with_draft(true);
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_with_prerelease() {
        let hook = GithubReleaseHook::new().with_prerelease(true);
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_with_assets() {
        let hook = GithubReleaseHook::new().with_assets(vec![
            "dist/app.zip".to_string(),
            "dist/app.tar.gz".to_string(),
        ]);
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_full_config() {
        let hook = GithubReleaseHook::new()
            .with_draft(true)
            .with_prerelease(true)
            .with_assets(vec!["release.zip".to_string()]);
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }
}
