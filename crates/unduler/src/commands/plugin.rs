//! Plugin management commands.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use unduler_plugin_manager::{PluginDiscovery, PluginRegistry, PluginStorage};

/// Plugin management commands.
#[derive(Debug, Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

#[derive(Debug, Subcommand)]
pub enum PluginCommand {
    /// Install a plugin from crates.io
    Install(InstallArgs),

    /// Remove an installed plugin
    Remove(RemoveArgs),

    /// Update installed plugins
    Update(UpdateArgs),

    /// List installed plugins
    List(ListArgs),

    /// Search for plugins on crates.io
    Search(SearchArgs),

    /// Show information about a plugin
    Info(InfoArgs),
}

/// Arguments for the `plugin install` command.
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Plugin name (e.g., "unduler-parser-conventional" or just "parser-conventional")
    pub name: String,

    /// Specific version to install (defaults to latest)
    #[arg(short, long)]
    pub version: Option<semver::Version>,
}

/// Arguments for the `plugin remove` command.
#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Plugin name
    pub name: String,
}

/// Arguments for the `plugin list` command.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by plugin type (parser, bumper, formatter, hook)
    #[arg(short = 't', long)]
    pub r#type: Option<String>,
}

/// Arguments for the `plugin search` command.
#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,
}

/// Arguments for the `plugin update` command.
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Plugin name (updates all if not specified)
    pub name: Option<String>,
}

/// Arguments for the `plugin info` command.
#[derive(Debug, Args)]
pub struct InfoArgs {
    /// Plugin name
    pub name: String,
}

/// Runs the plugin command.
pub fn run(args: PluginArgs) -> Result<()> {
    // Create a tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().context("failed to create async runtime")?;

    rt.block_on(async { run_async(args).await })
}

async fn run_async(args: PluginArgs) -> Result<()> {
    match args.command {
        PluginCommand::Install(args) => install(args).await,
        PluginCommand::Remove(ref args) => remove(args),
        PluginCommand::Update(args) => update(args).await,
        PluginCommand::List(ref args) => list(args),
        PluginCommand::Search(args) => search(args).await,
        PluginCommand::Info(args) => info(args).await,
    }
}

async fn install(args: InstallArgs) -> Result<()> {
    let crate_name = normalize_plugin_name(&args.name);

    println!("Installing {crate_name}...");

    let storage = PluginStorage::new().context("failed to initialize plugin storage")?;
    let mut registry = PluginRegistry::new(storage).context("failed to load plugin registry")?;
    let discovery = PluginDiscovery::new();

    let plugin = discovery
        .install(&mut registry, &crate_name, args.version.as_ref())
        .await
        .with_context(|| format!("failed to install {crate_name}"))?;

    println!(
        "Installed {} v{} ({})",
        plugin.short_name, plugin.version, plugin.crate_name
    );

    Ok(())
}

fn remove(args: &RemoveArgs) -> Result<()> {
    let crate_name = normalize_plugin_name(&args.name);

    let storage = PluginStorage::new().context("failed to initialize plugin storage")?;
    let mut registry = PluginRegistry::new(storage).context("failed to load plugin registry")?;
    let discovery = PluginDiscovery::new();

    discovery
        .uninstall(&mut registry, &crate_name)
        .with_context(|| format!("failed to remove {crate_name}"))?;

    println!("Removed {crate_name}");

    Ok(())
}

async fn update(args: UpdateArgs) -> Result<()> {
    let storage = PluginStorage::new().context("failed to initialize plugin storage")?;
    let mut registry = PluginRegistry::new(storage).context("failed to load plugin registry")?;
    let discovery = PluginDiscovery::new();

    let plugins_to_update: Vec<_> = if let Some(name) = &args.name {
        let crate_name = normalize_plugin_name(name);
        match registry.get(&crate_name) {
            Some(p) => vec![p.clone()],
            None => anyhow::bail!("plugin {crate_name} is not installed"),
        }
    } else {
        registry.list().into_iter().cloned().collect()
    };

    if plugins_to_update.is_empty() {
        println!("No plugins installed.");
        return Ok(());
    }

    let mut updated = 0;
    let mut up_to_date = 0;
    let mut errors = 0;

    for plugin in &plugins_to_update {
        print!("Checking {}... ", plugin.crate_name);

        match discovery.fetch_metadata(&plugin.crate_name).await {
            Ok(metadata) => {
                if metadata.latest_version > plugin.version {
                    println!("updating {} -> {}", plugin.version, metadata.latest_version);
                    match discovery
                        .install(&mut registry, &plugin.crate_name, None)
                        .await
                    {
                        Ok(_) => updated += 1,
                        Err(e) => {
                            println!("  error: {e}");
                            errors += 1;
                        }
                    }
                } else {
                    println!("up to date ({})", plugin.version);
                    up_to_date += 1;
                }
            }
            Err(e) => {
                println!("error: {e}");
                errors += 1;
            }
        }
    }

    println!();
    if updated > 0 {
        println!("Updated {updated} plugin(s)");
    }
    if up_to_date > 0 {
        println!("{up_to_date} plugin(s) already up to date");
    }
    if errors > 0 {
        println!("{errors} error(s) occurred");
    }

    Ok(())
}

fn list(args: &ListArgs) -> Result<()> {
    let storage = PluginStorage::new().context("failed to initialize plugin storage")?;
    let registry = PluginRegistry::new(storage).context("failed to load plugin registry")?;

    let plugins = if let Some(type_filter) = &args.r#type {
        let plugin_type = match type_filter.as_str() {
            "parser" => unduler_plugin_manager::storage::PluginType::Parser,
            "bumper" => unduler_plugin_manager::storage::PluginType::Bumper,
            "formatter" => unduler_plugin_manager::storage::PluginType::Formatter,
            "hook" => unduler_plugin_manager::storage::PluginType::Hook,
            _ => anyhow::bail!("unknown plugin type: {type_filter}"),
        };
        registry.list_by_type(plugin_type)
    } else {
        registry.list()
    };

    if plugins.is_empty() {
        println!("No plugins installed.");
        return Ok(());
    }

    println!("Installed plugins:\n");

    for plugin in plugins {
        println!("  {} v{}", plugin.crate_name, plugin.version);
        if let Some(desc) = &plugin.description {
            println!("    {desc}");
        }
    }

    Ok(())
}

async fn search(args: SearchArgs) -> Result<()> {
    let discovery = PluginDiscovery::new();

    println!("Searching for \"{}\"...\n", args.query);

    let results = discovery
        .search(&args.query)
        .await
        .context("failed to search crates.io")?;

    if results.is_empty() {
        println!("No plugins found.");
        return Ok(());
    }

    for result in results {
        let version = result.latest_version.as_deref().unwrap_or("?");
        println!(
            "  {} v{}  ({} downloads)",
            result.crate_name, version, result.downloads
        );
        if let Some(desc) = &result.description {
            println!("    {desc}");
        }
    }

    Ok(())
}

async fn info(args: InfoArgs) -> Result<()> {
    let crate_name = normalize_plugin_name(&args.name);
    let discovery = PluginDiscovery::new();

    println!("Fetching info for {crate_name}...\n");

    let metadata = discovery
        .fetch_metadata(&crate_name)
        .await
        .with_context(|| format!("failed to fetch info for {crate_name}"))?;

    println!("Name:        {}", metadata.crate_name);
    println!("Type:        {:?}", metadata.plugin_type);
    println!("Latest:      {}", metadata.latest_version);
    if let Some(desc) = &metadata.description {
        println!("Description: {desc}");
    }
    if let Some(repo) = &metadata.repository {
        println!("Repository:  {repo}");
    }

    if !metadata.versions.is_empty() {
        println!("\nAvailable versions:");
        for version in metadata.versions.iter().take(10) {
            println!("  - {version}");
        }
        if metadata.versions.len() > 10 {
            println!("  ... and {} more", metadata.versions.len() - 10);
        }
    }

    // Check if installed
    let storage = PluginStorage::new().context("failed to initialize plugin storage")?;
    let registry = PluginRegistry::new(storage).context("failed to load plugin registry")?;

    if let Some(installed) = registry.get(&crate_name) {
        println!("\nInstalled:   v{}", installed.version);
    }

    Ok(())
}

/// Normalizes a plugin name to its full crate name.
///
/// Accepts:
/// - Full name: "unduler-parser-conventional"
/// - Short name with prefix: "parser-conventional"
/// - Short name only: "conventional" (will try all prefixes)
fn normalize_plugin_name(name: &str) -> String {
    if name.starts_with("unduler-") {
        name.to_string()
    } else if name.starts_with("parser-")
        || name.starts_with("bumper-")
        || name.starts_with("formatter-")
        || name.starts_with("hook-")
    {
        format!("unduler-{name}")
    } else {
        // For now, just prepend unduler-parser- as the most common case
        // Could be smarter and try all prefixes
        format!("unduler-parser-{name}")
    }
}
