//! Changelog command.

use anyhow::Result;
use clap::Args;

/// Arguments for the changelog command.
#[derive(Debug, Args)]
pub struct ChangelogArgs {
    /// Output file (default: CHANGELOG.md)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Generate changelog for unreleased changes only
    #[arg(short, long)]
    pub unreleased: bool,
}

/// Runs the changelog command.
#[allow(clippy::unnecessary_wraps)] // Will return errors when implemented
pub fn run(_args: ChangelogArgs) -> Result<()> {
    // TODO: Implement changelog command
    println!("Changelog command not yet implemented");
    Ok(())
}
