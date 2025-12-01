//! Cargo/Rust hook plugin.

use unduler_plugin::{Plugin, PluginResult, ReleaseContext, ReleaseHook};

/// Cargo hook for Rust projects.
pub struct CargoHook {
    /// Publish to crates.io after release.
    publish: bool,
    /// Registry to publish to.
    registry: Option<String>,
}

impl CargoHook {
    /// Creates a new Cargo hook.
    #[must_use]
    pub fn new() -> Self {
        Self {
            publish: false,
            registry: None,
        }
    }

    /// Enables publishing to crates.io.
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

impl Default for CargoHook {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CargoHook {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        "Updates Cargo.toml version and optionally publishes to crates.io"
    }
}

impl ReleaseHook for CargoHook {
    fn on_post_bump(&self, ctx: &mut ReleaseContext) -> PluginResult<()> {
        if ctx.dry_run {
            return Ok(());
        }

        // TODO: Update Cargo.toml version
        // TODO: Run cargo check to update Cargo.lock

        Ok(())
    }

    fn on_post_tag(&self, ctx: &mut ReleaseContext) -> PluginResult<()> {
        if ctx.dry_run || !self.publish {
            return Ok(());
        }

        // TODO: Run cargo publish

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
        let hook = CargoHook::new();
        assert!(!hook.publish());
        assert!(hook.registry().is_none());
    }

    #[test]
    fn test_default() {
        let hook = CargoHook::default();
        assert!(!hook.publish());
        assert!(hook.registry().is_none());
    }

    #[test]
    fn test_with_publish() {
        let hook = CargoHook::new().with_publish(true);
        assert!(hook.publish());
    }

    #[test]
    fn test_with_publish_false() {
        let hook = CargoHook::new().with_publish(true).with_publish(false);
        assert!(!hook.publish());
    }

    #[test]
    fn test_with_registry() {
        let hook = CargoHook::new().with_registry("my-registry");
        assert_eq!(hook.registry(), Some("my-registry"));
    }

    #[test]
    fn test_with_registry_string() {
        let hook = CargoHook::new().with_registry(String::from("custom-registry"));
        assert_eq!(hook.registry(), Some("custom-registry"));
    }

    #[test]
    fn test_builder_chain() {
        let hook = CargoHook::new()
            .with_publish(true)
            .with_registry("my-registry");
        assert!(hook.publish());
        assert_eq!(hook.registry(), Some("my-registry"));
    }

    #[test]
    fn test_plugin_name() {
        let hook = CargoHook::new();
        assert_eq!(hook.name(), "cargo");
    }

    #[test]
    fn test_plugin_version() {
        let hook = CargoHook::new();
        assert_eq!(hook.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_plugin_description() {
        let hook = CargoHook::new();
        assert_eq!(
            hook.description(),
            "Updates Cargo.toml version and optionally publishes to crates.io"
        );
    }

    #[test]
    fn test_on_post_bump_dry_run() {
        let hook = CargoHook::new();
        let mut ctx = create_test_context(true);
        let result = hook.on_post_bump(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_bump_not_dry_run() {
        let hook = CargoHook::new();
        let mut ctx = create_test_context(false);
        let result = hook.on_post_bump(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_dry_run() {
        let hook = CargoHook::new().with_publish(true);
        let mut ctx = create_test_context(true);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_not_publishing() {
        let hook = CargoHook::new().with_publish(false);
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_publishing() {
        let hook = CargoHook::new().with_publish(true);
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_on_post_tag_with_registry() {
        let hook = CargoHook::new()
            .with_publish(true)
            .with_registry("private-registry");
        let mut ctx = create_test_context(false);
        let result = hook.on_post_tag(&mut ctx);
        assert!(result.is_ok());
    }
}
