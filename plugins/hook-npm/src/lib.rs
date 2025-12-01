//! npm/Node.js hook plugin.

use unduler_plugin::{Plugin, PluginResult, ReleaseContext, ReleaseHook};

/// npm hook for Node.js projects.
pub struct NpmHook {
    /// Publish to npm after release.
    publish: bool,
    /// Registry to publish to.
    registry: Option<String>,
}

impl NpmHook {
    /// Creates a new npm hook.
    #[must_use]
    pub fn new() -> Self {
        Self {
            publish: false,
            registry: None,
        }
    }

    /// Enables publishing to npm.
    #[must_use]
    pub fn with_publish(mut self, publish: bool) -> Self {
        self.publish = publish;
        self
    }

    /// Sets the registry to publish to.
    #[must_use]
    pub fn with_registry(mut self, registry: impl Into<String>) -> Self {
        self.registry = Some(registry.into());
        self
    }

    /// Returns whether publishing is enabled.
    #[must_use]
    pub fn publish(&self) -> bool {
        self.publish
    }

    /// Returns the registry, if set.
    #[must_use]
    pub fn registry(&self) -> Option<&str> {
        self.registry.as_deref()
    }
}

impl Default for NpmHook {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for NpmHook {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        "Updates package.json version and optionally publishes to npm"
    }
}

impl ReleaseHook for NpmHook {
    fn on_post_bump(&self, ctx: &mut ReleaseContext) -> PluginResult<()> {
        if ctx.dry_run {
            return Ok(());
        }

        // TODO: Update package.json version
        // TODO: Run npm install to update package-lock.json

        Ok(())
    }

    fn on_post_tag(&self, ctx: &mut ReleaseContext) -> PluginResult<()> {
        if ctx.dry_run || !self.publish {
            return Ok(());
        }

        // TODO: Run npm publish

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
        let hook = NpmHook::new();
        assert!(!hook.publish());
        assert!(hook.registry().is_none());
    }

    #[test]
    fn test_default() {
        let hook = NpmHook::default();
        assert!(!hook.publish());
        assert!(hook.registry().is_none());
    }

    #[test]
    fn test_with_publish() {
        let hook = NpmHook::new().with_publish(true);
        assert!(hook.publish());
    }

    #[test]
    fn test_with_publish_false() {
        let hook = NpmHook::new().with_publish(true).with_publish(false);
        assert!(!hook.publish());
    }

    #[test]
    fn test_with_registry() {
        let hook = NpmHook::new().with_registry("https://npm.private.com");
        assert_eq!(hook.registry(), Some("https://npm.private.com"));
    }

    #[test]
    fn test_with_registry_string() {
        let hook = NpmHook::new().with_registry(String::from("https://npm.example.com"));
        assert_eq!(hook.registry(), Some("https://npm.example.com"));
    }

    #[test]
    fn test_builder_chain() {
        let hook = NpmHook::new()
            .with_publish(true)
            .with_registry("https://npm.private.com");
        assert!(hook.publish());
        assert_eq!(hook.registry(), Some("https://npm.private.com"));
    }

    #[test]
    fn test_plugin_name() {
        let hook = NpmHook::new();
        assert_eq!(hook.name(), "npm");
    }

    #[test]
    fn test_plugin_version() {
        let hook = NpmHook::new();
        assert_eq!(hook.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_plugin_description() {
        let hook = NpmHook::new();
        assert_eq!(
            hook.description(),
            "Updates package.json version and optionally publishes to npm"
        );
    }

    #[test]
    fn test_on_post_bump_dry_run() {
        let hook = NpmHook::new();
        let mut ctx = create_test_context(true);
        let result = hook.on_post_bump(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_bump_not_dry_run() {
        let hook = NpmHook::new();
        let mut ctx = create_test_context(false);
        let result = hook.on_post_bump(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_dry_run() {
        let hook = NpmHook::new().with_publish(true);
        let mut ctx = create_test_context(true);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_not_publishing() {
        let hook = NpmHook::new().with_publish(false);
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_publishing() {
        let hook = NpmHook::new().with_publish(true);
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_with_registry() {
        let hook = NpmHook::new()
            .with_publish(true)
            .with_registry("https://npm.private.com");
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }
}
