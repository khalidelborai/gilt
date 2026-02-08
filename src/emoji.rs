//! Emoji widget for terminal rendering.
//!
//! Provides the `Emoji` type that resolves emoji names to Unicode characters
//! and can be rendered to the console.

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::emoji_codes::EMOJI;
use crate::emoji_replace::emoji_replace;
use crate::segment::Segment;
use crate::style::Style;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Error returned when an emoji name is not found in the emoji dictionary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoEmoji(pub String);

impl fmt::Display for NoEmoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No emoji called {:?}", self.0)
    }
}

impl std::error::Error for NoEmoji {}

// ---------------------------------------------------------------------------
// Emoji
// ---------------------------------------------------------------------------

/// A single emoji character that can be rendered to the console.
///
/// # Examples
///
/// ```
/// use gilt::emoji::Emoji;
///
/// let emoji = Emoji::new("heart").unwrap();
/// assert_eq!(emoji.to_string(), "\u{2764}");
/// ```
pub struct Emoji {
    /// The emoji name (e.g. "heart", "thumbs_up").
    pub name: String,
    /// The rendering style.
    pub style: Style,
    /// The resolved Unicode character(s).
    pub char: String,
}

impl Emoji {
    /// Create a new Emoji by looking up `name` in the emoji dictionary.
    ///
    /// Returns `Err(NoEmoji)` if the name is not found.
    pub fn new(name: &str) -> Result<Self, NoEmoji> {
        let emoji_char = EMOJI
            .get(name)
            .ok_or_else(|| NoEmoji(name.to_string()))?;

        Ok(Emoji {
            name: name.to_string(),
            style: Style::null(),
            char: emoji_char.to_string(),
        })
    }

    /// Set the rendering style (builder pattern).
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Append a variant selector to the emoji character.
    ///
    /// - `"text"` appends U+FE0E (text presentation selector)
    /// - `"emoji"` appends U+FE0F (emoji presentation selector)
    pub fn with_variant(mut self, variant: &str) -> Self {
        match variant {
            "text" => self.char.push('\u{FE0E}'),
            "emoji" => self.char.push('\u{FE0F}'),
            _ => {}
        }
        self
    }

    /// Replace all `:emoji_name:` patterns in `text` with their Unicode equivalents.
    ///
    /// This is a convenience wrapper around `emoji_replace`.
    pub fn replace(text: &str) -> String {
        emoji_replace(text, None)
    }
}

impl fmt::Display for Emoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.char)
    }
}

impl fmt::Debug for Emoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<emoji {:?}>", self.name)
    }
}

impl Renderable for Emoji {
    fn rich_console(&self, console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let style = if self.style.is_null() {
            match console.get_style("none") {
                Ok(s) => s,
                Err(_) => Style::null(),
            }
        } else {
            self.style.clone()
        };
        vec![Segment::styled(&self.char, style)]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid_name() {
        let emoji = Emoji::new("heart").unwrap();
        assert_eq!(emoji.name, "heart");
        assert_eq!(emoji.char, "\u{2764}");
    }

    #[test]
    fn test_new_invalid_name() {
        let result = Emoji::new("this_does_not_exist_xyz");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.0, "this_does_not_exist_xyz");
        assert!(err.to_string().contains("this_does_not_exist_xyz"));
    }

    #[test]
    fn test_display_trait() {
        let emoji = Emoji::new("heart").unwrap();
        assert_eq!(format!("{}", emoji), "\u{2764}");
    }

    #[test]
    fn test_debug_trait() {
        let emoji = Emoji::new("heart").unwrap();
        let debug = format!("{:?}", emoji);
        assert!(debug.contains("heart"));
        assert!(debug.contains("emoji"));
    }

    #[test]
    fn test_renderable_trait() {
        let emoji = Emoji::new("heart").unwrap();
        let console = Console::builder().width(80).build();
        let options = console.options();
        let segments = emoji.rich_console(&console, &options);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "\u{2764}");
    }

    #[test]
    fn test_replace_class_method() {
        let result = Emoji::replace("Hello :heart:!");
        assert_eq!(result, "Hello \u{2764}!");
    }

    #[test]
    fn test_with_style() {
        let style = Style::parse("bold red").unwrap();
        let emoji = Emoji::new("heart").unwrap().with_style(style.clone());
        assert_eq!(emoji.style, style);
    }

    #[test]
    fn test_with_style_renderable() {
        let style = Style::parse("bold").unwrap();
        let emoji = Emoji::new("heart").unwrap().with_style(style);
        let console = Console::builder().width(80).build();
        let options = console.options();
        let segments = emoji.rich_console(&console, &options);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].style.as_ref().unwrap().bold(), Some(true));
    }

    #[test]
    fn test_with_variant_text() {
        let emoji = Emoji::new("heart").unwrap().with_variant("text");
        assert_eq!(emoji.char, "\u{2764}\u{FE0E}");
        assert_eq!(emoji.to_string(), "\u{2764}\u{FE0E}");
    }

    #[test]
    fn test_with_variant_emoji() {
        let emoji = Emoji::new("heart").unwrap().with_variant("emoji");
        assert_eq!(emoji.char, "\u{2764}\u{FE0F}");
    }

    #[test]
    fn test_with_variant_unknown() {
        let emoji = Emoji::new("heart").unwrap().with_variant("other");
        // Unknown variant should not change the char
        assert_eq!(emoji.char, "\u{2764}");
    }

    #[test]
    fn test_no_emoji_error_display() {
        let err = NoEmoji("missing".to_string());
        assert_eq!(err.to_string(), "No emoji called \"missing\"");
    }

    #[test]
    fn test_chained_builders() {
        let emoji = Emoji::new("thumbs_up")
            .unwrap()
            .with_style(Style::parse("bold").unwrap())
            .with_variant("emoji");
        assert_eq!(emoji.style.bold(), Some(true));
        assert!(emoji.char.ends_with('\u{FE0F}'));
    }
}
