# Unduler

A modern Rust-based tool that automates version management and changelog generation for any Git-based project.

With a powerful plugin system, Unduler supports multiple commit conventions—Gitmoji, Conventional Commits, or custom
formats—and automatically determines the right version bump.

## Features

- **Automatic Versioning** — Increment your project's version based on commits since the last tag
- **Changelog Generation** — Create structured, readable changelogs with fully customizable formatting
- **Extensible Plugin System** — Customize parsing and versioning rules to match your internal conventions
- **Multiple Commit Formats** — Support for Conventional Commits, Gitmoji, or custom regex patterns
- **Ecosystem Hooks** — Integrate with Cargo, npm, and GitHub Releases
- **Consistent Release Workflow** — Streamline releases across different projects and ecosystems

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/jdevelop-io/unduler.git
cd unduler

# Build and install
cargo install --path crates/unduler
```

### Requirements

- Rust 1.90 or later
- Git

## Quick Start

1. **Initialize** a new configuration in your project:

```bash
unduler init
```

2. **Generate** a changelog based on your commits:

```bash
unduler changelog
```

3. **Bump** the version automatically:

```bash
unduler bump
```

4. **Release** (bump + changelog + tag):

```bash
unduler release
```

## Configuration

Unduler uses a `unduler.toml` configuration file at the root of your project:

```toml
[parser]
name = "conventional"  # or "conventional-gitmoji", "regex"

[bumper]
name = "semver"

[formatter]
name = "keepachangelog"

[version]
tag_prefix = "v"
files = ["Cargo.toml"]

[changelog]
output = "CHANGELOG.md"

[hooks]
pre_bump = []
post_bump = []
pre_commit = []
pre_tag = []
post_tag = []

# Plugin-specific configuration
[plugins.cargo]
publish = false

[plugins.npm]
publish = false

[plugins.github-release]
draft = false
prerelease = false
assets = []
```

### Parser Options

#### Conventional Commits

```toml
[parser]
name = "conventional"
```

Parses commits in the format: `type(scope): message`

#### Conventional + Gitmoji

```toml
[parser]
name = "conventional-gitmoji"

[parser.conventional-gitmoji]
infer_type_from_emoji = true
strict_emoji = false
```

Supports both emoji and text code formats:

- `✨ feat(api): add new endpoint`
- `:sparkles: feat(api): add new endpoint`
- `✨ add new feature` (with type inference)

#### Custom Regex

```toml
[parser]
name = "regex"

[parser.regex]
pattern = "^(?P<type>\\w+): (?P<message>.+)$"

[parser.regex.mapping]
type = "type"
message = "message"

[parser.regex.validation]
type = ["feat", "fix", "chore"]
```

## Architecture

Unduler is built with a modular architecture:

```
unduler/
├── crates/
│   ├── unduler/          # CLI binary
│   ├── unduler-core/     # Core release pipeline
│   ├── unduler-git/      # Git operations
│   ├── unduler-commit/   # Commit types (raw & parsed)
│   ├── unduler-plugin/   # Plugin traits & types
│   └── unduler-config/   # Configuration handling
└── plugins/
    ├── parser-conventional/      # Conventional Commits parser
    ├── parser-gitmoji/           # Gitmoji parser
    ├── parser-regex/             # Custom regex parser
    ├── bumper-semver/            # SemVer bump strategy
    ├── formatter-keepachangelog/ # Keep a Changelog formatter
    ├── hook-cargo/               # Cargo publish hook
    ├── hook-npm/                 # npm publish hook
    └── hook-github-release/      # GitHub Release hook
```

### Plugin Traits

- **CommitParser** — Parse raw commits into structured data
- **BumpStrategy** — Determine version bump type from commits
- **ChangelogFormatter** — Format releases into changelog output
- **ReleaseHook** — Execute actions at release lifecycle points

## Supported Gitmojis

Unduler supports all gitmojis from [gitmoji.dev](https://gitmoji.dev), including:

| Emoji | Code                 | Type     |
|-------|----------------------|----------|
| ✨     | `:sparkles:`         | feat     |
| 🐛    | `:bug:`              | fix      |
| 📝    | `:memo:`             | docs     |
| ♻️    | `:recycle:`          | refactor |
| ⚡     | `:zap:`              | perf     |
| ✅     | `:white_check_mark:` | test     |
| 🔧    | `:wrench:`           | chore    |
| 💥    | `:boom:`             | breaking |
| 🚀    | `:rocket:`           | release  |

## Development

```bash
# Run tests
cargo test --workspace

# Run lints
cargo clippy --workspace --all-targets -- -D warnings

# Format code
cargo fmt --all

# Run all checks
just check

# Run with coverage
just coverage
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes using conventional commits
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
