//! Conventional Commits parser plugin (WASM).
//!
//! This plugin parses commits following the Conventional Commits specification:
//! https://www.conventionalcommits.org/

// Use wit-bindgen directly instead of going through the SDK
wit_bindgen::generate!({
    world: "unduler-parser",
    path: "../../../crates/unduler-plugin-sdk/wit",
});

use exports::unduler::plugin::parser::Guest;
use unduler::plugin::types::*;

struct ConventionalParser;

impl Guest for ConventionalParser {
    fn info() -> PluginInfo {
        PluginInfo {
            name: "conventional".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Parses Conventional Commits format".to_string(),
            plugin_type: PluginType::Parser,
        }
    }

    fn parse(commit: RawCommit) -> Option<ParsedCommit> {
        let subject = get_subject(&commit.message);
        parse_conventional(subject, &commit)
    }

    fn can_parse(commit: RawCommit) -> bool {
        let subject = get_subject(&commit.message);
        is_conventional(subject)
    }
}

/// Extracts the subject line from a commit message.
fn get_subject(message: &str) -> &str {
    message.lines().next().unwrap_or("")
}

/// Checks if a subject line follows Conventional Commits format.
fn is_conventional(subject: &str) -> bool {
    // Simple check: must have type followed by optional scope and colon
    let chars: Vec<char> = subject.chars().collect();

    // Find the colon
    let Some(colon_pos) = chars.iter().position(|&c| c == ':') else {
        return false;
    };

    // Must have space after colon
    if colon_pos + 1 >= chars.len() || chars[colon_pos + 1] != ' ' {
        return false;
    }

    // Type must be alphanumeric
    let type_end = chars
        .iter()
        .position(|&c| c == '(' || c == '!' || c == ':')
        .unwrap_or(colon_pos);
    if type_end == 0 {
        return false;
    }

    // Check type characters are valid
    chars[..type_end].iter().all(|c| c.is_alphanumeric())
}

/// Parses a conventional commit subject line.
fn parse_conventional(subject: &str, commit: &RawCommit) -> Option<ParsedCommit> {
    let chars: Vec<char> = subject.chars().collect();

    // Find the colon
    let colon_pos = chars.iter().position(|&c| c == ':')?;

    // Must have space after colon
    if colon_pos + 1 >= chars.len() || chars[colon_pos + 1] != ' ' {
        return None;
    }

    // Parse type (everything before scope, breaking mark, or colon)
    let mut type_end = colon_pos;
    let mut scope_start = None;
    let mut scope_end = None;
    let mut breaking = false;

    for (i, &c) in chars.iter().enumerate() {
        if c == '(' && scope_start.is_none() {
            type_end = i;
            scope_start = Some(i + 1);
        } else if c == ')' && scope_start.is_some() {
            scope_end = Some(i);
        } else if c == '!' {
            breaking = true;
            if scope_start.is_none() {
                type_end = i;
            }
        } else if c == ':' {
            break;
        }
    }

    // Extract type
    let commit_type: String = chars[..type_end].iter().collect();
    if commit_type.is_empty() || !commit_type.chars().all(|c| c.is_alphanumeric()) {
        return None;
    }

    // Extract scope
    let scope = match (scope_start, scope_end) {
        (Some(start), Some(end)) if end > start => {
            Some(chars[start..end].iter().collect::<String>())
        }
        _ => None,
    };

    // Extract message (after ": ")
    let message: String = chars[colon_pos + 2..].iter().collect();
    if message.is_empty() {
        return None;
    }

    Some(ParsedCommit {
        hash: commit.hash.clone(),
        commit_type,
        scope,
        message,
        breaking,
        emoji: None,
        metadata: vec![],
        author: commit.author.clone(),
        timestamp: commit.timestamp,
    })
}

export!(ConventionalParser);

#[cfg(test)]
mod tests {
    use super::*;

    fn make_raw(message: &str) -> RawCommit {
        RawCommit {
            hash: "abc123".to_string(),
            message: message.to_string(),
            author: "Test".to_string(),
            email: "test@test.com".to_string(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_simple_commit() {
        let raw = make_raw("feat: add new feature");
        let parsed = parse_conventional("feat: add new feature", &raw).unwrap();

        assert_eq!(parsed.commit_type, "feat");
        assert!(parsed.scope.is_none());
        assert_eq!(parsed.message, "add new feature");
        assert!(!parsed.breaking);
    }

    #[test]
    fn test_with_scope() {
        let raw = make_raw("fix(parser): handle edge case");
        let parsed = parse_conventional("fix(parser): handle edge case", &raw).unwrap();

        assert_eq!(parsed.commit_type, "fix");
        assert_eq!(parsed.scope, Some("parser".to_string()));
        assert_eq!(parsed.message, "handle edge case");
    }

    #[test]
    fn test_breaking_change() {
        let raw = make_raw("feat(api)!: redesign endpoints");
        let parsed = parse_conventional("feat(api)!: redesign endpoints", &raw).unwrap();

        assert_eq!(parsed.commit_type, "feat");
        assert!(parsed.breaking);
    }

    #[test]
    fn test_breaking_without_scope() {
        let raw = make_raw("feat!: breaking feature");
        let parsed = parse_conventional("feat!: breaking feature", &raw).unwrap();

        assert_eq!(parsed.commit_type, "feat");
        assert!(parsed.breaking);
        assert!(parsed.scope.is_none());
    }

    #[test]
    fn test_invalid_commit() {
        let raw = make_raw("random commit message");
        assert!(parse_conventional("random commit message", &raw).is_none());
    }

    #[test]
    fn test_is_conventional() {
        assert!(is_conventional("feat: something"));
        assert!(is_conventional("fix(scope): something"));
        assert!(is_conventional("feat!: breaking"));
        assert!(!is_conventional("random message"));
        assert!(!is_conventional("feat:no space"));
    }
}
