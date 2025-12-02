//! End-to-end CLI integration tests.
//!
//! These tests verify the complete CLI workflow by:
//! 1. Creating a temporary git repository
//! 2. Running unduler commands
//! 3. Verifying the expected outputs

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

/// Gets the path to the unduler binary.
fn unduler_bin() -> std::path::PathBuf {
    // Try to find the binary in order of preference:
    // 1. CARGO_BIN_EXE_unduler (set by cargo test)
    // 2. target/release/unduler
    // 3. target/debug/unduler
    // 4. target/llvm-cov-target/debug/unduler (coverage builds)

    if let Ok(bin) = std::env::var("CARGO_BIN_EXE_unduler") {
        return std::path::PathBuf::from(bin);
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let candidates = [
        workspace_root.join("target/release/unduler"),
        workspace_root.join("target/debug/unduler"),
        workspace_root.join("target/llvm-cov-target/debug/unduler"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // If no binary found, build it in debug mode
    let status = Command::new("cargo")
        .args(["build", "-p", "unduler"])
        .current_dir(workspace_root)
        .status()
        .expect("failed to build unduler binary");

    assert!(status.success(), "failed to build unduler");

    workspace_root.join("target/debug/unduler")
}

/// Creates a temporary git repository with some initial setup.
fn setup_git_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let dir = temp_dir.path();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .expect("failed to init git repo");

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()
        .expect("failed to configure git email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir)
        .output()
        .expect("failed to configure git name");

    temp_dir
}

/// Creates a Cargo.toml file with the given version.
fn create_cargo_toml(dir: &Path, version: &str) {
    let content = format!(
        r#"[package]
name = "test-project"
version = "{version}"
edition = "2021"
"#
    );
    fs::write(dir.join("Cargo.toml"), content).expect("failed to write Cargo.toml");
}

/// Commits all changes with the given message.
fn git_commit(dir: &Path, message: &str) {
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .expect("failed to add files");

    Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(dir)
        .output()
        .expect("failed to commit");
}

/// Creates a git tag.
fn git_tag(dir: &Path, tag: &str) {
    Command::new("git")
        .args(["tag", tag])
        .current_dir(dir)
        .output()
        .expect("failed to create tag");
}

#[test]
fn test_init_creates_config() {
    let temp_dir = setup_git_repo();
    let dir = temp_dir.path();

    // Create Cargo.toml to simulate Rust project
    create_cargo_toml(dir, "0.1.0");

    // Run init
    let output = Command::new(unduler_bin())
        .args(["init", "--no-plugins"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler init");

    assert!(output.status.success(), "init should succeed");

    // Verify config file was created
    let config_path = dir.join("unduler.toml");
    assert!(config_path.exists(), "config file should exist");

    // Verify content
    let content = fs::read_to_string(&config_path).expect("failed to read config");
    assert!(
        content.contains("[parser]"),
        "config should have parser section"
    );
    assert!(
        content.contains("[version]"),
        "config should have version section"
    );
    assert!(
        content.contains("[changelog]"),
        "config should have changelog section"
    );
    assert!(
        content.contains("Cargo.toml"),
        "config should detect Cargo.toml"
    );
}

#[test]
fn test_init_force_overwrites() {
    let temp_dir = setup_git_repo();
    let dir = temp_dir.path();

    // Create initial config
    fs::write(dir.join("unduler.toml"), "# old config\n").expect("failed to write config");

    // Run init without force should fail
    let output = Command::new(unduler_bin())
        .args(["init", "--no-plugins"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler init");

    assert!(!output.status.success(), "init should fail without --force");

    // Run init with force should succeed
    let output = Command::new(unduler_bin())
        .args(["init", "--force", "--no-plugins"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler init");

    assert!(output.status.success(), "init --force should succeed");

    // Verify new config
    let content = fs::read_to_string(dir.join("unduler.toml")).expect("failed to read config");
    assert!(
        !content.contains("# old config"),
        "config should be overwritten"
    );
}

#[test]
fn test_init_with_gitmoji_parser() {
    let temp_dir = setup_git_repo();
    let dir = temp_dir.path();

    let output = Command::new(unduler_bin())
        .args(["init", "--parser", "conventional-gitmoji", "--no-plugins"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler init");

    assert!(output.status.success(), "init should succeed");

    let content = fs::read_to_string(dir.join("unduler.toml")).expect("failed to read config");
    assert!(
        content.contains("conventional-gitmoji"),
        "config should use gitmoji parser"
    );
    assert!(
        content.contains("infer_type_from_emoji"),
        "config should have gitmoji options"
    );
}

#[test]
fn test_changelog_generates_output() {
    let temp_dir = setup_git_repo();
    let dir = temp_dir.path();

    // Setup: create project, init, make commits
    create_cargo_toml(dir, "0.1.0");
    git_commit(dir, "chore: initial commit");
    git_tag(dir, "v0.1.0");

    // Make some feature commits
    fs::write(dir.join("src.rs"), "// new feature").expect("failed to write file");
    git_commit(dir, "feat: add new feature");

    fs::write(dir.join("fix.rs"), "// bug fix").expect("failed to write file");
    git_commit(dir, "fix: resolve critical bug");

    // Create unduler config
    let config = r#"
[parser]
name = "conventional"

[version]
tag_prefix = "v"
files = ["Cargo.toml"]

[changelog]
output = "CHANGELOG.md"
"#;
    fs::write(dir.join("unduler.toml"), config).expect("failed to write config");

    // Run changelog command
    let output = Command::new(unduler_bin())
        .args(["changelog"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler changelog");

    assert!(
        output.status.success(),
        "changelog should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify changelog was created
    let changelog_path = dir.join("CHANGELOG.md");
    assert!(changelog_path.exists(), "CHANGELOG.md should exist");

    let content = fs::read_to_string(&changelog_path).expect("failed to read changelog");
    assert!(content.contains("Added"), "should have Added section");
    assert!(content.contains("Fixed"), "should have Fixed section");
    assert!(
        content.contains("add new feature"),
        "should include feature commit"
    );
    assert!(
        content.contains("resolve critical bug"),
        "should include fix commit"
    );
}

#[test]
fn test_bump_dry_run() {
    let temp_dir = setup_git_repo();
    let dir = temp_dir.path();

    // Setup
    create_cargo_toml(dir, "0.1.0");
    git_commit(dir, "chore: initial commit");
    git_tag(dir, "v0.1.0");

    // Feature commit should trigger minor bump
    fs::write(dir.join("feature.rs"), "// feature").expect("failed to write file");
    git_commit(dir, "feat: add feature");

    // Create config
    let config = r#"
[parser]
name = "conventional"

[version]
tag_prefix = "v"
files = ["Cargo.toml"]
"#;
    fs::write(dir.join("unduler.toml"), config).expect("failed to write config");

    // Run bump with dry-run
    let output = Command::new(unduler_bin())
        .args(["bump", "--dry-run"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler bump");

    assert!(
        output.status.success(),
        "bump --dry-run should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("0.2.0") || stdout.contains("minor"),
        "should show new version or bump type: {stdout}"
    );

    // Verify Cargo.toml was NOT modified (dry-run)
    let cargo_content =
        fs::read_to_string(dir.join("Cargo.toml")).expect("failed to read Cargo.toml");
    assert!(
        cargo_content.contains("0.1.0"),
        "version should not be changed in dry-run"
    );
}

#[test]
fn test_bump_updates_version() {
    let temp_dir = setup_git_repo();
    let dir = temp_dir.path();

    // Setup
    create_cargo_toml(dir, "1.0.0");
    git_commit(dir, "chore: initial commit");
    git_tag(dir, "v1.0.0");

    // Fix commit should trigger patch bump
    fs::write(dir.join("fix.rs"), "// fix").expect("failed to write file");
    git_commit(dir, "fix: fix a bug");

    // Create config
    let config = r#"
[parser]
name = "conventional"

[version]
tag_prefix = "v"
files = ["Cargo.toml"]
"#;
    fs::write(dir.join("unduler.toml"), config).expect("failed to write config");

    // Run bump
    let output = Command::new(unduler_bin())
        .args(["bump"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler bump");

    assert!(
        output.status.success(),
        "bump should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify Cargo.toml was updated
    let cargo_content =
        fs::read_to_string(dir.join("Cargo.toml")).expect("failed to read Cargo.toml");
    assert!(
        cargo_content.contains("1.0.1"),
        "version should be bumped to 1.0.1"
    );
}

#[test]
fn test_release_dry_run() {
    let temp_dir = setup_git_repo();
    let dir = temp_dir.path();

    // Setup
    create_cargo_toml(dir, "0.1.0");
    git_commit(dir, "chore: initial commit");
    git_tag(dir, "v0.1.0");

    // Add a feature
    fs::write(dir.join("feature.rs"), "// feature").expect("failed to write file");
    git_commit(dir, "feat: add new feature");

    // Create config
    let config = r#"
[parser]
name = "conventional"

[version]
tag_prefix = "v"
files = ["Cargo.toml"]

[changelog]
output = "CHANGELOG.md"
"#;
    fs::write(dir.join("unduler.toml"), config).expect("failed to write config");

    // Run release with dry-run
    let output = Command::new(unduler_bin())
        .args(["release", "--dry-run"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler release");

    assert!(
        output.status.success(),
        "release --dry-run should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("0.2.0") || stdout.contains("minor"),
        "should show version info: {stdout}"
    );

    // Verify files were NOT modified (dry-run)
    let cargo_content =
        fs::read_to_string(dir.join("Cargo.toml")).expect("failed to read Cargo.toml");
    assert!(
        cargo_content.contains("0.1.0"),
        "version should not be changed in dry-run"
    );
}

#[test]
fn test_version_command() {
    let output = Command::new(unduler_bin())
        .args(["--version"])
        .output()
        .expect("failed to run unduler --version");

    assert!(output.status.success(), "--version should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("unduler") && stdout.contains("0.1.0"),
        "should show version: {stdout}"
    );
}

#[test]
fn test_help_command() {
    let output = Command::new(unduler_bin())
        .args(["--help"])
        .output()
        .expect("failed to run unduler --help");

    assert!(output.status.success(), "--help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("init"), "should show init command");
    assert!(stdout.contains("bump"), "should show bump command");
    assert!(
        stdout.contains("changelog"),
        "should show changelog command"
    );
    assert!(stdout.contains("release"), "should show release command");
}

#[test]
fn test_plugin_list_empty() {
    let output = Command::new(unduler_bin())
        .args(["plugin", "list"])
        .output()
        .expect("failed to run unduler plugin list");

    assert!(
        output.status.success(),
        "plugin list should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No plugins") || stdout.contains("Installed"),
        "should show plugin list status: {stdout}"
    );
}

#[test]
fn test_breaking_change_triggers_major_bump() {
    let temp_dir = setup_git_repo();
    let dir = temp_dir.path();

    // Setup
    create_cargo_toml(dir, "1.0.0");
    git_commit(dir, "chore: initial commit");
    git_tag(dir, "v1.0.0");

    // Breaking change commit
    fs::write(dir.join("api.rs"), "// breaking change").expect("failed to write file");
    git_commit(dir, "feat!: breaking API change");

    // Create config
    let config = r#"
[parser]
name = "conventional"

[version]
tag_prefix = "v"
files = ["Cargo.toml"]
"#;
    fs::write(dir.join("unduler.toml"), config).expect("failed to write config");

    // Run bump
    let output = Command::new(unduler_bin())
        .args(["bump"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler bump");

    assert!(
        output.status.success(),
        "bump should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify major bump
    let cargo_content =
        fs::read_to_string(dir.join("Cargo.toml")).expect("failed to read Cargo.toml");
    assert!(
        cargo_content.contains("2.0.0"),
        "version should be bumped to 2.0.0 for breaking change"
    );
}

#[test]
fn test_no_commits_since_tag() {
    let temp_dir = setup_git_repo();
    let dir = temp_dir.path();

    // Setup with no new commits
    create_cargo_toml(dir, "1.0.0");
    git_commit(dir, "chore: initial commit");
    git_tag(dir, "v1.0.0");

    // Create config
    let config = r#"
[parser]
name = "conventional"

[version]
tag_prefix = "v"
files = ["Cargo.toml"]
"#;
    fs::write(dir.join("unduler.toml"), config).expect("failed to write config");

    // Run bump - should indicate no changes needed
    let output = Command::new(unduler_bin())
        .args(["bump", "--dry-run"])
        .current_dir(dir)
        .output()
        .expect("failed to run unduler bump");

    // Either succeeds with "no bump needed" or returns specific exit code
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should indicate no bump needed
    let output_text = format!("{stdout}{stderr}");
    assert!(
        output_text.contains("no") || output_text.contains("None") || !output.status.success(),
        "should indicate no bump needed: stdout={stdout}, stderr={stderr}"
    );
}
