//! Demonstrates that gilt widgets implement `Display`, so they work with
//! `println!()`, `format!()`, and anywhere Rust expects `Display`.
//!
//! **No Console is used in this example.** The whole point is that widgets
//! render themselves to plain text via the standard `Display` trait.
//!
//! Run with: `cargo run --example display_trait`

use gilt::panel::Panel;
use gilt::style::Style;
use gilt::table::Table;
use gilt::text::Text;
use gilt::tree::Tree;

fn main() {
    // ── Table with println! ──────────────────────────────────────────────
    println!("=== Table via println! (default 80-column width) ===\n");

    let mut table = Table::new(&["Language", "Typing", "Year"]);
    table.title = Some("Programming Languages".to_string());
    table.add_row(&["Rust", "Static / Strong", "2015"]);
    table.add_row(&["Python", "Dynamic / Strong", "1991"]);
    table.add_row(&["Go", "Static / Strong", "2009"]);
    table.add_row(&["TypeScript", "Static / Gradual", "2012"]);

    println!("{table}");

    // ── Table with custom width via format specifier ─────────────────────
    println!("\n=== Table with {{:60}} width specifier ===\n");

    println!("{table:60}");

    // ── Panel with println! ──────────────────────────────────────────────
    println!("\n=== Panel via println! ===\n");

    let content = Text::new(
        "Widgets that implement Display can be used anywhere\n\
         Rust expects a string — println!, format!, write!, etc.",
        Style::null(),
    );
    let mut panel = Panel::new(content);
    panel.title = Some(Text::new("Display Trait", Style::null()));

    println!("{panel}");

    // ── Tree with format! ────────────────────────────────────────────────
    println!("\n=== Tree captured via format! ===\n");

    let mut tree = Tree::new(Text::new("gilt/", Style::null()));
    {
        let src = tree.add(Text::new("src/", Style::null()));
        src.add(Text::new("lib.rs", Style::null()));
        src.add(Text::new("console.rs", Style::null()));
        src.add(Text::new("styled_str.rs", Style::null()));
    }
    tree.add(Text::new("Cargo.toml", Style::null()));
    tree.add(Text::new("README.md", Style::null()));

    let tree_string = format!("{tree}");

    println!("Tree as a String ({} bytes):", tree_string.len());
    println!("{tree_string}");

    // ── Combining widgets into a single formatted string ─────────────────
    println!("\n=== Combined widgets in a single String ===\n");

    let mut small_table = Table::new(&["Key", "Value"]);
    small_table.add_row(&["Name", "gilt"]);
    small_table.add_row(&["Version", "0.1.0"]);
    small_table.add_row(&["License", "MIT"]);

    let table_str = format!("{small_table:50}");
    let combined = format!(
        "Project Info:\n{table_str}\n\nDirectory Tree:\n{tree}",
        tree = tree_string,
    );

    println!("{combined}");

    // ── Proof: no ANSI codes in output ───────────────────────────────────
    println!("\n=== Proof: output is plain text (no ANSI escape codes) ===\n");

    let output = format!("{small_table:40}");
    let has_ansi = output.contains('\x1b');
    println!("Contains ANSI escapes: {has_ansi}");
    println!(
        "First 80 chars (raw): {:?}",
        &output[..output.len().min(80)]
    );
}
