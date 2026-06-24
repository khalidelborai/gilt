//! Group widget -- renders a collection of renderables in sequence.
//!
//! The `Group` widget holds a list of `RenderableArc` items and renders them
//! one after another. It supports two modes:
//!
//! - **Default (`new`)**: fills the available width (measurement returns
//!   `max_width` for both minimum and maximum).
//! - **Fit (`fit`)**: constrains the width to the widest item in the group
//!   (measurement returns the combined measurement of all items).
//!

use std::fmt;
use std::sync::Arc;

use crate::console::{Console, ConsoleOptions, Renderable, RenderableArc};
use crate::measure::Measurement;
use crate::segment::Segment;

// ---------------------------------------------------------------------------
// Group
// ---------------------------------------------------------------------------

/// A group of renderables that are rendered in sequence.
///
/// When `fit` is `true`, the group's measurement is derived from its contents
/// so that it occupies only as much width as the widest item requires. When
/// `fit` is `false`, the group fills the entire available width.
///
/// Items can be any type implementing [`Renderable`]: `Text`, `Panel`, `Rule`,
/// `Table`, etc.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use gilt::group::Group;
/// use gilt::text::Text;
/// use gilt::style::Style;
///
/// let items: Vec<gilt::RenderableArc> = vec![
///     Arc::new(Text::new("Hello", Style::null())),
///     Arc::new(Text::new("World", Style::null())),
/// ];
/// let group = Group::new(items);
/// ```
#[derive(Clone)]
pub struct Group {
    /// The renderable items in this group.
    items: Vec<RenderableArc>,
    /// When `true`, constrain width to the widest item.
    /// When `false`, fill the available width.
    fit: bool,
}

// Manual Debug — RenderableArc (Arc<dyn Renderable + Send + Sync>) doesn't
// implement Debug, so we print a placeholder for each item.
impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Group")
            .field(
                "items",
                &self
                    .items
                    .iter()
                    .map(|_| "<renderable>")
                    .collect::<Vec<_>>(),
            )
            .field("fit", &self.fit)
            .finish()
    }
}

impl Group {
    /// Create a new `Group` from a vector of [`RenderableArc`] items.
    ///
    /// By default, `fit` is `false` -- the group fills the available width.
    /// Use [`Group::fit`] to create a group that constrains to content width.
    pub fn new(items: Vec<RenderableArc>) -> Self {
        Group { items, fit: false }
    }

    /// Create a new `Group` that constrains its width to the widest item.
    ///
    /// This is equivalent to `Group::new(items)` with `fit` set to `true`.
    pub fn fit(items: Vec<RenderableArc>) -> Self {
        Group { items, fit: true }
    }

    /// Push a single item into the group.
    ///
    /// Accepts any type that implements [`Renderable`]; wraps it in an [`Arc`]
    /// internally.
    pub fn push(&mut self, item: impl Renderable + Send + Sync + 'static) {
        self.items.push(Arc::new(item));
    }

    /// Return `true` if this group constrains width to content.
    pub fn is_fit(&self) -> bool {
        self.fit
    }

    /// Return a reference to the items in this group.
    pub fn items(&self) -> &[RenderableArc] {
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
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        if self.fit {
            self.measure_renderables(console, options)
        } else {
            Measurement::new(options.max_width, options.max_width)
        }
    }

    /// Compute the combined measurement of all items.
    ///
    /// The minimum width is the maximum of all individual minimums, and the
    /// maximum width is the maximum of all individual maximums, clamped to
    /// `options.max_width`.
    fn measure_renderables(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        if self.items.is_empty() {
            return Measurement::new(1, 1);
        }
        let mut min_width = 0usize;
        let mut max_width = 0usize;
        for item in &self.items {
            let m = item.gilt_measure(console, options);
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
    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }

    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let render_options = if self.fit {
            let measurement = self.measure_renderables(console, options);
            options.update_width(measurement.maximum.min(options.max_width))
        } else {
            options.clone()
        };

        let mut segments = Vec::new();
        for item in &self.items {
            segments.extend(item.as_ref().gilt_console(console, &render_options));
        }
        segments
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = f.width().unwrap_or(80);
        let mut console = Console::builder()
            .width(w)
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
    use crate::panel::Panel;
    use crate::rule::Rule;
    use crate::style::Style;
    use crate::text::Text;

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

    // Helper: wrap a Text into a RenderableArc
    fn text_arc(s: &str) -> RenderableArc {
        Arc::new(Text::new(s, Style::null()))
    }

    // -- NEW: heterogeneous widget tests ------------------------------------

    #[test]
    fn group_new_with_text_panel_rule_renders_all() {
        let console = make_console(80);
        let opts = console.options();

        let text = Text::new("plain text", Style::null());
        let panel = Panel::new(Text::new("panel content", Style::null()));
        let rule = Rule::new();

        let items: Vec<RenderableArc> = vec![Arc::new(text), Arc::new(panel), Arc::new(rule)];
        let group = Group::new(items);
        let segments = group.gilt_console(&console, &opts);
        let output = segments_text(&segments);

        // Text item renders
        assert!(
            output.contains("plain text"),
            "Text item not found in output"
        );
        // Panel item renders its content
        assert!(
            output.contains("panel content"),
            "Panel content not found in output"
        );
        // Rule item renders (produces non-empty output)
        assert!(
            !segments.is_empty(),
            "Group with Rule should produce segments"
        );
    }

    #[test]
    fn group_push_adds_widget() {
        let console = make_console(80);
        let opts = console.options();

        let mut group = Group::new(vec![Arc::new(Text::new("first", Style::null()))]);
        group.push(Text::new("second", Style::null()));
        assert_eq!(group.len(), 2);

        let segments = group.gilt_console(&console, &opts);
        let output = segments_text(&segments);
        assert!(output.contains("first"));
        assert!(output.contains("second"));
    }

    #[test]
    fn group_items_returns_renderable_arc_slice() {
        let items: Vec<RenderableArc> = vec![
            Arc::new(Text::new("a", Style::null())),
            Arc::new(Text::new("b", Style::null())),
        ];
        let group = Group::new(items);
        assert_eq!(group.items().len(), 2);
    }

    #[test]
    fn group_debug_impl() {
        let items: Vec<RenderableArc> = vec![Arc::new(Text::new("debug", Style::null()))];
        let group = Group::new(items);
        let debug_str = format!("{:?}", group);
        assert!(debug_str.contains("Group"));
        assert!(debug_str.contains("<renderable>"));
    }

    // -- Construction -------------------------------------------------------

    #[test]
    fn test_new_creates_non_fit_group() {
        let items = vec![text_arc("Hello"), text_arc("World")];
        let group = Group::new(items);
        assert!(!group.is_fit());
        assert_eq!(group.len(), 2);
    }

    #[test]
    fn test_fit_creates_fit_group() {
        let items = vec![text_arc("Hello"), text_arc("World")];
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
        let items = vec![text_arc("Alpha"), text_arc("Beta")];
        let group = Group::new(items);
        assert_eq!(group.items().len(), 2);
    }

    // -- Measure (non-fit) --------------------------------------------------

    #[test]
    fn test_measure_non_fit_fills_width() {
        let console = make_console(80);
        let opts = console.options();
        let items = vec![text_arc("Short"), text_arc("A bit longer text")];
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
            text_arc("Hi"),          // max=2
            text_arc("Hello World"), // max=11
            text_arc("Foo"),         // max=3
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
        let items = vec![text_arc("A very long line of text")];
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
        let items = vec![text_arc("First"), text_arc("Second"), text_arc("Third")];
        let group = Group::new(items);
        let segments = group.gilt_console(&console, &opts);
        let text = segments_text(&segments);
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
        assert!(text.contains("Third"));
    }

    #[test]
    fn test_render_preserves_order() {
        let console = make_console(80);
        let opts = console.options();
        let items = vec![text_arc("AAA"), text_arc("BBB"), text_arc("CCC")];
        let group = Group::new(items);
        let segments = group.gilt_console(&console, &opts);
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
        let segments = group.gilt_console(&console, &opts);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_render_single_item() {
        let console = make_console(80);
        let opts = console.options();
        let mut t = Text::new("Only one", Style::null());
        t.end = String::new();
        let group = Group::new(vec![Arc::new(t)]);
        let segments = group.gilt_console(&console, &opts);
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
        let items_fit = vec![text_arc("Short"), text_arc("Medium text")];
        let items_no_fit = vec![text_arc("Short"), text_arc("Medium text")];
        let group_fit = Group::fit(items_fit);
        let group_no_fit = Group::new(items_no_fit);

        let seg_fit = group_fit.gilt_console(&console, &opts);
        let seg_no_fit = group_no_fit.gilt_console(&console, &opts);

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
        let items = vec![text_arc("via console render")];
        let group = Group::new(items);
        let segments = console.render(&group, None);
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("via console render"));
    }

    // -- Clone and Debug ---------------------------------------------------

    #[test]
    fn test_clone() {
        let items = vec![text_arc("cloneable")];
        let group = Group::fit(items);
        let cloned = group.clone();
        assert_eq!(cloned.len(), group.len());
        assert_eq!(cloned.is_fit(), group.is_fit());
    }

    #[test]
    fn test_debug() {
        let items = vec![text_arc("debug")];
        let group = Group::new(items);
        let debug_str = format!("{:?}", group);
        assert!(debug_str.contains("Group"));
    }

    // -- Styled content preserved ------------------------------------------

    #[test]
    fn test_styled_content_preserved() {
        let console = make_console(80);
        let opts = console.options();
        let items: Vec<RenderableArc> = vec![
            Arc::new(Text::styled("Bold item", "bold")),
            Arc::new(Text::styled("Italic item", "italic")),
        ];
        let group = Group::new(items);
        let segments = group.gilt_console(&console, &opts);

        let has_bold = segments.iter().any(|s| {
            s.text.contains("Bold item")
                && s.style.as_ref().is_some_and(|st| st.bold() == Some(true))
        });
        let has_italic = segments.iter().any(|s| {
            s.text.contains("Italic item")
                && s.style.as_ref().is_some_and(|st| st.italic() == Some(true))
        });
        assert!(has_bold, "Expected bold segment in output");
        assert!(has_italic, "Expected italic segment in output");
    }

    // -- gilt_measure override -----------------------------------------------

    #[test]
    fn group_gilt_measure_non_fit_matches_standalone() {
        let console = make_console(80);
        let opts = console.options();
        let items = vec![text_arc("Short"), text_arc("A bit longer text")];
        let group = Group::new(items);
        let m_standalone = group.measure(&console, &opts);
        let m_trait = group.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Group::gilt_measure (non-fit) must delegate to Group::measure"
        );
    }

    #[test]
    fn group_gilt_measure_fit_matches_standalone() {
        let console = make_console(80);
        let opts = console.options();
        let items = vec![text_arc("Hi"), text_arc("Hello World")];
        let group = Group::fit(items);
        let m_standalone = group.measure(&console, &opts);
        let m_trait = group.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Group::gilt_measure (fit) must delegate to Group::measure"
        );
    }
}
