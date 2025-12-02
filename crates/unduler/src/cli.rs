//! CLI definition.

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands;

/// Automate version management and changelog generation for Git-based projects.
#[derive(Debug, Parser)]
#[command(name = "unduler")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize a new unduler configuration
    Init(commands::init::InitArgs),

    /// Bump the version based on commits
    Bump(commands::bump::BumpArgs),

    /// Generate changelog
    Changelog(commands::changelog::ChangelogArgs),

    /// Run a full release (bump + changelog + tag)
    Release(commands::release::ReleaseArgs),

    /// Manage plugins (install, remove, list, search)
    Plugin(commands::plugin::PluginArgs),
}

impl Cli {
    /// Runs the CLI command.
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::Init(args) => commands::init::run(args),
            Commands::Bump(args) => commands::bump::run(args),
            Commands::Changelog(args) => commands::changelog::run(args),
            Commands::Release(args) => commands::release::run(args),
            Commands::Plugin(args) => commands::plugin::run(args),
        }
    }
}
