//! Emoji replacement in text strings.
//!
//! Replaces `:emoji_name:` patterns in text with actual Unicode emoji characters.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::emoji_codes::EMOJI;

static EMOJI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r":(\S*?)(?:(?:\-)(emoji|text))?:").unwrap()
});

/// Replace `:emoji_name:` patterns in text with corresponding Unicode emoji.
///
/// Supports optional variant suffixes:
/// - `:name-text:` appends the text presentation selector (U+FE0E)
/// - `:name-emoji:` appends the emoji presentation selector (U+FE0F)
///
/// If `default_variant` is provided ("text" or "emoji"), that variant selector
/// is appended to all replacements that don't specify one explicitly.
///
/// Unknown emoji names are left unchanged (e.g. `:unknown:` stays as-is).
pub fn emoji_replace(text: &str, default_variant: Option<&str>) -> String {
    let default_variant_code = match default_variant {
        Some("text") => "\u{FE0E}",
        Some("emoji") => "\u{FE0F}",
        _ => "",
    };

    EMOJI_RE
        .replace_all(text, |caps: &regex::Captures| {
            let full_match = caps.get(0).unwrap().as_str();
            let emoji_name = caps.get(1).unwrap().as_str().to_lowercase();
            let variant = caps.get(2).map(|m| m.as_str());

            match EMOJI.get(emoji_name.as_str()) {
                Some(emoji_char) => {
                    let variant_code = match variant {
                        Some("text") => "\u{FE0E}",
                        Some("emoji") => "\u{FE0F}",
                        _ => default_variant_code,
                    };
                    format!("{}{}", emoji_char, variant_code)
                }
                None => full_match.to_string(),
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_replacement() {
        let result = emoji_replace(":heart:", None);
        assert_eq!(result, "\u{2764}");
    }

    #[test]
    fn test_unknown_emoji_unchanged() {
        let result = emoji_replace(":unknown_xyz:", None);
        assert_eq!(result, ":unknown_xyz:");
    }

    #[test]
    fn test_variant_text() {
        let result = emoji_replace(":heart-text:", None);
        assert_eq!(result, "\u{2764}\u{FE0E}");
    }

    #[test]
    fn test_variant_emoji() {
        let result = emoji_replace(":heart-emoji:", None);
        assert_eq!(result, "\u{2764}\u{FE0F}");
    }

    #[test]
    fn test_multiple_emojis() {
        let result = emoji_replace("Hello :heart: world :thumbs_up:!", None);
        assert_eq!(result, "Hello \u{2764} world \u{1F44D}!");
    }

    #[test]
    fn test_no_emojis_passthrough() {
        let result = emoji_replace("Hello, world!", None);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_default_variant_text() {
        let result = emoji_replace(":heart:", Some("text"));
        assert_eq!(result, "\u{2764}\u{FE0E}");
    }

    #[test]
    fn test_default_variant_emoji() {
        let result = emoji_replace(":heart:", Some("emoji"));
        assert_eq!(result, "\u{2764}\u{FE0F}");
    }

    #[test]
    fn test_explicit_variant_overrides_default() {
        // Explicit -text should use text variant even when default is emoji
        let result = emoji_replace(":heart-text:", Some("emoji"));
        assert_eq!(result, "\u{2764}\u{FE0E}");
    }

    #[test]
    fn test_empty_string() {
        let result = emoji_replace("", None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_case_insensitive_name() {
        let result = emoji_replace(":HEART:", None);
        assert_eq!(result, "\u{2764}");
    }
}
