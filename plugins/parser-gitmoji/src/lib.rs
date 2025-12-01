//! Conventional Commits + Gitmoji parser plugin.
//!
//! Supports all gitmojis from <https://gitmoji.dev> in both emoji and text format.

use std::collections::HashMap;
use std::sync::LazyLock;
use unduler_commit::{ParsedCommit, RawCommit};
use unduler_parser_conventional::ConventionalParser;
use unduler_plugin::{CommitParser, Plugin};

/// A gitmoji entry with its emoji, text code, and commit type.
#[derive(Debug, Clone, Copy)]
struct Gitmoji {
    emoji: &'static str,
    code: &'static str,
    commit_type: &'static str,
}

/// All gitmojis from gitmoji.dev with their corresponding commit types.
const GITMOJIS: &[Gitmoji] = &[
    // Features
    Gitmoji {
        emoji: "✨",
        code: ":sparkles:",
        commit_type: "feat",
    },
    Gitmoji {
        emoji: "🎉",
        code: ":tada:",
        commit_type: "feat",
    },
    Gitmoji {
        emoji: "🚩",
        code: ":triangular_flag_on_post:",
        commit_type: "feat",
    },
    Gitmoji {
        emoji: "👔",
        code: ":necktie:",
        commit_type: "feat",
    },
    Gitmoji {
        emoji: "🦺",
        code: ":safety_vest:",
        commit_type: "feat",
    },
    Gitmoji {
        emoji: "🦖",
        code: ":t-rex:",
        commit_type: "feat",
    },
    Gitmoji {
        emoji: "✈️",
        code: ":airplane:",
        commit_type: "feat",
    },
    // Bug fixes
    Gitmoji {
        emoji: "🐛",
        code: ":bug:",
        commit_type: "fix",
    },
    Gitmoji {
        emoji: "🚑️",
        code: ":ambulance:",
        commit_type: "fix",
    },
    Gitmoji {
        emoji: "🚑",
        code: ":ambulance:",
        commit_type: "fix",
    },
    Gitmoji {
        emoji: "🩹",
        code: ":adhesive_bandage:",
        commit_type: "fix",
    },
    Gitmoji {
        emoji: "✏️",
        code: ":pencil2:",
        commit_type: "fix",
    },
    Gitmoji {
        emoji: "🥅",
        code: ":goal_net:",
        commit_type: "fix",
    },
    Gitmoji {
        emoji: "🚨",
        code: ":rotating_light:",
        commit_type: "fix",
    },
    // Documentation
    Gitmoji {
        emoji: "📝",
        code: ":memo:",
        commit_type: "docs",
    },
    Gitmoji {
        emoji: "💡",
        code: ":bulb:",
        commit_type: "docs",
    },
    Gitmoji {
        emoji: "📄",
        code: ":page_facing_up:",
        commit_type: "docs",
    },
    Gitmoji {
        emoji: "👥",
        code: ":busts_in_silhouette:",
        commit_type: "docs",
    },
    // Style / UI
    Gitmoji {
        emoji: "🎨",
        code: ":art:",
        commit_type: "style",
    },
    Gitmoji {
        emoji: "💄",
        code: ":lipstick:",
        commit_type: "style",
    },
    Gitmoji {
        emoji: "📱",
        code: ":iphone:",
        commit_type: "style",
    },
    // Refactoring
    Gitmoji {
        emoji: "♻️",
        code: ":recycle:",
        commit_type: "refactor",
    },
    Gitmoji {
        emoji: "🚚",
        code: ":truck:",
        commit_type: "refactor",
    },
    Gitmoji {
        emoji: "🏗️",
        code: ":building_construction:",
        commit_type: "refactor",
    },
    Gitmoji {
        emoji: "🏗",
        code: ":building_construction:",
        commit_type: "refactor",
    },
    // Performance
    Gitmoji {
        emoji: "⚡️",
        code: ":zap:",
        commit_type: "perf",
    },
    Gitmoji {
        emoji: "⚡",
        code: ":zap:",
        commit_type: "perf",
    },
    Gitmoji {
        emoji: "🧵",
        code: ":thread:",
        commit_type: "perf",
    },
    // Tests
    Gitmoji {
        emoji: "✅",
        code: ":white_check_mark:",
        commit_type: "test",
    },
    Gitmoji {
        emoji: "🧪",
        code: ":test_tube:",
        commit_type: "test",
    },
    Gitmoji {
        emoji: "🤡",
        code: ":clown_face:",
        commit_type: "test",
    },
    // Build
    Gitmoji {
        emoji: "📦️",
        code: ":package:",
        commit_type: "build",
    },
    Gitmoji {
        emoji: "📦",
        code: ":package:",
        commit_type: "build",
    },
    Gitmoji {
        emoji: "🔨",
        code: ":hammer:",
        commit_type: "build",
    },
    // CI
    Gitmoji {
        emoji: "👷",
        code: ":construction_worker:",
        commit_type: "ci",
    },
    Gitmoji {
        emoji: "💚",
        code: ":green_heart:",
        commit_type: "ci",
    },
    // Chore / Maintenance
    Gitmoji {
        emoji: "🔧",
        code: ":wrench:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🚧",
        code: ":construction:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🗑️",
        code: ":wastebasket:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🗑",
        code: ":wastebasket:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🙈",
        code: ":see_no_evil:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "💩",
        code: ":poop:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🍻",
        code: ":beers:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🥚",
        code: ":egg:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🔥",
        code: ":fire:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "⚰️",
        code: ":coffin:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "⚰",
        code: ":coffin:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🔊",
        code: ":loud_sound:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🔇",
        code: ":mute:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "📈",
        code: ":chart_with_upwards_trend:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "👽️",
        code: ":alien:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "👽",
        code: ":alien:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🍱",
        code: ":bento:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "📸",
        code: ":camera_flash:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "💬",
        code: ":speech_balloon:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🔀",
        code: ":twisted_rightwards_arrows:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🔍️",
        code: ":mag:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🔍",
        code: ":mag:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "⚗️",
        code: ":alembic:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "⚗",
        code: ":alembic:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🩺",
        code: ":stethoscope:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🧱",
        code: ":bricks:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "💸",
        code: ":money_with_wings:",
        commit_type: "chore",
    },
    Gitmoji {
        emoji: "🧐",
        code: ":monocle_face:",
        commit_type: "chore",
    },
    // Dependencies
    Gitmoji {
        emoji: "⬆️",
        code: ":arrow_up:",
        commit_type: "deps",
    },
    Gitmoji {
        emoji: "⬇️",
        code: ":arrow_down:",
        commit_type: "deps",
    },
    Gitmoji {
        emoji: "📌",
        code: ":pushpin:",
        commit_type: "deps",
    },
    Gitmoji {
        emoji: "➕",
        code: ":heavy_plus_sign:",
        commit_type: "deps",
    },
    Gitmoji {
        emoji: "➖",
        code: ":heavy_minus_sign:",
        commit_type: "deps",
    },
    // Security
    Gitmoji {
        emoji: "🔒️",
        code: ":lock:",
        commit_type: "security",
    },
    Gitmoji {
        emoji: "🔒",
        code: ":lock:",
        commit_type: "security",
    },
    Gitmoji {
        emoji: "🔐",
        code: ":closed_lock_with_key:",
        commit_type: "security",
    },
    Gitmoji {
        emoji: "🛂",
        code: ":passport_control:",
        commit_type: "security",
    },
    // Release / Deploy
    Gitmoji {
        emoji: "🚀",
        code: ":rocket:",
        commit_type: "release",
    },
    Gitmoji {
        emoji: "🔖",
        code: ":bookmark:",
        commit_type: "release",
    },
    // Breaking changes
    Gitmoji {
        emoji: "💥",
        code: ":boom:",
        commit_type: "breaking",
    },
    // Revert
    Gitmoji {
        emoji: "⏪️",
        code: ":rewind:",
        commit_type: "revert",
    },
    Gitmoji {
        emoji: "⏪",
        code: ":rewind:",
        commit_type: "revert",
    },
    // Internationalization
    Gitmoji {
        emoji: "🌐",
        code: ":globe_with_meridians:",
        commit_type: "i18n",
    },
    // Accessibility
    Gitmoji {
        emoji: "♿️",
        code: ":wheelchair:",
        commit_type: "a11y",
    },
    Gitmoji {
        emoji: "♿",
        code: ":wheelchair:",
        commit_type: "a11y",
    },
    // Database
    Gitmoji {
        emoji: "🗃️",
        code: ":card_file_box:",
        commit_type: "db",
    },
    Gitmoji {
        emoji: "🗃",
        code: ":card_file_box:",
        commit_type: "db",
    },
    Gitmoji {
        emoji: "🌱",
        code: ":seedling:",
        commit_type: "db",
    },
    // UX improvements
    Gitmoji {
        emoji: "🚸",
        code: ":children_crossing:",
        commit_type: "ux",
    },
    Gitmoji {
        emoji: "💫",
        code: ":dizzy:",
        commit_type: "ux",
    },
    // Types
    Gitmoji {
        emoji: "🏷️",
        code: ":label:",
        commit_type: "types",
    },
    Gitmoji {
        emoji: "🏷",
        code: ":label:",
        commit_type: "types",
    },
    // Developer experience
    Gitmoji {
        emoji: "🧑‍💻",
        code: ":technologist:",
        commit_type: "dx",
    },
];

/// Maps emoji characters to their commit type.
static EMOJI_MAP: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| GITMOJIS.iter().map(|g| (g.emoji, g.commit_type)).collect());

/// Maps text codes (like :bug:) to their emoji and commit type.
static CODE_MAP: LazyLock<HashMap<&'static str, (&'static str, &'static str)>> =
    LazyLock::new(|| {
        GITMOJIS
            .iter()
            .map(|g| (g.code, (g.emoji, g.commit_type)))
            .collect()
    });

/// Result of extracting a gitmoji from a string.
#[derive(Debug)]
struct ExtractedGitmoji<'a> {
    /// The emoji character (even if input was text code).
    emoji: &'a str,
    /// The remaining string after the gitmoji.
    rest: &'a str,
}

/// Configuration for the Gitmoji parser.
#[derive(Debug, Clone)]
pub struct GitmojiParserConfig {
    /// Infer type from emoji if not explicitly provided.
    pub infer_type_from_emoji: bool,
    /// Reject commits with unknown emojis.
    pub strict_emoji: bool,
}

impl Default for GitmojiParserConfig {
    fn default() -> Self {
        Self {
            infer_type_from_emoji: true,
            strict_emoji: false,
        }
    }
}

/// Conventional Commits + Gitmoji parser.
///
/// Supports both emoji format (✨) and text format (:sparkles:).
pub struct ConventionalGitmojiParser {
    conventional: ConventionalParser,
    config: GitmojiParserConfig,
}

impl ConventionalGitmojiParser {
    /// Creates a new parser with the default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            conventional: ConventionalParser::new(),
            config: GitmojiParserConfig::default(),
        }
    }

    /// Creates a new parser with custom configuration.
    #[must_use]
    pub fn with_config(config: GitmojiParserConfig) -> Self {
        Self {
            conventional: ConventionalParser::new(),
            config,
        }
    }

    /// Extracts gitmoji from the beginning of a string.
    ///
    /// Supports both emoji format (✨) and text format (:sparkles:).
    fn extract_gitmoji(s: &str) -> Option<ExtractedGitmoji<'_>> {
        // Try text code format first (:code:)
        if let Some(after_colon) = s.strip_prefix(':')
            && let Some(end) = after_colon.find(':')
        {
            let code = &s[..end + 2];
            if let Some(&(emoji, _)) = CODE_MAP.get(code) {
                let rest = s[end + 2..].trim_start();
                return Some(ExtractedGitmoji { emoji, rest });
            }
        }

        // Try known emojis
        for emoji in EMOJI_MAP.keys() {
            if let Some(rest) = s.strip_prefix(emoji) {
                return Some(ExtractedGitmoji {
                    emoji,
                    rest: rest.trim_start(),
                });
            }
        }

        // Try to extract any emoji-like character (for unknown emojis)
        let mut chars = s.chars();
        if let Some(c) = chars.next()
            && is_emoji_char(c)
        {
            let mut emoji_len = c.len_utf8();

            // Handle variation selectors and zero-width joiners
            let remaining = &s[emoji_len..];
            for next_char in remaining.chars() {
                if is_emoji_modifier(next_char) {
                    emoji_len += next_char.len_utf8();
                } else {
                    break;
                }
            }

            let emoji = &s[..emoji_len];
            let rest = s[emoji_len..].trim_start();
            return Some(ExtractedGitmoji { emoji, rest });
        }

        None
    }
}

/// Checks if a character is likely an emoji.
fn is_emoji_char(c: char) -> bool {
    let code = c as u32;
    // Common emoji ranges
    (0x1F300..=0x1F9FF).contains(&code) // Misc Symbols, Emoticons, etc.
        || (0x2600..=0x26FF).contains(&code) // Misc Symbols
        || (0x2700..=0x27BF).contains(&code) // Dingbats
        || (0x1F600..=0x1F64F).contains(&code) // Emoticons
        || (0x1F680..=0x1F6FF).contains(&code) // Transport
        || (0x2300..=0x23FF).contains(&code) // Misc Technical
        || code == 0x2B50 // Star
        || code == 0x2714 // Check mark
        || code == 0x2716 // X mark
}

/// Checks if a character is an emoji modifier (variation selector, ZWJ, skin tone).
fn is_emoji_modifier(c: char) -> bool {
    let code = c as u32;
    code == 0xFE0F // Variation Selector-16
        || code == 0xFE0E // Variation Selector-15
        || code == 0x200D // Zero Width Joiner
        || (0x1F3FB..=0x1F3FF).contains(&code) // Skin tone modifiers
}

impl Default for ConventionalGitmojiParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ConventionalGitmojiParser {
    fn name(&self) -> &'static str {
        "gitmoji"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &'static str {
        "Parses Conventional Commits with Gitmoji prefix (emoji or :code: format)"
    }
}

impl CommitParser for ConventionalGitmojiParser {
    fn parse(&self, raw: &RawCommit) -> Option<ParsedCommit> {
        let subject = raw.subject();

        // Try to extract gitmoji
        if let Some(extracted) = Self::extract_gitmoji(subject) {
            // Check if emoji is known (if strict mode)
            let is_known = EMOJI_MAP.contains_key(extracted.emoji);
            if self.config.strict_emoji && !is_known {
                return None;
            }

            // Create a modified raw commit without emoji for conventional parsing
            let modified_raw =
                RawCommit::new(&raw.hash, extracted.rest, &raw.author, &raw.email, raw.date);

            // Try conventional parsing on the rest
            if let Some(mut parsed) = self.conventional.parse(&modified_raw) {
                parsed.emoji = Some(extracted.emoji.to_string());
                return Some(parsed);
            }

            // If conventional parsing fails and infer_type_from_emoji is enabled
            if self.config.infer_type_from_emoji
                && let Some(&commit_type) = EMOJI_MAP.get(extracted.emoji)
            {
                return Some(
                    ParsedCommit::builder(&raw.hash, commit_type)
                        .message(extracted.rest)
                        .emoji(extracted.emoji)
                        .author(&raw.author)
                        .date(raw.date)
                        .build(),
                );
            }
        }

        // Fallback to conventional parsing without emoji
        self.conventional.parse(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_raw(message: &str) -> RawCommit {
        RawCommit::new("abc123", message, "Test", "test@test.com", Utc::now())
    }

    #[test]
    fn test_emoji_with_conventional() {
        let parser = ConventionalGitmojiParser::new();
        let raw = make_raw("✨ feat(api): add new endpoint");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert_eq!(parsed.emoji.as_deref(), Some("✨"));
        assert_eq!(parsed.message, "add new endpoint");
    }

    #[test]
    fn test_text_code_with_conventional() {
        let parser = ConventionalGitmojiParser::new();
        let raw = make_raw(":sparkles: feat(api): add new endpoint");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert_eq!(parsed.emoji.as_deref(), Some("✨"));
        assert_eq!(parsed.message, "add new endpoint");
    }

    #[test]
    fn test_emoji_only() {
        let parser = ConventionalGitmojiParser::new();
        let raw = make_raw("✨ add new feature");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert_eq!(parsed.emoji.as_deref(), Some("✨"));
        assert_eq!(parsed.message, "add new feature");
    }

    #[test]
    fn test_text_code_only() {
        let parser = ConventionalGitmojiParser::new();
        let raw = make_raw(":bug: fix login issue");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "fix");
        assert_eq!(parsed.emoji.as_deref(), Some("🐛"));
        assert_eq!(parsed.message, "fix login issue");
    }

    #[test]
    fn test_conventional_without_emoji() {
        let parser = ConventionalGitmojiParser::new();
        let raw = make_raw("fix: resolve bug");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "fix");
        assert!(parsed.emoji.is_none());
    }

    #[test]
    fn test_various_gitmojis() {
        let parser = ConventionalGitmojiParser::new();

        let test_cases = [
            ("🐛 fix a bug", "fix", "🐛"),
            (":fire: remove dead code", "chore", "🔥"),
            ("🚀 deploy to production", "release", "🚀"),
            (":lock: fix security issue", "security", "🔒"),
            ("♻️ refactor authentication", "refactor", "♻️"),
            (":arrow_up: upgrade dependencies", "deps", "⬆️"),
            ("💥 breaking api change", "breaking", "💥"),
            (":memo: update readme", "docs", "📝"),
        ];

        for (message, expected_type, expected_emoji) in test_cases {
            let raw = make_raw(message);
            let parsed = parser.parse(&raw).unwrap();
            assert_eq!(parsed.r#type, expected_type, "Failed for: {message}");
            assert_eq!(
                parsed.emoji.as_deref(),
                Some(expected_emoji),
                "Failed for: {message}"
            );
        }
    }

    #[test]
    fn test_unknown_emoji_non_strict() {
        let parser = ConventionalGitmojiParser::new();
        // Unknown emoji should still be extracted but won't infer type
        let raw = make_raw("🦄 feat: add unicorn support");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert_eq!(parsed.emoji.as_deref(), Some("🦄"));
    }

    #[test]
    fn test_unknown_text_code() {
        let parser = ConventionalGitmojiParser::new();
        // Unknown text code should not be treated as gitmoji
        let raw = make_raw(":unknown: feat: something");
        let parsed = parser.parse(&raw);

        // Should fail since :unknown: is not recognized and "feat: something" alone isn't valid
        assert!(parsed.is_none());
    }

    #[test]
    fn test_default() {
        let parser = ConventionalGitmojiParser::default();
        let raw = make_raw("✨ feat: test");
        let parsed = parser.parse(&raw).unwrap();
        assert_eq!(parsed.r#type, "feat");
    }

    #[test]
    fn test_plugin_name() {
        let parser = ConventionalGitmojiParser::new();
        assert_eq!(parser.name(), "gitmoji");
    }

    #[test]
    fn test_plugin_version() {
        let parser = ConventionalGitmojiParser::new();
        assert_eq!(parser.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_plugin_description() {
        let parser = ConventionalGitmojiParser::new();
        assert!(!parser.description().is_empty());
    }

    #[test]
    fn test_with_config_strict_mode() {
        let config = GitmojiParserConfig {
            infer_type_from_emoji: true,
            strict_emoji: true,
        };
        let parser = ConventionalGitmojiParser::with_config(config);
        // Unknown emoji should fail in strict mode
        let raw = make_raw("🦄 add unicorn");
        assert!(parser.parse(&raw).is_none());
    }

    #[test]
    fn test_with_config_no_infer() {
        let config = GitmojiParserConfig {
            infer_type_from_emoji: false,
            strict_emoji: false,
        };
        let parser = ConventionalGitmojiParser::with_config(config);
        // Should not infer type from emoji alone
        let raw = make_raw("✨ add new feature");
        // Will fail because it's not valid conventional commits format
        assert!(parser.parse(&raw).is_none());
    }

    #[test]
    fn test_config_default() {
        let config = GitmojiParserConfig::default();
        assert!(config.infer_type_from_emoji);
        assert!(!config.strict_emoji);
    }

    #[test]
    fn test_can_parse_valid() {
        let parser = ConventionalGitmojiParser::new();
        let raw = make_raw("✨ feat: something");
        assert!(parser.can_parse(&raw));
    }

    #[test]
    fn test_can_parse_invalid() {
        let parser = ConventionalGitmojiParser::new();
        let raw = make_raw("not a valid commit");
        assert!(!parser.can_parse(&raw));
    }

    #[test]
    fn test_preserves_author() {
        let parser = ConventionalGitmojiParser::new();
        let raw = RawCommit::new(
            "hash",
            "✨ feat: test",
            "John Doe",
            "john@test.com",
            Utc::now(),
        );
        let parsed = parser.parse(&raw).unwrap();
        assert_eq!(parsed.author, "John Doe");
    }

    #[test]
    fn test_emoji_with_variation_selector() {
        let parser = ConventionalGitmojiParser::new();
        // Ambulance with variation selector
        let raw = make_raw("🚑️ fix: urgent bug");
        let parsed = parser.parse(&raw).unwrap();
        assert_eq!(parsed.r#type, "fix");
    }

    #[test]
    fn test_all_feature_emojis() {
        let parser = ConventionalGitmojiParser::new();
        for emoji in ["✨", "🎉", "🚩", "👔", "🦺"] {
            let raw = make_raw(&format!("{emoji} add feature"));
            let parsed = parser.parse(&raw);
            assert!(parsed.is_some(), "Failed for emoji: {emoji}");
            assert_eq!(parsed.unwrap().r#type, "feat", "Failed for emoji: {emoji}");
        }
    }

    #[test]
    fn test_is_emoji_char() {
        assert!(is_emoji_char('✨'));
        assert!(is_emoji_char('🐛'));
        assert!(!is_emoji_char('a'));
        assert!(!is_emoji_char('1'));
    }

    #[test]
    fn test_is_emoji_modifier() {
        assert!(is_emoji_modifier('\u{FE0F}')); // Variation Selector-16
        assert!(is_emoji_modifier('\u{200D}')); // Zero Width Joiner
        assert!(!is_emoji_modifier('a'));
    }

    #[test]
    fn test_empty_message() {
        let parser = ConventionalGitmojiParser::new();
        let raw = make_raw("");
        assert!(parser.parse(&raw).is_none());
    }

    #[test]
    fn test_emoji_with_scope_and_breaking() {
        let parser = ConventionalGitmojiParser::new();
        let raw = make_raw("💥 feat(api)!: breaking change");
        let parsed = parser.parse(&raw).unwrap();

        assert_eq!(parsed.r#type, "feat");
        assert!(parsed.breaking);
        assert_eq!(parsed.scope.as_deref(), Some("api"));
        assert_eq!(parsed.emoji.as_deref(), Some("💥"));
    }
}
