//! Initialize command.

use std::fmt::Write;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};

use unduler_config::CONFIG_FILE_NAME;

/// Parser type argument.
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum ParserType {
    /// Standard Conventional Commits
    #[default]
    Conventional,
    /// Conventional Commits with Gitmoji
    ConventionalGitmoji,
    /// Custom regex pattern
    Regex,
}

impl ParserType {
    fn as_config_name(self) -> &'static str {
        match self {
            Self::Conventional => "conventional",
            Self::ConventionalGitmoji => "conventional-gitmoji",
            Self::Regex => "regex",
        }
    }
}

/// Detected project type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectType {
    Rust,
    Node,
    RustAndNode,
    Unknown,
}

impl ProjectType {
    fn detect() -> Self {
        let has_cargo = Path::new("Cargo.toml").exists();
        let has_package_json = Path::new("package.json").exists();

        match (has_cargo, has_package_json) {
            (true, true) => Self::RustAndNode,
            (true, false) => Self::Rust,
            (false, true) => Self::Node,
            (false, false) => Self::Unknown,
        }
    }

    fn version_files(self) -> Vec<&'static str> {
        match self {
            Self::Rust => vec!["Cargo.toml"],
            Self::Node => vec!["package.json"],
            Self::RustAndNode => vec!["Cargo.toml", "package.json"],
            Self::Unknown => vec![],
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Node => "Node.js",
            Self::RustAndNode => "Rust + Node.js",
            Self::Unknown => "Unknown",
        }
    }
}

/// Arguments for the init command.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Force overwrite existing configuration
    #[arg(short, long)]
    pub force: bool,

    /// Parser to use
    #[arg(short, long, value_enum, default_value_t = ParserType::Conventional)]
    pub parser: ParserType,

    /// Skip plugin installation suggestions
    #[arg(long)]
    pub no_plugins: bool,
}

/// Generates the configuration file content.
fn generate_config(parser: ParserType, project_type: ProjectType) -> String {
    let parser_name = parser.as_config_name();
    let version_files = project_type.version_files();

    let mut config = String::new();

    // Parser section
    config.push_str("[parser]\n");
    let _ = writeln!(config, "name = \"{parser_name}\"");

    // Add gitmoji-specific config if using conventional-gitmoji
    if matches!(parser, ParserType::ConventionalGitmoji) {
        config.push_str("\n[parser.conventional-gitmoji]\n");
        config.push_str("infer_type_from_emoji = true\n");
        config.push_str("strict_emoji = false\n");
    }

    // Add regex placeholder if using regex
    if matches!(parser, ParserType::Regex) {
        config.push_str("\n[parser.regex]\n");
        config.push_str("# pattern = \"^(?P<type>\\\\w+)(?:\\\\((?P<scope>\\\\w+)\\\\))?:\\\\s+(?P<message>.+)$\"\n");
        config.push_str("\n[parser.regex.mapping]\n");
        config.push_str("# type = \"type\"\n");
        config.push_str("# scope = \"scope\"\n");
        config.push_str("# message = \"message\"\n");
        config.push_str("\n[parser.regex.validation]\n");
        config.push_str(
            "# type = [\"feat\", \"fix\", \"docs\", \"chore\", \"refactor\", \"test\", \"ci\"]\n",
        );
    }

    // Version section
    config.push_str("\n[version]\n");
    config.push_str("tag_prefix = \"v\"\n");

    if !version_files.is_empty() {
        let files_str = version_files
            .iter()
            .map(|f| format!("\"{f}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(config, "files = [{files_str}]");
    }

    // Changelog section
    config.push_str("\n[changelog]\n");
    config.push_str("output = \"CHANGELOG.md\"\n");

    config
}

/// Runs the init command.
#[allow(clippy::needless_pass_by_value)]
pub fn run(args: InitArgs) -> Result<()> {
    let config_path = Path::new(CONFIG_FILE_NAME);

    // Check if config already exists
    if config_path.exists() && !args.force {
        bail!("{CONFIG_FILE_NAME} already exists. Use --force to overwrite.");
    }

    // Detect project type
    let project_type = ProjectType::detect();

    println!("Initializing unduler configuration...\n");
    println!("  Project type: {}", project_type.description());
    println!("  Parser: {}", args.parser.as_config_name());

    let version_files = project_type.version_files();
    if version_files.is_empty() {
        println!("  Version files: (none detected)");
    } else {
        println!("  Version files: {}", version_files.join(", "));
    }

    // Generate and write config
    let config_content = generate_config(args.parser, project_type);

    fs::write(config_path, &config_content)
        .with_context(|| format!("failed to write {CONFIG_FILE_NAME}"))?;

    println!("\nCreated {CONFIG_FILE_NAME}");

    if matches!(args.parser, ParserType::Regex) {
        println!("\nNote: Regex parser requires manual configuration.");
        println!("Edit {CONFIG_FILE_NAME} to set your custom pattern.");
    }

    if version_files.is_empty() {
        println!("\nWarning: No version files detected.");
        println!("Edit {CONFIG_FILE_NAME} to add your version files manually.");
    }

    // Suggest plugins based on configuration
    if !args.no_plugins {
        suggest_plugins(args.parser, project_type);
    }

    Ok(())
}

/// Suggests plugins to install based on configuration.
fn suggest_plugins(parser: ParserType, project_type: ProjectType) {
    let mut plugins = Vec::new();

    // Parser plugin based on selection
    let parser_plugin = match parser {
        ParserType::Conventional => "unduler-parser-conventional",
        ParserType::ConventionalGitmoji => "unduler-parser-gitmoji",
        ParserType::Regex => "unduler-parser-regex",
    };
    plugins.push(parser_plugin);

    // Always suggest these core plugins
    plugins.push("unduler-bumper-semver");
    plugins.push("unduler-formatter-keepachangelog");

    // Hook plugins based on project type
    match project_type {
        ProjectType::Rust => {
            plugins.push("unduler-hook-cargo");
        }
        ProjectType::Node => {
            plugins.push("unduler-hook-npm");
        }
        ProjectType::RustAndNode => {
            plugins.push("unduler-hook-cargo");
            plugins.push("unduler-hook-npm");
        }
        ProjectType::Unknown => {}
    }

    println!("\nRecommended plugins:");
    for plugin in &plugins {
        println!("  - {plugin}");
    }

    println!("\nInstall all with:");
    println!(
        "  unduler plugin install {}",
        plugins
            .iter()
            .map(|p| p.strip_prefix("unduler-").unwrap_or(p))
            .collect::<Vec<_>>()
            .join(" ")
    );

    println!("\nOr install individually:");
    println!("  unduler plugin install <name>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_type_as_config_name() {
        assert_eq!(ParserType::Conventional.as_config_name(), "conventional");
        assert_eq!(
            ParserType::ConventionalGitmoji.as_config_name(),
            "conventional-gitmoji"
        );
        assert_eq!(ParserType::Regex.as_config_name(), "regex");
    }

    #[test]
    fn test_project_type_version_files() {
        assert_eq!(ProjectType::Rust.version_files(), vec!["Cargo.toml"]);
        assert_eq!(ProjectType::Node.version_files(), vec!["package.json"]);
        assert_eq!(
            ProjectType::RustAndNode.version_files(),
            vec!["Cargo.toml", "package.json"]
        );
        assert!(ProjectType::Unknown.version_files().is_empty());
    }

    #[test]
    fn test_project_type_description() {
        assert_eq!(ProjectType::Rust.description(), "Rust");
        assert_eq!(ProjectType::Node.description(), "Node.js");
        assert_eq!(ProjectType::RustAndNode.description(), "Rust + Node.js");
        assert_eq!(ProjectType::Unknown.description(), "Unknown");
    }

    #[test]
    fn test_generate_config_conventional() {
        let config = generate_config(ParserType::Conventional, ProjectType::Rust);
        assert!(config.contains("name = \"conventional\""));
        assert!(config.contains("files = [\"Cargo.toml\"]"));
        assert!(config.contains("tag_prefix = \"v\""));
        assert!(config.contains("output = \"CHANGELOG.md\""));
    }

    #[test]
    fn test_generate_config_gitmoji() {
        let config = generate_config(ParserType::ConventionalGitmoji, ProjectType::Node);
        assert!(config.contains("name = \"conventional-gitmoji\""));
        assert!(config.contains("[parser.conventional-gitmoji]"));
        assert!(config.contains("infer_type_from_emoji = true"));
        assert!(config.contains("files = [\"package.json\"]"));
    }

    #[test]
    fn test_generate_config_regex() {
        let config = generate_config(ParserType::Regex, ProjectType::Unknown);
        assert!(config.contains("name = \"regex\""));
        assert!(config.contains("[parser.regex]"));
        assert!(config.contains("# pattern ="));
        assert!(!config.contains("files = ")); // No version files for unknown
    }

    #[test]
    fn test_generate_config_hybrid_project() {
        let config = generate_config(ParserType::Conventional, ProjectType::RustAndNode);
        assert!(config.contains("files = [\"Cargo.toml\", \"package.json\"]"));
    }
}
