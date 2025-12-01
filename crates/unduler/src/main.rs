//! Unduler CLI - Automate version management and changelog generation.

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod cli;
mod commands;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Parse CLI arguments and run
    let cli = cli::Cli::parse();
    cli.run()
}
