//! Badge/Tag widget for displaying status indicators.
//!
//! Compact visual badges like GitHub badges for modern CLIs.
//!
//! # Examples
//!
//! ```
//! use gilt::badge::{Badge, BadgeStyle};
//!
//! // Create a success badge
//! let badge = Badge::success("Success");
//!
//! // Create a custom styled badge
//! let badge = Badge::new("Custom")
//!     .style(BadgeStyle::Info)
//!     .icon("ℹ")
//!     .rounded(true);
//! ```

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;


// -----------------------------------------------------------------------------
// BadgeStyle
// -----------------------------------------------------------------------------

/// Predefined badge styles for common status indicators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BadgeStyle {
    /// Green background, white text - indicates success/completion.
    Success,
    /// Red background, white text - indicates error/failure.
    Error,
    /// Yellow background, black text - indicates warning/caution.
    Warning,
    /// Blue background, white text - indicates information.
    Info,
    /// Gray background, white text - neutral/default state.
    Neutral,
    /// Custom style with full control over appearance.
    Custom(Style),
}

impl BadgeStyle {
    /// Get the background color for this badge style.
    fn bg_style(&self) -> Style {
        match self {
            BadgeStyle::Success => Style::parse("on green").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Error => Style::parse("on red").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Warning => Style::parse("on yellow").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Info => Style::parse("on blue").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Neutral => Style::parse("on grey").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Custom(style) => {
                // Extract just the background style from the custom style
                style.background_style()
            }
        }
    }

    /// Get the foreground (text) color for this badge style.
    fn fg_style(&self) -> Style {
        match self {
            BadgeStyle::Success => Style::parse("white").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Error => Style::parse("white").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Warning => Style::parse("black").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Info => Style::parse("white").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Neutral => Style::parse("white").unwrap_or_else(|_| Style::null()),
            BadgeStyle::Custom(style) => {
                // Create a style with just the foreground color
                Style::from_color(style.color().cloned(), None)
            }
        }
    }

    /// Get the default icon for this badge style.
    fn default_icon(&self) -> Option<&'static str> {
        match self {
            BadgeStyle::Success => Some("✓"),
            BadgeStyle::Error => Some("✗"),
            BadgeStyle::Warning => Some("⚠"),
            BadgeStyle::Info => Some("ℹ"),
            BadgeStyle::Neutral => None,
            BadgeStyle::Custom(_) => None,
        }
    }
}

// -----------------------------------------------------------------------------
// Badge
// -----------------------------------------------------------------------------

/// A badge/tag widget for displaying status indicators.
///
/// Badges are compact visual elements with background colors and optional icons,
/// similar to GitHub badges or status tags in modern CLIs.
///
/// # Examples
///
/// ```
/// use gilt::badge::{Badge, BadgeStyle};
///
/// // Using convenience constructors
/// let success = Badge::success("Build Passed");
/// let error = Badge::error("Failed");
/// let warning = Badge::warning("Deprecated");
/// let info = Badge::info("New Feature");
///
/// // Using builder pattern
/// let custom = Badge::new("Beta")
///     .style(BadgeStyle::Info)
///     .icon("β")
///     .rounded(true);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Badge {
    text: String,
    style: BadgeStyle,
    icon: Option<String>,
    rounded: bool,
}

impl Badge {
    /// Create a new badge with the given text.
    ///
    /// Defaults to `BadgeStyle::Neutral` with no icon and square corners.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::badge::Badge;
    ///
    /// let badge = Badge::new("Status");
    /// ```
    pub fn new(text: impl Into<String>) -> Self {
        Badge {
            text: text.into(),
            style: BadgeStyle::Neutral,
            icon: None,
            rounded: false,
        }
    }

    /// Set the badge style.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::badge::{Badge, BadgeStyle};
    ///
    /// let badge = Badge::new("Done").style(BadgeStyle::Success);
    /// ```
    #[must_use]
    pub fn style(mut self, style: BadgeStyle) -> Self {
        self.style = style;
        self
    }

    /// Create a success badge (green background, white text, ✓ icon).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::badge::Badge;
    ///
    /// let badge = Badge::success("Complete");
    /// ```
    pub fn success(text: impl Into<String>) -> Self {
        Badge::new(text)
            .style(BadgeStyle::Success)
            .icon("✓")
    }

    /// Create an error badge (red background, white text, ✗ icon).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::badge::Badge;
    ///
    /// let badge = Badge::error("Failed");
    /// ```
    pub fn error(text: impl Into<String>) -> Self {
        Badge::new(text)
            .style(BadgeStyle::Error)
            .icon("✗")
    }

    /// Create a warning badge (yellow background, black text, ⚠ icon).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::badge::Badge;
    ///
    /// let badge = Badge::warning("Caution");
    /// ```
    pub fn warning(text: impl Into<String>) -> Self {
        Badge::new(text)
            .style(BadgeStyle::Warning)
            .icon("⚠")
    }

    /// Create an info badge (blue background, white text, ℹ icon).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::badge::Badge;
    ///
    /// let badge = Badge::info("Note");
    /// ```
    pub fn info(text: impl Into<String>) -> Self {
        Badge::new(text)
            .style(BadgeStyle::Info)
            .icon("ℹ")
    }

    /// Set the icon displayed before the text.
    ///
    /// Set to `None` to remove the icon.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::badge::Badge;
    ///
    /// let badge = Badge::new("Star").icon("★");
    /// ```
    #[must_use]
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set whether to use rounded corners.
    ///
    /// When `true`, uses rounded box characters (`╭╮╰╯`).
    /// When `false`, uses square box characters (`┌┐└┘`).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::badge::Badge;
    ///
    /// let badge = Badge::new("Rounded").rounded(true);
    /// ```
    #[must_use]
    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    /// Get the text content of this badge.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the style of this badge.
    pub fn badge_style(&self) -> &BadgeStyle {
        &self.style
    }

    /// Get the icon of this badge, if any.
    pub fn icon_str(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// Check if this badge uses rounded corners.
    pub fn is_rounded(&self) -> bool {
        self.rounded
    }

    /// Get the effective icon to display (explicit or default for style).
    fn effective_icon(&self) -> Option<&str> {
        self.icon.as_deref().or_else(|| self.style.default_icon())
    }

    /// Get the box characters based on rounded setting.
    fn box_chars(&self) -> (char, char, char, char, char, char) {
        if self.rounded {
            // Rounded: ╭ ╮ ╰ ╯ ─ │
            ('╭', '╮', '╰', '╯', '─', '│')
        } else {
            // Square: ┌ ┐ └ ┘ ─ │
            ('┌', '┐', '└', '┘', '─', '│')
        }
    }
}

impl Renderable for Badge {
    fn gilt_console(&self, _console: &Console, _options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();

        let (tl, tr, bl, br, horiz, vert) = self.box_chars();
        let bg_style = self.style.bg_style();
        let fg_style = self.style.fg_style();

        // Build the content (icon + text)
        let content = match self.effective_icon() {
            Some(icon) => format!("{} {}", icon, self.text),
            None => self.text.clone(),
        };

        let content_width = crate::cells::cell_len(&content);
        let inner_width = content_width + 2; // +2 for padding spaces

        // Build top border
        let mut top = String::new();
        top.push(tl);
        for _ in 0..inner_width {
            top.push(horiz);
        }
        top.push(tr);
        segments.push(Segment::styled(&top, bg_style.clone()));
        segments.push(Segment::line());

        // Build middle row with content
        let mut middle = String::new();
        middle.push(vert);
        middle.push(' ');
        middle.push_str(&content);
        middle.push(' ');
        middle.push(vert);
        
        // The middle row needs both background and foreground styling
        // We create a combined style
        let combined_style = fg_style + bg_style.clone();
        segments.push(Segment::styled(&middle, combined_style));
        segments.push(Segment::line());

        // Build bottom border
        let mut bottom = String::new();
        bottom.push(bl);
        for _ in 0..inner_width {
            bottom.push(horiz);
        }
        bottom.push(br);
        segments.push(Segment::styled(&bottom, bg_style));
        segments.push(Segment::line());

        segments
    }
}

impl std::fmt::Display for Badge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut console = Console::builder()
            .width(f.width().unwrap_or(80))
            .force_terminal(true)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(self);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_console(width: usize) -> Console {
        Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build()
    }

    fn render_badge(console: &Console, badge: &Badge) -> String {
        let opts = console.options();
        let segments = badge.gilt_console(console, &opts);
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    // -- Construction ---------------------------------------------------------

    #[test]
    fn test_new() {
        let badge = Badge::new("Test");
        assert_eq!(badge.text(), "Test");
        assert_eq!(badge.badge_style(), &BadgeStyle::Neutral);
        assert_eq!(badge.icon_str(), None);
        assert!(!badge.is_rounded());
    }

    #[test]
    fn test_success() {
        let badge = Badge::success("Done");
        assert_eq!(badge.text(), "Done");
        assert_eq!(badge.badge_style(), &BadgeStyle::Success);
        assert_eq!(badge.icon_str(), Some("✓"));
    }

    #[test]
    fn test_error() {
        let badge = Badge::error("Failed");
        assert_eq!(badge.text(), "Failed");
        assert_eq!(badge.badge_style(), &BadgeStyle::Error);
        assert_eq!(badge.icon_str(), Some("✗"));
    }

    #[test]
    fn test_warning() {
        let badge = Badge::warning("Caution");
        assert_eq!(badge.text(), "Caution");
        assert_eq!(badge.badge_style(), &BadgeStyle::Warning);
        assert_eq!(badge.icon_str(), Some("⚠"));
    }

    #[test]
    fn test_info() {
        let badge = Badge::info("Note");
        assert_eq!(badge.text(), "Note");
        assert_eq!(badge.badge_style(), &BadgeStyle::Info);
        assert_eq!(badge.icon_str(), Some("ℹ"));
    }

    // -- Builder pattern ------------------------------------------------------

    #[test]
    fn test_builder_style() {
        let badge = Badge::new("Test").style(BadgeStyle::Success);
        assert_eq!(badge.badge_style(), &BadgeStyle::Success);
    }

    #[test]
    fn test_builder_icon() {
        let badge = Badge::new("Test").icon("★");
        assert_eq!(badge.icon_str(), Some("★"));
    }

    #[test]
    fn test_builder_rounded() {
        let badge = Badge::new("Test").rounded(true);
        assert!(badge.is_rounded());
    }

    #[test]
    fn test_builder_chain() {
        let badge = Badge::new("Test")
            .style(BadgeStyle::Info)
            .icon("→")
            .rounded(true);
        
        assert_eq!(badge.text(), "Test");
        assert_eq!(badge.badge_style(), &BadgeStyle::Info);
        assert_eq!(badge.icon_str(), Some("→"));
        assert!(badge.is_rounded());
    }

    #[test]
    fn test_icon_override() {
        // Start with success (has default icon ✓)
        let badge = Badge::success("Done").icon("✔");
        assert_eq!(badge.icon_str(), Some("✔"));
    }

    #[test]
    fn test_icon_remove() {
        let badge = Badge::success("Done").icon("");
        assert_eq!(badge.icon_str(), Some(""));
    }

    // -- Rendering ------------------------------------------------------------

    #[test]
    fn test_render_square() {
        let console = make_console(80);
        let badge = Badge::new("OK");
        let output = render_badge(&console, &badge);
        
        // Should contain square box characters
        assert!(output.contains('┌'));
        assert!(output.contains('┐'));
        assert!(output.contains('└'));
        assert!(output.contains('┘'));
        assert!(output.contains("OK"));
    }

    #[test]
    fn test_render_rounded() {
        let console = make_console(80);
        let badge = Badge::new("OK").rounded(true);
        let output = render_badge(&console, &badge);
        
        // Should contain rounded box characters
        assert!(output.contains('╭'));
        assert!(output.contains('╮'));
        assert!(output.contains('╰'));
        assert!(output.contains('╯'));
        assert!(output.contains("OK"));
    }

    #[test]
    fn test_render_with_icon() {
        let console = make_console(80);
        let badge = Badge::success("Done");
        let output = render_badge(&console, &badge);
        
        assert!(output.contains("✓"));
        assert!(output.contains("Done"));
    }

    #[test]
    fn test_render_no_icon() {
        let console = make_console(80);
        let badge = Badge::new("Plain").icon("");
        let output = render_badge(&console, &badge);
        
        // Just text with padding, no icon
        assert!(output.contains(" Plain "));
    }

    #[test]
    fn test_render_multiline() {
        let console = make_console(80);
        let badge = Badge::new("Test");
        let output = render_badge(&console, &badge);
        
        // Badge should be 3 lines (top, middle, bottom)
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    // -- Display trait --------------------------------------------------------

    #[test]
    fn test_display_trait() {
        let badge = Badge::info("Info");
        let s = format!("{}", badge);
        assert!(s.contains("Info"));
        assert!(s.contains('┌') || s.contains('╭'));
    }

    // -- BadgeStyle ------------------------------------------------------------

    #[test]
    fn test_badge_style_default_icons() {
        assert_eq!(BadgeStyle::Success.default_icon(), Some("✓"));
        assert_eq!(BadgeStyle::Error.default_icon(), Some("✗"));
        assert_eq!(BadgeStyle::Warning.default_icon(), Some("⚠"));
        assert_eq!(BadgeStyle::Info.default_icon(), Some("ℹ"));
        assert_eq!(BadgeStyle::Neutral.default_icon(), None);
        assert_eq!(BadgeStyle::Custom(Style::null()).default_icon(), None);
    }

    #[test]
    fn test_badge_style_equality() {
        assert_eq!(BadgeStyle::Success, BadgeStyle::Success);
        assert_ne!(BadgeStyle::Success, BadgeStyle::Error);
        
        let style1 = BadgeStyle::Custom(Style::parse("red").unwrap());
        let style2 = BadgeStyle::Custom(Style::parse("red").unwrap());
        assert_eq!(style1, style2);
    }

    // -- Effective icon -------------------------------------------------------

    #[test]
    fn test_effective_icon_explicit() {
        let badge = Badge::new("Test").icon("★");
        assert_eq!(badge.effective_icon(), Some("★"));
    }

    #[test]
    fn test_effective_icon_default() {
        let badge = Badge::success("Test");
        // Icon is set explicitly in success()
        assert_eq!(badge.effective_icon(), Some("✓"));
    }

    #[test]
    fn test_effective_icon_none() {
        let badge = Badge::new("Test"); // Neutral style, no explicit icon
        assert_eq!(badge.effective_icon(), None);
    }

    // -- Box characters -------------------------------------------------------

    #[test]
    fn test_box_chars_square() {
        let badge = Badge::new("Test").rounded(false);
        let (tl, tr, bl, br, horiz, vert) = badge.box_chars();
        assert_eq!(tl, '┌');
        assert_eq!(tr, '┐');
        assert_eq!(bl, '└');
        assert_eq!(br, '┘');
        assert_eq!(horiz, '─');
        assert_eq!(vert, '│');
    }

    #[test]
    fn test_box_chars_rounded() {
        let badge = Badge::new("Test").rounded(true);
        let (tl, tr, bl, br, horiz, vert) = badge.box_chars();
        assert_eq!(tl, '╭');
        assert_eq!(tr, '╮');
        assert_eq!(bl, '╰');
        assert_eq!(br, '╯');
        assert_eq!(horiz, '─');
        assert_eq!(vert, '│');
    }

    // -- Clone ----------------------------------------------------------------

    #[test]
    fn test_clone() {
        let badge = Badge::success("Done").rounded(true);
        let cloned = badge.clone();
        
        assert_eq!(badge.text(), cloned.text());
        assert_eq!(badge.badge_style(), cloned.badge_style());
        assert_eq!(badge.icon_str(), cloned.icon_str());
        assert_eq!(badge.is_rounded(), cloned.is_rounded());
    }
}
