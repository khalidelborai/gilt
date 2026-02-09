//! Demonstrates gilt's Diff widget — colored text diffs in unified and side-by-side styles.

use gilt::console::Console;
use gilt::diff::{Diff, DiffStyle};
use gilt::rule::Rule;

fn main() {
    let mut console = Console::builder()
        .width(90)
        .force_terminal(true)
        .no_color(false)
        .build();

    let old_code = r#"fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    greet("world");
}
"#;

    let new_code = r#"fn greet(name: &str) {
    println!("Hi there, {}!", name);
    println!("Welcome!");
}

fn main() {
    greet("Rust");
    greet("World");
}
"#;

    // -- 1. Unified Diff -----------------------------------------------------

    console.print(&Rule::with_title("Unified Diff"));

    let diff = Diff::new(old_code, new_code)
        .with_labels("a/main.rs", "b/main.rs")
        .with_context(3);

    console.print(&diff);

    // -- 2. Side-by-Side Diff ------------------------------------------------

    console.print(&Rule::with_title("Side-by-Side Diff"));

    let diff = Diff::side_by_side(old_code, new_code).with_labels("old/main.rs", "new/main.rs");

    console.print(&diff);

    // -- 3. Custom Labels and Context ----------------------------------------

    console.print(&Rule::with_title("Custom Context (1 line)"));

    let old_config = "host = localhost\nport = 8080\ndebug = false\nlog = info\n";
    let new_config = "host = localhost\nport = 9090\ndebug = true\nlog = debug\n";

    let diff = Diff::new(old_config, new_config)
        .with_labels("config.old.toml", "config.new.toml")
        .with_style(DiffStyle::Unified)
        .with_context(1);

    console.print(&diff);
}
