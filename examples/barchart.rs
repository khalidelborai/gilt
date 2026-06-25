//! `BarChart` — horizontal Unicode bar charts with sub-cell precision.
//!
//! Each row is `label  ███████▌  value`: a right-aligned label, a proportional
//! bar drawn with full blocks plus an eighth-block partial cell for fractional
//! precision, and an optional value. This demo frames each chart in a `Panel`
//! (gilt 2.0 lets a `Panel` hold any `Renderable`) for a dashboard feel.
//!
//! Run with: `cargo run --example barchart`

use gilt::barchart::BarChart;
use gilt::color::Color;
use gilt::console::Console;
use gilt::gradient::Gradient;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::Text;

/// A subtle Dracula-grey panel border.
fn border() -> Style {
    Style::parse("#44475a")
}

fn titled(title: &str, color: &str, chart: BarChart) -> Panel {
    Panel::new(chart)
        .with_title(Text::new(
            &format!(" {title} "),
            Style::parse(&format!("bold {color}")),
        ))
        .with_border_style(border())
}

fn main() {
    let mut console = Console::builder()
        .width(72)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Gradient::new(
        "gilt · BarChart — sub-cell precision, any data, any style",
        vec![
            Color::from_rgb(189, 147, 249),
            Color::from_rgb(255, 121, 198),
            Color::from_rgb(139, 233, 253),
        ],
    ));
    console.line(1);

    // 1. Auto-scaled to the largest value (cyan).
    let stars = BarChart::from_pairs([
        ("ratatui", 12_400.0),
        ("clap", 15_900.0),
        ("indicatif", 4_300.0),
        ("crossterm", 3_100.0),
        ("gilt", 9_001.0),
    ])
    .with_width(54)
    .with_bar_style(Style::parse("#8be9fd"))
    .with_label_style(Style::parse("bold #f8f8f2"))
    .with_value_style(Style::parse("dim"));
    console.print(&titled("GitHub stars · auto-scaled", "#8be9fd", stars));
    console.line(1);

    // 2. Fixed 0..100 scale so bars read as true percentages (green).
    let disk = BarChart::from_pairs([
        ("/", 72.0),
        ("/home", 48.5),
        ("/var", 93.0),
        ("/tmp", 6.0),
        ("/boot", 31.0),
    ])
    .with_width(54)
    .with_max(100.0)
    .with_bar_style(Style::parse("#50fa7b"))
    .with_label_style(Style::parse("bold"))
    .with_value_style(Style::parse("dim #50fa7b"));
    console.print(&titled("disk usage % · fixed max 100", "#50fa7b", disk));
    console.line(1);

    // 3. Built incrementally with push(); fractional values show the eighth-block
    //    partial cell that gives the bars their sub-cell precision (pink).
    let mut build = BarChart::new()
        .with_width(54)
        .with_bar_style(Style::parse("#ff79c6"))
        .with_label_style(Style::parse("bold"))
        .with_value_style(Style::parse("dim"));
    build.push("parse", 0.8);
    build.push("typecheck", 4.25);
    build.push("codegen", 9.7);
    build.push("link", 2.1);
    console.print(&titled(
        "build phase · seconds (push-built)",
        "#ff79c6",
        build,
    ));
    console.line(1);

    // 4. Values hidden — clean bars only (yellow).
    let poll = BarChart::from_pairs([("Rust", 58.0), ("Go", 22.0), ("Zig", 13.0), ("C", 7.0)])
        .with_width(54)
        .with_max(100.0)
        .with_show_values(false)
        .with_bar_style(Style::parse("#f1fa8c"))
        .with_label_style(Style::parse("bold"));
    console.print(&titled("favourite language · bars only", "#f1fa8c", poll));

    console.line(1);
    console.print(&Text::new(
        "  every bar is a Renderable — drop one in a Panel, Table cell, or Live display.",
        Style::parse("italic #6272a4"),
    ));
}
