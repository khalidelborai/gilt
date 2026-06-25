//! Demonstrates the BarChart widget -- horizontal Unicode bar charts.
//!
//! Each row is `label  ███████▌░░  value`: a right-padded label, a
//! proportional bar (full blocks plus a fractional partial block for sub-cell
//! precision over a `░` track), and an optional value.
//!
//! Run with: `cargo run --example barchart`

use gilt::barchart::BarChart;
use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;

fn main() {
    let mut console = Console::builder()
        .width(72)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Basic chart, auto-scaled to the largest value -----------------------
    console.print(&Rule::with_title("Languages by stars (auto scale)"));

    let langs = BarChart::from_pairs([
        ("Rust", 81_000.0),
        ("Go", 119_000.0),
        ("Zig", 33_000.0),
        ("Python", 59_000.0),
    ])
    .with_width(60)
    .with_bar_style(Style::parse("cyan"))
    .with_label_style(Style::parse("bold"))
    .with_value_style(Style::parse("dim"));
    console.print(&langs);

    // -- Explicit max (0..100) keeps every bar on the same percentage scale --
    console.print(&Rule::with_title("Disk usage % (fixed max = 100)"));

    let disks = BarChart::from_pairs([("/", 72.0), ("/home", 48.5), ("/var", 91.0), ("/tmp", 6.0)])
        .with_width(60)
        .with_max(100.0)
        .with_bar_style(Style::parse("green"));
    console.print(&disks);

    // -- Incremental building via push, values hidden ------------------------
    console.print(&Rule::with_title("Votes (built with push, no values)"));

    let mut votes = BarChart::new()
        .with_width(60)
        .with_show_values(false)
        .with_bar_style(Style::parse("magenta"));
    votes.push("Alice", 12.0);
    votes.push("Bob", 7.0);
    votes.push("Carol", 19.0);
    console.print(&votes);

    // -- Display trait (via println!) ----------------------------------------
    console.print(&Rule::with_title("Display trait (via println!)"));

    let quick = BarChart::from_pairs([("cpu", 63.0), ("mem", 41.0), ("net", 88.0)])
        .with_width(48)
        .with_max(100.0);
    println!("{quick}");
}
