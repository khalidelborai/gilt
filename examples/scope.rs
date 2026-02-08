//! Scope demo -- renders key-value pairs in a styled panel.
//!
//! Run with: `cargo run --example scope`
//!
//! Port of Python rich's scope functionality. Demonstrates the Scope widget
//! which displays variables and their values in a bordered panel, useful
//! for debugging and inspection.

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::scope::Scope;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- 1. Simple Scope ------------------------------------------------------

    console.print(&Rule::with_title("Simple Scope"));

    let scope = Scope::from_pairs(&[("name", "Alice"), ("age", "30"), ("language", "Rust")]);
    console.print(&scope);

    // -- 2. Scope with Custom Title -------------------------------------------

    console.print(&Rule::with_title("Titled Scope"));

    let scope = Scope::from_pairs(&[
        ("host", "localhost"),
        ("port", "8080"),
        ("debug", "true"),
        ("workers", "4"),
    ])
    .title("Server Config");
    console.print(&scope);

    // -- 3. Sorted Keys -------------------------------------------------------

    console.print(&Rule::with_title("Sorted Keys"));

    let scope = Scope::from_pairs(&[
        ("zebra", "last alphabetically"),
        ("alpha", "first alphabetically"),
        ("middle", "somewhere in between"),
    ])
    .sort_keys(true)
    .title("Sorted Variables");
    console.print(&scope);

    // -- 4. Application State -------------------------------------------------

    console.print(&Rule::with_title("Application State"));

    let scope = Scope::from_pairs(&[
        ("version", "0.1.0"),
        ("build", "release"),
        ("target", "x86_64-unknown-linux-gnu"),
        ("features", "color, unicode, spinners"),
        ("log_level", "info"),
        ("max_retries", "3"),
        ("timeout_ms", "5000"),
    ])
    .title("App State")
    .sort_keys(true);
    console.print(&scope);

    // -- 5. Empty Scope -------------------------------------------------------

    console.print(&Rule::with_title("Empty Scope"));

    let scope = Scope::new(vec![]).title("No Variables");
    console.print(&scope);
}
