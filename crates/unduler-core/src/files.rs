//! Version file updaters.
//!
//! Supports updating version numbers in various file formats:
//! - Cargo.toml (TOML)
//! - package.json (JSON)
//! - Generic files via regex pattern

use std::fs;
use std::path::Path;

use semver::Version;
use thiserror::Error;

/// Errors that can occur when updating version files.
#[derive(Debug, Error)]
pub enum FileUpdateError {
    /// File not found.
    #[error("file not found: {0}")]
    NotFound(String),

    /// Failed to read file.
    #[error("failed to read file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to parse file.
    #[error("failed to parse {file}: {reason}")]
    ParseError { file: String, reason: String },

    /// Version not found in file.
    #[error("version not found in {0}")]
    VersionNotFound(String),

    /// Unsupported file type.
    #[error("unsupported file type: {0}")]
    UnsupportedFileType(String),
}

/// Result type for file operations.
pub type FileResult<T> = Result<T, FileUpdateError>;

/// Updates version in a file based on its type.
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist
/// - The file type is not supported
/// - The version field is not found in the file
/// - The file cannot be read or written
pub fn update_version_file(path: &Path, new_version: &Version, dry_run: bool) -> FileResult<()> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    match filename {
        "Cargo.toml" => update_cargo_toml(path, new_version, dry_run),
        "package.json" => update_package_json(path, new_version, dry_run),
        _ => {
            // Try to detect by extension
            match path.extension().and_then(|e| e.to_str()) {
                Some("toml") => update_cargo_toml(path, new_version, dry_run),
                Some("json") => update_package_json(path, new_version, dry_run),
                _ => Err(FileUpdateError::UnsupportedFileType(
                    path.display().to_string(),
                )),
            }
        }
    }
}

/// Updates version in a Cargo.toml file.
fn update_cargo_toml(path: &Path, new_version: &Version, dry_run: bool) -> FileResult<()> {
    if !path.exists() {
        return Err(FileUpdateError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;

    // Use regex to update version while preserving formatting
    let version_regex =
        regex::Regex::new(r#"(?m)^(\s*version\s*=\s*)"([^"]+)"#).expect("invalid regex");

    if !version_regex.is_match(&content) {
        return Err(FileUpdateError::VersionNotFound(path.display().to_string()));
    }

    let new_content = version_regex
        .replace(&content, format!(r#"$1"{new_version}""#))
        .to_string();

    if !dry_run {
        fs::write(path, new_content)?;
    }

    Ok(())
}

/// Updates version in a package.json file.
fn update_package_json(path: &Path, new_version: &Version, dry_run: bool) -> FileResult<()> {
    if !path.exists() {
        return Err(FileUpdateError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;

    // Parse JSON
    let mut json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| FileUpdateError::ParseError {
            file: path.display().to_string(),
            reason: e.to_string(),
        })?;

    // Update version
    if let Some(obj) = json.as_object_mut() {
        if !obj.contains_key("version") {
            return Err(FileUpdateError::VersionNotFound(path.display().to_string()));
        }
        obj.insert(
            "version".to_string(),
            serde_json::Value::String(new_version.to_string()),
        );
    } else {
        return Err(FileUpdateError::ParseError {
            file: path.display().to_string(),
            reason: "not a JSON object".to_string(),
        });
    }

    if !dry_run {
        // Write with pretty formatting and trailing newline
        let new_content =
            serde_json::to_string_pretty(&json).map_err(|e| FileUpdateError::ParseError {
                file: path.display().to_string(),
                reason: e.to_string(),
            })?;
        fs::write(path, format!("{new_content}\n"))?;
    }

    Ok(())
}

/// Reads the current version from a file.
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist
/// - The file type is not supported
/// - The version field is not found in the file
/// - The version string is not valid semver
pub fn read_version_from_file(path: &Path) -> FileResult<Version> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    match filename {
        "Cargo.toml" => read_cargo_toml_version(path),
        "package.json" => read_package_json_version(path),
        _ => match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => read_cargo_toml_version(path),
            Some("json") => read_package_json_version(path),
            _ => Err(FileUpdateError::UnsupportedFileType(
                path.display().to_string(),
            )),
        },
    }
}

/// Reads version from a Cargo.toml file.
fn read_cargo_toml_version(path: &Path) -> FileResult<Version> {
    if !path.exists() {
        return Err(FileUpdateError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;

    let version_regex =
        regex::Regex::new(r#"(?m)^\s*version\s*=\s*"([^"]+)""#).expect("invalid regex");

    let captures = version_regex
        .captures(&content)
        .ok_or_else(|| FileUpdateError::VersionNotFound(path.display().to_string()))?;

    let version_str = captures.get(1).map(|m| m.as_str()).unwrap_or_default();

    Version::parse(version_str).map_err(|e| FileUpdateError::ParseError {
        file: path.display().to_string(),
        reason: e.to_string(),
    })
}

/// Reads version from a package.json file.
fn read_package_json_version(path: &Path) -> FileResult<Version> {
    if !path.exists() {
        return Err(FileUpdateError::NotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)?;

    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| FileUpdateError::ParseError {
            file: path.display().to_string(),
            reason: e.to_string(),
        })?;

    let version_str = json
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| FileUpdateError::VersionNotFound(path.display().to_string()))?;

    Version::parse(version_str).map_err(|e| FileUpdateError::ParseError {
        file: path.display().to_string(),
        reason: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_update_cargo_toml() {
        let mut file = NamedTempFile::with_suffix(".toml").unwrap();
        writeln!(
            file,
            r#"[package]
name = "test"
version = "1.0.0"

[dependencies]
"#
        )
        .unwrap();

        let version = Version::new(2, 0, 0);
        update_cargo_toml(file.path(), &version, false).unwrap();

        let content = fs::read_to_string(file.path()).unwrap();
        assert!(content.contains(r#"version = "2.0.0""#));
    }

    #[test]
    fn test_update_cargo_toml_dry_run() {
        let mut file = NamedTempFile::with_suffix(".toml").unwrap();
        writeln!(
            file,
            r#"[package]
name = "test"
version = "1.0.0"
"#
        )
        .unwrap();

        let version = Version::new(2, 0, 0);
        update_cargo_toml(file.path(), &version, true).unwrap();

        let content = fs::read_to_string(file.path()).unwrap();
        assert!(content.contains(r#"version = "1.0.0""#)); // Unchanged
    }

    #[test]
    fn test_update_package_json() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(
            file,
            r#"{{
  "name": "test",
  "version": "1.0.0"
}}"#
        )
        .unwrap();

        let version = Version::new(2, 0, 0);
        update_package_json(file.path(), &version, false).unwrap();

        let content = fs::read_to_string(file.path()).unwrap();
        assert!(content.contains(r#""version": "2.0.0""#));
    }

    #[test]
    fn test_read_cargo_toml_version() {
        let mut file = NamedTempFile::with_suffix(".toml").unwrap();
        writeln!(
            file,
            r#"[package]
name = "test"
version = "1.2.3"
"#
        )
        .unwrap();

        let version = read_cargo_toml_version(file.path()).unwrap();
        assert_eq!(version, Version::new(1, 2, 3));
    }

    #[test]
    fn test_read_package_json_version() {
        let mut file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(
            file,
            r#"{{
  "name": "test",
  "version": "1.2.3"
}}"#
        )
        .unwrap();

        let version = read_package_json_version(file.path()).unwrap();
        assert_eq!(version, Version::new(1, 2, 3));
    }

    #[test]
    fn test_update_version_file_cargo() {
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"[package]
name = "test"
version = "1.0.0"
"#
        )
        .unwrap();

        let version = Version::new(2, 0, 0);
        update_version_file(file.path(), &version, false).unwrap();

        let content = fs::read_to_string(file.path()).unwrap();
        assert!(content.contains(r#"version = "2.0.0""#));
    }

    #[test]
    fn test_read_version_from_file_cargo() {
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"[package]
name = "test"
version = "3.2.1"
"#
        )
        .unwrap();

        let version = read_version_from_file(file.path()).unwrap();
        assert_eq!(version, Version::new(3, 2, 1));
    }

    #[test]
    fn test_version_not_found() {
        let mut file = NamedTempFile::with_suffix(".toml").unwrap();
        writeln!(
            file,
            r#"[package]
name = "test"
"#
        )
        .unwrap();

        let result = read_cargo_toml_version(file.path());
        assert!(matches!(result, Err(FileUpdateError::VersionNotFound(_))));
    }

    #[test]
    fn test_file_not_found() {
        let path = Path::new("/nonexistent/Cargo.toml");
        let result = read_cargo_toml_version(path);
        assert!(matches!(result, Err(FileUpdateError::NotFound(_))));
    }

    #[test]
    fn test_unsupported_file_type() {
        let file = NamedTempFile::with_suffix(".txt").unwrap();
        let result = update_version_file(file.path(), &Version::new(1, 0, 0), false);
        assert!(matches!(
            result,
            Err(FileUpdateError::UnsupportedFileType(_))
        ));
    }

    #[test]
    fn test_error_display() {
        let err = FileUpdateError::NotFound("test.toml".to_string());
        assert_eq!(err.to_string(), "file not found: test.toml");

        let err = FileUpdateError::VersionNotFound("Cargo.toml".to_string());
        assert_eq!(err.to_string(), "version not found in Cargo.toml");

        let err = FileUpdateError::UnsupportedFileType("test.txt".to_string());
        assert_eq!(err.to_string(), "unsupported file type: test.txt");

        let err = FileUpdateError::ParseError {
            file: "test.json".to_string(),
            reason: "invalid JSON".to_string(),
        };
        assert_eq!(err.to_string(), "failed to parse test.json: invalid JSON");
    }
}
