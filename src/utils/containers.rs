//! Container types for grouping renderables.
//!
//! This module provides `Renderables`, a container that renders multiple items
//! in sequence, and implements the `Renderable` trait for `Lines` so that
//! collections of `Text` objects can be rendered to the console.
//!
//! Port of Python's `rich/containers.py`.

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::text::Text;

// ---------------------------------------------------------------------------
// Renderables
// ---------------------------------------------------------------------------

/// A container of renderable items that renders them in sequence.
///
/// This is the Rust equivalent of Python's `rich.containers.Renderables`.
/// In gilt, renderables are `Text` objects.
#[derive(Clone, Debug, Default)]
pub struct Renderables {
    items: Vec<Text>,
}

impl Renderables {
    /// Create a new `Renderables` from a vector of `Text` items.
    pub fn new(items: Vec<Text>) -> Self {
        Renderables { items }
    }

    /// Append a `Text` item to the container.
    pub fn append(&mut self, item: Text) {
        self.items.push(item);
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
    /// If there are no items, returns `Measurement(1, 1)` (matching Python's rich).
    pub fn measure(&self) -> Measurement {
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
        Measurement::new(min_width, max_width)
    }
}

impl Renderable for Renderables {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut segments = Vec::new();
        for item in &self.items {
            segments.extend(item.gilt_console(console, options));
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
    use crate::text::{JustifyMethod, OverflowMethod};

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

    // -- Renderables: construction ------------------------------------------

    #[test]
    fn test_renderables_empty() {
        let r = Renderables::new(vec![]);
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn test_renderables_single_item() {
        let t = Text::new("Hello", Style::null());
        let r = Renderables::new(vec![t]);
        assert!(!r.is_empty());
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_renderables_multiple_items() {
        let items = vec![
            Text::new("Hello", Style::null()),
            Text::new("World", Style::null()),
            Text::new("Foo", Style::null()),
        ];
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
        let r = Renderables::new(vec![]);
        let m = r.measure();
        assert_eq!(m, Measurement::new(1, 1));
    }

    #[test]
    fn test_renderables_measure_single() {
        let t = Text::new("Hello World", Style::null());
        let r = Renderables::new(vec![t]);
        let m = r.measure();
        // "Hello World" -> min=5 (longest word "Hello" or "World"), max=11
        assert_eq!(m.minimum, 5);
        assert_eq!(m.maximum, 11);
    }

    #[test]
    fn test_renderables_measure_multiple() {
        let items = vec![
            Text::new("Hi", Style::null()),          // min=2, max=2
            Text::new("Hello World", Style::null()), // min=5, max=11
            Text::new("Foo", Style::null()),         // min=3, max=3
        ];
        let r = Renderables::new(items);
        let m = r.measure();
        // min = max(2, 5, 3) = 5
        // max = max(2, 11, 3) = 11
        assert_eq!(m.minimum, 5);
        assert_eq!(m.maximum, 11);
    }

    #[test]
    fn test_renderables_measure_correct_min_max() {
        let items = vec![
            Text::new("abcdefghij", Style::null()), // single word, min=10, max=10
            Text::new("ab cd ef", Style::null()),   // min=2, max=8
        ];
        let r = Renderables::new(items);
        let m = r.measure();
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
        let r = Renderables::new(vec![t]);
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
        let r = Renderables::new(vec![t1, t2]);
        let segments = r.gilt_console(&console, &options);
        let text = segments_text(&segments);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_renderables_renderable_preserves_order() {
        let console = make_console();
        let options = console.options();
        let mut items = Vec::new();
        for i in 0..5 {
            let mut t = Text::new(&format!("item{}", i), Style::null());
            t.end = String::new();
            items.push(t);
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
        let mut lines = Lines::new(vec![
            Text::new("Hi", Style::null()),
            Text::new("Hello", Style::null()),
        ]);
        lines.justify(10, JustifyMethod::Left, OverflowMethod::Fold);
        // Left justify truncates with padding to width
        assert_eq!(lines[0].cell_len(), 10);
        assert_eq!(lines[1].cell_len(), 10);
        assert!(lines[0].plain().starts_with("Hi"));
        assert!(lines[1].plain().starts_with("Hello"));
    }

    // -- Lines: justify center ----------------------------------------------

    #[test]
    fn test_lines_justify_center() {
        let mut lines = Lines::new(vec![Text::new("Hi", Style::null())]);
        lines.justify(10, JustifyMethod::Center, OverflowMethod::Fold);
        let plain = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 10);
        // "Hi" is 2 chars, centered in 10: left_pad=4, right_pad=4
        assert_eq!(plain, "    Hi    ");
    }

    // -- Lines: justify right -----------------------------------------------

    #[test]
    fn test_lines_justify_right() {
        let mut lines = Lines::new(vec![Text::new("Hi", Style::null())]);
        lines.justify(10, JustifyMethod::Right, OverflowMethod::Fold);
        let plain = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 10);
        // "Hi" is 2 chars, right justified in 10: 8 spaces then "Hi"
        assert_eq!(plain, "        Hi");
    }

    // -- Lines: justify full ------------------------------------------------

    #[test]
    fn test_lines_justify_full() {
        let mut lines = Lines::new(vec![
            Text::new("a b c", Style::null()),
            Text::new("end", Style::null()),
        ]);
        lines.justify(10, JustifyMethod::Full, OverflowMethod::Fold);
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
        let mut lines = Lines::new(vec![
            Text::new("aa bb cc dd", Style::null()),
            Text::new("last", Style::null()),
        ]);
        // "aa bb cc dd" = 11 chars, width = 15 -> 4 extra spaces among 3 gaps
        lines.justify(15, JustifyMethod::Full, OverflowMethod::Fold);
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
        let mut lines = Lines::new(vec![
            Text::new("a b c", Style::null()),
            Text::new("end", Style::null()),
        ]);
        // "a b c" = 5 chars, width = 9 -> 4 extra spaces among 2 gaps
        // 4/2 = 2 per gap, 0 remainder => "a   b   c"
        lines.justify(9, JustifyMethod::Full, OverflowMethod::Fold);
        let first = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 9);
        assert_eq!(first, "a   b   c");
    }

    #[test]
    fn test_lines_justify_full_odd_remainder() {
        let mut lines = Lines::new(vec![
            Text::new("a b c", Style::null()),
            Text::new("end", Style::null()),
        ]);
        // "a b c" = 5 chars, width = 10 -> 5 extra spaces among 2 gaps
        // 5/2 = 2 per gap, 1 remainder -> distributed right-to-left
        // gap 1 (between b and c) gets 3, gap 0 (between a and b) gets 2
        // => "a   b    c"
        lines.justify(10, JustifyMethod::Full, OverflowMethod::Fold);
        let first = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 10);
        assert!(first.starts_with('a'));
        assert!(first.ends_with('c'));
    }

    #[test]
    fn test_lines_justify_full_single_word() {
        let mut lines = Lines::new(vec![
            Text::new("hello", Style::null()),
            Text::new("end", Style::null()),
        ]);
        // Single word can't distribute spaces - should just truncate with padding
        lines.justify(10, JustifyMethod::Full, OverflowMethod::Fold);
        let first = lines[0].plain().to_string();
        assert_eq!(lines[0].cell_len(), 10);
        assert!(first.starts_with("hello"));
    }
}
