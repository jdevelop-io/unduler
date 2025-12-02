//! Bump command.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};
use semver::Version;
use tracing::info;

use unduler_bumper_semver::SemverBumper;
use unduler_commit::{ParsedCommit, RawCommit};
use unduler_config::{Config, find_and_load_config};
use unduler_core::update_version_file;
use unduler_git::Repository;
use unduler_parser_conventional::ConventionalParser;
use unduler_parser_gitmoji::{ConventionalGitmojiParser, GitmojiParserConfig};
use unduler_parser_regex::{FieldMapping, RegexParser, RegexParserConfig};
use unduler_plugin::{BumpStrategy, BumpType, CommitParser};

/// Bump type argument.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BumpTypeArg {
    /// Major version bump (breaking changes)
    Major,
    /// Minor version bump (new features)
    Minor,
    /// Patch version bump (bug fixes)
    Patch,
}

impl From<BumpTypeArg> for BumpType {
    fn from(arg: BumpTypeArg) -> Self {
        match arg {
            BumpTypeArg::Major => BumpType::Major,
            BumpTypeArg::Minor => BumpType::Minor,
            BumpTypeArg::Patch => BumpType::Patch,
        }
    }
}

/// Arguments for the bump command.
#[derive(Debug, Args)]
pub struct BumpArgs {
    /// Perform a dry run without making changes
    #[arg(short, long)]
    pub dry_run: bool,

    /// Force a specific bump type (major, minor, patch)
    #[arg(short = 't', long, value_enum)]
    pub bump_type: Option<BumpTypeArg>,
}

/// Creates the appropriate parser based on configuration.
fn create_parser(config: &Config) -> Box<dyn CommitParser> {
    match config.parser.name.as_str() {
        "gitmoji" | "conventional-gitmoji" => create_gitmoji_parser(config),
        "regex" => create_regex_parser(config),
        _ => Box::new(ConventionalParser::new()),
    }
}

fn create_gitmoji_parser(config: &Config) -> Box<dyn CommitParser> {
    let parser_config = GitmojiParserConfig {
        infer_type_from_emoji: config.parser.conventional_gitmoji.infer_type_from_emoji,
        strict_emoji: config.parser.conventional_gitmoji.strict_emoji,
    };
    Box::new(ConventionalGitmojiParser::with_config(parser_config))
}

fn create_regex_parser(config: &Config) -> Box<dyn CommitParser> {
    let Some(ref pattern) = config.parser.regex.pattern else {
        info!("no regex pattern configured, falling back to conventional");
        return Box::new(ConventionalParser::new());
    };

    let mut metadata_mapping = std::collections::HashMap::new();
    for (field, capture) in &config.parser.regex.mapping {
        if !["type", "scope", "message"].contains(&field.as_str()) {
            metadata_mapping.insert(field.clone(), capture.clone());
        }
    }

    let mapping = FieldMapping {
        r#type: config
            .parser
            .regex
            .mapping
            .get("type")
            .cloned()
            .unwrap_or_else(|| "type".to_string()),
        scope: config.parser.regex.mapping.get("scope").cloned(),
        message: config
            .parser
            .regex
            .mapping
            .get("message")
            .cloned()
            .unwrap_or_else(|| "message".to_string()),
        metadata: metadata_mapping,
    };

    let parser_config = RegexParserConfig {
        pattern: pattern.clone(),
        mapping,
        validation: config.parser.regex.validation.clone(),
    };

    match RegexParser::new(parser_config) {
        Ok(parser) => Box::new(parser),
        Err(e) => {
            info!("invalid regex pattern, falling back to conventional: {e}");
            Box::new(ConventionalParser::new())
        }
    }
}

/// Parses raw commits using the given parser.
fn parse_commits(parser: &dyn CommitParser, raw_commits: &[RawCommit]) -> Vec<ParsedCommit> {
    raw_commits
        .iter()
        .filter_map(|raw| {
            let parsed = parser.parse(raw);
            if parsed.is_none() {
                info!(
                    hash = %raw.short_hash(),
                    subject = %raw.subject(),
                    "skipping unparseable commit"
                );
            }
            parsed
        })
        .collect()
}

/// Determines the bump type from commits.
fn determine_bump_type(parsed_commits: &[ParsedCommit]) -> BumpType {
    let bumper = SemverBumper::new();
    bumper.determine(parsed_commits)
}

/// Calculates the next version.
fn calculate_next_version(current: &Version, bump_type: BumpType) -> Version {
    match bump_type {
        BumpType::Major => Version::new(current.major + 1, 0, 0),
        BumpType::Minor => Version::new(current.major, current.minor + 1, 0),
        BumpType::Patch | BumpType::None => {
            Version::new(current.major, current.minor, current.patch + 1)
        }
    }
}

/// Runs the bump command.
#[allow(clippy::needless_pass_by_value)]
pub fn run(args: BumpArgs) -> Result<()> {
    let config = find_and_load_config().context("failed to load configuration")?;
    let repo = Repository::discover().context("failed to open git repository")?;
    let tag_prefix = &config.version.tag_prefix;

    // Get latest version tag
    let latest_tag = repo
        .latest_version_tag(tag_prefix)
        .context("failed to get latest version tag")?;

    info!(tag = ?latest_tag, "found latest version tag");

    // Determine bump type
    let bump_type = if let Some(forced) = args.bump_type {
        info!(bump_type = ?forced, "using forced bump type");
        forced.into()
    } else {
        // Get commits and determine from them
        let raw_commits = repo
            .commits_since(latest_tag.as_deref())
            .context("failed to get commits")?;

        if raw_commits.is_empty() {
            bail!("no commits found since last release");
        }

        info!(count = raw_commits.len(), "found commits to analyze");

        let parser = create_parser(&config);
        info!(parser = parser.name(), "using parser");

        let parsed_commits = parse_commits(parser.as_ref(), &raw_commits);

        if parsed_commits.is_empty() {
            bail!("no parseable commits found");
        }

        let determined = determine_bump_type(&parsed_commits);
        info!(bump_type = %determined, "determined bump type from commits");
        determined
    };

    // Calculate versions
    let current_version = latest_tag
        .as_ref()
        .and_then(|tag| tag.strip_prefix(tag_prefix))
        .and_then(|v| Version::parse(v).ok());

    let (current_version, new_version) = if let Some(current) = current_version {
        let new = calculate_next_version(&current, bump_type);
        (current, new)
    } else {
        // No tag: first release is 0.1.0
        (Version::new(0, 0, 0), Version::new(0, 1, 0))
    };

    info!(
        current = %current_version,
        new = %new_version,
        "version bump"
    );

    // Update version files
    let version_files = &config.version.files;

    if version_files.is_empty() {
        println!("No version files configured. Would bump {current_version} -> {new_version}");
        return Ok(());
    }

    let mut updated_count = 0;
    let mut errors = Vec::new();

    for file_path in version_files {
        let path = PathBuf::from(file_path);

        if args.dry_run {
            println!("Would update {file_path} to version {new_version}");
        } else {
            match update_version_file(&path, &new_version, false) {
                Ok(()) => {
                    println!("Updated {file_path} to version {new_version}");
                    updated_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to update {file_path}: {e}");
                    errors.push((file_path.clone(), e));
                }
            }
        }
    }

    // Summary
    if args.dry_run {
        println!("\nDry run: would bump version {current_version} -> {new_version}");
    } else if errors.is_empty() {
        println!(
            "\nBumped version {current_version} -> {new_version} ({updated_count} file(s) updated)"
        );
    } else {
        let error_count = errors.len();
        println!(
            "\nPartially bumped version {current_version} -> {new_version} ({updated_count} file(s) updated, {error_count} error(s))"
        );
    }

    Ok(())
}
