# Unduler

A modern Rust-based tool that automates version management and changelog generation for any Git-based project.

## Project Overview

**Unduler** provides:

- **Automatic Versioning:** Increment project version based on commits since the last tag
- **Changelog Generation:** Create structured, readable changelogs with customizable formatting
- **Extensible Plugin System:** Everything is a plugin - parsers, bumpers, formatters, and hooks
- **Multi-Convention Support:** Conventional Commits, Gitmoji, or custom regex formats

## Architecture

This is a Cargo workspace with multiple crates for separation of concerns.

```
unduler/
├── Cargo.toml                      # Workspace root
│
├── crates/
│   ├── unduler/                    # Binary crate (CLI)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── cli.rs              # CLI definition (clap)
│   │       └── commands/
│   │           ├── mod.rs
│   │           ├── bump.rs         # Version bump command
│   │           ├── changelog.rs    # Changelog generation command
│   │           ├── release.rs      # Full release workflow
│   │           └── init.rs         # Config initialization
│   │
│   ├── unduler-core/               # Core library
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── version.rs          # SemVer types and logic
│   │       ├── changelog.rs        # Changelog generation orchestration
│   │       ├── release.rs          # Release orchestration
│   │       ├── pipeline.rs         # Plugin pipeline execution
│   │       └── error.rs            # Error types
│   │
│   ├── unduler-git/                # Git abstraction layer
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── repository.rs       # Repository operations
│   │       ├── commit.rs           # Commit retrieval
│   │       ├── tag.rs              # Tag management
│   │       └── diff.rs             # Commits since tag
│   │
│   ├── unduler-commit/             # Commit types (no parsing logic)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── raw.rs              # RawCommit (from git)
│   │       └── parsed.rs           # ParsedCommit (after parsing)
│   │
│   ├── unduler-plugin/             # Unified plugin system
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits/
│   │       │   ├── mod.rs
│   │       │   ├── parser.rs       # CommitParser trait
│   │       │   ├── bumper.rs       # BumpStrategy trait
│   │       │   ├── formatter.rs    # ChangelogFormatter trait
│   │       │   └── hook.rs         # ReleaseHook trait
│   │       ├── registry.rs         # Plugin registry
│   │       ├── loader.rs           # Plugin loading
│   │       └── context.rs          # ReleaseContext shared state
│   │
│   └── unduler-config/             # Configuration management
│       └── src/
│           ├── lib.rs
│           ├── schema.rs           # Config structure
│           └── loader.rs           # File loading (TOML)
│
└── plugins/                        # Built-in plugins
    ├── parser-conventional/        # Conventional Commits parser
    ├── parser-conventional-gitmoji/# Conventional + Gitmoji (depends on parser-conventional)
    ├── parser-regex/               # Custom format via regex
    ├── bumper-semver/              # SemVer bump strategy
    ├── formatter-keepachangelog/   # Keep a Changelog format
    ├── hook-cargo/                 # Rust/Cargo support
    ├── hook-npm/                   # Node.js/npm support
    └── hook-github-release/        # GitHub Release creation
```

## Parser Plugins

Three parsers with increasing flexibility levels.

### parser-conventional

Base parser for standard Conventional Commits format.

```
type(scope): message
type(scope)!: message    # Breaking change
type: message            # Without scope
```

Examples:
- `feat(api): add new endpoint`
- `fix(auth)!: change token format`
- `docs: update readme`

### parser-conventional-gitmoji

Extends `parser-conventional` with Gitmoji prefix support. Depends on `parser-conventional`.

```
emoji type(scope): message
emoji message              # Type inferred from emoji
```

**Flow:**

```
"✨ feat(api): add endpoint"
         │
         ▼
┌────────────────────────────┐
│  parser-conventional-gitmoji│
│  1. Extract ✨              │
│  2. Remaining: "feat(api): add endpoint"
└──────────────┬─────────────┘
               │ delegate
               ▼
┌────────────────────────────┐
│   parser-conventional      │
│   Parse type, scope, msg   │
└──────────────┬─────────────┘
               │
               ▼
ParsedCommit {
    emoji: Some("✨"),
    type: "feat",
    scope: Some("api"),
    message: "add endpoint",
}
```

**Emoji without explicit type:**

```
"✨ add endpoint"
         │
         ▼
ParsedCommit {
    emoji: Some("✨"),
    type: "feat",        # Inferred from emoji mapping
    scope: None,
    message: "add endpoint",
}
```

### parser-regex

Full flexibility for custom/legacy formats via named capture groups.

```toml
[parser]
name = "regex"

[parser.regex]
pattern = '''
  ^(?P<ticket>[A-Z]+-\d+)\s+
  (?P<type>\w+)
  (?:\((?P<scope>\w+)\))?:\s+
  (?P<message>.+)$
'''

[parser.regex.mapping]
type = "type"
scope = "scope"
message = "message"
ticket = "ticket"        # Custom field → metadata

[parser.regex.validation]
type = ["feat", "fix", "docs", "chore", "refactor", "test"]
```

**Supported custom formats examples:**

```
# JIRA-style
PROJ-123 feat(api): add endpoint
→ ^(?P<ticket>[A-Z]+-\d+)\s+(?P<type>\w+)(?:\((?P<scope>\w+)\))?:\s+(?P<message>.+)$

# Simple internal format
[API] Added new endpoint
→ ^\[(?P<scope>\w+)\]\s+(?P<message>.+)$

# Legacy format
bugfix/login: fix authentication
→ ^(?P<type>\w+)/(?P<scope>\w+):\s+(?P<message>.+)$
```

### Parser Summary

| Parser | Format | Use Case |
|--------|--------|----------|
| `conventional` | `type(scope): msg` | Standard projects |
| `conventional-gitmoji` | `✨ type(scope): msg` | Projects using emojis |
| `regex` | Custom pattern | Legacy/internal formats |

## Commit Types

```rust
// In unduler-commit crate

pub struct RawCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Utc>,
}

pub struct ParsedCommit {
    pub hash: String,
    pub r#type: String,
    pub scope: Option<String>,
    pub message: String,
    pub breaking: bool,
    pub emoji: Option<String>,
    pub metadata: HashMap<String, String>,  // Custom fields (ticket, etc.)
}
```

## Plugin System

Everything is a plugin. The core orchestrates plugins through a configurable pipeline.

### Plugin Traits

```rust
/// Base trait for all plugins
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
}

/// Parses raw commit messages into structured data
pub trait CommitParser: Plugin {
    fn parse(&self, raw: &RawCommit) -> Option<ParsedCommit>;
}

/// Determines version bump type from parsed commits
pub trait BumpStrategy: Plugin {
    fn determine(&self, commits: &[ParsedCommit]) -> BumpType;
}

/// Formats changelog output
pub trait ChangelogFormatter: Plugin {
    fn format(&self, release: &Release, config: &FormatterConfig) -> String;
}

/// Lifecycle hooks during release process
pub trait ReleaseHook: Plugin {
    fn on_pre_bump(&self, ctx: &mut ReleaseContext) -> Result<()> { Ok(()) }
    fn on_post_bump(&self, ctx: &mut ReleaseContext) -> Result<()> { Ok(()) }
    fn on_pre_commit(&self, ctx: &mut ReleaseContext) -> Result<()> { Ok(()) }
    fn on_pre_tag(&self, ctx: &mut ReleaseContext) -> Result<()> { Ok(()) }
    fn on_post_tag(&self, ctx: &mut ReleaseContext) -> Result<()> { Ok(()) }
}
```

### Release Context

Shared state passed to all hooks:

```rust
pub struct ReleaseContext {
    pub repo: Repository,
    pub previous_version: Version,
    pub next_version: Version,
    pub bump_type: BumpType,
    pub commits: Vec<ParsedCommit>,
    pub changelog: Option<String>,
    pub dry_run: bool,
    pub metadata: HashMap<String, Value>,  // Inter-hook communication
}
```

## Release Pipeline

```
unduler release
       │
       ▼
┌──────────────────┐
│  Parser          │  Plugin: conventional-gitmoji, conventional, or regex
│  RawCommit →     │
│  ParsedCommit    │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Bump Strategy   │  Plugin: bumper-semver
│  Determine bump  │
└────────┬─────────┘
         │
         ▼
    PRE_BUMP ←────────── Hooks: validate, prepare
         │
         ▼
┌──────────────────┐
│  Update version  │
│  files           │
└────────┬─────────┘
         │
         ▼
    POST_BUMP ←───────── Hooks: sync lock files, build
         │
         ▼
┌──────────────────┐
│  Formatter       │  Plugin: formatter-keepachangelog
│  Generate        │
│  changelog       │
└────────┬─────────┘
         │
         ▼
    PRE_COMMIT ←──────── Hooks: lint, format
         │
         ▼
┌──────────────────┐
│  Git commit      │
└────────┬─────────┘
         │
         ▼
    PRE_TAG ←─────────── Hooks: final checks
         │
         ▼
┌──────────────────┐
│  Git tag         │
└────────┬─────────┘
         │
         ▼
    POST_TAG ←────────── Hooks: push, publish, notify
```

## Hook Lifecycle

| Hook | When | Use Cases |
|------|------|-----------|
| `pre_bump` | Before version modification | Validation, monorepo package detection |
| `post_bump` | After version modification | Update lock files, internal deps |
| `pre_commit` | Before git commit | Linting, formatting |
| `pre_tag` | Before git tag | Final verification |
| `post_tag` | After git tag | Publish, deploy, notify |

## Configuration

Configuration file: `unduler.toml`

```toml
[parser]
name = "conventional-gitmoji"    # or "conventional", "regex"

# Gitmoji-specific options
[parser.conventional-gitmoji]
infer_type_from_emoji = true     # If no type, infer from emoji
strict_emoji = false             # Reject unknown emojis

# Regex-specific options (when parser.name = "regex")
[parser.regex]
pattern = "..."
[parser.regex.mapping]
type = "type"
scope = "scope"
message = "message"

[bumper]
name = "semver"

[formatter]
name = "keepachangelog"

[hooks]
pre_bump = []
post_bump = ["cargo", "npm"]
pre_commit = []
pre_tag = []
post_tag = ["github-release"]

[version]
files = ["Cargo.toml", "package.json"]
tag_prefix = "v"

[changelog]
output = "CHANGELOG.md"

# Plugin-specific configuration
[plugins.cargo]
publish = true

[plugins.github-release]
draft = false
assets = ["target/release/unduler"]
```

## Key Dependencies

```toml
# Git operations
git2 = "0.19"

# Configuration
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }

# CLI
clap = { version = "4", features = ["derive"] }

# Core
semver = "1.0"
thiserror = "2.0"
anyhow = "1.0"

# Regex (for parser-regex)
regex = "1.10"
```

## Commit Conventions

This project uses **Conventional Commits** with **Gitmoji** prefix.

Format: `<gitmoji> <type>(<scope>): <subject>`

Examples:
- `✨ feat(parser): add support for custom commit formats`
- `🐛 fix(git): handle detached HEAD state`
- `♻️ refactor(core): simplify pipeline execution`

## Development Guidelines

- Each crate has a single responsibility
- Plugins implement traits from `unduler-plugin`
- Use `thiserror` for library errors, `anyhow` for CLI
- Prefer composition over inheritance (e.g., `parser-conventional-gitmoji` depends on `parser-conventional`)
- All public APIs must be documented
