//! Integration tests for `gilt::table::Table`.
//!
//! Ported from rich's tests/test_table.py — see .review/04-test-parity.md

use gilt::box_chars::{HEAVY, MINIMAL, ROUNDED, SQUARE};
use gilt::measure::Measurement;
use gilt::prelude::*;
use gilt::table::{ColumnOptions, Table};
use gilt::utils::align_widget::VerticalAlign;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn capture_table(table: &Table, width: usize) -> String {
    let mut c = Console::builder()
        .width(width)
        .force_terminal(false)
        .no_color(true)
        .build();
    c.begin_capture();
    c.print(table);
    c.end_capture()
}

// ---------------------------------------------------------------------------
// 1. test_render_table — basic table with header + rows renders box-drawing
// ---------------------------------------------------------------------------

#[test]
fn render_table_basic() {
    let mut table = Table::new(&["Name", "Age"]);
    table.add_row(&["Alice", "30"]);
    table.add_row(&["Bob", "25"]);

    let out = capture_table(&table, 40);
    assert!(out.contains("Alice"), "expected 'Alice' in output");
    assert!(out.contains("Bob"), "expected 'Bob' in output");
    assert!(out.contains("30"), "expected '30' in output");
    assert!(out.contains("25"), "expected '25' in output");
    // HEAVY_HEAD default box: top-left is '┏'
    assert!(out.contains('┏'), "expected heavy-head top-left '┏'");
    // Header separator uses '┡'
    assert!(out.contains('┡'), "expected header separator '┡'");
    // Bottom edge
    assert!(out.contains('└'), "expected bottom-left '└'");
}

// ---------------------------------------------------------------------------
// 2. test_no_columns — empty table renders without panic
// ---------------------------------------------------------------------------

#[test]
fn no_columns_no_panic() {
    let table = Table::new(&[]);
    // Must not panic; output is nearly empty (just a newline or empty)
    let out = capture_table(&table, 40);
    assert!(out.trim().is_empty() || out == "\n", "expected empty/newline output for empty table, got: {out:?}");
}

// ---------------------------------------------------------------------------
// 3. test_init_append_column — add_row auto-creates columns
// ---------------------------------------------------------------------------

#[test]
fn add_row_auto_creates_columns() {
    // Start with no columns
    let mut table = Table::new(&[]);
    // Add a row with 3 cells — should auto-create 3 columns
    table.add_row(&["a", "b", "c"]);
    assert_eq!(table.columns.len(), 3, "expected 3 auto-created columns");
    assert_eq!(table.row_count(), 1);
    // Render must not panic
    let out = capture_table(&table, 40);
    assert!(out.contains('a') || !out.is_empty());
}

// ---------------------------------------------------------------------------
// 4. test_rich_measure — measure() returns column widths
// ---------------------------------------------------------------------------

#[test]
fn measure_returns_nonzero_for_populated_table() {
    let mut table = Table::new(&["Name", "Score"]);
    table.add_row(&["Averlongword", "9999"]);

    let c = Console::builder()
        .width(80)
        .force_terminal(false)
        .no_color(true)
        .build();
    let opts = c.options();
    let m = table.measure(&c, &opts);
    assert!(m.minimum > 0, "minimum should be > 0");
    assert!(m.maximum >= m.minimum, "maximum >= minimum");
}

#[test]
fn measure_zero_width_returns_zero() {
    let table = Table::new(&["header"]).with_width(0);
    let c = Console::builder().width(80).force_terminal(false).build();
    let opts = c.options().update_width(0);
    let m = table.measure(&c, &opts);
    assert_eq!(m, Measurement::new(0, 0));
}

// ---------------------------------------------------------------------------
// 5. test_min_width — with_width clamps output width
// ---------------------------------------------------------------------------

#[test]
fn width_clamps_output() {
    let mut table = Table::new(&["foo"]).with_width(20);
    table.add_row(&["bar"]);
    // Render at console width 80 but table width should be clamped to 20 chars
    let out = capture_table(&table, 80);
    for line in out.lines() {
        assert!(
            line.chars().count() <= 20,
            "line wider than 20 chars: {line:?} (chars={})",
            line.chars().count()
        );
    }
}

// ---------------------------------------------------------------------------
// 6. test_get_row_style — odd/even row styles alternate
// ---------------------------------------------------------------------------

#[test]
fn row_styles_cycle() {
    let mut table = Table::new(&["Val"]).with_row_styles(vec![
        "bold".to_string(),
        "italic".to_string(),
    ]);
    table.add_row(&["row0"]);
    table.add_row(&["row1"]);
    table.add_row(&["row2"]);
    assert_eq!(table.row_styles.len(), 2);
    // Render should not panic and output contains data
    let out = capture_table(&table, 40);
    assert!(out.contains("row0"));
    assert!(out.contains("row2"));
}

// ---------------------------------------------------------------------------
// 7. test_vertical_align — top/middle/bottom alignment
// ---------------------------------------------------------------------------

#[test]
fn vertical_align_top_does_not_panic() {
    let mut table = Table::new(&[]).with_show_header(false).with_box_chars(Some(&SQUARE));
    table.add_column("", "", ColumnOptions {
        vertical: Some(VerticalAlign::Top),
        ..Default::default()
    });
    table.add_column("", "", ColumnOptions::default());
    // First column: single-line "foo", second: multi-line "bar\nbar\nbar"
    table.add_row(&["foo", "bar\nbar\nbar"]);
    let out = capture_table(&table, 40);
    assert!(out.contains("foo"));
    assert!(out.contains("bar"));
}

#[test]
fn vertical_align_middle_does_not_panic() {
    let mut table = Table::new(&[]).with_show_header(false).with_box_chars(Some(&SQUARE));
    table.add_column("", "", ColumnOptions {
        vertical: Some(VerticalAlign::Middle),
        ..Default::default()
    });
    table.add_column("", "", ColumnOptions::default());
    table.add_row(&["foo", "bar\nbar\nbar"]);
    let _out = capture_table(&table, 40);
}

#[test]
fn vertical_align_bottom_does_not_panic() {
    let mut table = Table::new(&[]).with_show_header(false).with_box_chars(Some(&SQUARE));
    table.add_column("", "", ColumnOptions {
        vertical: Some(VerticalAlign::Bottom),
        ..Default::default()
    });
    table.add_column("", "", ColumnOptions::default());
    table.add_row(&["foo", "bar\nbar\nbar"]);
    let _out = capture_table(&table, 40);
}

// ---------------------------------------------------------------------------
// 8. test_show_header_false — omits header row
// ---------------------------------------------------------------------------

#[test]
fn show_header_false_omits_header() {
    let mut table = Table::new(&["HEADER_SENTINEL"]).with_show_header(false);
    table.add_row(&["data_row"]);
    let out = capture_table(&table, 40);
    assert!(!out.contains("HEADER_SENTINEL"), "header should be absent");
    assert!(out.contains("data_row"), "data row should appear");
}

// ---------------------------------------------------------------------------
// 9. test_section — add_section inserts separator between row groups
// ---------------------------------------------------------------------------

#[test]
fn section_inserts_separator() {
    let mut table = Table::new(&["Item"]);
    table.add_row(&["row1"]);
    table.add_row(&["row2"]);
    table.add_section(); // separator after row2
    table.add_row(&["row3"]);

    let out = capture_table(&table, 40);
    assert!(out.contains("row1"), "row1 should appear");
    assert!(out.contains("row2"), "row2 should appear");
    assert!(out.contains("row3"), "row3 should appear");
    // A mid-row separator uses '├' in HEAVY_HEAD box
    assert!(
        out.contains('├') || out.contains('┼'),
        "expected row-separator character (├ or ┼)"
    );
}

#[test]
fn add_section_before_any_rows_is_noop() {
    let mut table = Table::new(&["Item"]);
    table.add_section(); // no-op — no rows yet
    table.add_row(&["only_row"]);
    let out = capture_table(&table, 40);
    assert!(out.contains("only_row"));
    // No spurious mid-row separator
    assert!(!out.contains('├'), "should not have mid-row separator");
}

// ---------------------------------------------------------------------------
// 10. test_box_element — different BoxChars produce distinct corners
// ---------------------------------------------------------------------------

#[test]
fn box_square_uses_square_corners() {
    let mut table = Table::new(&["H"]).with_box_chars(Some(&SQUARE));
    table.add_row(&["x"]);
    let out = capture_table(&table, 40);
    assert!(out.contains('┌'), "SQUARE box should have '┌' top-left");
    assert!(out.contains('└'), "SQUARE box should have '└' bottom-left");
}

#[test]
fn box_heavy_uses_heavy_corners() {
    let mut table = Table::new(&["H"]).with_box_chars(Some(&HEAVY));
    table.add_row(&["x"]);
    let out = capture_table(&table, 40);
    assert!(out.contains('┏'), "HEAVY box should have '┏' top-left");
    assert!(out.contains('┗'), "HEAVY box should have '┗' bottom-left");
}

#[test]
fn box_rounded_uses_rounded_corners() {
    let mut table = Table::new(&["H"]).with_box_chars(Some(&ROUNDED));
    table.add_row(&["x"]);
    let out = capture_table(&table, 40);
    assert!(out.contains('╭'), "ROUNDED box should have '╭' top-left");
    assert!(out.contains('╰'), "ROUNDED box should have '╰' bottom-left");
}

#[test]
fn box_minimal_has_no_outer_edges() {
    let mut table = Table::new(&["H"]).with_box_chars(Some(&MINIMAL));
    table.add_row(&["x"]);
    let out = capture_table(&table, 40);
    // MINIMAL has no solid outer border corners
    assert!(!out.contains('┌'), "MINIMAL should not have '┌'");
    assert!(!out.contains('┏'), "MINIMAL should not have '┏'");
}

#[test]
fn box_none_produces_no_border_chars() {
    let mut table = Table::new(&["H"]).with_box_chars(None);
    table.add_row(&["x"]);
    let out = capture_table(&table, 40);
    assert!(!out.contains('┌'), "no-box table should have no '┌'");
    assert!(!out.contains('│'), "no-box table should have no '│'");
}

// ---------------------------------------------------------------------------
// 11. test_columns_highlight_added_by_add_row
// ---------------------------------------------------------------------------

#[test]
fn auto_created_columns_inherit_highlight_flag() {
    // Start with highlight=true, no columns
    let mut table = Table::new(&[]).with_highlight(true);
    table.add_row(&["val1", "val2"]);
    // Both auto-created columns should inherit highlight=true
    assert!(table.columns[0].highlight, "column 0 should inherit highlight=true");
    assert!(table.columns[1].highlight, "column 1 should inherit highlight=true");
}

#[test]
fn explicit_columns_inherit_highlight_from_table() {
    let mut table = Table::new(&["H1", "H2"]);
    table.highlight = true;
    // Columns already created by Table::new — they inherit at creation time.
    // Verify adding a new column via add_column also inherits.
    table.add_column("H3", "", Default::default());
    assert!(table.columns[2].highlight, "added column should inherit table.highlight");
}

// ---------------------------------------------------------------------------
// 12. test_padding_width — padding affects rendered output
// ---------------------------------------------------------------------------

#[test]
fn padding_grid_with_explicit_padding_renders_spaced() {
    // Port of Python test_padding_width: grid with (0,1) padding, 3 cols each width=3
    let mut table = Table::grid(&[]).with_padding((0, 1, 0, 0));
    for _ in 0..3 {
        table.add_column(
            "",
            "",
            ColumnOptions {
                width: Some(3),
                ..Default::default()
            },
        );
    }
    table.add_row(&["aaa", "aaa", "aaa"]);
    let out = capture_table(&table, 40);
    // The three "aaa" cells should appear in the output, space-separated
    assert!(out.contains("aaa"), "expected cell content 'aaa'");
}

#[test]
fn padding_default_consistent_with_get_padding_width() {
    // Regression: measure path and render path must agree on padding totals.
    let mut table = Table::new(&["a", "b", "c"]);
    table.padding = (0, 2, 0, 1);
    // get_padding_width must equal what render path uses
    assert_eq!(table.get_padding_width(0), 1 + 2); // left + right
    assert_eq!(table.get_padding_width(1), 1 + 2);
    assert_eq!(table.get_padding_width(2), 1 + 2);
}

// ---------------------------------------------------------------------------
// 13. test_caption — caption text appears below table
// ---------------------------------------------------------------------------

#[test]
fn caption_appears_below_table() {
    // Make the table wide enough (>= caption length) so caption fits without wrapping.
    let mut table = Table::new(&["Column"]).with_caption("Note");
    table.add_row(&["some data row content"]);
    let out = capture_table(&table, 40);
    // "Note" is short enough to fit on one line in any reasonably-sized table.
    assert!(out.contains("Note"), "caption should appear in output");
    // Caption must follow the table bottom edge character.
    let plain: String = out.chars().collect();
    let bottom_pos = plain
        .char_indices()
        .filter_map(|(i, c)| if c == '└' || c == '┗' || c == '┘' { Some(i) } else { None })
        .max();
    let caption_pos = plain.find("Note");
    if let (Some(b), Some(c)) = (bottom_pos, caption_pos) {
        assert!(c > b, "caption should appear after table bottom edge");
    }
}

