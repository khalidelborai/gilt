//! Demonstrates gilt's Columns widget — auto-fitting grid layout of items.

use gilt::columns::Columns;
use gilt::console::Console;
use gilt::panel::Panel;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Programming Languages in Columns (plain strings) --------------------

    console.print(&Rule::with_title("Programming Languages (Columns)"));

    let languages = [
        "Rust", "Python", "Go", "TypeScript", "Java", "C++",
        "Ruby", "Swift", "Kotlin", "Haskell", "Elixir", "Zig",
    ];

    let mut cols = Columns::new();
    for lang in &languages {
        cols.add_renderable(lang);
    }

    console.print(&cols);

    // -- Equal-width columns ------------------------------------------------

    console.print(&Rule::with_title("Equal-Width Columns"));

    let mut cols = Columns::new().with_equal(true).with_expand(true);
    for lang in &languages {
        cols.add_renderable(lang);
    }

    console.print(&cols);

    // -- Each language wrapped in a Panel -----------------------------------

    console.print(&Rule::with_title("Languages in Panels"));

    // Since Columns stores items as strings and renders them via render_str,
    // we can't directly add Panel renderables. Instead we demonstrate Columns
    // with descriptive labels, then show a separate panel grid below.

    let descriptions: &[(&str, &str)] = &[
        ("Rust",       "Systems programming with safety guarantees"),
        ("Python",     "Versatile scripting and data science"),
        ("Go",         "Concurrent server software made simple"),
        ("TypeScript", "JavaScript with static types"),
        ("Java",       "Enterprise & Android powerhouse"),
        ("C++",        "High-performance systems & games"),
    ];

    // Print each description inside its own panel
    for (name, desc) in descriptions {
        let content = Text::new(desc, Style::null());
        let panel = Panel::fit(content)
            .title(Text::new(name, Style::null()));

        console.print(&panel);
    }

    // -- Column-first ordering ---------------------------------------------

    console.print(&Rule::with_title("Column-First Ordering (fill top-to-bottom)"));

    let mut cols = Columns::new()
        .with_column_first(true)
        .with_width(12);
    for lang in &languages {
        cols.add_renderable(lang);
    }

    console.print(&cols);
}
