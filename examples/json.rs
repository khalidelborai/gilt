//! JSON pretty-printing example -- demonstrates the Json widget.
//!
//! Run with: `cargo run --example json`

use gilt::console::Console;
use gilt::json::{Json, JsonOptions};
use gilt::rule::Rule;

fn main() {
    let mut console = Console::builder().width(80).force_terminal(true).build();

    let json_str = r#"{
        "name": "Gilt",
        "version": "0.1.0",
        "description": "A rich terminal rendering library for Rust",
        "features": ["syntax", "markdown", "json", "tables"],
        "metadata": {
            "stars": 42,
            "active": true,
            "license": null,
            "contributors": [
                {"name": "Alice", "commits": 120},
                {"name": "Bob", "commits": 85}
            ]
        },
        "pi": 3.14159
    }"#;

    // -- 2-space indent (default) -------------------------------------------

    console.print(&Rule::with_title("JSON (2-space indent)"));
    let json = Json::new(json_str, JsonOptions::default()).unwrap();
    console.print(&json);

    // -- 4-space indent -----------------------------------------------------

    console.print(&Rule::with_title("JSON (4-space indent)"));
    let wide_opts = JsonOptions {
        indent: Some(4),
        ..Default::default()
    };
    let json_wide = Json::new(json_str, wide_opts).unwrap();
    console.print(&json_wide);
}
