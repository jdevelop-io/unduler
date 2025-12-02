//! Integration tests for WASM parser plugins.

use std::path::PathBuf;

use unduler_wasm_runtime::{WasmEngine, WasmParser};

fn test_plugin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-plugins/parser-conventional.wasm")
}

#[test]
fn test_load_parser_plugin() {
    let path = test_plugin_path();
    if !path.exists() {
        eprintln!("Skipping test: plugin not found at {path:?}");
        eprintln!("Build the example plugin first with:");
        eprintln!("  cd examples/plugins/parser-conventional-wasm");
        eprintln!("  cargo build --release --target wasm32-unknown-unknown");
        eprintln!(
            "  wasm-tools component new target/wasm32-unknown-unknown/release/parser_conventional_wasm.wasm -o ../../test-plugins/parser-conventional.wasm"
        );
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut parser = WasmParser::from_file(&engine, &path).expect("Failed to load parser");

    let info = parser.info().expect("Failed to get info");
    assert_eq!(info.name, "conventional");
    assert_eq!(info.description, "Parses Conventional Commits format");
}

#[test]
fn test_parser_can_parse() {
    let path = test_plugin_path();
    if !path.exists() {
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut parser = WasmParser::from_file(&engine, &path).expect("Failed to load parser");

    let commit = unduler_wasm_runtime::parser::RawCommit {
        hash: "abc123".to_string(),
        message: "feat: add new feature".to_string(),
        author: "Test".to_string(),
        email: "test@test.com".to_string(),
        timestamp: 0,
    };

    assert!(parser.can_parse(&commit).expect("can_parse failed"));

    let non_conventional = unduler_wasm_runtime::parser::RawCommit {
        hash: "abc123".to_string(),
        message: "random commit message".to_string(),
        author: "Test".to_string(),
        email: "test@test.com".to_string(),
        timestamp: 0,
    };

    assert!(
        !parser
            .can_parse(&non_conventional)
            .expect("can_parse failed")
    );
}

#[test]
fn test_parser_parse() {
    let path = test_plugin_path();
    if !path.exists() {
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut parser = WasmParser::from_file(&engine, &path).expect("Failed to load parser");

    let commit = unduler_wasm_runtime::parser::RawCommit {
        hash: "abc123".to_string(),
        message: "feat(api): add new endpoint".to_string(),
        author: "Test".to_string(),
        email: "test@test.com".to_string(),
        timestamp: 1_234_567_890,
    };

    let parsed = parser
        .parse(&commit)
        .expect("parse failed")
        .expect("should parse");
    assert_eq!(parsed.commit_type, "feat");
    assert_eq!(parsed.scope, Some("api".to_string()));
    assert_eq!(parsed.message, "add new endpoint");
    assert!(!parsed.breaking);
}

#[test]
fn test_parser_breaking_change() {
    let path = test_plugin_path();
    if !path.exists() {
        return;
    }

    let engine = WasmEngine::new().expect("Failed to create engine");
    let mut parser = WasmParser::from_file(&engine, &path).expect("Failed to load parser");

    let commit = unduler_wasm_runtime::parser::RawCommit {
        hash: "abc123".to_string(),
        message: "feat(api)!: redesign endpoints".to_string(),
        author: "Test".to_string(),
        email: "test@test.com".to_string(),
        timestamp: 0,
    };

    let parsed = parser
        .parse(&commit)
        .expect("parse failed")
        .expect("should parse");
    assert_eq!(parsed.commit_type, "feat");
    assert!(parsed.breaking);
}
