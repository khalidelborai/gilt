//! Rust-idiomatic style extension trait for string types.
//!
//! This module provides a `Stylize` extension trait that enables method chaining
//! on `&str`, `String`, and `StyledStr` to build styled text, similar to the
//! `colored` crate's API. This is a distinctly Rusty API that Python's rich
//! cannot offer.
//!
//! # Examples
//!
//! ```
//! use gilt::styled_str::Stylize;
//!
//! let styled = "Hello".bold().red().on_blue();
//! assert_eq!(styled.text, "Hello");
//! ```

use crate::color::Color;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

use std::fmt;

/// A string with an associated style, created via the [`Stylize`] extension trait.
///
/// `StyledStr` implements [`Renderable`], so it can be passed directly to
/// [`Console::print`] and friends.
///
/// # Examples
///
/// ```
/// use gilt::styled_str::Stylize;
///
/// let greeting = "Hello, world!".bold().green();
/// assert_eq!(greeting.text, "Hello, world!");
/// ```
#[derive(Clone, Debug)]
pub struct StyledStr {
    /// The plain text content.
    pub text: String,
    /// The accumulated style.
    pub style: Style,
}

impl StyledStr {
    /// Create a new `StyledStr` with the given text and style.
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        StyledStr {
            text: text.into(),
            style,
        }
    }

    /// Convert this `StyledStr` into a [`Text`] with the style applied as a span.
    pub fn to_text(&self) -> Text {
        Text::styled(&self.text, self.style.clone())
    }
}

impl fmt::Display for StyledStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display shows the plain text without ANSI codes.
        write!(f, "{}", self.text)
    }
}

impl Renderable for StyledStr {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.to_text().rich_console(console, options)
    }
}

// ---------------------------------------------------------------------------
// Helper: construct a Style with a single attribute set
// ---------------------------------------------------------------------------

/// Build an attribute-only style. Uses `Style::parse` for known-valid attribute
/// names, which will never fail for the strings we pass.
fn attr_style(name: &str) -> Style {
    Style::parse(name).expect("internal: known-valid attribute name")
}

/// Build a foreground-color style.
fn fg_style(name: &str) -> Style {
    let color = Color::parse(name).expect("internal: known-valid color name");
    Style::from_color(Some(color), None)
}

/// Build a background-color style.
fn bg_style(name: &str) -> Style {
    let color = Color::parse(name).expect("internal: known-valid color name");
    Style::from_color(None, Some(color))
}

// ---------------------------------------------------------------------------
// Stylize trait
// ---------------------------------------------------------------------------

/// Extension trait for adding rich-style formatting to strings via method chaining.
///
/// Implemented for `&str`, `String`, and `StyledStr`. When called on a plain
/// string type, the first method creates a [`StyledStr`]. Subsequent chained
/// calls merge additional style attributes using the `Style` `+` operator.
///
/// # Examples
///
/// ```
/// use gilt::styled_str::Stylize;
///
/// // Single attribute
/// let bold = "hello".bold();
/// assert_eq!(bold.text, "hello");
///
/// // Chained attributes + foreground + background
/// let fancy = "world".bold().italic().red().on_blue();
/// assert_eq!(fancy.text, "world");
/// ```
pub trait Stylize: Sized {
    /// Apply an arbitrary [`Style`] to produce a [`StyledStr`].
    fn styled(self, style: Style) -> StyledStr;

    // -- Attribute methods --------------------------------------------------

    /// Make the text bold.
    fn bold(self) -> StyledStr {
        self.styled(attr_style("bold"))
    }
    /// Make the text dim / faint.
    fn dim(self) -> StyledStr {
        self.styled(attr_style("dim"))
    }
    /// Make the text italic.
    fn italic(self) -> StyledStr {
        self.styled(attr_style("italic"))
    }
    /// Underline the text.
    fn underline(self) -> StyledStr {
        self.styled(attr_style("underline"))
    }
    /// Apply strikethrough to the text.
    fn strikethrough(self) -> StyledStr {
        self.styled(attr_style("strike"))
    }
    /// Make the text blink.
    fn blink(self) -> StyledStr {
        self.styled(attr_style("blink"))
    }
    /// Reverse foreground and background colors.
    fn reverse(self) -> StyledStr {
        self.styled(attr_style("reverse"))
    }
    /// Hide / conceal the text.
    fn conceal(self) -> StyledStr {
        self.styled(attr_style("conceal"))
    }
    /// Apply double underline.
    fn underline2(self) -> StyledStr {
        self.styled(attr_style("underline2"))
    }
    /// Apply overline.
    fn overline(self) -> StyledStr {
        self.styled(attr_style("overline"))
    }

    // -- Foreground color methods -------------------------------------------

    /// Set foreground to red.
    fn red(self) -> StyledStr {
        self.styled(fg_style("red"))
    }
    /// Set foreground to green.
    fn green(self) -> StyledStr {
        self.styled(fg_style("green"))
    }
    /// Set foreground to blue.
    fn blue(self) -> StyledStr {
        self.styled(fg_style("blue"))
    }
    /// Set foreground to yellow.
    fn yellow(self) -> StyledStr {
        self.styled(fg_style("yellow"))
    }
    /// Set foreground to magenta.
    fn magenta(self) -> StyledStr {
        self.styled(fg_style("magenta"))
    }
    /// Set foreground to cyan.
    fn cyan(self) -> StyledStr {
        self.styled(fg_style("cyan"))
    }
    /// Set foreground to white.
    fn white(self) -> StyledStr {
        self.styled(fg_style("white"))
    }
    /// Set foreground to black.
    fn black(self) -> StyledStr {
        self.styled(fg_style("black"))
    }
    /// Set foreground to bright red.
    fn bright_red(self) -> StyledStr {
        self.styled(fg_style("bright_red"))
    }
    /// Set foreground to bright green.
    fn bright_green(self) -> StyledStr {
        self.styled(fg_style("bright_green"))
    }
    /// Set foreground to bright blue.
    fn bright_blue(self) -> StyledStr {
        self.styled(fg_style("bright_blue"))
    }
    /// Set foreground to bright yellow.
    fn bright_yellow(self) -> StyledStr {
        self.styled(fg_style("bright_yellow"))
    }
    /// Set foreground to bright magenta.
    fn bright_magenta(self) -> StyledStr {
        self.styled(fg_style("bright_magenta"))
    }
    /// Set foreground to bright cyan.
    fn bright_cyan(self) -> StyledStr {
        self.styled(fg_style("bright_cyan"))
    }
    /// Set foreground to bright white.
    fn bright_white(self) -> StyledStr {
        self.styled(fg_style("bright_white"))
    }

    // -- Background color methods -------------------------------------------

    /// Set background to red.
    fn on_red(self) -> StyledStr {
        self.styled(bg_style("red"))
    }
    /// Set background to green.
    fn on_green(self) -> StyledStr {
        self.styled(bg_style("green"))
    }
    /// Set background to blue.
    fn on_blue(self) -> StyledStr {
        self.styled(bg_style("blue"))
    }
    /// Set background to yellow.
    fn on_yellow(self) -> StyledStr {
        self.styled(bg_style("yellow"))
    }
    /// Set background to magenta.
    fn on_magenta(self) -> StyledStr {
        self.styled(bg_style("magenta"))
    }
    /// Set background to cyan.
    fn on_cyan(self) -> StyledStr {
        self.styled(bg_style("cyan"))
    }
    /// Set background to white.
    fn on_white(self) -> StyledStr {
        self.styled(bg_style("white"))
    }
    /// Set background to black.
    fn on_black(self) -> StyledStr {
        self.styled(bg_style("black"))
    }

    // -- Arbitrary color methods --------------------------------------------

    /// Set the foreground to an arbitrary color by name or hex string.
    ///
    /// # Panics
    /// Panics if `color` is not a valid color string.
    fn fg(self, color: &str) -> StyledStr {
        self.styled(fg_style(color))
    }

    /// Set the background to an arbitrary color by name or hex string.
    ///
    /// # Panics
    /// Panics if `color` is not a valid color string.
    fn bg(self, color: &str) -> StyledStr {
        self.styled(bg_style(color))
    }

    /// Apply a hyperlink.
    fn link(self, url: &str) -> StyledStr {
        self.styled(Style::with_link(url))
    }
}

// ---------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------

impl Stylize for &str {
    fn styled(self, style: Style) -> StyledStr {
        StyledStr {
            text: self.to_string(),
            style,
        }
    }
}

impl Stylize for String {
    fn styled(self, style: Style) -> StyledStr {
        StyledStr { text: self, style }
    }
}

impl Stylize for StyledStr {
    fn styled(self, style: Style) -> StyledStr {
        StyledStr {
            text: self.text,
            style: self.style + style,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleBuilder;

    #[test]
    fn test_bold_str() {
        let s = "hello".bold();
        assert_eq!(s.text, "hello");
        assert_eq!(s.style.bold(), Some(true));
    }

    #[test]
    fn test_chain_bold_red() {
        let s = "hello".bold().red();
        assert_eq!(s.text, "hello");
        assert_eq!(s.style.bold(), Some(true));
        let color = s.style.color().expect("should have foreground color");
        assert_eq!(color.name, "red");
    }

    #[test]
    fn test_chain_on_color() {
        let s = "hello".red().on_blue();
        assert_eq!(s.text, "hello");
        let fg = s.style.color().expect("should have fg");
        assert_eq!(fg.name, "red");
        let bg = s.style.bgcolor().expect("should have bg");
        assert_eq!(bg.name, "blue");
    }

    #[test]
    fn test_styled_str_renderable() {
        let styled = "Hello".bold().green();
        let console = ConsoleBuilder::new().force_terminal(true).width(80).build();
        let options = console.options();
        let segments = styled.rich_console(&console, &options);
        // Should produce at least one segment containing "Hello"
        let text: String = segments.iter().map(|seg| seg.text.as_str()).collect();
        assert!(text.contains("Hello"));
    }

    #[test]
    fn test_stylize_string() {
        let owned = String::from("world");
        let s = owned.italic().cyan();
        assert_eq!(s.text, "world");
        assert_eq!(s.style.italic(), Some(true));
        let color = s.style.color().expect("should have foreground color");
        assert_eq!(color.name, "cyan");
    }

    #[test]
    fn test_display() {
        let s = "plain text".bold().red();
        // Display should show the plain text, not ANSI codes.
        assert_eq!(format!("{}", s), "plain text");
    }

    #[test]
    fn test_styled_str_text_conversion() {
        let s = "convert me".underline().magenta();
        let text = s.to_text();
        assert_eq!(text.plain(), "convert me");
        // The text should have a span with the style applied.
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_all_colors() {
        let colors = vec![
            ("red", "hello".red()),
            ("green", "hello".green()),
            ("blue", "hello".blue()),
            ("yellow", "hello".yellow()),
            ("magenta", "hello".magenta()),
            ("cyan", "hello".cyan()),
            ("white", "hello".white()),
            ("black", "hello".black()),
            ("bright_red", "hello".bright_red()),
            ("bright_green", "hello".bright_green()),
            ("bright_blue", "hello".bright_blue()),
            ("bright_yellow", "hello".bright_yellow()),
            ("bright_magenta", "hello".bright_magenta()),
            ("bright_cyan", "hello".bright_cyan()),
            ("bright_white", "hello".bright_white()),
        ];

        for (name, styled) in colors {
            let color = styled
                .style
                .color()
                .unwrap_or_else(|| panic!("color method '{}' should set fg color", name));
            assert_eq!(
                color.name, name,
                "expected color name '{}', got '{}'",
                name, color.name
            );
        }
    }

    #[test]
    fn test_all_bg_colors() {
        let colors = vec![
            ("red", "hello".on_red()),
            ("green", "hello".on_green()),
            ("blue", "hello".on_blue()),
            ("yellow", "hello".on_yellow()),
            ("magenta", "hello".on_magenta()),
            ("cyan", "hello".on_cyan()),
            ("white", "hello".on_white()),
            ("black", "hello".on_black()),
        ];

        for (name, styled) in colors {
            let bg = styled
                .style
                .bgcolor()
                .unwrap_or_else(|| panic!("on_{} should set bg color", name));
            assert_eq!(
                bg.name, name,
                "expected bg color '{}', got '{}'",
                name, bg.name
            );
        }
    }

    #[test]
    fn test_all_attributes() {
        assert_eq!("x".bold().style.bold(), Some(true));
        assert_eq!("x".dim().style.dim(), Some(true));
        assert_eq!("x".italic().style.italic(), Some(true));
        assert_eq!("x".underline().style.underline(), Some(true));
        assert_eq!("x".strikethrough().style.strike(), Some(true));
        assert_eq!("x".blink().style.blink(), Some(true));
        assert_eq!("x".reverse().style.reverse(), Some(true));
        assert_eq!("x".conceal().style.conceal(), Some(true));
        assert_eq!("x".overline().style.overline(), Some(true));
    }

    #[test]
    fn test_complex_chain() {
        let s = "fancy"
            .bold()
            .italic()
            .underline()
            .bright_yellow()
            .on_blue();
        assert_eq!(s.text, "fancy");
        assert_eq!(s.style.bold(), Some(true));
        assert_eq!(s.style.italic(), Some(true));
        assert_eq!(s.style.underline(), Some(true));
        assert_eq!(s.style.color().unwrap().name, "bright_yellow");
        assert_eq!(s.style.bgcolor().unwrap().name, "blue");
    }

    #[test]
    fn test_fg_and_bg_arbitrary() {
        let s = "hex".fg("#ff0000").bg("#00ff00");
        assert!(s.style.color().is_some());
        assert!(s.style.bgcolor().is_some());
    }

    #[test]
    fn test_link() {
        let s = "click me".blue().underline().link("https://example.com");
        assert_eq!(s.style.link(), Some("https://example.com"));
        assert_eq!(s.style.color().unwrap().name, "blue");
    }

    #[test]
    fn test_styled_method_with_parsed_style() {
        let style = Style::parse("bold italic red on white").unwrap();
        let s = "custom".styled(style);
        assert_eq!(s.style.bold(), Some(true));
        assert_eq!(s.style.italic(), Some(true));
        assert_eq!(s.style.color().unwrap().name, "red");
        assert_eq!(s.style.bgcolor().unwrap().name, "white");
    }
}
