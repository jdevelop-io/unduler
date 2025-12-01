//! Release command.

use anyhow::Result;
use clap::Args;

/// Arguments for the release command.
#[derive(Debug, Args)]
pub struct ReleaseArgs {
    /// Perform a dry run without making changes
    #[arg(short, long)]
    pub dry_run: bool,

    /// Skip changelog generation
    #[arg(long)]
    pub no_changelog: bool,

    /// Skip git tag creation
    #[arg(long)]
    pub no_tag: bool,

    /// Skip git push
    #[arg(long)]
    pub no_push: bool,
}

/// Runs the release command.
#[allow(clippy::unnecessary_wraps)] // Will return errors when implemented
pub fn run(_args: ReleaseArgs) -> Result<()> {
    // TODO: Implement release command
    println!("Release command not yet implemented");
    Ok(())
}
