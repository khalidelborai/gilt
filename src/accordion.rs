//! Accordion widget -- collapsible content panels for organizing complex output.
//!
//! Port of Python's `rich` collapsible panels concept.

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Accordion
// ---------------------------------------------------------------------------

/// A collapsible accordion section.
///
/// Displays a title with an expand/collapse icon. When expanded, the content
/// is shown below the title with indentation. When collapsed, only the title
/// is displayed.
///
/// # Examples
///
/// ```
/// use gilt::prelude::*;
/// use gilt::accordion::Accordion;
///
/// // Create an expanded accordion
/// let accordion = Accordion::new(
///     "Section Title",
///     Text::new("This is the content that will be shown when expanded.", Style::null())
/// );
///
/// // Create a collapsed accordion
/// let collapsed = Accordion::new(
///     "Hidden Section",
///     Text::new("You can't see this until expanded.", Style::null())
/// ).collapsed(true);
/// ```
#[derive(Debug, Clone)]
pub struct Accordion {
    /// The title displayed on the accordion header.
    pub title: String,
    /// The content to display when expanded.
    pub content: Text,
    /// Whether the accordion is currently collapsed.
    pub collapsed: bool,
    /// Style applied to the entire accordion.
    pub style: Style,
    /// Style applied to the title.
    pub title_style: Style,
    /// Style applied to the expand/collapse icons.
    pub icon_style: Style,
    /// Icon shown when collapsed (default: "▶").
    pub expand_icon: String,
    /// Icon shown when expanded (default: "▼").
    pub collapse_icon: String,
    /// Number of spaces to indent content when expanded.
    pub indent: usize,
}

impl Accordion {
    /// Create a new accordion with the given title and content.
    ///
    /// The accordion is initially expanded (collapsed = false).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let accordion = Accordion::new(
    ///     "My Section",
    ///     Text::new("Content here", Style::null())
    /// );
    /// ```
    pub fn new(title: impl Into<String>, content: impl Into<Text>) -> Self {
        Accordion {
            title: title.into(),
            content: content.into(),
            collapsed: false,
            style: Style::null(),
            title_style: Style::null(),
            icon_style: Style::null(),
            expand_icon: "▶".to_string(),
            collapse_icon: "▼".to_string(),
            indent: 2,
        }
    }

    /// Builder: set the initial collapsed state.
    ///
    /// Default is `false` (expanded).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let accordion = Accordion::new(
    ///     "Collapsed",
    ///     Text::new("Hidden content", Style::null())
    /// ).collapsed(true);
    /// ```
    #[must_use]
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Toggle the collapsed state.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let mut accordion = Accordion::new(
    ///     "Section",
    ///     Text::new("Content", Style::null())
    /// );
    ///
    /// accordion.toggle(); // Now collapsed
    /// accordion.toggle(); // Now expanded again
    /// ```
    pub fn toggle(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Expand the accordion (set collapsed to false).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let mut accordion = Accordion::new(
    ///     "Section",
    ///     Text::new("Content", Style::null())
    /// ).collapsed(true);
    ///
    /// accordion.expand();
    /// assert!(!accordion.is_collapsed());
    /// ```
    pub fn expand(&mut self) {
        self.collapsed = false;
    }

    /// Collapse the accordion (set collapsed to true).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let mut accordion = Accordion::new(
    ///     "Section",
    ///     Text::new("Content", Style::null())
    /// );
    ///
    /// accordion.collapse();
    /// assert!(accordion.is_collapsed());
    /// ```
    pub fn collapse(&mut self) {
        self.collapsed = true;
    }

    /// Check if the accordion is currently collapsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let accordion = Accordion::new(
    ///     "Section",
    ///     Text::new("Content", Style::null())
    /// );
    ///
    /// assert!(!accordion.is_collapsed()); // Default is expanded
    /// ```
    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    /// Builder: set the base style for the accordion.
    ///
    /// This style is applied to all content.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let accordion = Accordion::new(
    ///     "Section",
    ///     Text::new("Content", Style::null())
    /// ).style(Style::parse("dim").unwrap());
    /// ```
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Builder: set the title style.
    ///
    /// This style is applied to the title text (in addition to the base style).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let accordion = Accordion::new(
    ///     "Important",
    ///     Text::new("Content", Style::null())
    /// ).title_style(Style::parse("bold yellow").unwrap());
    /// ```
    #[must_use]
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Builder: set the icon style.
    ///
    /// This style is applied to the expand/collapse icons.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let accordion = Accordion::new(
    ///     "Section",
    ///     Text::new("Content", Style::null())
    /// ).icon_style(Style::parse("cyan").unwrap());
    /// ```
    #[must_use]
    pub fn icon_style(mut self, style: Style) -> Self {
        self.icon_style = style;
        self
    }

    /// Builder: set custom expand/collapse icons.
    ///
    /// Default icons are "▶" (collapsed) and "▼" (expanded).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// // Use +/− symbols instead
    /// let accordion = Accordion::new(
    ///     "Section",
    ///     Text::new("Content", Style::null())
    /// ).icons("+", "−");
    ///
    /// // Use arrow variants
    /// let accordion2 = Accordion::new(
    ///     "Section",
    ///     Text::new("Content", Style::null())
    /// ).icons("►", "▼");
    /// ```
    #[must_use]
    pub fn icons(mut self, expand: impl Into<String>, collapse: impl Into<String>) -> Self {
        self.expand_icon = expand.into();
        self.collapse_icon = collapse.into();
        self
    }

    /// Builder: set the content indentation when expanded.
    ///
    /// Default is 2 spaces.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::Accordion;
    ///
    /// let accordion = Accordion::new(
    ///     "Section",
    ///     Text::new("Content", Style::null())
    /// ).indent(4);
    /// ```
    #[must_use]
    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Get the current icon based on collapsed state.
    fn current_icon(&self) -> &str {
        if self.collapsed {
            &self.expand_icon
        } else {
            &self.collapse_icon
        }
    }
}

// ---------------------------------------------------------------------------
// AccordionGroup
// ---------------------------------------------------------------------------

/// A group of accordions with optional mutual exclusion.
///
/// When `allow_multiple_open` is `false` (the default), expanding one accordion
/// will automatically collapse all others in the group.
///
/// # Examples
///
/// ```
/// use gilt::prelude::*;
/// use gilt::accordion::{Accordion, AccordionGroup};
///
/// let group = AccordionGroup::new(vec![
///     Accordion::new("First", Text::new("Content 1", Style::null())),
///     Accordion::new("Second", Text::new("Content 2", Style::null())),
///     Accordion::new("Third", Text::new("Content 3", Style::null())),
/// ]);
///
/// // Only one accordion can be open at a time
/// let group = AccordionGroup::new(vec![
///     Accordion::new("A", Text::new("...", Style::null())).collapsed(false),
///     Accordion::new("B", Text::new("...", Style::null())).collapsed(true),
/// ])
/// .allow_multiple_open(false);
/// ```
#[derive(Debug, Clone)]
pub struct AccordionGroup {
    /// The accordions in this group.
    pub items: Vec<Accordion>,
    /// Whether multiple accordions can be open simultaneously.
    pub allow_multiple_open: bool,
}

impl AccordionGroup {
    /// Create a new accordion group from a vector of accordions.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::{Accordion, AccordionGroup};
    ///
    /// let group = AccordionGroup::new(vec![
    ///     Accordion::new("Section 1", Text::new("Content 1", Style::null())),
    ///     Accordion::new("Section 2", Text::new("Content 2", Style::null())),
    /// ]);
    /// ```
    pub fn new(items: Vec<Accordion>) -> Self {
        AccordionGroup {
            items,
            allow_multiple_open: true,
        }
    }

    /// Builder: set whether multiple accordions can be open at once.
    ///
    /// Default is `true`. When set to `false`, expanding one accordion
    /// will collapse all others.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::{Accordion, AccordionGroup};
    ///
    /// let group = AccordionGroup::new(vec![
    ///     Accordion::new("A", Text::new("...", Style::null())),
    ///     Accordion::new("B", Text::new("...", Style::null())),
    /// ])
    /// .allow_multiple_open(false);
    /// ```
    #[must_use]
    pub fn allow_multiple_open(mut self, allow: bool) -> Self {
        self.allow_multiple_open = allow;
        self
    }

    /// Expand all accordions in the group.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::{Accordion, AccordionGroup};
    ///
    /// let mut group = AccordionGroup::new(vec![
    ///     Accordion::new("A", Text::new("...", Style::null())).collapsed(true),
    ///     Accordion::new("B", Text::new("...", Style::null())).collapsed(true),
    /// ]);
    ///
    /// group.expand_all();
    /// assert!(!group.items[0].is_collapsed());
    /// assert!(!group.items[1].is_collapsed());
    /// ```
    pub fn expand_all(&mut self) {
        for item in &mut self.items {
            item.expand();
        }
    }

    /// Collapse all accordions in the group.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::{Accordion, AccordionGroup};
    ///
    /// let mut group = AccordionGroup::new(vec![
    ///     Accordion::new("A", Text::new("...", Style::null())),
    ///     Accordion::new("B", Text::new("...", Style::null())),
    /// ]);
    ///
    /// group.collapse_all();
    /// assert!(group.items[0].is_collapsed());
    /// assert!(group.items[1].is_collapsed());
    /// ```
    pub fn collapse_all(&mut self) {
        for item in &mut self.items {
            item.collapse();
        }
    }

    /// Add an accordion to the group.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::{Accordion, AccordionGroup};
    ///
    /// let mut group = AccordionGroup::new(vec![]);
    ///
    /// group.push(Accordion::new("Section", Text::new("Content", Style::null())));
    /// ```
    pub fn push(&mut self, accordion: Accordion) {
        self.items.push(accordion);
    }

    /// Expand a specific accordion by index, optionally collapsing others.
    ///
    /// If `allow_multiple_open` is false, this will collapse all other accordions.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::prelude::*;
    /// use gilt::accordion::{Accordion, AccordionGroup};
    ///
    /// let mut group = AccordionGroup::new(vec![
    ///     Accordion::new("A", Text::new("...", Style::null())).collapsed(true),
    ///     Accordion::new("B", Text::new("...", Style::null())).collapsed(true),
    /// ])
    /// .allow_multiple_open(false);
    ///
    /// group.expand_item(1);
    /// assert!(group.items[0].is_collapsed()); // First is collapsed
    /// assert!(!group.items[1].is_collapsed()); // Second is expanded
    /// ```
    pub fn expand_item(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }

        if !self.allow_multiple_open {
            for (i, item) in self.items.iter_mut().enumerate() {
                if i == index {
                    item.expand();
                } else {
                    item.collapse();
                }
            }
        } else {
            self.items[index].expand();
        }
    }
}

// ---------------------------------------------------------------------------
// Renderable implementations
// ---------------------------------------------------------------------------

impl Renderable for Accordion {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();

        // Calculate available width for content
        let max_width = options.max_width;
        let icon_width = self.current_icon().chars().count() + 1; // icon + space
        let content_width = max_width.saturating_sub(icon_width.max(self.indent));

        // Build header line: icon + space + title
        let icon = self.current_icon();

        // Icon segment
        if !self.icon_style.is_null() {
            segments.push(Segment::styled(icon, self.icon_style.clone()));
        } else {
            segments.push(Segment::text(icon));
        }

        // Space after icon
        segments.push(Segment::text(" "));

        // Title segment
        let title_text = if !self.title_style.is_null() {
            Segment::styled(&self.title, self.title_style.clone())
        } else if !self.style.is_null() {
            Segment::styled(&self.title, self.style.clone())
        } else {
            Segment::text(&self.title)
        };
        segments.push(title_text);

        // End of header line
        segments.push(Segment::line());

        // If expanded, render content with indentation
        if !self.collapsed {
            // Create a copy of content for rendering
            let mut content = self.content.clone();
            content.end = String::new(); // We'll handle newlines ourselves

            // Render the content text
            let content_opts = options.update_width(content_width);
            let content_segments = content.gilt_console(console, &content_opts);

            // Split content into lines and add indentation
            let lines = Segment::split_lines(&content_segments);
            let indent_str = " ".repeat(self.indent);

            for line in lines {
                // Add indentation at the start of each line
                if !line.is_empty() {
                    segments.push(Segment::text(&indent_str));
                    segments.extend(line);
                } else {
                    // Empty line - just add newline
                    segments.push(Segment::line());
                }
            }
        }

        segments
    }
}

impl Renderable for AccordionGroup {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();

        for (i, item) in self.items.iter().enumerate() {
            // Render each accordion
            let item_segments = item.gilt_console(console, options);
            segments.extend(item_segments);

            // Add a blank line between accordions (except after the last one)
            if i < self.items.len() - 1 {
                segments.push(Segment::line());
            }
        }

        segments
    }
}

// ---------------------------------------------------------------------------
// Display trait implementations
// ---------------------------------------------------------------------------

impl std::fmt::Display for Accordion {
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

impl std::fmt::Display for AccordionGroup {
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

    fn render_accordion(console: &Console, accordion: &Accordion) -> String {
        let opts = console.options();
        let segments = accordion.gilt_console(console, &opts);
        segments_to_text(&segments)
    }

    // ── 1. Basic accordion rendering ─────────────────────────────────────────

    #[test]
    fn test_accordion_expanded() {
        let console = make_console(80);
        let accordion = Accordion::new(
            "Section Title",
            Text::new("Content line 1\nContent line 2", Style::null()),
        );
        let output = render_accordion(&console, &accordion);

        // Should show icon, title, and content
        assert!(output.contains("▼"));
        assert!(output.contains("Section Title"));
        assert!(output.contains("Content line 1"));
        assert!(output.contains("Content line 2"));
    }

    #[test]
    fn test_accordion_collapsed() {
        let console = make_console(80);
        let accordion = Accordion::new(
            "Section Title",
            Text::new("Content line 1\nContent line 2", Style::null()),
        )
        .collapsed(true);
        let output = render_accordion(&console, &accordion);

        // Should show icon and title only (collapsed state)
        assert!(output.contains("▶"));
        assert!(output.contains("Section Title"));
        // Content should NOT be visible when collapsed
        assert!(!output.contains("Content line 1"));
        assert!(!output.contains("Content line 2"));
    }

    // ── 2. Toggle functionality ─────────────────────────────────────────────

    #[test]
    fn test_toggle() {
        let mut accordion = Accordion::new("Test", Text::new("Content", Style::null()));

        // Default is expanded
        assert!(!accordion.is_collapsed());

        // Toggle to collapsed
        accordion.toggle();
        assert!(accordion.is_collapsed());

        // Toggle back to expanded
        accordion.toggle();
        assert!(!accordion.is_collapsed());
    }

    #[test]
    fn test_expand_collapse() {
        let mut accordion = Accordion::new("Test", Text::new("Content", Style::null()));

        accordion.collapse();
        assert!(accordion.is_collapsed());

        accordion.expand();
        assert!(!accordion.is_collapsed());
    }

    // ── 3. Custom icons ─────────────────────────────────────────────────────

    #[test]
    fn test_custom_icons() {
        let accordion = Accordion::new("Test", Text::new("Content", Style::null())).icons("+", "−");

        assert_eq!(accordion.expand_icon, "+");
        assert_eq!(accordion.collapse_icon, "−");

        // When expanded, should show collapse icon
        assert_eq!(accordion.current_icon(), "−");

        let collapsed = Accordion::new("Test", Text::new("Content", Style::null()))
            .icons("+", "−")
            .collapsed(true);

        // When collapsed, should show expand icon
        assert_eq!(collapsed.current_icon(), "+");
    }

    #[test]
    fn test_custom_icons_rendering() {
        let console = make_console(80);

        let expanded = Accordion::new("Test", Text::new("Content", Style::null())).icons("►", "▼");
        let output = render_accordion(&console, &expanded);
        assert!(output.contains("▼"));

        let collapsed = Accordion::new("Test", Text::new("Content", Style::null()))
            .icons("►", "▼")
            .collapsed(true);
        let output = render_accordion(&console, &collapsed);
        assert!(output.contains("►"));
    }

    // ── 4. Indentation ──────────────────────────────────────────────────────

    #[test]
    fn test_indentation() {
        let console = make_console(80);
        let accordion =
            Accordion::new("Title", Text::new("Line 1\nLine 2", Style::null())).indent(4);
        let output = render_accordion(&console, &accordion);

        // Check that content is indented
        let lines: Vec<&str> = output.lines().collect();
        // First line is header, second line should have 4-space indent
        assert!(lines[1].starts_with("    Line 1"));
    }

    // ── 5. AccordionGroup ───────────────────────────────────────────────────

    #[test]
    fn test_accordion_group_new() {
        let group = AccordionGroup::new(vec![
            Accordion::new("A", Text::new("Content A", Style::null())),
            Accordion::new("B", Text::new("Content B", Style::null())),
        ]);

        assert_eq!(group.items.len(), 2);
        assert!(group.allow_multiple_open);
    }

    #[test]
    fn test_accordion_group_allow_multiple_open() {
        let group = AccordionGroup::new(vec![]).allow_multiple_open(false);

        assert!(!group.allow_multiple_open);
    }

    #[test]
    fn test_accordion_group_expand_all() {
        let mut group = AccordionGroup::new(vec![
            Accordion::new("A", Text::new("...", Style::null())).collapsed(true),
            Accordion::new("B", Text::new("...", Style::null())).collapsed(true),
        ]);

        group.expand_all();

        assert!(!group.items[0].is_collapsed());
        assert!(!group.items[1].is_collapsed());
    }

    #[test]
    fn test_accordion_group_collapse_all() {
        let mut group = AccordionGroup::new(vec![
            Accordion::new("A", Text::new("...", Style::null())),
            Accordion::new("B", Text::new("...", Style::null())),
        ]);

        group.collapse_all();

        assert!(group.items[0].is_collapsed());
        assert!(group.items[1].is_collapsed());
    }

    #[test]
    fn test_accordion_group_push() {
        let mut group = AccordionGroup::new(vec![]);
        group.push(Accordion::new("New", Text::new("Content", Style::null())));

        assert_eq!(group.items.len(), 1);
    }

    #[test]
    fn test_accordion_group_expand_item_mutual_exclusion() {
        let mut group = AccordionGroup::new(vec![
            Accordion::new("A", Text::new("...", Style::null())),
            Accordion::new("B", Text::new("...", Style::null())),
            Accordion::new("C", Text::new("...", Style::null())),
        ])
        .allow_multiple_open(false);

        // Initially all expanded
        assert!(!group.items[0].is_collapsed());
        assert!(!group.items[1].is_collapsed());
        assert!(!group.items[2].is_collapsed());

        // Expand item 1 - should collapse others
        group.expand_item(1);

        assert!(group.items[0].is_collapsed());
        assert!(!group.items[1].is_collapsed());
        assert!(group.items[2].is_collapsed());
    }

    #[test]
    fn test_accordion_group_render() {
        let console = make_console(80);
        let group = AccordionGroup::new(vec![
            Accordion::new("First", Text::new("Content 1", Style::null())),
            Accordion::new("Second", Text::new("Content 2", Style::null())),
        ]);

        let opts = console.options();
        let segments = group.gilt_console(&console, &opts);
        let output = segments_to_text(&segments);

        // Should contain both titles and content
        assert!(output.contains("First"));
        assert!(output.contains("Second"));
        assert!(output.contains("Content 1"));
        assert!(output.contains("Content 2"));
    }

    // ── 6. Builder pattern chain ────────────────────────────────────────────

    #[test]
    fn test_builder_chain() {
        let accordion = Accordion::new("Title", Text::new("Content", Style::null()))
            .collapsed(true)
            .style(Style::parse("dim").unwrap())
            .title_style(Style::parse("bold").unwrap())
            .icon_style(Style::parse("cyan").unwrap())
            .icons("+", "−")
            .indent(4);

        assert!(accordion.collapsed);
        assert!(!accordion.style.is_null());
        assert!(!accordion.title_style.is_null());
        assert!(!accordion.icon_style.is_null());
        assert_eq!(accordion.expand_icon, "+");
        assert_eq!(accordion.collapse_icon, "−");
        assert_eq!(accordion.indent, 4);
    }

    // ── 7. Display trait ────────────────────────────────────────────────────

    #[test]
    fn test_accordion_display() {
        let accordion = Accordion::new("Title", Text::new("Content", Style::null()));
        let output = format!("{}", accordion);

        assert!(output.contains("Title"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn test_accordion_group_display() {
        let group = AccordionGroup::new(vec![Accordion::new(
            "A",
            Text::new("Content A", Style::null()),
        )]);
        let output = format!("{}", group);

        assert!(output.contains("A"));
        assert!(output.contains("Content A"));
    }

    // ── 8. Edge cases ───────────────────────────────────────────────────────

    #[test]
    fn test_empty_content() {
        let console = make_console(80);
        let accordion = Accordion::new("Title", Text::new("", Style::null()));
        let output = render_accordion(&console, &accordion);

        // Should still render header
        assert!(output.contains("Title"));
        assert!(output.contains("▼"));
    }

    #[test]
    fn test_empty_title() {
        let console = make_console(80);
        let accordion = Accordion::new("", Text::new("Content", Style::null()));
        let output = render_accordion(&console, &accordion);

        // Should still render icon and content
        assert!(output.contains("▼"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn test_long_content_wrapping() {
        let console = make_console(40);
        let long_text =
            "This is a very long line that should wrap when rendered in a narrow console width.";
        let accordion = Accordion::new("Title", Text::new(long_text, Style::null()));
        let output = render_accordion(&console, &accordion);

        // Content should be wrapped (multiple lines)
        let line_count = output.lines().count();
        // Header (▼ Title) + at least one content line (may wrap to 2+ lines)
        assert!(
            line_count >= 2,
            "Expected at least 2 lines, got {}",
            line_count
        );
    }

    #[test]
    fn test_expand_item_out_of_bounds() {
        let mut group =
            AccordionGroup::new(vec![Accordion::new("A", Text::new("...", Style::null()))]);

        // Should not panic with out-of-bounds index
        group.expand_item(10);

        // Original item should remain unchanged
        assert!(!group.items[0].is_collapsed());
    }

    #[test]
    fn test_accordion_group_empty() {
        let console = make_console(80);
        let group = AccordionGroup::new(vec![]);

        let opts = console.options();
        let segments = group.gilt_console(&console, &opts);

        // Empty group should produce no segments
        assert!(segments.is_empty());
    }
}
