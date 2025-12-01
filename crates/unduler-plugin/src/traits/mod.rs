//! Plugin traits.

pub mod bumper;
pub mod formatter;
pub mod hook;
pub mod parser;

/// Base trait for all plugins.
pub trait Plugin: Send + Sync {
    /// Returns the plugin name.
    fn name(&self) -> &'static str;

    /// Returns the plugin version.
    fn version(&self) -> &'static str;

    /// Returns a short description of the plugin.
    fn description(&self) -> &'static str {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MinimalPlugin;

    impl Plugin for MinimalPlugin {
        fn name(&self) -> &'static str {
            "minimal"
        }
        fn version(&self) -> &'static str {
            "1.0.0"
        }
        // Using default description
    }

    struct CustomPlugin;

    impl Plugin for CustomPlugin {
        fn name(&self) -> &'static str {
            "custom"
        }
        fn version(&self) -> &'static str {
            "2.0.0"
        }
        fn description(&self) -> &'static str {
            "Custom description"
        }
    }

    #[test]
    fn test_default_description() {
        let plugin = MinimalPlugin;
        assert_eq!(plugin.description(), "");
    }

    #[test]
    fn test_custom_description() {
        let plugin = CustomPlugin;
        assert_eq!(plugin.description(), "Custom description");
    }

    #[test]
    fn test_plugin_name() {
        let plugin = MinimalPlugin;
        assert_eq!(plugin.name(), "minimal");
    }

    #[test]
    fn test_plugin_version() {
        let plugin = MinimalPlugin;
        assert_eq!(plugin.version(), "1.0.0");
    }
}
