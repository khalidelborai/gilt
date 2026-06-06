//! Table with column-spanning cells via `add_row_spanned`.
//!
//! Demonstrates:
//! - `Table::new(&headers)` — a standard bordered table.
//! - `table.add_row(&cells)` — a plain row with one cell per column.
//! - `table.add_row_spanned(&cells, &spans)` — a row where `cells[i]` spans
//!   `spans[i]` physical columns.
//!
//! Run with: cargo run --example table_colspan

use gilt::table::Table;

fn main() {
    // A 4-column conference schedule table.
    let mut table = Table::new(&["Time", "Track A", "Track B", "Track C"]);

    // Regular rows — one cell per column.
    table.add_row(&["09:00", "Keynote", "", ""]);

    // This cell spans all three track columns ("All Tracks" fills cols 1-3).
    table.add_row_spanned(&["10:00", "Morning Break (all tracks)"], &[1, 3]);

    table.add_row(&["11:00", "Systems", "Web Dev", "Data Eng"]);
    table.add_row(&["12:00", "Rustaceans", "Async I/O", "ML in Rust"]);

    // This row has one cell per column plus a 2-column span.
    table.add_row_spanned(&["13:00", "Lunch", "Joint Workshop"], &[1, 1, 2]);

    table.add_row(&["14:00", "Unsafe", "WASM", "Embedded"]);

    // Span the first two columns together and the last two together.
    table.add_row_spanned(
        &["15:30", "Coffee + Networking", "Lightning Talks"],
        &[1, 2, 1],
    );

    table.add_row(&["16:00", "Panel", "Panel", "Panel"]);
    table.add_row_spanned(&["17:00", "Closing Keynote (all)"], &[1, 3]);

    // Print to stdout.
    print!("{}", table);

    // Sanity check: the rendered string contains our spanned content.
    let rendered = format!("{}", table);
    assert!(
        rendered.contains("Morning Break"),
        "spanned cell content must appear"
    );
    assert!(
        rendered.contains("Closing Keynote"),
        "closing spanned row must appear"
    );

    println!("\n[assertion passed] spanned cells rendered correctly");
}
