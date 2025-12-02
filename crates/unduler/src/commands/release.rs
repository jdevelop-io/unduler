//! Release command.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::Utc;
use clap::{Args, ValueEnum};
use semver::Version;
use tracing::info;

use unduler_bumper_semver::SemverBumper;
use unduler_commit::{ParsedCommit, RawCommit};
use unduler_config::{Config, find_and_load_config};
use unduler_core::update_version_file;
use unduler_formatter_keepachangelog::KeepAChangelogFormatter;
use unduler_git::Repository;
use unduler_parser_conventional::ConventionalParser;
use unduler_parser_gitmoji::{ConventionalGitmojiParser, GitmojiParserConfig};
use unduler_parser_regex::{FieldMapping, RegexParser, RegexParserConfig};
use unduler_plugin::{
    BumpStrategy, BumpType, ChangelogFormatter, CommitParser, FormatterConfig, Release,
};

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

/// Arguments for the release command.
#[derive(Debug, Args)]
pub struct ReleaseArgs {
    /// Perform a dry run without making changes
    #[arg(short, long)]
    pub dry_run: bool,

    /// Force a specific bump type (major, minor, patch)
    #[arg(short = 't', long, value_enum)]
    pub bump_type: Option<BumpTypeArg>,

    /// Skip changelog generation
    #[arg(long)]
    pub no_changelog: bool,

    /// Skip git tag creation
    #[arg(long)]
    pub no_tag: bool,

    /// Skip git commit
    #[arg(long)]
    pub no_commit: bool,
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

/// Updates all version files.
fn update_version_files(
    version_files: &[String],
    new_version: &Version,
    dry_run: bool,
) -> Vec<String> {
    let mut updated = Vec::new();

    for file_path in version_files {
        let path = PathBuf::from(file_path);

        if dry_run {
            println!("  Would update {file_path}");
            updated.push(file_path.clone());
        } else {
            match update_version_file(&path, new_version, false) {
                Ok(()) => {
                    println!("  Updated {file_path}");
                    updated.push(file_path.clone());
                }
                Err(e) => {
                    eprintln!("  Failed to update {file_path}: {e}");
                }
            }
        }
    }

    updated
}

/// Writes changelog to file.
fn write_changelog(
    changelog: &str,
    output_path: &PathBuf,
    version: &Version,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        println!("  Would update {}", output_path.display());
        return Ok(());
    }

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

    println!("  Updated {} for version {version}", output_path.display());
    Ok(())
}

/// Runs the release command.
#[allow(clippy::needless_pass_by_value)]
pub fn run(args: ReleaseArgs) -> Result<()> {
    let config = find_and_load_config().context("failed to load configuration")?;
    let repo = Repository::discover().context("failed to open git repository")?;
    let tag_prefix = &config.version.tag_prefix;

    println!("Starting release process...\n");

    // Step 1: Get latest version tag
    let latest_tag = repo
        .latest_version_tag(tag_prefix)
        .context("failed to get latest version tag")?;

    info!(tag = ?latest_tag, "found latest version tag");

    // Step 2: Determine bump type
    let bump_type = if let Some(forced) = args.bump_type {
        info!(bump_type = ?forced, "using forced bump type");
        forced.into()
    } else {
        let raw_commits = repo
            .commits_since(latest_tag.as_deref())
            .context("failed to get commits")?;

        if raw_commits.is_empty() {
            bail!("no commits found since last release");
        }

        info!(count = raw_commits.len(), "found commits to analyze");

        let parser = create_parser(&config);
        let parsed_commits = parse_commits(parser.as_ref(), &raw_commits);

        if parsed_commits.is_empty() {
            bail!("no parseable commits found");
        }

        let determined = determine_bump_type(&parsed_commits);
        info!(bump_type = %determined, "determined bump type from commits");
        determined
    };

    // Step 3: Calculate versions
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

    println!("Version: {current_version} -> {new_version} ({bump_type})\n");

    // Step 4: Update version files
    let version_files = &config.version.files;
    if !version_files.is_empty() {
        println!("Updating version files:");
        let updated = update_version_files(version_files, &new_version, args.dry_run);
        if updated.is_empty() && !args.dry_run {
            eprintln!("Warning: no version files were updated");
        }
        println!();
    }

    // Step 5: Generate and write changelog
    if !args.no_changelog {
        println!("Generating changelog:");

        // Re-parse commits for changelog generation
        let raw_commits = repo
            .commits_since(latest_tag.as_deref())
            .context("failed to get commits")?;

        let parser = create_parser(&config);
        let parsed_commits = parse_commits(parser.as_ref(), &raw_commits);

        let mut release = Release::new(new_version.clone(), Utc::now(), parsed_commits);
        if current_version != Version::new(0, 0, 0) {
            release = release.with_previous_version(current_version.clone());
        }

        let formatter = KeepAChangelogFormatter::new();
        let changelog = formatter.format(&release, &FormatterConfig::default());

        let output_path = PathBuf::from(&config.changelog.output);
        write_changelog(&changelog, &output_path, &new_version, args.dry_run)?;
        println!();
    }

    // Step 6: Create git commit
    if !args.no_commit {
        println!("Creating git commit:");
        let commit_message = format!("chore(release): {new_version}");

        if args.dry_run {
            println!("  Would create commit: {commit_message}");
        } else {
            repo.commit(&commit_message)
                .context("failed to create commit")?;
            println!("  Created commit: {commit_message}");
        }
        println!();
    }

    // Step 7: Create git tag
    if !args.no_tag {
        println!("Creating git tag:");
        let tag_name = format!("{tag_prefix}{new_version}");
        let tag_message = format!("Release {new_version}");

        if args.dry_run {
            println!("  Would create tag: {tag_name}");
        } else {
            repo.create_tag(&tag_name, &tag_message)
                .context("failed to create tag")?;
            println!("  Created tag: {tag_name}");
        }
        println!();
    }

    // Summary
    if args.dry_run {
        println!("Dry run completed. No changes were made.");
    } else {
        println!("Release {new_version} completed successfully!");
        println!("\nNext steps:");
        println!("  git push origin main --tags");
    }

    Ok(())
}
