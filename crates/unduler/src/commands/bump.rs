//! Bump command.

use anyhow::Result;
use clap::Args;

/// Arguments for the bump command.
#[derive(Debug, Args)]
pub struct BumpArgs {
    /// Perform a dry run without making changes
    #[arg(short, long)]
    pub dry_run: bool,

    /// Force a specific bump type (major, minor, patch)
    #[arg(short = 't', long)]
    pub bump_type: Option<String>,
}

/// Runs the bump command.
#[allow(clippy::unnecessary_wraps)] // Will return errors when implemented
pub fn run(_args: BumpArgs) -> Result<()> {
    // TODO: Implement bump command
    println!("Bump command not yet implemented");
    Ok(())
}
