//! Columns demo. Run: `cargo run --example columns`
//!
//! Equivalent to rich/examples/columns.py — without the network round-trip,
//! since gilt avoids pulling reqwest for an example.

use gilt::columns::Columns;
use gilt::console::Console;

fn main() {
    let users = [
        ("Alice Chen", "United States"),
        ("Lukas Müller", "Germany"),
        ("Yuki Tanaka", "Japan"),
        ("Olivia Smith", "United Kingdom"),
        ("Carlos Diaz", "Spain"),
        ("Priya Patel", "India"),
    ];
    let items: Vec<String> = users
        .iter()
        .map(|(n, c)| format!("[b]{n}[/]\n[yellow]{c}"))
        .collect();
    Console::default().print(&Columns::from_items(items).with_expand(true));
}
