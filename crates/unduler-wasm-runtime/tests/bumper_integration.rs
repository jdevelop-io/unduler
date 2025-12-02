//! Integration tests for WASM bumper plugins.

use std::path::PathBuf;

use unduler_wasm_runtime::{WasmBumper, WasmEngine};

fn test_plugin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-plugins/bumper-semver.wasm")
}

#[test]
fn test_load_bumper_plugin() {
    let path = test_plugin_path();
    if !path.exists() {
        eprintln!("Skipping test: plugin not found at {path:?}");
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut bumper = WasmBumper::from_file(&engine, &path).expect("Failed to load bumper");

    let info = bumper.info().expect("Failed to get info");
    assert_eq!(info.name, "semver");
    assert_eq!(
        info.description,
        "Determines version bump using SemVer conventions"
    );
}

#[test]
fn test_bumper_breaking_change() {
    let path = test_plugin_path();
    if !path.exists() {
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut bumper = WasmBumper::from_file(&engine, &path).expect("Failed to load bumper");

    let commits = vec![
        unduler_wasm_runtime::bumper::ParsedCommit {
            hash: "abc123".to_string(),
            commit_type: "feat".to_string(),
            scope: None,
            message: "add feature".to_string(),
            breaking: false,
            emoji: None,
            metadata: vec![],
            author: "Test".to_string(),
            timestamp: 0,
        },
        unduler_wasm_runtime::bumper::ParsedCommit {
            hash: "def456".to_string(),
            commit_type: "fix".to_string(),
            scope: None,
            message: "breaking fix".to_string(),
            breaking: true, // Breaking change
            emoji: None,
            metadata: vec![],
            author: "Test".to_string(),
            timestamp: 0,
        },
    ];

    let bump = bumper.determine(&commits).expect("determine failed");
    assert!(matches!(
        bump,
        unduler_wasm_runtime::bumper::BumpType::Major
    ));
}

#[test]
fn test_bumper_feature() {
    let path = test_plugin_path();
    if !path.exists() {
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut bumper = WasmBumper::from_file(&engine, &path).expect("Failed to load bumper");

    let commits = vec![unduler_wasm_runtime::bumper::ParsedCommit {
        hash: "abc123".to_string(),
        commit_type: "feat".to_string(),
        scope: None,
        message: "add feature".to_string(),
        breaking: false,
        emoji: None,
        metadata: vec![],
        author: "Test".to_string(),
        timestamp: 0,
    }];

    let bump = bumper.determine(&commits).expect("determine failed");
    assert!(matches!(
        bump,
        unduler_wasm_runtime::bumper::BumpType::Minor
    ));
}

#[test]
fn test_bumper_fix() {
    let path = test_plugin_path();
    if !path.exists() {
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut bumper = WasmBumper::from_file(&engine, &path).expect("Failed to load bumper");

    let commits = vec![unduler_wasm_runtime::bumper::ParsedCommit {
        hash: "abc123".to_string(),
        commit_type: "fix".to_string(),
        scope: None,
        message: "bug fix".to_string(),
        breaking: false,
        emoji: None,
        metadata: vec![],
        author: "Test".to_string(),
        timestamp: 0,
    }];

    let bump = bumper.determine(&commits).expect("determine failed");
    assert!(matches!(
        bump,
        unduler_wasm_runtime::bumper::BumpType::Patch
    ));
}

#[test]
fn test_bumper_chore_only() {
    let path = test_plugin_path();
    if !path.exists() {
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut bumper = WasmBumper::from_file(&engine, &path).expect("Failed to load bumper");

    let commits = vec![unduler_wasm_runtime::bumper::ParsedCommit {
        hash: "abc123".to_string(),
        commit_type: "chore".to_string(),
        scope: None,
        message: "update deps".to_string(),
        breaking: false,
        emoji: None,
        metadata: vec![],
        author: "Test".to_string(),
        timestamp: 0,
    }];

    let bump = bumper.determine(&commits).expect("determine failed");
    assert!(matches!(bump, unduler_wasm_runtime::bumper::BumpType::None));
}

#[test]
fn test_bumper_empty_commits() {
    let path = test_plugin_path();
    if !path.exists() {
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut bumper = WasmBumper::from_file(&engine, &path).expect("Failed to load bumper");

    let commits: Vec<unduler_wasm_runtime::bumper::ParsedCommit> = vec![];

    let bump = bumper.determine(&commits).expect("determine failed");
    assert!(matches!(bump, unduler_wasm_runtime::bumper::BumpType::None));
}
