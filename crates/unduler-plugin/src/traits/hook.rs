//! Release hook trait.

use super::Plugin;
use crate::{PluginResult, ReleaseContext};

/// Lifecycle hooks during the release process.
///
/// Hooks are executed at specific points in the release pipeline:
/// 1. `pre_bump` - Before version files are modified
/// 2. `post_bump` - After version files are modified
/// 3. `pre_commit` - Before the release commit is created
/// 4. `pre_tag` - Before the git tag is created
/// 5. `post_tag` - After the git tag is created
#[allow(unused_variables)]
pub trait ReleaseHook: Plugin {
    /// Called before version files are modified.
    ///
    /// Use this for validation or preparation.
    ///
    /// # Errors
    ///
    /// Returns an error if the hook's pre-bump validation fails.
    fn on_pre_bump(&self, _ctx: &mut ReleaseContext) -> PluginResult<()> {
        Ok(())
    }

    /// Called after version files are modified.
    ///
    /// Use this to sync lock files or update internal dependencies.
    ///
    /// # Errors
    ///
    /// Returns an error if post-bump operations fail (e.g., lock file sync).
    fn on_post_bump(&self, _ctx: &mut ReleaseContext) -> PluginResult<()> {
        Ok(())
    }

    /// Called before the release commit is created.
    ///
    /// Use this for linting or formatting.
    ///
    /// # Errors
    ///
    /// Returns an error if pre-commit checks fail (e.g., linting errors).
    fn on_pre_commit(&self, _ctx: &mut ReleaseContext) -> PluginResult<()> {
        Ok(())
    }

    /// Called before the git tag is created.
    ///
    /// Use this for final verification.
    ///
    /// # Errors
    ///
    /// Returns an error if pre-tag verification fails.
    fn on_pre_tag(&self, _ctx: &mut ReleaseContext) -> PluginResult<()> {
        Ok(())
    }

    /// Called after the git tag is created.
    ///
    /// Use this for publishing, deployment, or notifications.
    ///
    /// # Errors
    ///
    /// Returns an error if post-tag operations fail (e.g., publishing).
    fn on_post_tag(&self, _ctx: &mut ReleaseContext) -> PluginResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BumpType;
    use semver::Version;

    // A minimal hook that uses all defaults
    struct MinimalHook;

    impl Plugin for MinimalHook {
        fn name(&self) -> &'static str {
            "minimal-hook"
        }
        fn version(&self) -> &'static str {
            "1.0.0"
        }
    }

    impl ReleaseHook for MinimalHook {}

    fn create_test_context() -> ReleaseContext {
        ReleaseContext::new(
            "/tmp/test",
            Version::new(1, 0, 0),
            Version::new(1, 1, 0),
            BumpType::Minor,
            vec![],
        )
    }

    #[test]
    fn test_default_pre_bump() {
        let hook = MinimalHook;
        let mut ctx = create_test_context();
        assert!(hook.on_pre_bump(&mut ctx).is_ok());
    }

    #[test]
    fn test_default_post_bump() {
        let hook = MinimalHook;
        let mut ctx = create_test_context();
        assert!(hook.on_post_bump(&mut ctx).is_ok());
    }

    #[test]
    fn test_default_pre_commit() {
        let hook = MinimalHook;
        let mut ctx = create_test_context();
        assert!(hook.on_pre_commit(&mut ctx).is_ok());
    }

    #[test]
    fn test_default_pre_tag() {
        let hook = MinimalHook;
        let mut ctx = create_test_context();
        assert!(hook.on_pre_tag(&mut ctx).is_ok());
    }

    #[test]
    fn test_default_post_tag() {
        let hook = MinimalHook;
        let mut ctx = create_test_context();
        assert!(hook.on_post_tag(&mut ctx).is_ok());
    }
}
