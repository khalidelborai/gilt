//! Container types for grouping renderables.
//!
//! This module provides `Renderables`, a container that renders multiple items
//! in sequence, and implements the `Renderable` trait for `Lines` so that
//! collections of `Text` objects can be rendered to the console.
//!

use std::sync::Arc;

use crate::console::{Console, ConsoleOptions, Renderable, RenderableArc};
use crate::measure::Measurement;
use crate::segment::Segment;

// ---------------------------------------------------------------------------
// Renderables
// ---------------------------------------------------------------------------

/// A container of renderable items that renders them in sequence.
///
/// This is the Rust equivalent of rich's `Renderables`. Items can be any type
/// implementing [`Renderable`]: `Text`, `Table`, `Panel`, `Tree`, etc.
#[derive(Clone, Default)]
pub struct Renderables {
    items: Vec<RenderableArc>,
}

// Manual Debug — RenderableArc (Arc<dyn Renderable + Send + Sync>) doesn't
// implement Debug, so we print a placeholder for each item.
impl std::fmt::Debug for Renderables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderables")
            .field(
                "items",
                &self
                    .items
                    .iter()
                    .map(|_| "<renderable>")
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Renderables {
    /// Create a new `Renderables` from a vector of [`RenderableArc`] items.
    pub fn new(items: Vec<RenderableArc>) -> Self {
        Renderables { items }
    }

    /// Append any [`Renderable`] item to the container.
    ///
    /// The value is wrapped in an [`Arc`] internally.
    pub fn append(&mut self, item: impl Renderable + Send + Sync + 'static) {
        self.items.push(Arc::new(item));
    }

    /// Return the number of items in the container.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Return `true` if the container has no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Measure the renderables, returning the combined `Measurement`.
    ///
    /// The minimum width is the maximum of all individual minimums,
    /// and the maximum width is the maximum of all individual maximums.
    /// If there are no items, returns `Measurement(1, 1)` (matching Python rich).
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
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
        Measurement::new(min_width, max_width)
    }
}

impl Renderable for Renderables {
    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }

    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();
        for item in &self.items {
            segments.extend(item.as_ref().gilt_console(console, options));
        }
        segments
    }
}

// ---------------------------------------------------------------------------
// Renderable implementation for Lines
// ---------------------------------------------------------------------------

use crate::text::Lines;

impl Renderable for Lines {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();
        for line in self.iter() {
            segments.extend(line.gilt_console(console, options));
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
    use crate::text::{JustifyMethod, Lines, OverflowMethod, Span, Text};
    use crate::widgets::table::Table;

    fn make_console() -> Console {
        Console::builder()
            .width(80)
            .markup(false)
            .no_color(true)
            .build()
    }

    fn segments_text(segments: &[Segment]) -> String {
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    fn text_arc(s: &str) -> RenderableArc {
        Arc::new(Text::new(s, Style::null()))
    }

    // -- NEW: heterogeneous widget tests ------------------------------------

    #[test]
    fn renderables_append_table_renders() {
        let console = make_console();
        let options = console.options();

        let mut r = Renderables::new(vec![]);
        let mut table = Table::new(&["Col A", "Col B"]);
        table.add_row(&["r1c1", "r1c2"]);
        r.append(table);
        assert_eq!(r.len(), 1);

        let segments = r.gilt_console(&console, &options);
        let output = segments_text(&segments);

        assert!(output.contains("Col A"), "Table header Col A not found");
        assert!(output.contains("r1c1"), "Table cell not found");
    }

    #[test]
    fn renderables_new_accepts_renderable_arcs() {
        let console = make_console();
        let options = console.options();

        let items: Vec<RenderableArc> = vec![text_arc("item one"), text_arc("item two")];
        let r = Renderables::new(items);
        assert_eq!(r.len(), 2);

        let segments = r.gilt_console(&console, &options);
        let output = segments_text(&segments);
        assert!(output.contains("item one"));
        assert!(output.contains("item two"));
    }

    #[test]
    fn renderables_debug_impl() {
        let r = Renderables::new(vec![text_arc("debug")]);
        let debug_str = format!("{:?}", r);
        assert!(debug_str.contains("Renderables"));
        assert!(debug_str.contains("<renderable>"));
    }

    // -- Renderables: construction ------------------------------------------

    #[test]
    fn test_renderables_empty() {
        let r = Renderables::new(vec![]);
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn test_renderables_single_item() {
        let r = Renderables::new(vec![text_arc("Hello")]);
        assert!(!r.is_empty());
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_renderables_multiple_items() {
        let items: Vec<RenderableArc> = vec![text_arc("Hello"), text_arc("World"), text_arc("Foo")];
        let r = Renderables::new(items);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_renderables_append() {
        let mut r = Renderables::new(vec![]);
        assert!(r.is_empty());
        r.append(Text::new("First", Style::null()));
        assert_eq!(r.len(), 1);
        r.append(Text::new("Second", Style::null()));
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_renderables_default() {
        let r = Renderables::default();
        assert!(r.is_empty());
    }

    // -- Renderables: measure -----------------------------------------------

    #[test]
    fn test_renderables_measure_empty() {
        let console = make_console();
        let opts = console.options();
        let r = Renderables::new(vec![]);
        let m = r.measure(&console, &opts);
        assert_eq!(m, Measurement::new(1, 1));
    }

    #[test]
    fn test_renderables_measure_single() {
        let console = make_console();
        let opts = console.options();
        let r = Renderables::new(vec![text_arc("Hello World")]);
        let m = r.measure(&console, &opts);
        // "Hello World" -> min=5 (longest word "Hello" or "World"), max=11
        assert_eq!(m.minimum, 5);
        assert_eq!(m.maximum, 11);
    }

    #[test]
    fn test_renderables_measure_multiple() {
        let console = make_console();
        let opts = console.options();
        let items: Vec<RenderableArc> = vec![
            text_arc("Hi"),          // min=2, max=2
            text_arc("Hello World"), // min=5, max=11
            text_arc("Foo"),         // min=3, max=3
        ];
        let r = Renderables::new(items);
        let m = r.measure(&console, &opts);
        // min = max(2, 5, 3) = 5
        // max = max(2, 11, 3) = 11
        assert_eq!(m.minimum, 5);
        assert_eq!(m.maximum, 11);
    }

    #[test]
    fn test_renderables_measure_correct_min_max() {
        let console = make_console();
        let opts = console.options();
        let items: Vec<RenderableArc> = vec![
            text_arc("abcdefghij"), // single word, min=10, max=10
            text_arc("ab cd ef"),   // min=2, max=8
        ];
        let r = Renderables::new(items);
        let m = r.measure(&console, &opts);
        assert_eq!(m.minimum, 10); // max(10, 2)
        assert_eq!(m.maximum, 10); // max(10, 8)
    }

    // -- Renderables: Renderable trait --------------------------------------

    #[test]
    fn test_renderables_renderable_empty() {
        let console = make_console();
        let options = console.options();
        let r = Renderables::new(vec![]);
        let segments = r.gilt_console(&console, &options);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_renderables_renderable_single() {
        let console = make_console();
        let options = console.options();
        let mut t = Text::new("Hello", Style::null());
        t.end = String::new();
        let r = Renderables::new(vec![Arc::new(t)]);
        let segments = r.gilt_console(&console, &options);
        let text = segments_text(&segments);
        assert!(text.contains("Hello"));
    }

    #[test]
    fn test_renderables_renderable_multiple() {
        let console = make_console();
        let options = console.options();
        let mut t1 = Text::new("Hello", Style::null());
        t1.end = String::new();
        let mut t2 = Text::new("World", Style::null());
        t2.end = String::new();
        let r = Renderables::new(vec![Arc::new(t1), Arc::new(t2)]);
        let segments = r.gilt_console(&console, &options);
        let text = segments_text(&segments);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_renderables_renderable_preserves_order() {
        let console = make_console();
        let options = console.options();
        let mut items: Vec<RenderableArc> = Vec::new();
        for i in 0..5 {
            let mut t = Text::new(&format!("item{}", i), Style::null());
            t.end = String::new();
            items.push(Arc::new(t));
        }
        let r = Renderables::new(items);
        let segments = r.gilt_console(&console, &options);
        let text = segments_text(&segments);
        // Items should appear in order
        let pos0 = text.find("item0").unwrap();
        let pos1 = text.find("item1").unwrap();
        let pos2 = text.find("item2").unwrap();
        let pos3 = text.find("item3").unwrap();
        let pos4 = text.find("item4").unwrap();
        assert!(pos0 < pos1);
        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
        assert!(pos3 < pos4);
    }

    // -- Lines: construction ------------------------------------------------

    #[test]
    fn test_lines_construction_empty() {
        let lines = Lines::new(vec![]);
        assert!(lines.is_empty());
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn test_lines_construction_with_items() {
        let lines = Lines::new(vec![
            Text::new("Line 1", Style::null()),
            Text::new("Line 2", Style::null()),
        ]);
        assert_eq!(lines.len(), 2);
        assert!(!lines.is_empty());
    }

    // -- Lines: append, extend, pop -----------------------------------------

    #[test]
    fn test_lines_append() {
        let mut lines = Lines::new(vec![]);
        lines.push(Text::new("First", Style::null()));
        assert_eq!(lines.len(), 1);
        lines.push(Text::new("Second", Style::null()));
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_lines_extend() {
        let mut lines = Lines::new(vec![Text::new("First", Style::null())]);
        lines.extend(vec![
            Text::new("Second", Style::null()),
            Text::new("Third", Style::null()),
        ]);
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_lines_pop() {
        let mut lines = Lines::new(vec![
            Text::new("First", Style::null()),
            Text::new("Second", Style::null()),
        ]);
        let popped = lines.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().plain(), "Second");
        assert_eq!(lines.len(), 1);

        let popped2 = lines.pop();
        assert!(popped2.is_some());
        assert_eq!(popped2.unwrap().plain(), "First");

        let popped3 = lines.pop();
        assert!(popped3.is_none());
    }

    #[test]
    fn test_lines_len_is_empty() {
        let mut lines = Lines::new(vec![]);
        assert!(lines.is_empty());
        assert_eq!(lines.len(), 0);

        lines.push(Text::new("Item", Style::null()));
        assert!(!lines.is_empty());
        assert_eq!(lines.len(), 1);
    }

    // -- Lines: index access ------------------------------------------------

    #[test]
    fn test_lines_index_access() {
        let lines = Lines::new(vec![
            Text::new("Alpha", Style::null()),
            Text::new("Beta", Style::null()),
            Text::new("Gamma", Style::null()),
        ]);
        assert_eq!(lines[0].plain(), "Alpha");
        assert_eq!(lines[1].plain(), "Beta");
        assert_eq!(lines[2].plain(), "Gamma");
    }

    #[test]
    fn test_lines_index_mut_access() {
        let mut lines = Lines::new(vec![Text::new("Before", Style::null())]);
        lines[0].set_plain("After");
        assert_eq!(lines[0].plain(), "After");
    }

    // -- Lines: Renderable implementation -----------------------------------

    #[test]
    fn test_lines_renderable_empty() {
        let console = make_console();
        let options = console.options();
        let lines = Lines::new(vec![]);
        let segments = lines.gilt_console(&console, &options);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_lines_renderable_single() {
        let console = make_console();
        let options = console.options();
        let mut t = Text::new("Hello", Style::null());
        t.end = String::new();
        let lines = Lines::new(vec![t]);
        let segments = lines.gilt_console(&console, &options);
        let text = segments_text(&segments);
        assert!(text.contains("Hello"));
    }

    #[test]
    fn test_lines_renderable_multiple() {
        let console = make_console();
        let options = console.options();
        let mut t1 = Text::new("Line One", Style::null());
        t1.end = String::new();
        let mut t2 = Text::new("Line Two", Style::null());
        t2.end = String::new();
        let lines = Lines::new(vec![t1, t2]);
        let segments = lines.gilt_console(&console, &options);
        let text = segments_text(&segments);
        assert!(text.contains("Line One"));
        assert!(text.contains("Line Two"));
    }

    // -- Lines: justify left ------------------------------------------------

    #[test]
    fn test_lines_justify_left() {
        let console = make_console();
        let mut lines = Lines::new(vec![
            Text::new("Hi", Style::null()),
            Text::new("Hello", Style::null()),
        ]);
        lines.justify(&console, 10, JustifyMethod::Left, OverflowMethod::Fold);
        // Left justify truncates with padding to width
        assert_eq!(lines[0].cell_len(), 10);
        assert_eq!(lines[1].cell_len(), 10);
        assert!(lines[0].plain().starts_with("Hi"));
        assert!(lines[1].plain().starts_with("Hello"));
    }

    // -- Lines: justify center ----------------------------------------------

    #[test]
    fn test_lines_justify_center() {
        let console = make_console();
        let mut lines = Lines::new(vec![Text::new("Hi", Style::null())]);
        lines.justify(&console, 10, JustifyMethod::Center, OverflowMethod::Fold);
        let plain = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 10);
        // "Hi" is 2 chars, centered in 10: left_pad=4, right_pad=4
        assert_eq!(plain, "    Hi    ");
    }

    #[test]
    fn test_lines_justify_right() {
        let console = make_console();
        let mut lines = Lines::new(vec![Text::new("Hi", Style::null())]);
        lines.justify(&console, 10, JustifyMethod::Right, OverflowMethod::Fold);
        let plain = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 10);
        // "Hi" is 2 chars, right justified in 10: 8 spaces then "Hi"
        assert_eq!(plain, "        Hi");
    }

    // -- Lines: justify full ------------------------------------------------
    #[test]
    fn test_lines_justify_full() {
        let console = make_console();
        let mut lines = Lines::new(vec![
            Text::new("a b c", Style::null()),
            Text::new("end", Style::null()),
        ]);
        lines.justify(&console, 10, JustifyMethod::Full, OverflowMethod::Fold);
        // First line "a b c" (5 chars) should be expanded to 10 with extra spaces
        // between words. Last line should be left-justified.
        let first = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 10);
        // Verify spaces were distributed among the 2 gaps
        assert!(first.starts_with('a'));
        assert!(first.ends_with('c'));
        // Last line: left justified
        assert!(lines[1].plain().starts_with("end"));
    }

    #[test]
    fn test_lines_justify_full_multiple_words() {
        let console = make_console();
        let mut lines = Lines::new(vec![
            Text::new("aa bb cc dd", Style::null()),
            Text::new("last", Style::null()),
        ]);
        // "aa bb cc dd" = 11 chars, width = 15 -> 4 extra spaces among 3 gaps
        lines.justify(&console, 15, JustifyMethod::Full, OverflowMethod::Fold);
        let first = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 15);
        // Words should still be present
        assert!(first.contains("aa"));
        assert!(first.contains("bb"));
        assert!(first.contains("cc"));
        assert!(first.contains("dd"));
    }
    #[test]
    fn test_lines_justify_full_uneven_spacing() {
        let console = make_console();
        let mut lines = Lines::new(vec![
            Text::new("a b c", Style::null()),
            Text::new("end", Style::null()),
        ]);
        // "a b c" = 5 chars, width = 9 -> 4 extra spaces among 2 gaps
        // 4/2 = 2 per gap, 0 remainder => "a   b   c"
        lines.justify(&console, 9, JustifyMethod::Full, OverflowMethod::Fold);
        let first = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 9);
        assert_eq!(first, "a   b   c");
    }
    #[test]
    fn test_lines_justify_full_odd_remainder() {
        let console = make_console();
        let mut lines = Lines::new(vec![
            Text::new("a b c", Style::null()),
            Text::new("end", Style::null()),
        ]);
        // "a b c" = 5 chars, width = 10 -> 5 extra spaces among 2 gaps
        // 5/2 = 2 per gap, 1 remainder -> distributed right-to-left
        // gap 1 (between b and c) gets 3, gap 0 (between a and b) gets 2
        // => "a   b    c"
        lines.justify(&console, 10, JustifyMethod::Full, OverflowMethod::Fold);
        let first = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 10);
        assert!(first.starts_with('a'));
        assert!(first.ends_with('c'));
    }
    #[test]
    fn test_lines_justify_full_single_word() {
        let console = make_console();
        let mut lines = Lines::new(vec![
            Text::new("hello", Style::null()),
            Text::new("end", Style::null()),
        ]);
        // Single word can't distribute spaces - should just truncate with padding
        lines.justify(&console, 10, JustifyMethod::Full, OverflowMethod::Fold);
        let first = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 10);
        assert!(first.starts_with("hello"));
    }

    // -- Lines: justify full with adjacent-word style (Phase 7) -------------

    /// Phase 7 — `Full` justification should resolve the inter-word space
    /// style from the adjacent word's style. With equal adjacent styles the
    /// space carries that style; with different adjacent styles it falls back
    /// to the line's root style (rich's `Lines.justify` contract).
    #[test]
    fn test_lines_justify_full_adjacent_word_style_equal() {
        let console = make_console();
        let red = Style::parse("red");
        let mut text = Text::new("A B", Style::null());
        text.spans_mut().push(Span::new(0, 1, red.clone())); // "A" red
        text.spans_mut().push(Span::new(2, 3, red.clone())); // "B" red
        let mut lines = Lines::new(vec![text, Text::new("end", Style::null())]);
        lines.justify(&console, 5, JustifyMethod::Full, OverflowMethod::Fold);
        // "A B" (3 chars) -> "A   B" (5 chars; gap 1..4 = 3 spaces).
        assert_eq!(lines[0].plain(), "A   B");
        assert_eq!(lines[0].cell_len(), 5);
        // Gap must be covered by a span carrying the adjacent word's style
        // (red, since both adjacent words are red).
        let spans = lines[0].spans();
        let gap = spans
            .iter()
            .find(|s| s.start == 1 && s.end == 4)
            .expect("gap between A and B must be covered by a span");
        assert_eq!(gap.style, red);
    }

    #[test]
    fn test_lines_justify_full_adjacent_word_style_differs() {
        let console = make_console();
        let red = Style::parse("red");
        let blue = Style::parse("blue");
        let line_style = Style::parse("italic");
        let mut text = Text::new("A B", line_style.clone());
        text.spans_mut().push(Span::new(0, 1, red.clone())); // "A" red
        text.spans_mut().push(Span::new(2, 3, blue.clone())); // "B" blue
        let mut lines = Lines::new(vec![text, Text::new("end", Style::null())]);
        lines.justify(&console, 5, JustifyMethod::Full, OverflowMethod::Fold);
        assert_eq!(lines[0].plain(), "A   B");
        let spans = lines[0].spans();
        let gap = spans
            .iter()
            .find(|s| s.start == 1 && s.end == 4)
            .expect("gap between A and B must be covered by a span");
        // Adjacent styles differ -> rich falls back to the line's root style.
        assert_eq!(gap.style, line_style);
    }

    /// Phase 7 — multi-gap styled justification. Three styled words with two
    /// gaps; verify BOTH gaps are independently styled and that
    /// `running_shift` correctly offsets the second gap's span start.
    #[test]
    fn test_lines_justify_full_multi_gap_styled() {
        let console = make_console();
        let red = Style::parse("red");
        let blue = Style::parse("blue");
        // "aa bb cc" -> 8 chars; width=11 -> 3 extras, 2 gaps -> per_gap=1, remainder=1.
        // Right-to-left distribution: gap[1] (between bb and cc) gets the
        // remainder -> extras = [1, 2]. New text = "aa bb  cc" (11 chars).
        let mut text = Text::new("aa bb cc", Style::null());
        text.spans_mut().push(Span::new(0, 2, red.clone())); // "aa"
        text.spans_mut().push(Span::new(3, 5, blue.clone())); // "bb"
        text.spans_mut().push(Span::new(6, 8, red.clone())); // "cc"
        let mut lines = Lines::new(vec![text, Text::new("end", Style::null())]);
        lines.justify(&console, 11, JustifyMethod::Full, OverflowMethod::Fold);
        // "aa bb cc" -> 8 chars; width=11 -> 3 extras, 2 gaps -> per_gap=1, remainder=1.
        // Right-to-left distribution: gap[1] (between bb and cc) gets the
        // remainder -> extras = [1, 2]. New text = "aa  bb   cc" (11 chars):
        // gap0 [2..4] (2 spaces), gap1 [6..9] (3 spaces).
        assert_eq!(lines[0].plain(), "aa  bb   cc");
        let spans = lines[0].spans();
        // Gap 0 (aa|bb): red vs blue -> differ -> falls back to line.style (null).
        let gap0 = spans
            .iter()
            .find(|s| s.start == 2 && s.end == 4)
            .expect("gap 0 (between aa and bb) must be covered");
        assert_eq!(gap0.style, Style::null());
        // Gap 1 (bb|cc): blue vs red -> differ -> falls back to line.style (null).
        // Position: sp_pos=5, running_shift after gap 0 = 1, gap_len=1+2=3.
        let gap1 = spans
            .iter()
            .find(|s| s.start == 6 && s.end == 9)
            .expect("gap 1 (between bb and cc) must be covered");
        assert_eq!(gap1.style, Style::null());
    }

    /// Phase 7 — multi-gap with one equal-styled and one differing gap. The
    /// equal gap should carry the shared style; the differing gap should
    /// fall back to the line's root style.
    #[test]
    fn test_lines_justify_full_mixed_styled_gaps() {
        let console = make_console();
        let red = Style::parse("red");
        let italic = Style::parse("italic");
        // "aa bb cc" -> 8 chars; width=10 -> 2 extras, 2 gaps -> per_gap=1, remainder=0.
        // Both gaps get 1 extra. New text = "aa bb cc" with each gap = 2 chars.
        let mut text = Text::new("aa bb cc", italic.clone());
        text.spans_mut().push(Span::new(0, 2, red.clone())); // "aa"
        text.spans_mut().push(Span::new(3, 5, red.clone())); // "bb"
        text.spans_mut().push(Span::new(6, 8, Style::parse("blue"))); // "cc"
        let mut lines = Lines::new(vec![text, Text::new("end", Style::null())]);
        lines.justify(&console, 10, JustifyMethod::Full, OverflowMethod::Fold);
        // Gap 0 (red/red equal) -> gap carries italic + red (line.style + span style).
        // Gap 1 (red/blue differ) -> falls back to italic only.
        // Both gap styles are themed-composed by get_style_at_offset_themed,
        // which combines the line's root style with overlapping span styles.
        let spans = lines[0].spans();
        let italic_red = italic.clone() + red.clone();
        let gap0 = spans
            .iter()
            .find(|s| s.start == 2 && s.end == 4)
            .expect("gap 0 must be covered");
        assert_eq!(gap0.style, italic_red);
        let gap1 = spans
            .iter()
            .find(|s| s.start == 6 && s.end == 8)
            .expect("gap 1 must be covered");
        assert_eq!(gap1.style, italic);
    }

    // -- gilt_measure override -----------------------------------------------

    #[test]
    fn renderables_gilt_measure_matches_standalone() {
        let console = make_console();
        let opts = console.options();
        let items: Vec<RenderableArc> =
            vec![text_arc("Hello"), text_arc("Hello, World!"), text_arc("Hi")];
        let r = Renderables::new(items);
        let m_standalone = r.measure(&console, &opts);
        let m_trait = r.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Renderables::gilt_measure must delegate to Renderables::measure"
        );
    }

    #[test]
    fn renderables_gilt_measure_empty_matches_standalone() {
        let console = make_console();
        let opts = console.options();
        let r = Renderables::new(vec![]);
        let m_standalone = r.measure(&console, &opts);
        let m_trait = r.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Renderables::gilt_measure (empty) must delegate to Renderables::measure"
        );
    }
}
