//! Initialize command.

use anyhow::Result;
use clap::Args;

/// Arguments for the init command.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Force overwrite existing configuration
    #[arg(short, long)]
    pub force: bool,
}

/// Runs the init command.
#[allow(clippy::unnecessary_wraps)] // Will return errors when implemented
pub fn run(_args: InitArgs) -> Result<()> {
    // TODO: Implement init command
    println!("Init command not yet implemented");
    Ok(())
}
