//! Bump strategy trait.

use serde::{Deserialize, Serialize};
use unduler_commit::ParsedCommit;

use super::Plugin;

/// Version bump type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BumpType {
    /// Major version bump (breaking changes).
    Major,
    /// Minor version bump (new features).
    Minor,
    /// Patch version bump (bug fixes).
    Patch,
    /// No version bump needed.
    None,
}

impl BumpType {
    /// Returns true if this bump type is greater than another.
    #[must_use]
    pub fn is_greater_than(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Major, Self::Minor | Self::Patch | Self::None)
                | (Self::Minor, Self::Patch | Self::None)
                | (Self::Patch, Self::None)
        )
    }

    /// Returns the maximum of two bump types.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        if self.is_greater_than(other) {
            self
        } else {
            other
        }
    }
}

impl std::fmt::Display for BumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Patch => write!(f, "patch"),
            Self::None => write!(f, "none"),
        }
    }
}

/// Determines version bump type from parsed commits.
pub trait BumpStrategy: Plugin {
    /// Determines the bump type based on the given commits.
    fn determine(&self, commits: &[ParsedCommit]) -> BumpType;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_type_display_major() {
        assert_eq!(BumpType::Major.to_string(), "major");
    }

    #[test]
    fn test_bump_type_display_minor() {
        assert_eq!(BumpType::Minor.to_string(), "minor");
    }

    #[test]
    fn test_bump_type_display_patch() {
        assert_eq!(BumpType::Patch.to_string(), "patch");
    }

    #[test]
    fn test_bump_type_display_none() {
        assert_eq!(BumpType::None.to_string(), "none");
    }

    #[test]
    fn test_is_greater_than_major() {
        assert!(BumpType::Major.is_greater_than(BumpType::Minor));
        assert!(BumpType::Major.is_greater_than(BumpType::Patch));
        assert!(BumpType::Major.is_greater_than(BumpType::None));
        assert!(!BumpType::Major.is_greater_than(BumpType::Major));
    }

    #[test]
    fn test_is_greater_than_minor() {
        assert!(!BumpType::Minor.is_greater_than(BumpType::Major));
        assert!(BumpType::Minor.is_greater_than(BumpType::Patch));
        assert!(BumpType::Minor.is_greater_than(BumpType::None));
        assert!(!BumpType::Minor.is_greater_than(BumpType::Minor));
    }

    #[test]
    fn test_is_greater_than_patch() {
        assert!(!BumpType::Patch.is_greater_than(BumpType::Major));
        assert!(!BumpType::Patch.is_greater_than(BumpType::Minor));
        assert!(BumpType::Patch.is_greater_than(BumpType::None));
        assert!(!BumpType::Patch.is_greater_than(BumpType::Patch));
    }

    #[test]
    fn test_is_greater_than_none() {
        assert!(!BumpType::None.is_greater_than(BumpType::Major));
        assert!(!BumpType::None.is_greater_than(BumpType::Minor));
        assert!(!BumpType::None.is_greater_than(BumpType::Patch));
        assert!(!BumpType::None.is_greater_than(BumpType::None));
    }

    #[test]
    fn test_max_major_wins() {
        assert_eq!(BumpType::Major.max(BumpType::Minor), BumpType::Major);
        assert_eq!(BumpType::Major.max(BumpType::Patch), BumpType::Major);
        assert_eq!(BumpType::Major.max(BumpType::None), BumpType::Major);
        assert_eq!(BumpType::Minor.max(BumpType::Major), BumpType::Major);
    }

    #[test]
    fn test_max_minor_wins() {
        assert_eq!(BumpType::Minor.max(BumpType::Patch), BumpType::Minor);
        assert_eq!(BumpType::Minor.max(BumpType::None), BumpType::Minor);
        assert_eq!(BumpType::Patch.max(BumpType::Minor), BumpType::Minor);
    }

    #[test]
    fn test_max_patch_wins() {
        assert_eq!(BumpType::Patch.max(BumpType::None), BumpType::Patch);
        assert_eq!(BumpType::None.max(BumpType::Patch), BumpType::Patch);
    }

    #[test]
    fn test_max_same() {
        assert_eq!(BumpType::Major.max(BumpType::Major), BumpType::Major);
        assert_eq!(BumpType::Minor.max(BumpType::Minor), BumpType::Minor);
        assert_eq!(BumpType::Patch.max(BumpType::Patch), BumpType::Patch);
        assert_eq!(BumpType::None.max(BumpType::None), BumpType::None);
    }

    #[test]
    fn test_clone() {
        let bump = BumpType::Major;
        let cloned = bump;
        assert_eq!(bump, cloned);
    }

    #[test]
    fn test_debug() {
        let debug = format!("{:?}", BumpType::Major);
        assert!(debug.contains("Major"));
    }

    #[test]
    fn test_eq() {
        assert_eq!(BumpType::Major, BumpType::Major);
        assert_ne!(BumpType::Major, BumpType::Minor);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(BumpType::Major);
        set.insert(BumpType::Minor);
        assert!(set.contains(&BumpType::Major));
        assert!(set.contains(&BumpType::Minor));
        assert!(!set.contains(&BumpType::Patch));
    }

    #[test]
    fn test_serialize() {
        let bump = BumpType::Major;
        let json = serde_json::to_string(&bump).unwrap();
        assert_eq!(json, "\"Major\"");
    }

    #[test]
    fn test_deserialize() {
        let bump: BumpType = serde_json::from_str("\"Minor\"").unwrap();
        assert_eq!(bump, BumpType::Minor);
    }
}
