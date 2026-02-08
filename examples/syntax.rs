//! Syntax highlighting example -- demonstrates the Syntax widget.
//!
//! Run with: `cargo run --example syntax`

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::syntax::Syntax;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .build();

    // -- Rust snippet with line numbers and a highlighted line ----------------

    console.print(&Rule::with_title("Rust (line 3 highlighted)"));

    let rust_code = r#"use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<&str, i32> = HashMap::new();
    scores.insert("Alice", 100);
    scores.insert("Bob", 85);

    for (name, score) in &scores {
        println!("{name}: {score}");
    }
}"#;

    let rust_syntax = Syntax::new(rust_code, "rs")
        .with_line_numbers(true)
        .with_theme("base16-ocean.dark")
        .with_highlight_lines(vec![3]);

    console.print(&rust_syntax);

    // -- Python snippet for contrast -----------------------------------------

    console.print(&Rule::with_title("Python"));

    let python_code = r#"from dataclasses import dataclass

@dataclass
class Point:
    x: float
    y: float

    def distance(self, other: "Point") -> float:
        return ((self.x - other.x) ** 2 + (self.y - other.y) ** 2) ** 0.5

if __name__ == "__main__":
    a = Point(0.0, 0.0)
    b = Point(3.0, 4.0)
    print(f"Distance: {a.distance(b)}")"#;

    let python_syntax = Syntax::new(python_code, "py")
        .with_line_numbers(true)
        .with_theme("base16-ocean.dark");

    console.print(&python_syntax);
}
