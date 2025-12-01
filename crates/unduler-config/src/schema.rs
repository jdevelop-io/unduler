//! Configuration schema.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Main configuration structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Parser configuration.
    #[serde(default)]
    pub parser: ParserConfig,

    /// Bumper configuration.
    #[serde(default)]
    pub bumper: BumperConfig,

    /// Formatter configuration.
    #[serde(default)]
    pub formatter: FormatterPluginConfig,

    /// Hooks configuration.
    #[serde(default)]
    pub hooks: HooksConfig,

    /// Version configuration.
    #[serde(default)]
    pub version: VersionConfig,

    /// Changelog configuration.
    #[serde(default)]
    pub changelog: ChangelogConfig,

    /// Plugin-specific configuration.
    #[serde(default)]
    pub plugins: PluginsConfig,
}

/// Parser configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Parser plugin name.
    #[serde(default = "default_parser")]
    pub name: String,

    /// Gitmoji-specific options.
    #[serde(default, rename = "conventional-gitmoji")]
    pub conventional_gitmoji: ConventionalGitmojiConfig,

    /// Regex-specific options.
    #[serde(default)]
    pub regex: RegexParserConfig,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            name: default_parser(),
            conventional_gitmoji: ConventionalGitmojiConfig::default(),
            regex: RegexParserConfig::default(),
        }
    }
}

fn default_parser() -> String {
    "conventional".to_string()
}

/// Conventional + Gitmoji parser options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConventionalGitmojiConfig {
    /// Infer type from emoji if not explicitly provided.
    #[serde(default = "default_true")]
    pub infer_type_from_emoji: bool,

    /// Reject commits with unknown emojis.
    #[serde(default)]
    pub strict_emoji: bool,
}

fn default_true() -> bool {
    true
}

/// Regex parser options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegexParserConfig {
    /// The regex pattern.
    pub pattern: Option<String>,

    /// Mapping of capture groups to commit fields.
    #[serde(default)]
    pub mapping: HashMap<String, String>,

    /// Validation rules for captured values.
    #[serde(default)]
    pub validation: HashMap<String, Vec<String>>,
}

/// Bumper configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BumperConfig {
    /// Bumper plugin name.
    #[serde(default = "default_bumper")]
    pub name: String,
}

impl Default for BumperConfig {
    fn default() -> Self {
        Self {
            name: default_bumper(),
        }
    }
}

fn default_bumper() -> String {
    "semver".to_string()
}

/// Formatter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatterPluginConfig {
    /// Formatter plugin name.
    #[serde(default = "default_formatter")]
    pub name: String,
}

impl Default for FormatterPluginConfig {
    fn default() -> Self {
        Self {
            name: default_formatter(),
        }
    }
}

fn default_formatter() -> String {
    "keepachangelog".to_string()
}

/// Hooks configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Hooks to run before bump.
    #[serde(default)]
    pub pre_bump: Vec<String>,

    /// Hooks to run after bump.
    #[serde(default)]
    pub post_bump: Vec<String>,

    /// Hooks to run before commit.
    #[serde(default)]
    pub pre_commit: Vec<String>,

    /// Hooks to run before tag.
    #[serde(default)]
    pub pre_tag: Vec<String>,

    /// Hooks to run after tag.
    #[serde(default)]
    pub post_tag: Vec<String>,
}

/// Version configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConfig {
    /// Files containing version information.
    #[serde(default)]
    pub files: Vec<String>,

    /// Tag prefix (e.g., "v").
    #[serde(default = "default_tag_prefix")]
    pub tag_prefix: String,
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            files: vec!["Cargo.toml".to_string()],
            tag_prefix: default_tag_prefix(),
        }
    }
}

fn default_tag_prefix() -> String {
    "v".to_string()
}

/// Changelog configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogConfig {
    /// Output file path.
    #[serde(default = "default_changelog_output")]
    pub output: String,
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        Self {
            output: default_changelog_output(),
        }
    }
}

fn default_changelog_output() -> String {
    "CHANGELOG.md".to_string()
}

/// Plugin-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginsConfig {
    /// Cargo hook configuration.
    #[serde(default)]
    pub cargo: CargoPluginConfig,

    /// npm hook configuration.
    #[serde(default)]
    pub npm: NpmPluginConfig,

    /// GitHub Release hook configuration.
    #[serde(default, rename = "github-release")]
    pub github_release: GithubReleasePluginConfig,
}

/// Cargo plugin configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CargoPluginConfig {
    /// Publish to crates.io after release.
    #[serde(default)]
    pub publish: bool,

    /// Registry to publish to.
    pub registry: Option<String>,
}

/// npm plugin configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NpmPluginConfig {
    /// Publish to npm after release.
    #[serde(default)]
    pub publish: bool,

    /// Registry to publish to.
    pub registry: Option<String>,
}

/// GitHub Release plugin configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GithubReleasePluginConfig {
    /// Create release as draft.
    #[serde(default)]
    pub draft: bool,

    /// Mark release as prerelease.
    #[serde(default)]
    pub prerelease: bool,

    /// Assets to upload.
    #[serde(default)]
    pub assets: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.parser.name, "conventional");
        assert_eq!(config.bumper.name, "semver");
        assert_eq!(config.formatter.name, "keepachangelog");
        assert_eq!(config.version.tag_prefix, "v");
        assert_eq!(config.changelog.output, "CHANGELOG.md");
    }

    #[test]
    fn test_default_parser_config() {
        let config = ParserConfig::default();
        assert_eq!(config.name, "conventional");
        // Default trait doesn't use serde default functions
        assert!(!config.conventional_gitmoji.infer_type_from_emoji);
        assert!(!config.conventional_gitmoji.strict_emoji);
    }

    #[test]
    fn test_default_bumper_config() {
        let config = BumperConfig::default();
        assert_eq!(config.name, "semver");
    }

    #[test]
    fn test_default_formatter_config() {
        let config = FormatterPluginConfig::default();
        assert_eq!(config.name, "keepachangelog");
    }

    #[test]
    fn test_default_version_config() {
        let config = VersionConfig::default();
        assert_eq!(config.tag_prefix, "v");
        assert_eq!(config.files, vec!["Cargo.toml".to_string()]);
    }

    #[test]
    fn test_default_changelog_config() {
        let config = ChangelogConfig::default();
        assert_eq!(config.output, "CHANGELOG.md");
    }

    #[test]
    fn test_default_hooks_config() {
        let config = HooksConfig::default();
        assert!(config.pre_bump.is_empty());
        assert!(config.post_bump.is_empty());
        assert!(config.pre_commit.is_empty());
        assert!(config.pre_tag.is_empty());
        assert!(config.post_tag.is_empty());
    }

    #[test]
    fn test_default_plugins_config() {
        let config = PluginsConfig::default();
        assert!(!config.cargo.publish);
        assert!(config.cargo.registry.is_none());
        assert!(!config.npm.publish);
        assert!(config.npm.registry.is_none());
        assert!(!config.github_release.draft);
        assert!(!config.github_release.prerelease);
        assert!(config.github_release.assets.is_empty());
    }

    #[test]
    fn test_deserialize_minimal() {
        let toml = r#"
            [parser]
            name = "conventional-gitmoji"
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.parser.name, "conventional-gitmoji");
        assert_eq!(config.bumper.name, "semver"); // default
    }

    #[test]
    fn test_deserialize_empty() {
        let toml = "";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.parser.name, "conventional");
    }

    #[test]
    fn test_deserialize_full() {
        let toml = r#"
            [parser]
            name = "regex"

            [parser.regex]
            pattern = "^(?P<type>\\w+): (?P<message>.+)$"

            [bumper]
            name = "semver"

            [formatter]
            name = "keepachangelog"

            [version]
            tag_prefix = "release-"
            files = ["package.json", "version.txt"]

            [changelog]
            output = "HISTORY.md"

            [hooks]
            pre_bump = ["cargo fmt"]
            post_bump = ["cargo check"]
            pre_tag = ["cargo test"]

            [plugins.cargo]
            publish = true
            registry = "my-registry"

            [plugins.npm]
            publish = true
            registry = "https://npm.example.com"

            [plugins.github-release]
            draft = true
            prerelease = true
            assets = ["dist/*.zip", "dist/*.tar.gz"]
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.parser.name, "regex");
        assert!(config.parser.regex.pattern.is_some());
        assert_eq!(config.version.tag_prefix, "release-");
        assert_eq!(config.version.files.len(), 2);
        assert_eq!(config.changelog.output, "HISTORY.md");
        assert_eq!(config.hooks.pre_bump.len(), 1);
        assert_eq!(config.hooks.post_bump.len(), 1);
        assert_eq!(config.hooks.pre_tag.len(), 1);
        assert!(config.plugins.cargo.publish);
        assert_eq!(
            config.plugins.cargo.registry,
            Some("my-registry".to_string())
        );
        assert!(config.plugins.npm.publish);
        assert!(config.plugins.github_release.draft);
        assert!(config.plugins.github_release.prerelease);
        assert_eq!(config.plugins.github_release.assets.len(), 2);
    }

    #[test]
    fn test_deserialize_gitmoji_config() {
        let toml = r#"
            [parser]
            name = "conventional-gitmoji"

            [parser.conventional-gitmoji]
            infer_type_from_emoji = false
            strict_emoji = true
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.parser.name, "conventional-gitmoji");
        assert!(!config.parser.conventional_gitmoji.infer_type_from_emoji);
        assert!(config.parser.conventional_gitmoji.strict_emoji);
    }

    #[test]
    fn test_deserialize_regex_config() {
        let toml = r#"
            [parser]
            name = "regex"

            [parser.regex]
            pattern = "^(?P<type>\\w+): (?P<message>.+)$"

            [parser.regex.mapping]
            type = "type"
            message = "message"

            [parser.regex.validation]
            type = ["feat", "fix", "chore"]
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.parser.name, "regex");
        assert!(config.parser.regex.pattern.is_some());
        assert_eq!(
            config.parser.regex.mapping.get("type"),
            Some(&"type".to_string())
        );
        assert_eq!(config.parser.regex.validation.get("type").unwrap().len(), 3);
    }

    #[test]
    fn test_serialize_config() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[parser]"));
        assert!(toml_str.contains("name = \"conventional\""));
    }

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();
        assert_eq!(config.parser.name, cloned.parser.name);
    }

    #[test]
    fn test_config_debug() {
        let config = Config::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("Config"));
        assert!(debug.contains("parser"));
    }
}
