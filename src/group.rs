//! Group widget -- renders a collection of renderables in sequence.
//!
//! The `Group` widget holds a list of `Text` renderables and renders them one
//! after another. It supports two modes:
//!
//! - **Default (`new`)**: fills the available width (measurement returns
//!   `max_width` for both minimum and maximum).
//! - **Fit (`fit`)**: constrains the width to the widest item in the group
//!   (measurement returns the combined measurement of all items).
//!
//! Rust port of Python's `rich.console.Group`.

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Group
// ---------------------------------------------------------------------------

/// A group of renderables that are rendered in sequence.
///
/// When `fit` is `true` (the default, matching Python rich), the group's
/// measurement is derived from its contents so that it occupies only as much
/// width as the widest item requires. When `fit` is `false`, the group fills
/// the entire available width.
///
/// # Examples
///
/// ```
/// use gilt::group::Group;
/// use gilt::text::Text;
/// use gilt::style::Style;
///
/// let items = vec![
///     Text::new("Hello", Style::null()),
///     Text::new("World", Style::null()),
/// ];
/// let group = Group::new(items);
/// ```
#[derive(Debug, Clone)]
pub struct Group {
    /// The renderable items in this group.
    items: Vec<Text>,
    /// When `true`, constrain width to the widest item.
    /// When `false`, fill the available width.
    fit: bool,
}

impl Group {
    /// Create a new `Group` from a vector of `Text` items.
    ///
    /// By default, `fit` is `false` -- the group fills the available width.
    /// Use [`Group::fit`] to create a group that constrains to content width.
    pub fn new(items: Vec<Text>) -> Self {
        Group { items, fit: false }
    }

    /// Create a new `Group` that constrains its width to the widest item.
    ///
    /// This is equivalent to `Group::new(items)` with `fit` set to `true`,
    /// matching Python rich's `Group(*renderables, fit=True)`.
    pub fn fit(items: Vec<Text>) -> Self {
        Group { items, fit: true }
    }

    /// Return `true` if this group constrains width to content.
    pub fn is_fit(&self) -> bool {
        self.fit
    }

    /// Return a reference to the items in this group.
    pub fn items(&self) -> &[Text] {
        &self.items
    }

    /// Return the number of items in this group.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Return `true` if this group has no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Measure the group.
    ///
    /// When `fit` is `true`, the measurement is the combined measurement of all
    /// items (maximum of each item's min and max). When `fit` is `false`, both
    /// minimum and maximum are set to `options.max_width` (fill available space).
    pub fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        if self.fit {
            self.measure_renderables(options)
        } else {
            Measurement::new(options.max_width, options.max_width)
        }
    }

    /// Compute the combined measurement of all items.
    ///
    /// The minimum width is the maximum of all individual minimums, and the
    /// maximum width is the maximum of all individual maximums, clamped to
    /// `options.max_width`.
    fn measure_renderables(&self, options: &ConsoleOptions) -> Measurement {
        if self.items.is_empty() {
            return Measurement::new(1, 1);
        }
        let mut min_width = 0usize;
        let mut max_width = 0usize;
        for item in &self.items {
            let m = item.measure();
            min_width = min_width.max(m.minimum);
            max_width = max_width.max(m.maximum);
        }
        Measurement::new(
            min_width.min(options.max_width),
            max_width.min(options.max_width),
        )
    }
}

impl Renderable for Group {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let render_options = if self.fit {
            let measurement = self.measure_renderables(options);
            options.update_width(measurement.maximum.min(options.max_width))
        } else {
            options.clone()
        };

        let mut segments = Vec::new();
        for item in &self.items {
            segments.extend(item.rich_console(console, &render_options));
        }
        segments
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    fn make_console(width: usize) -> Console {
        Console::builder()
            .width(width)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build()
    }

    fn segments_text(segments: &[Segment]) -> String {
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    // -- Construction -------------------------------------------------------

    #[test]
    fn test_new_creates_non_fit_group() {
        let items = vec![
            Text::new("Hello", Style::null()),
            Text::new("World", Style::null()),
        ];
        let group = Group::new(items);
        assert!(!group.is_fit());
        assert_eq!(group.len(), 2);
    }

    #[test]
    fn test_fit_creates_fit_group() {
        let items = vec![
            Text::new("Hello", Style::null()),
            Text::new("World", Style::null()),
        ];
        let group = Group::fit(items);
        assert!(group.is_fit());
        assert_eq!(group.len(), 2);
    }

    #[test]
    fn test_empty_group() {
        let group = Group::new(vec![]);
        assert!(group.is_empty());
        assert_eq!(group.len(), 0);
    }

    #[test]
    fn test_items_accessor() {
        let items = vec![
            Text::new("Alpha", Style::null()),
            Text::new("Beta", Style::null()),
        ];
        let group = Group::new(items);
        assert_eq!(group.items().len(), 2);
        assert_eq!(group.items()[0].plain(), "Alpha");
        assert_eq!(group.items()[1].plain(), "Beta");
    }

    // -- Measure (non-fit) --------------------------------------------------

    #[test]
    fn test_measure_non_fit_fills_width() {
        let console = make_console(80);
        let opts = console.options();
        let items = vec![
            Text::new("Short", Style::null()),
            Text::new("A bit longer text", Style::null()),
        ];
        let group = Group::new(items);
        let m = group.measure(&console, &opts);
        // Non-fit group fills available width
        assert_eq!(m.minimum, 80);
        assert_eq!(m.maximum, 80);
    }

    // -- Measure (fit) ------------------------------------------------------

    #[test]
    fn test_measure_fit_matches_content() {
        let console = make_console(80);
        let opts = console.options();
        let items = vec![
            Text::new("Hi", Style::null()),          // max=2
            Text::new("Hello World", Style::null()), // max=11
            Text::new("Foo", Style::null()),         // max=3
        ];
        let group = Group::fit(items);
        let m = group.measure(&console, &opts);
        // max should be the widest item = 11
        assert_eq!(m.maximum, 11);
        // min should be the longest word = 5 ("Hello" or "World")
        assert_eq!(m.minimum, 5);
    }

    #[test]
    fn test_measure_fit_empty() {
        let console = make_console(80);
        let opts = console.options();
        let group = Group::fit(vec![]);
        let m = group.measure(&console, &opts);
        assert_eq!(m, Measurement::new(1, 1));
    }

    #[test]
    fn test_measure_fit_clamped_to_max_width() {
        let console = make_console(5);
        let opts = console.options();
        let items = vec![Text::new("A very long line of text", Style::null())];
        let group = Group::fit(items);
        let m = group.measure(&console, &opts);
        // Should be clamped to console width of 5
        assert!(m.maximum <= 5);
    }

    // -- Rendering ----------------------------------------------------------

    #[test]
    fn test_render_contains_all_items() {
        let console = make_console(80);
        let opts = console.options();
        let items = vec![
            Text::new("First", Style::null()),
            Text::new("Second", Style::null()),
            Text::new("Third", Style::null()),
        ];
        let group = Group::new(items);
        let segments = group.rich_console(&console, &opts);
        let text = segments_text(&segments);
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
        assert!(text.contains("Third"));
    }

    #[test]
    fn test_render_preserves_order() {
        let console = make_console(80);
        let opts = console.options();
        let items = vec![
            Text::new("AAA", Style::null()),
            Text::new("BBB", Style::null()),
            Text::new("CCC", Style::null()),
        ];
        let group = Group::new(items);
        let segments = group.rich_console(&console, &opts);
        let text = segments_text(&segments);
        let pos_a = text.find("AAA").unwrap();
        let pos_b = text.find("BBB").unwrap();
        let pos_c = text.find("CCC").unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_render_empty_group() {
        let console = make_console(80);
        let opts = console.options();
        let group = Group::new(vec![]);
        let segments = group.rich_console(&console, &opts);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_render_single_item() {
        let console = make_console(80);
        let opts = console.options();
        let mut t = Text::new("Only one", Style::null());
        t.end = String::new();
        let group = Group::new(vec![t]);
        let segments = group.rich_console(&console, &opts);
        let text = segments_text(&segments);
        assert!(text.contains("Only one"));
    }

    // -- Fit rendering constrains width ------------------------------------

    #[test]
    fn test_fit_rendering_constrains_width() {
        // Console is 80 wide, but items are narrow.
        // With fit=true, the render width should be constrained.
        let console = make_console(80);
        let opts = console.options();
        let items = vec![
            Text::new("Short", Style::null()),
            Text::new("Medium text", Style::null()),
        ];
        let group_fit = Group::fit(items.clone());
        let group_no_fit = Group::new(items);

        let seg_fit = group_fit.rich_console(&console, &opts);
        let seg_no_fit = group_no_fit.rich_console(&console, &opts);

        // Both should contain the same text content
        let text_fit = segments_text(&seg_fit);
        let text_no_fit = segments_text(&seg_no_fit);
        assert!(text_fit.contains("Short"));
        assert!(text_fit.contains("Medium text"));
        assert!(text_no_fit.contains("Short"));
        assert!(text_no_fit.contains("Medium text"));
    }

    // -- Console integration -----------------------------------------------

    #[test]
    fn test_console_render_integration() {
        let console = make_console(80);
        let items = vec![Text::new("via console render", Style::null())];
        let group = Group::new(items);
        let segments = console.render(&group, None);
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("via console render"));
    }

    // -- Clone and Debug ---------------------------------------------------

    #[test]
    fn test_clone() {
        let items = vec![Text::new("cloneable", Style::null())];
        let group = Group::fit(items);
        let cloned = group.clone();
        assert_eq!(cloned.len(), group.len());
        assert_eq!(cloned.is_fit(), group.is_fit());
        assert_eq!(cloned.items()[0].plain(), "cloneable");
    }

    #[test]
    fn test_debug() {
        let items = vec![Text::new("debug", Style::null())];
        let group = Group::new(items);
        let debug_str = format!("{:?}", group);
        assert!(debug_str.contains("Group"));
    }

    // -- Styled content preserved ------------------------------------------

    #[test]
    fn test_styled_content_preserved() {
        let console = make_console(80);
        let opts = console.options();
        let items = vec![
            Text::styled("Bold item", Style::parse("bold").unwrap()),
            Text::styled("Italic item", Style::parse("italic").unwrap()),
        ];
        let group = Group::new(items);
        let segments = group.rich_console(&console, &opts);

        let has_bold = segments.iter().any(|s| {
            s.text.contains("Bold item")
                && s.style.as_ref().map_or(false, |st| st.bold() == Some(true))
        });
        let has_italic = segments.iter().any(|s| {
            s.text.contains("Italic item")
                && s.style
                    .as_ref()
                    .map_or(false, |st| st.italic() == Some(true))
        });
        assert!(has_bold, "Expected bold segment in output");
        assert!(has_italic, "Expected italic segment in output");
    }
}
