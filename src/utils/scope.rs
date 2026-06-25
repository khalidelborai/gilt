//! Scope module -- renders a mapping of key-value pairs in a panel.
//!
//!
//! In Python, `render_scope` introspects a `dict` and displays variable names
//! and their repr values inside a bordered panel. In Rust we accept
//! pre-formatted `&str` pairs instead.

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::panel::Panel;
use crate::segment::Segment;
use crate::style::Style;
use crate::table::{ColumnOptions, Table};
use crate::text::{JustifyMethod, Text};
use crate::utils::highlighter::Highlighter;
use crate::utils::padding::PaddingDimensions;

// ---------------------------------------------------------------------------
// Scope
// ---------------------------------------------------------------------------

/// A renderable that displays key-value pairs inside a bordered panel.
///
/// This is the Rust equivalent of Python's `render_scope`.
#[derive(Debug, Clone)]
pub struct Scope {
    /// The key-value pairs to display.
    items: Vec<(String, String)>,
    /// Optional title for the panel.
    title: Option<String>,
    /// Whether to sort keys (default: `true`).
    sort_keys: bool,
}

impl Scope {
    /// Create a new `Scope` with the given key-value pairs.
    pub fn new(items: Vec<(String, String)>) -> Self {
        Scope {
            items,
            title: None,
            sort_keys: true,
        }
    }

    /// Create a `Scope` from borrowed string slices.
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        Scope {
            items: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            title: None,
            sort_keys: true,
        }
    }

    /// Set the panel title.
    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set whether keys should be sorted.
    #[must_use]
    pub fn sort_keys(mut self, sort: bool) -> Self {
        self.sort_keys = sort;
        self
    }

    /// Add a key-value pair.
    pub fn add(&mut self, key: &str, value: &str) {
        self.items.push((key.to_string(), value.to_string()));
    }

    /// Return a sorted or unsorted iterator over the items, following Python
    /// rich's sort order: dunder keys first (alphabetically), then regular
    /// keys (alphabetically, case-insensitive).
    fn ordered_items(&self) -> Vec<(&str, &str)> {
        let mut pairs: Vec<(&str, &str)> = self
            .items
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        if self.sort_keys {
            // Pre-compute lowercased keys (one alloc per pair) instead of
            // re-lowercasing on every comparison (O(N log N) allocs).
            let lower: Vec<String> = pairs.iter().map(|(k, _)| k.to_lowercase()).collect();
            let mut indices: Vec<usize> = (0..pairs.len()).collect();
            indices.sort_by(|&i, &j| {
                let a_regular = !pairs[i].0.starts_with("__");
                let b_regular = !pairs[j].0.starts_with("__");
                a_regular
                    .cmp(&b_regular)
                    .then_with(|| lower[i].cmp(&lower[j]))
            });
            pairs = indices.into_iter().map(|i| pairs[i]).collect();
        }

        pairs
    }

    /// Build the inner table and wrap it in a panel, returning segments.
    fn render_panel(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        // Build a grid table with padding (0, 1)
        let mut grid = Table::grid(&[]);
        grid.padding = (0, 1, 0, 1);

        // Key column: right-justified
        grid.add_column(
            "",
            "",
            ColumnOptions {
                justify: Some(JustifyMethod::Right),
                ..Default::default()
            },
        );
        // Equals column: left-justified, no extra padding so "=" sits next
        // to the key. The column-style is the equals theme style.
        grid.add_column(
            "",
            "scope.equals",
            ColumnOptions {
                justify: Some(JustifyMethod::Left),
                ..Default::default()
            },
        );
        // Value column: default (left-justified)
        grid.add_column("", "", ColumnOptions::default());

        // Resolve the per-cell theme styles once. `get_style` falls back
        // to `Style::null()` on any failure (missing theme entry, parse
        // error) so the render never breaks on a custom theme.
        let key_style = console
            .get_style("scope.key")
            .unwrap_or_else(|_| Style::null());
        let key_special_style = console
            .get_style("scope.key.special")
            .unwrap_or_else(|_| Style::null());
        let equals_style = console
            .get_style("scope.equals")
            .unwrap_or_else(|_| Style::null());
        let highlighter = crate::utils::highlighter::ReprHighlighter::new();

        let items = self.ordered_items();

        for (key, value) in &items {
            // Per-cell styling (Phase 7 / Plan 7.27): mirror rich's
            // `render_scope` by splitting the row into three styled cells
            // — key, "=", value. The key uses `scope.key` for normal
            // identifiers and `scope.key.special` for dunder / special
            // names (start *and* end with `_`); the equals gets
            // `scope.equals`; the value is run through `ReprHighlighter`
            // so numbers / strings / paths / etc. are colored regardless
            // of the console's `highlight` setting.
            let is_special = key.starts_with('_') && key.ends_with('_');
            let cell_key_style = if is_special {
                key_special_style.clone()
            } else {
                key_style.clone()
            };
            let key_text = Text::new(key, cell_key_style);
            let equals_text = Text::new(" =", equals_style.clone());
            let mut value_text = Text::new(value, Style::null());
            highlighter.highlight(&mut value_text);
            grid.add_row_text(&[key_text, equals_text, value_text]);
        }

        // Get the border style
        let border_style = console
            .get_style("scope.border")
            .unwrap_or_else(|_| Style::null());

        // Render the grid table and rebuild a styled Text from the resulting
        // segments — preserves per-segment styles (was previously lost via
        // `seg.text.push_str` into a plain String).
        let table_segments = grid.gilt_console(console, options);
        let mut content = Text::new("", Style::null());
        for seg in &table_segments {
            // Skip the trailing terminator-only segments; the Text appender
            // handles internal newlines correctly.
            content.append_str(&seg.text, seg.style().cloned());
        }
        // Trim the final newline for cleaner Panel rendering.
        if content.plain().ends_with('\n') {
            let new_len = content.plain().len() - 1;
            let new_text = content.plain()[..new_len].to_string();
            content.set_plain(&new_text);
        }

        // Build the panel
        let mut panel = Panel::fit(content)
            .with_border_style(border_style)
            .with_padding(PaddingDimensions::Pair(0, 1));

        if let Some(ref title) = self.title {
            panel = panel.with_title(Text::new(title, Style::null()));
        }

        panel.gilt_console(console, options)
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Scope {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.render_panel(console, options)
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Scope {
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
// render_scope (free function)
// ---------------------------------------------------------------------------

/// Render key-value pairs as a panel with a table grid inside.
///
/// This is the direct Rust equivalent of Python's `render_scope()`.
///
/// # Arguments
///
/// * `scope` - Slice of `(key, value)` string pairs to display.
/// * `title` - Optional title for the panel border.
/// * `sort_keys` - If `true`, sort keys with dunder keys first.
///
/// # Returns
///
/// A `Vec<Segment>` ready for console output.
pub fn render_scope(scope: &[(&str, &str)], title: Option<&str>, sort_keys: bool) -> Vec<Segment> {
    let console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(true)
        .markup(false)
        .build();
    let options = console.options();

    let mut builder = Scope::from_pairs(scope);
    builder.sort_keys = sort_keys;
    if let Some(t) = title {
        builder.title = Some(t.to_string());
    }

    builder.gilt_console(&console, &options)
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

    fn render_scope_output(scope: &Scope, width: usize) -> String {
        let console = make_console(width);
        let opts = console.options();
        let segments = scope.gilt_console(&console, &opts);
        segments_to_text(&segments)
    }

    // -- Empty scope --------------------------------------------------------

    #[test]
    fn test_empty_scope() {
        let scope = Scope::new(vec![]);
        let output = render_scope_output(&scope, 40);
        // Should still produce a panel (empty content)
        assert!(output.contains('\u{256d}') || output.contains('\u{250c}') || output.contains('+'));
    }

    // -- Single key-value pair ----------------------------------------------

    #[test]
    fn test_single_pair() {
        let scope = Scope::from_pairs(&[("name", "Alice")]);
        let output = render_scope_output(&scope, 40);
        assert!(output.contains("name"), "output was: {:?}", output);
        assert!(output.contains("Alice"), "output was: {:?}", output);
        assert!(output.contains("="), "output was: {:?}", output);
    }

    // -- Multiple pairs -----------------------------------------------------

    #[test]
    fn test_multiple_pairs() {
        let scope = Scope::from_pairs(&[("x", "1"), ("y", "2"), ("z", "3")]);
        let output = render_scope_output(&scope, 40);
        assert!(output.contains("x"));
        assert!(output.contains("y"));
        assert!(output.contains("z"));
        assert!(output.contains("1"));
        assert!(output.contains("2"));
        assert!(output.contains("3"));
    }

    // -- Sort keys (alphabetical) -------------------------------------------

    #[test]
    fn test_sort_keys_alphabetical() {
        let scope = Scope::from_pairs(&[("zebra", "z"), ("apple", "a"), ("mango", "m")]);
        let output = render_scope_output(&scope, 40);
        // All keys should appear
        assert!(output.contains("zebra"));
        assert!(output.contains("apple"));
        assert!(output.contains("mango"));

        // Check ordering: apple should come before mango, mango before zebra
        let apple_pos = output.find("apple").unwrap();
        let mango_pos = output.find("mango").unwrap();
        let zebra_pos = output.find("zebra").unwrap();
        assert!(apple_pos < mango_pos);
        assert!(mango_pos < zebra_pos);
    }

    // -- Special keys (__dunder__) sorted first -----------------------------

    #[test]
    fn test_dunder_keys_sorted_first() {
        let scope = Scope::from_pairs(&[
            ("regular", "val"),
            ("__special__", "dunder"),
            ("another", "val2"),
        ]);
        let output = render_scope_output(&scope, 50);
        // __special__ should appear before regular keys
        let special_pos = output.find("__special__").unwrap();
        let regular_pos = output.find("regular").unwrap();
        let another_pos = output.find("another").unwrap();
        assert!(
            special_pos < regular_pos,
            "dunder key should come first: special={}, regular={}",
            special_pos,
            regular_pos
        );
        assert!(
            special_pos < another_pos,
            "dunder key should come first: special={}, another={}",
            special_pos,
            another_pos
        );
    }

    #[test]
    fn test_multiple_dunder_keys_sorted() {
        let scope = Scope::from_pairs(&[
            ("beta", "b"),
            ("__z__", "z"),
            ("alpha", "a"),
            ("__a__", "a"),
        ]);
        let output = render_scope_output(&scope, 50);
        // Dunder keys should come first, then regular keys
        let z_dunder_pos = output.find("__z__").unwrap();
        let a_dunder_pos = output.find("__a__").unwrap();
        let alpha_pos = output.find("alpha").unwrap();
        let beta_pos = output.find("beta").unwrap();

        // __a__ before __z__ (alphabetical among dunders)
        assert!(a_dunder_pos < z_dunder_pos);
        // Both dunders before regular keys
        assert!(z_dunder_pos < alpha_pos);
        assert!(z_dunder_pos < beta_pos);
        // alpha before beta
        assert!(alpha_pos < beta_pos);
    }

    // -- Title displayed ----------------------------------------------------

    #[test]
    fn test_title_displayed() {
        let scope = Scope::from_pairs(&[("key", "value")]).title("My Scope");
        let output = render_scope_output(&scope, 40);
        assert!(output.contains("My Scope"), "output was: {:?}", output);
    }

    // -- No title -----------------------------------------------------------

    #[test]
    fn test_no_title() {
        let scope = Scope::from_pairs(&[("key", "value")]);
        assert!(scope.title.is_none());
        let output = render_scope_output(&scope, 40);
        // Should still render without error
        assert!(output.contains("key"));
        assert!(output.contains("value"));
    }

    // -- Key styling --------------------------------------------------------

    #[test]
    fn test_key_contains_equals() {
        let scope = Scope::from_pairs(&[("myvar", "42")]);
        let output = render_scope_output(&scope, 40);
        // The output should contain "myvar =" (key followed by equals)
        assert!(output.contains("myvar ="), "output was: {:?}", output);
    }

    #[test]
    fn test_special_key_format() {
        let scope = Scope::from_pairs(&[("__init__", "method")]);
        let output = render_scope_output(&scope, 40);
        assert!(output.contains("__init__"), "output was: {:?}", output);
        assert!(output.contains("="), "output was: {:?}", output);
    }

    // -- Values rendered in output ------------------------------------------

    #[test]
    fn test_value_rendered() {
        let scope = Scope::from_pairs(&[("count", "42"), ("name", "hello")]);
        let output = render_scope_output(&scope, 40);
        assert!(output.contains("42"));
        assert!(output.contains("hello"));
    }

    // -- Renderable trait integration ---------------------------------------

    #[test]
    fn test_renderable_trait() {
        let scope = Scope::from_pairs(&[("a", "1")]);
        let console = make_console(40);
        let opts = console.options();
        let segments = scope.gilt_console(&console, &opts);
        assert!(!segments.is_empty());
        let text = segments_to_text(&segments);
        assert!(text.contains("a"));
        assert!(text.contains("1"));
    }

    #[test]
    fn test_renderable_produces_panel() {
        let scope = Scope::from_pairs(&[("x", "10")]);
        let console = make_console(30);
        let opts = console.options();
        let segments = scope.gilt_console(&console, &opts);
        let text = segments_to_text(&segments);
        // Panel should have border characters (rounded box)
        assert!(
            text.contains('\u{256d}') || text.contains('\u{250c}'),
            "expected panel border in: {:?}",
            text
        );
    }

    // -- Builder methods ----------------------------------------------------

    #[test]
    fn test_builder_title() {
        let scope = Scope::from_pairs(&[("k", "v")]).title("Title");
        assert_eq!(scope.title, Some("Title".to_string()));
    }

    #[test]
    fn test_builder_sort_keys() {
        let scope = Scope::from_pairs(&[("k", "v")]).sort_keys(false);
        assert!(!scope.sort_keys);
    }

    #[test]
    fn test_builder_chain() {
        let scope = Scope::from_pairs(&[("k", "v")]).title("T").sort_keys(false);
        assert_eq!(scope.title, Some("T".to_string()));
        assert!(!scope.sort_keys);
    }

    // -- Add method ---------------------------------------------------------

    #[test]
    fn test_add_items() {
        let mut scope = Scope::new(vec![]);
        scope.add("first", "1");
        scope.add("second", "2");
        assert_eq!(scope.items.len(), 2);
        assert_eq!(scope.items[0], ("first".to_string(), "1".to_string()));
        assert_eq!(scope.items[1], ("second".to_string(), "2".to_string()));
    }

    // -- Sort keys disabled -------------------------------------------------

    #[test]
    fn test_no_sort_preserves_order() {
        let scope =
            Scope::from_pairs(&[("zebra", "z"), ("apple", "a"), ("mango", "m")]).sort_keys(false);
        let output = render_scope_output(&scope, 40);
        // Original order should be preserved
        let zebra_pos = output.find("zebra").unwrap();
        let apple_pos = output.find("apple").unwrap();
        let mango_pos = output.find("mango").unwrap();
        assert!(zebra_pos < apple_pos);
        assert!(apple_pos < mango_pos);
    }

    // -- render_scope free function -----------------------------------------

    #[test]
    fn test_render_scope_function() {
        let segments = render_scope(&[("key", "value")], Some("Title"), true);
        let text = segments_to_text(&segments);
        assert!(text.contains("key"));
        assert!(text.contains("value"));
        assert!(text.contains("Title"));
    }

    #[test]
    fn test_render_scope_no_title() {
        let segments = render_scope(&[("a", "b")], None, true);
        let text = segments_to_text(&segments);
        assert!(text.contains("a"));
        assert!(text.contains("b"));
    }

    #[test]
    fn test_render_scope_empty() {
        let segments = render_scope(&[], None, true);
        let text = segments_to_text(&segments);
        // Should produce a panel even with no items
        assert!(!text.is_empty());
    }

    // -- Ordered items internal method --------------------------------------

    #[test]
    fn test_ordered_items_sort() {
        let scope = Scope::from_pairs(&[("B", "2"), ("a", "1"), ("C", "3")]);
        let ordered = scope.ordered_items();
        assert_eq!(ordered[0].0, "a");
        assert_eq!(ordered[1].0, "B");
        assert_eq!(ordered[2].0, "C");
    }

    #[test]
    fn test_ordered_items_no_sort() {
        let scope = Scope::from_pairs(&[("B", "2"), ("a", "1"), ("C", "3")]).sort_keys(false);
        let ordered = scope.ordered_items();
        assert_eq!(ordered[0].0, "B");
        assert_eq!(ordered[1].0, "a");
        assert_eq!(ordered[2].0, "C");
    }

    #[test]
    fn test_ordered_items_dunders_first() {
        let scope = Scope::from_pairs(&[("normal", "n"), ("__dunder__", "d"), ("alpha", "a")]);
        let ordered = scope.ordered_items();
        assert_eq!(ordered[0].0, "__dunder__");
        assert_eq!(ordered[1].0, "alpha");
        assert_eq!(ordered[2].0, "normal");
    }

    // -- Case insensitive sort ----------------------------------------------

    #[test]
    fn test_case_insensitive_sort() {
        let scope = Scope::from_pairs(&[("Zulu", "z"), ("alpha", "a"), ("Beta", "b")]);
        let ordered = scope.ordered_items();
        assert_eq!(ordered[0].0, "alpha");
        assert_eq!(ordered[1].0, "Beta");
        assert_eq!(ordered[2].0, "Zulu");
    }

    // -- Panel border style -------------------------------------------------

    #[test]
    fn test_panel_has_border() {
        let scope = Scope::from_pairs(&[("x", "1")]);
        let console = make_console(30);
        let opts = console.options();
        let segments = scope.gilt_console(&console, &opts);
        let text = segments_to_text(&segments);
        let lines: Vec<&str> = text.split('\n').filter(|l| !l.is_empty()).collect();
        // Should have at least 3 lines: top border, content, bottom border
        assert!(
            lines.len() >= 3,
            "expected at least 3 lines, got {}: {:?}",
            lines.len(),
            lines
        );
    }

    // -- Scope with long values ---------------------------------------------

    #[test]
    fn test_long_values() {
        let scope = Scope::from_pairs(&[
            ("short", "ok"),
            ("long_key", "this is a rather long value string"),
        ]);
        let output = render_scope_output(&scope, 60);
        assert!(output.contains("short"));
        assert!(output.contains("long_key"));
        assert!(output.contains("this is a rather long value string"));
    }

    // -- Scope with numeric-looking values ----------------------------------

    #[test]
    fn test_numeric_values() {
        let scope = Scope::from_pairs(&[("pi", "3.14159"), ("count", "42"), ("hex", "0xFF")]);
        let output = render_scope_output(&scope, 40);
        assert!(output.contains("3.14159"));
        assert!(output.contains("42"));
        assert!(output.contains("0xFF"));
    }

    // -- from_pairs constructor ---------------------------------------------

    #[test]
    fn test_from_pairs() {
        let scope = Scope::from_pairs(&[("a", "1"), ("b", "2")]);
        assert_eq!(scope.items.len(), 2);
        assert_eq!(scope.items[0].0, "a");
        assert_eq!(scope.items[0].1, "1");
        assert!(scope.sort_keys);
        assert!(scope.title.is_none());
    }

    // -- Clone --------------------------------------------------------------

    #[test]
    fn test_scope_clone() {
        let scope = Scope::from_pairs(&[("k", "v")]).title("T").sort_keys(false);
        let cloned = scope.clone();
        assert_eq!(cloned.items, scope.items);
        assert_eq!(cloned.title, scope.title);
        assert_eq!(cloned.sort_keys, scope.sort_keys);
    }

    #[test]
    fn render_preserves_styling_from_inner_table() {
        // Regression for B2: render_panel was flattening the inner table's
        // styled segments into a plain String, dropping all SGR markup. Verify
        // ANSI escapes from styled cells reach the captured output.
        use crate::console::Console;
        let scope = Scope::from_pairs(&[("user", "alice"), ("role", "admin")]).title("Login");
        let mut console = Console::builder()
            .width(60)
            .force_terminal(true)
            .color_system("truecolor")
            .build();
        console.begin_capture();
        console.print(&scope);
        let out = console.end_capture();
        // Some SGR sequence must be present — the cell-key style ("scope.key")
        // and panel border style produce ANSI escapes.
        assert!(
            out.contains('\x1b'),
            "expected ANSI escapes in scope render, got: {out:?}"
        );
        assert!(out.contains("user"));
        assert!(out.contains("alice"));
    }
    // -- Per-cell styling (Phase 7 / Plan 7.27) ----------------------------

    #[test]
    fn test_key_cell_carries_scope_key_style() {
        // Per-cell key styling (Phase 7 / Plan 7.27): the segment carrying
        // the key text "count" must carry the resolved "scope.key" theme
        // style. The segment carrying "42" must carry the "repr.number"
        // theme style (from ReprHighlighter). The "=" segment must carry
        // the resolved "scope.equals" theme style.
        let scope = Scope::from_pairs(&[("count", "42")]);
        let console = Console::builder()
            .width(60)
            .force_terminal(true)
            .color_system("truecolor")
            .highlight(false)
            .build();
        let opts = console.options();
        let key_style = console
            .get_style("scope.key")
            .expect("scope.key must be defined in default theme");
        let equals_style = console
            .get_style("scope.equals")
            .expect("scope.equals must be defined in default theme");
        let number_style = console
            .get_style("repr.number")
            .expect("repr.number must be defined in default theme");
        let segments = scope.gilt_console(&console, &opts);
        let segments: Vec<&Segment> = segments.iter().filter(|s| !s.text.is_empty()).collect();
        let key_segs: Vec<&&Segment> = segments
            .iter()
            .filter(|s| s.text.contains("count"))
            .collect();
        let eq_segs: Vec<&&Segment> = segments.iter().filter(|s| s.text.contains("=")).collect();
        let val_segs: Vec<&&Segment> = segments.iter().filter(|s| s.text.contains("42")).collect();
        assert!(
            !key_segs.is_empty(),
            "expected at least one segment containing key text 'count'"
        );
        assert!(
            !eq_segs.is_empty(),
            "expected at least one segment containing '='"
        );
        assert!(
            !val_segs.is_empty(),
            "expected at least one segment containing value '42'"
        );
        for seg in &key_segs {
            let st = seg.style().expect("key cell segment must carry a style");
            assert_eq!(
                st, &key_style,
                "key cell must carry scope.key style; got {st:?}"
            );
        }
        for seg in &eq_segs {
            let st = seg.style().expect("equals cell segment must carry a style");
            assert_eq!(
                st, &equals_style,
                "equals cell must carry scope.equals style; got {st:?}"
            );
        }
        for seg in &val_segs {
            let st = seg.style().expect("value cell segment must carry a style");
            assert_eq!(
                st, &number_style,
                "value cell must carry repr.number style (ReprHighlighter); got {st:?}"
            );
        }
    }

    #[test]
    fn test_dunder_key_uses_special_style() {
        // A dunder key like "__init__" must use the "scope.key.special"
        // theme style for its key cell, not the regular "scope.key".
        let scope = Scope::from_pairs(&[("__init__", "method")]);
        let console = Console::builder()
            .width(60)
            .force_terminal(true)
            .color_system("truecolor")
            .highlight(false)
            .build();
        let opts = console.options();
        let special_style = console
            .get_style("scope.key.special")
            .expect("scope.key.special must be defined in default theme");
        let key_style = console.get_style("scope.key").unwrap();
        assert_ne!(
            special_style, key_style,
            "scope.key.special must differ from scope.key in the theme"
        );
        let segments = scope.gilt_console(&console, &opts);
        let segments: Vec<&Segment> = segments.iter().filter(|s| !s.text.is_empty()).collect();
        let key_segs: Vec<&&Segment> = segments
            .iter()
            .filter(|s| s.text.contains("__init__"))
            .collect();
        assert!(
            !key_segs.is_empty(),
            "expected at least one segment containing '__init__'"
        );
        for seg in &key_segs {
            let st = seg.style().expect("dunder key segment must be styled");
            assert_eq!(
                st, &special_style,
                "dunder key cell must carry scope.key.special; got {st:?}"
            );
        }
    }

    #[test]
    fn test_value_repr_highlighted() {
        // A numeric value "3.14" must be ReprHighlighter-annotated — the
        // segment carrying the digits must carry the resolved
        // "repr.number" theme style.
        let scope = Scope::from_pairs(&[("count", "3.14")]);
        let console = Console::builder()
            .width(60)
            .force_terminal(true)
            .color_system("truecolor")
            .highlight(false)
            .build();
        let opts = console.options();
        let number_style = console
            .get_style("repr.number")
            .expect("repr.number must be defined in default theme");
        let segments = scope.gilt_console(&console, &opts);
        let segments: Vec<&Segment> = segments.iter().filter(|s| !s.text.is_empty()).collect();
        let val_segs: Vec<&&Segment> = segments
            .iter()
            .filter(|s| s.text.contains("3.14"))
            .collect();
        assert!(
            !val_segs.is_empty(),
            "expected at least one segment containing value '3.14'"
        );
        for seg in &val_segs {
            let st = seg
                .style()
                .expect("value segment must carry a style (ReprHighlighter)");
            assert_eq!(
                st, &number_style,
                "value cell must carry repr.number style; got {st:?}"
            );
        }
    }
}
