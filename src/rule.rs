//! Rule widget -- a horizontal line with optional title.
//!
//! Port of Python's `rich/rule.py`.

use crate::align_widget::HorizontalAlign;
use crate::cells::{cell_len, set_cell_size};
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{OverflowMethod, Text};

// ---------------------------------------------------------------------------
// Rule
// ---------------------------------------------------------------------------

/// A horizontal rule (line) with optional centered, left-, or right-aligned title.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Optional title text displayed within the rule.
    pub title: Option<Text>,
    /// Character(s) used to draw the line.
    pub characters: String,
    /// Style for the rule line characters.
    pub style: Style,
    /// String appended after the rule (default `"\n"`).
    pub end: String,
    /// Alignment of the title within the rule.
    pub align: HorizontalAlign,
}

impl Rule {
    /// Create a new `Rule` with defaults: no title, "━" character,
    /// `rule.line` style, newline end, center alignment.
    pub fn new() -> Self {
        Rule {
            title: None,
            characters: "\u{2501}".to_string(), // ━ (heavy horizontal)
            style: Style::null(),
            end: "\n".to_string(),
            align: HorizontalAlign::Center,
        }
    }

    /// Create a rule with a title string.
    pub fn with_title(title: &str) -> Self {
        let mut rule = Rule::new();
        rule.title = Some(Text::new(title, Style::null()));
        rule
    }

    /// Set the line characters.
    #[must_use]
    pub fn characters(mut self, chars: &str) -> Self {
        self.characters = chars.to_string();
        self
    }

    /// Set the rule style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the alignment.
    #[must_use]
    pub fn align(mut self, align: HorizontalAlign) -> Self {
        self.align = align;
        self
    }

    /// Set the end string.
    #[must_use]
    pub fn end(mut self, end: &str) -> Self {
        self.end = end.to_string();
        self
    }

    /// Build a line of repeated characters to fill the given width.
    fn rule_line(&self, width: usize) -> String {
        if width == 0 {
            return String::new();
        }
        let char_len = cell_len(&self.characters);
        if char_len == 0 {
            return " ".repeat(width);
        }
        let repeats = width / char_len;
        let remainder = width % char_len;
        let mut line = self.characters.repeat(repeats);
        if remainder > 0 {
            let partial = set_cell_size(&self.characters, remainder);
            line.push_str(&partial);
        }
        line
    }
}

impl Default for Rule {
    fn default() -> Self {
        Rule::new()
    }
}

impl Renderable for Rule {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let width = options.max_width;

        // Resolve the style: try "rule.line" from the console theme, fall back to self.style
        let rule_style = if self.style.is_null() {
            console
                .get_style("rule.line")
                .unwrap_or_else(|_| self.style.clone())
        } else {
            self.style.clone()
        };

        // Use ASCII fallback if needed
        let chars = if options.ascii_only() && !self.characters.is_ascii() {
            "-".to_string()
        } else {
            self.characters.clone()
        };

        // Temporarily replace characters for rule_line calls below
        let rule_with_chars = Rule {
            title: self.title.clone(),
            characters: chars,
            style: rule_style.clone(),
            end: self.end.clone(),
            align: self.align,
        };

        let mut segments = Vec::new();

        match &self.title {
            None => {
                // No title: just a full-width line
                let line_text = rule_with_chars.rule_line(width);
                let mut text = Text::new(&line_text, rule_style.clone());
                text.overflow = Some(OverflowMethod::Crop);
                let exact = set_cell_size(text.plain(), width);
                segments.push(Segment::styled(&exact, rule_style));
                segments.push(Segment::new(&self.end, None, None));
            }
            Some(title) => {
                let mut title_text = title.clone();

                // Resolve title style
                let title_style = console
                    .get_style("rule.text")
                    .unwrap_or_else(|_| Style::null());

                // Apply title style as a span if it's not null
                if !title_style.is_null() {
                    let len = title_text.len();
                    if len > 0 {
                        title_text.stylize(title_style, 0, Some(len));
                    }
                }

                let char_len = cell_len(&rule_with_chars.characters);
                if char_len == 0 {
                    // Degenerate case: no rule characters
                    segments.push(Segment::new(title_text.plain(), None, None));
                    segments.push(Segment::new(&self.end, None, None));
                    return segments;
                }

                // Minimum rule needs at least some line chars on each side
                // We need at least 1 char + space + title + space + 1 char for center
                // Or title + space + chars for left/right
                let min_side = char_len.max(1);

                match self.align {
                    HorizontalAlign::Center => {
                        // Center: "rule_chars  title  rule_chars"
                        // Need at least min_side + 1 + title + 1 + min_side
                        let title_max_width = width.saturating_sub(min_side * 2 + 2);
                        if title_max_width == 0 || title_text.cell_len() == 0 {
                            // Title doesn't fit, just draw line
                            let line_text = rule_with_chars.rule_line(width);
                            let exact = set_cell_size(&line_text, width);
                            segments.push(Segment::styled(&exact, rule_style));
                            segments.push(Segment::new(&self.end, None, None));
                            return segments;
                        }

                        // Truncate title if necessary
                        title_text.truncate(title_max_width, Some(OverflowMethod::Ellipsis), false);

                        let title_width = title_text.cell_len();
                        let side_width = (width.saturating_sub(title_width + 2)) / 2;
                        let left_width = side_width;
                        let right_width = width.saturating_sub(left_width + title_width + 2);

                        // Left rule
                        let left_line = rule_with_chars.rule_line(left_width);
                        let left_exact = set_cell_size(&left_line, left_width);
                        segments.push(Segment::styled(&left_exact, rule_style.clone()));

                        // Space + title + space
                        segments.push(Segment::new(" ", None, None));
                        segments.extend(title_text.render().into_iter().filter(|s| s.text != "\n"));
                        segments.push(Segment::new(" ", None, None));

                        // Right rule
                        let right_line = rule_with_chars.rule_line(right_width);
                        let right_exact = set_cell_size(&right_line, right_width);
                        segments.push(Segment::styled(&right_exact, rule_style));

                        segments.push(Segment::new(&self.end, None, None));
                    }
                    HorizontalAlign::Left => {
                        // Left: "title  rule_chars..."
                        let title_max_width = width.saturating_sub(min_side + 2);
                        if title_max_width == 0 || title_text.cell_len() == 0 {
                            let line_text = rule_with_chars.rule_line(width);
                            let exact = set_cell_size(&line_text, width);
                            segments.push(Segment::styled(&exact, rule_style));
                            segments.push(Segment::new(&self.end, None, None));
                            return segments;
                        }

                        title_text.truncate(title_max_width, Some(OverflowMethod::Ellipsis), false);

                        let title_width = title_text.cell_len();
                        let rule_width = width.saturating_sub(title_width + 2);

                        // Title + space
                        segments.extend(title_text.render().into_iter().filter(|s| s.text != "\n"));
                        segments.push(Segment::new(" ", None, None));

                        // Rule line
                        let line = rule_with_chars.rule_line(rule_width + 1);
                        let exact = set_cell_size(&line, rule_width + 1);
                        segments.push(Segment::styled(&exact, rule_style));

                        segments.push(Segment::new(&self.end, None, None));
                    }
                    HorizontalAlign::Right => {
                        // Right: "rule_chars...  title"
                        let title_max_width = width.saturating_sub(min_side + 2);
                        if title_max_width == 0 || title_text.cell_len() == 0 {
                            let line_text = rule_with_chars.rule_line(width);
                            let exact = set_cell_size(&line_text, width);
                            segments.push(Segment::styled(&exact, rule_style));
                            segments.push(Segment::new(&self.end, None, None));
                            return segments;
                        }

                        title_text.truncate(title_max_width, Some(OverflowMethod::Ellipsis), false);

                        let title_width = title_text.cell_len();
                        let rule_width = width.saturating_sub(title_width + 2);

                        // Rule line + space
                        let line = rule_with_chars.rule_line(rule_width + 1);
                        let exact = set_cell_size(&line, rule_width + 1);
                        segments.push(Segment::styled(&exact, rule_style));

                        segments.push(Segment::new(" ", None, None));

                        // Title
                        segments.extend(title_text.render().into_iter().filter(|s| s.text != "\n"));

                        segments.push(Segment::new(&self.end, None, None));
                    }
                }
            }
        }

        segments
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Rule {
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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

    fn segments_to_text(segments: &[Segment]) -> String {
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    fn render_rule(console: &Console, rule: &Rule) -> String {
        let opts = console.options();
        let segments = rule.rich_console(console, &opts);
        segments_to_text(&segments)
    }

    // -- No title -----------------------------------------------------------

    #[test]
    fn test_no_title() {
        let console = make_console(20);
        let rule = Rule::new();
        let output = render_rule(&console, &rule);
        // Should be 20 cells of rule character plus newline
        let line = output.trim_end_matches('\n');
        assert_eq!(cell_len(line), 20);
    }

    #[test]
    fn test_no_title_custom_char() {
        let console = make_console(10);
        let rule = Rule::new().characters("-");
        let output = render_rule(&console, &rule);
        let line = output.trim_end_matches('\n');
        assert_eq!(line, "----------");
    }

    #[test]
    fn test_no_title_double_char() {
        let console = make_console(10);
        let rule = Rule::new().characters("=-");
        let output = render_rule(&console, &rule);
        let line = output.trim_end_matches('\n');
        assert_eq!(cell_len(line), 10);
        assert_eq!(line, &"=-=-=-=-=-=-="[..10]);
    }

    // -- Centered title -----------------------------------------------------

    #[test]
    fn test_centered_title() {
        let console = make_console(40);
        let rule = Rule::with_title("Title").characters("-");
        let output = render_rule(&console, &rule);
        let line = output.trim_end_matches('\n');
        assert_eq!(cell_len(line), 40);
        assert!(line.contains("Title"));
        assert!(line.contains(" Title "));
    }

    #[test]
    fn test_centered_title_has_rule_chars_both_sides() {
        let console = make_console(20);
        let rule = Rule::with_title("X").characters("-");
        let output = render_rule(&console, &rule);
        let line = output.trim_end_matches('\n');
        // Should have dashes on both sides of " X "
        assert!(line.contains('-'));
        let parts: Vec<&str> = line.split(" X ").collect();
        assert!(parts.len() >= 2);
        assert!(parts[0].contains('-'));
        assert!(parts[1].contains('-'));
    }

    // -- Left-aligned title -------------------------------------------------

    #[test]
    fn test_left_title() {
        let console = make_console(30);
        let rule = Rule::with_title("Left")
            .characters("-")
            .align(HorizontalAlign::Left);
        let output = render_rule(&console, &rule);
        let line = output.trim_end_matches('\n');
        assert_eq!(cell_len(line), 30);
        assert!(line.starts_with("Left"));
    }

    // -- Right-aligned title ------------------------------------------------

    #[test]
    fn test_right_title() {
        let console = make_console(30);
        let rule = Rule::with_title("Right")
            .characters("-")
            .align(HorizontalAlign::Right);
        let output = render_rule(&console, &rule);
        let line = output.trim_end_matches('\n');
        assert_eq!(cell_len(line), 30);
        assert!(line.ends_with("Right"));
    }

    // -- ASCII fallback -----------------------------------------------------

    #[test]
    fn test_ascii_fallback() {
        let console = Console::builder()
            .width(20)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build();
        let rule = Rule::new();
        let mut opts = console.options();
        opts.encoding = "ascii".to_string();
        let segments = rule.rich_console(&console, &opts);
        let output = segments_to_text(&segments);
        let line = output.trim_end_matches('\n');
        // All characters should be ASCII (dash)
        assert!(line.is_ascii());
        assert_eq!(cell_len(line), 20);
    }

    // -- Custom style -------------------------------------------------------

    #[test]
    fn test_custom_style() {
        let console = make_console(20);
        let custom_style = Style::parse("bold").unwrap();
        let rule = Rule::new().style(custom_style);
        let opts = console.options();
        let segments = rule.rich_console(&console, &opts);
        // The rule line segment should have a style
        let rule_segs: Vec<&Segment> = segments
            .iter()
            .filter(|s| s.text.trim().len() > 0 && s.text != "\n")
            .collect();
        assert!(!rule_segs.is_empty());
        for seg in &rule_segs {
            assert!(seg.style.is_some());
        }
    }

    // -- Default constructor ------------------------------------------------

    #[test]
    fn test_default() {
        let rule = Rule::default();
        assert!(rule.title.is_none());
        assert_eq!(rule.end, "\n");
        assert_eq!(rule.align, HorizontalAlign::Center);
    }

    // -- Builder pattern ----------------------------------------------------

    #[test]
    fn test_builder_chain() {
        let rule = Rule::new()
            .characters("=")
            .align(HorizontalAlign::Left)
            .end("")
            .style(Style::parse("bold").unwrap());
        assert_eq!(rule.characters, "=");
        assert_eq!(rule.align, HorizontalAlign::Left);
        assert_eq!(rule.end, "");
        assert!(rule.style.bold() == Some(true));
    }

    // -- rule_line helper ---------------------------------------------------

    #[test]
    fn test_rule_line_exact() {
        let rule = Rule::new().characters("-");
        let line = rule.rule_line(5);
        assert_eq!(line, "-----");
    }

    #[test]
    fn test_rule_line_multi_char() {
        let rule = Rule::new().characters("=-");
        let line = rule.rule_line(6);
        assert_eq!(line, "=-=-=-");
    }

    #[test]
    fn test_rule_line_remainder() {
        let rule = Rule::new().characters("=-");
        let line = rule.rule_line(5);
        // "=-" repeated 2 times = "=-=-" (4 cells), plus 1 cell remainder = "="
        assert_eq!(cell_len(&line), 5);
    }

    #[test]
    fn test_rule_line_zero_width() {
        let rule = Rule::new().characters("-");
        let line = rule.rule_line(0);
        assert_eq!(line, "");
    }

    // -- Title truncation ---------------------------------------------------

    #[test]
    fn test_title_truncation() {
        let console = make_console(10);
        let rule = Rule::with_title("This is a very long title").characters("-");
        let output = render_rule(&console, &rule);
        let line = output.trim_end_matches('\n');
        assert_eq!(cell_len(line), 10);
    }

    // -- Newline end --------------------------------------------------------

    #[test]
    fn test_ends_with_newline() {
        let console = make_console(20);
        let rule = Rule::new();
        let output = render_rule(&console, &rule);
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_no_end() {
        let console = make_console(20);
        let rule = Rule::new().end("");
        let output = render_rule(&console, &rule);
        assert!(!output.ends_with('\n'));
    }

    // -- with_title constructor ---------------------------------------------

    #[test]
    fn test_with_title() {
        let rule = Rule::with_title("Hello");
        assert!(rule.title.is_some());
        assert_eq!(rule.title.as_ref().unwrap().plain(), "Hello");
    }

    #[test]
    fn test_display_trait() {
        let rule = Rule::new();
        let s = format!("{}", rule);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_display_with_title() {
        let rule = Rule::with_title("Section");
        let s = format!("{}", rule);
        assert!(s.contains("Section"));
    }
}
