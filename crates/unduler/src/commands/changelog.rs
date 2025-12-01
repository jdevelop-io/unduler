//! Changelog command.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Args;
use semver::Version;
use tracing::info;

use unduler_bumper_semver::SemverBumper;
use unduler_commit::{ParsedCommit, RawCommit};
use unduler_config::{Config, find_and_load_config};
use unduler_formatter_keepachangelog::KeepAChangelogFormatter;
use unduler_git::Repository;
use unduler_parser_conventional::ConventionalParser;
use unduler_parser_gitmoji::{ConventionalGitmojiParser, GitmojiParserConfig};
use unduler_parser_regex::{FieldMapping, RegexParser, RegexParserConfig};
use unduler_plugin::{
    BumpStrategy, BumpType, ChangelogFormatter, CommitParser, FormatterConfig, Release,
};

/// Arguments for the changelog command.
#[derive(Debug, Args)]
pub struct ChangelogArgs {
    /// Output file (default: from config or CHANGELOG.md)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Generate changelog for unreleased changes only
    #[arg(short, long)]
    pub unreleased: bool,

    /// Print to stdout instead of writing to file
    #[arg(long)]
    pub dry_run: bool,
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

    // Build field mapping from config
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

/// Determines the next version based on commits and current version.
fn determine_next_version(current_version: &Version, parsed_commits: &[ParsedCommit]) -> Version {
    let bumper = SemverBumper::new();
    let bump_type = bumper.determine(parsed_commits);

    match bump_type {
        BumpType::Major => Version::new(current_version.major + 1, 0, 0),
        BumpType::Minor => Version::new(current_version.major, current_version.minor + 1, 0),
        BumpType::Patch | BumpType::None => Version::new(
            current_version.major,
            current_version.minor,
            current_version.patch + 1,
        ),
    }
}

/// Writes the changelog to a file, merging with existing content.
fn write_changelog(
    changelog: &str,
    output_path: &PathBuf,
    version: &Version,
    unreleased: bool,
) -> Result<()> {
    let existing = fs::read_to_string(output_path).unwrap_or_default();

    let new_content = if existing.is_empty() {
        format!(
            "# Changelog\n\n\
             All notable changes to this project will be documented in this file.\n\n\
             The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),\n\
             and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n\
             {changelog}"
        )
    } else if let Some(pos) = existing.find("\n## ") {
        let (header, rest) = existing.split_at(pos + 1);
        format!("{header}{changelog}{rest}")
    } else {
        format!("{existing}\n{changelog}")
    };

    fs::write(output_path, new_content)
        .with_context(|| format!("failed to write changelog to {}", output_path.display()))?;

    if unreleased {
        println!(
            "Changelog updated with unreleased changes: {}",
            output_path.display()
        );
    } else {
        println!(
            "Changelog updated for version {version}: {}",
            output_path.display()
        );
    }

    Ok(())
}

/// Runs the changelog command.
pub fn run(args: ChangelogArgs) -> Result<()> {
    let config = find_and_load_config().context("failed to load configuration")?;
    let repo = Repository::discover().context("failed to open git repository")?;
    let tag_prefix = &config.version.tag_prefix;

    let latest_tag = repo
        .latest_version_tag(tag_prefix)
        .context("failed to get latest version tag")?;

    info!(tag = ?latest_tag, "found latest version tag");

    let raw_commits = repo
        .commits_since(latest_tag.as_deref())
        .context("failed to get commits")?;

    if raw_commits.is_empty() {
        println!("No commits found since last release");
        return Ok(());
    }

    info!(count = raw_commits.len(), "found commits to process");

    let parser = create_parser(&config);
    info!(parser = parser.name(), "using parser");

    let parsed_commits = parse_commits(parser.as_ref(), &raw_commits);

    if parsed_commits.is_empty() {
        println!("No parseable commits found");
        return Ok(());
    }

    info!(count = parsed_commits.len(), "parsed commits");

    let version = if args.unreleased {
        Version::new(0, 0, 0)
    } else if let Some(current_version) = latest_tag
        .as_ref()
        .and_then(|tag| tag.strip_prefix(tag_prefix))
        .and_then(|v| Version::parse(v).ok())
    {
        // Tag exists: bump based on commits
        determine_next_version(&current_version, &parsed_commits)
    } else {
        // No tag: first release is 0.1.0 (standard SemVer convention)
        Version::new(0, 1, 0)
    };

    let mut release = Release::new(version.clone(), Utc::now(), parsed_commits);

    if let Some(ref tag) = latest_tag
        && let Some(prev_version) = tag.strip_prefix(tag_prefix)
        && let Ok(v) = Version::parse(prev_version)
    {
        release = release.with_previous_version(v);
    }

    let formatter = KeepAChangelogFormatter::new();
    let changelog = formatter.format(&release, &FormatterConfig::default());

    if args.dry_run {
        println!("{changelog}");
    } else {
        let output_path = args
            .output
            .map_or_else(|| PathBuf::from(&config.changelog.output), PathBuf::from);
        write_changelog(&changelog, &output_path, &version, args.unreleased)?;
    }

    Ok(())
}
