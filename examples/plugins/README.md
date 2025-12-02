# Creating Unduler WASM Plugins

This directory contains example WASM plugins for Unduler. Use these as templates for creating your own plugins.

## Prerequisites

- Rust toolchain (via rustup)
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`
- wasm-tools: `cargo install wasm-tools`

## Plugin Types

Unduler supports four plugin types:

| Type | Interface | Description |
|------|-----------|-------------|
| **Parser** | `unduler:plugin/parser` | Parses raw git commits into structured data |
| **Bumper** | `unduler:plugin/bumper` | Determines version bump type from commits |
| **Formatter** | `unduler:plugin/formatter` | Formats changelog output |
| **Hook** | `unduler:plugin/hook` | Executes actions during release lifecycle |

## Quick Start

### 1. Create a new project

```bash
mkdir my-plugin && cd my-plugin
cargo init --lib
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2024"

# Exclude from workspace
[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.41"

[profile.release]
opt-level = "s"
lto = true
```

### 3. Implement the plugin

```rust
// src/lib.rs
wit_bindgen::generate!({
    world: "unduler-parser",  // or unduler-bumper, unduler-formatter, unduler-hook
    path: "path/to/unduler/crates/unduler-plugin-sdk/wit",
});

use exports::unduler::plugin::parser::Guest;
use unduler::plugin::types::*;

struct MyParser;

impl Guest for MyParser {
    fn info() -> PluginInfo {
        PluginInfo {
            name: "my-parser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "My custom parser".to_string(),
            plugin_type: PluginType::Parser,
        }
    }

    fn parse(commit: RawCommit) -> Option<ParsedCommit> {
        // Your parsing logic here
        None
    }

    fn can_parse(commit: RawCommit) -> bool {
        false
    }
}

export!(MyParser);
```

### 4. Build the WASM module

```bash
cargo build --release --target wasm32-unknown-unknown
```

### 5. Convert to WASM component

```bash
wasm-tools component new \
    target/wasm32-unknown-unknown/release/my_plugin.wasm \
    -o my-plugin.wasm
```

### 6. Install the plugin

```bash
# Copy to Unduler plugins directory
cp my-plugin.wasm ~/.unduler/plugins/

# Or use the CLI (when publishing to crates.io)
unduler plugin install my-plugin
```

## Plugin Interfaces

### Parser Plugin

```rust
impl Guest for MyParser {
    fn info() -> PluginInfo;
    fn parse(commit: RawCommit) -> Option<ParsedCommit>;
    fn can_parse(commit: RawCommit) -> bool;
}
```

### Bumper Plugin

```rust
impl Guest for MyBumper {
    fn info() -> PluginInfo;
    fn determine(commits: Vec<ParsedCommit>) -> BumpType;
}
```

### Formatter Plugin

```rust
impl Guest for MyFormatter {
    fn info() -> PluginInfo;
    fn format(release: Release, config: FormatterConfig) -> String;
}
```

### Hook Plugin

Hooks can return actions that the host will execute:

```rust
impl Guest for MyHook {
    fn info() -> PluginInfo;
    fn on_pre_bump(ctx: ReleaseContext) -> HookResult;
    fn on_post_bump(ctx: ReleaseContext) -> HookResult;
    fn on_pre_commit(ctx: ReleaseContext) -> HookResult;
    fn on_pre_tag(ctx: ReleaseContext) -> HookResult;
    fn on_post_tag(ctx: ReleaseContext) -> HookResult;
}
```

Available hook actions:
- `run-command`: Execute whitelisted commands (cargo, npm, yarn, pnpm, gh, git)
- `write-file`: Write content to a file
- `log-message`: Log a message

## Types Reference

### RawCommit

```rust
struct RawCommit {
    hash: String,
    message: String,
    author: String,
    email: String,
    timestamp: i64,
}
```

### ParsedCommit

```rust
struct ParsedCommit {
    hash: String,
    commit_type: String,
    scope: Option<String>,
    message: String,
    breaking: bool,
    emoji: Option<String>,
    metadata: Vec<(String, String)>,
    author: String,
    timestamp: i64,
}
```

### BumpType

```rust
enum BumpType {
    Major,
    Minor,
    Patch,
    None,
}
```

### HookResult

```rust
struct HookResult {
    success: bool,
    error_message: Option<String>,
    metadata_updates: Vec<(String, String)>,
    actions: Vec<HookAction>,
}
```

## Examples

- `parser-conventional-wasm/` - Conventional Commits parser
- `bumper-semver-wasm/` - SemVer bump strategy

## Publishing to crates.io

1. Add metadata to Cargo.toml:

```toml
[package]
# ... other fields
keywords = ["unduler", "plugin", "parser"]
categories = ["development-tools"]

[package.metadata.unduler]
plugin-type = "parser"
```

2. Publish your crate:

```bash
cargo publish
```

3. Create a GitHub Release with the compiled `.wasm` file attached.

4. Users can then install with:

```bash
unduler plugin install your-plugin-name
```

## Troubleshooting

### "can't find crate for `core`"

Make sure you're using rustup's toolchain:

```bash
# Check active toolchain
rustup show

# Reinstall WASM target if needed
rustup target remove wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown
```

### Build fails with SIGKILL

Reduce parallel compilation:

```bash
cargo build --release --target wasm32-unknown-unknown -j 2
```

### Component conversion fails

Ensure you're using a compatible version of wasm-tools:

```bash
cargo install wasm-tools
```
