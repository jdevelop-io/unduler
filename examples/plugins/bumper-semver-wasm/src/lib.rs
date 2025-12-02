//! SemVer bump strategy plugin (WASM).
//!
//! Determines version bump based on Conventional Commits:
//! - Breaking changes → Major
//! - `feat` → Minor
//! - `fix`, `perf` → Patch
//! - Other → None

wit_bindgen::generate!({
    world: "unduler-bumper",
    path: "../../../crates/unduler-plugin-sdk/wit",
});

use exports::unduler::plugin::bumper::Guest;
use unduler::plugin::types::*;

struct SemverBumper;

impl Guest for SemverBumper {
    fn info() -> PluginInfo {
        PluginInfo {
            name: "semver".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Determines version bump using SemVer conventions".to_string(),
            plugin_type: PluginType::Bumper,
        }
    }

    fn determine(commits: Vec<ParsedCommit>) -> BumpType {
        let mut bump = BumpType::None;

        for commit in &commits {
            // Breaking changes always win
            if commit.breaking {
                return BumpType::Major;
            }

            // Check for minor bump (feat)
            if commit.commit_type == "feat" {
                bump = max_bump(bump, BumpType::Minor);
            }

            // Check for patch bump (fix, perf)
            if commit.commit_type == "fix" || commit.commit_type == "perf" {
                bump = max_bump(bump, BumpType::Patch);
            }
        }

        bump
    }
}

/// Returns the larger of two bump types.
fn max_bump(a: BumpType, b: BumpType) -> BumpType {
    let priority = |bt: &BumpType| match bt {
        BumpType::Major => 3,
        BumpType::Minor => 2,
        BumpType::Patch => 1,
        BumpType::None => 0,
    };

    if priority(&a) >= priority(&b) {
        a
    } else {
        b
    }
}

export!(SemverBumper);

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(commit_type: &str, breaking: bool) -> ParsedCommit {
        ParsedCommit {
            hash: "abc123".to_string(),
            commit_type: commit_type.to_string(),
            scope: None,
            message: "test".to_string(),
            breaking,
            emoji: None,
            metadata: vec![],
            author: "Test".to_string(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_breaking_change() {
        let commits = vec![make_commit("feat", false), make_commit("fix", true)];
        assert!(matches!(
            SemverBumper::determine(commits),
            BumpType::Major
        ));
    }

    #[test]
    fn test_feature() {
        let commits = vec![make_commit("feat", false)];
        assert!(matches!(
            SemverBumper::determine(commits),
            BumpType::Minor
        ));
    }

    #[test]
    fn test_fix() {
        let commits = vec![make_commit("fix", false)];
        assert!(matches!(
            SemverBumper::determine(commits),
            BumpType::Patch
        ));
    }

    #[test]
    fn test_perf() {
        let commits = vec![make_commit("perf", false)];
        assert!(matches!(
            SemverBumper::determine(commits),
            BumpType::Patch
        ));
    }

    #[test]
    fn test_chore_only() {
        let commits = vec![make_commit("chore", false)];
        assert!(matches!(SemverBumper::determine(commits), BumpType::None));
    }

    #[test]
    fn test_mixed_commits() {
        let commits = vec![
            make_commit("docs", false),
            make_commit("fix", false),
            make_commit("feat", false),
            make_commit("chore", false),
        ];
        // feat wins over fix
        assert!(matches!(
            SemverBumper::determine(commits),
            BumpType::Minor
        ));
    }

    #[test]
    fn test_empty_commits() {
        let commits: Vec<ParsedCommit> = vec![];
        assert!(matches!(SemverBumper::determine(commits), BumpType::None));
    }
}
