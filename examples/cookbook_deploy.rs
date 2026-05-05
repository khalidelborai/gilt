//! Cookbook: Simulated Deployment Script
//!
//! A polished deployment output that shows a header, a checklist
//! table of deployment steps, a tree of deployment targets, and a
//! success banner. Demonstrates panels, tables with styled cells,
//! trees, and rules working together.
//!
//! Run with: `cargo run --example cookbook_deploy`

use gilt::box_chars::{DOUBLE, HEAVY, ROUNDED};
use gilt::prelude::*;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── Deploy Header ───────────────────────────────────────────────────
    let banner = Gradient::two_color(
        "  Deploying myapp v2.1.0  ",
        Color::from_rgb(0, 180, 255),
        Color::from_rgb(100, 255, 100),
    )
    .with_style(Style::parse("bold"));

    let header = Panel::new(Text::new(
        "Application: myapp\nVersion:     2.1.0\nEnvironment: production\nInitiated:   2026-02-09 14:30:00 UTC",
        Style::null(),
    ))
    .with_title(Text::styled("Deployment", "bold bright_white"))
    .with_border_style(Style::parse("bright_cyan"))
    .with_box_chars(&DOUBLE);

    console.print(&banner);
    console.print(&header);

    // ── Deployment Steps Checklist ──────────────────────────────────────
    console.print(&Rule::with_title("Pipeline Steps").with_style(Style::parse("bright_cyan")));

    let mut table = Table::new(&["#", "Step", "Duration", "Status"]);
    table.header_style = "bold bright_white on grey23".to_string();
    table.border_style = "bright_cyan".to_string();
    table.box_chars = Some(&ROUNDED);

    // Each step: (number, name, duration, passed)
    let steps: &[(&str, &str, &str, bool)] = &[
        ("1", "Build release binary", "1m 42s", true),
        ("2", "Run test suite (284 tests)", "0m 38s", true),
        ("3", "Package artifacts", "0m 12s", true),
        ("4", "Upload to artifact store", "0m 24s", true),
        ("5", "Rolling deploy to targets", "2m 05s", true),
        ("6", "Health check verification", "0m 15s", true),
    ];

    for &(num, name, duration, passed) in steps {
        let num_text = Text::styled(num, "dim");
        let name_text = Text::styled(name, "bold");
        let dur_text = Text::styled(duration, "cyan");
        let status_text = if passed {
            Text::styled("\u{2714} passed", "bold green")
        } else {
            Text::styled("\u{2718} failed", "bold red")
        };
        table.add_row_text(&[num_text, name_text, dur_text, status_text]);
    }

    console.print(&table);

    // ── Deployment Targets Tree ─────────────────────────────────────────
    console
        .print(&Rule::with_title("Target Infrastructure").with_style(Style::parse("bright_cyan")));

    let bold_cyan = Style::parse("bold bright_cyan");
    let bold_blue = Style::parse("bold blue");
    let green = Style::parse("green");
    let dim = Style::parse("dim");

    let mut tree = Tree::new(Text::styled_with("production", bold_cyan.clone()))
        .with_guide_style(Style::parse("bright_cyan"));

    // us-east region
    {
        let region = tree.add(Text::styled_with("us-east-1", bold_blue.clone()));
        let web = region.add(Text::styled_with("web-tier", dim.clone()));
        web.add(Text::styled_with("web-01  \u{2714} healthy", green.clone()));
        web.add(Text::styled_with("web-02  \u{2714} healthy", green.clone()));
        let api = region.add(Text::styled_with("api-tier", dim.clone()));
        api.add(Text::styled_with("api-01  \u{2714} healthy", green.clone()));
    }

    // eu-west region
    {
        let region = tree.add(Text::styled_with("eu-west-1", bold_blue.clone()));
        let web = region.add(Text::styled_with("web-tier", dim.clone()));
        web.add(Text::styled_with("web-03  \u{2714} healthy", green.clone()));
        let api = region.add(Text::styled_with("api-tier", dim.clone()));
        api.add(Text::styled_with("api-02  \u{2714} healthy", green.clone()));
    }

    // ap-south region
    {
        let region = tree.add(Text::styled_with("ap-south-1", bold_blue.clone()));
        let db = region.add(Text::styled_with("data-tier", dim.clone()));
        db.add(Text::styled_with("db-01   \u{2714} healthy", green.clone()));
    }

    console.print(&tree);

    // ── Deployment Summary ──────────────────────────────────────────────
    console.print(
        &Rule::new()
            .with_characters("\u{2550}")
            .with_style(Style::parse("bright_cyan")),
    );

    let total_duration = "4m 36s";
    let summary_content = format!(
        "\u{2714}  Deployment complete!\n\n\
         All 6 pipeline steps passed.\n\
         7 targets healthy across 3 regions.\n\
         Total duration: {total_duration}"
    );

    let success_panel = Panel::new(Text::styled(&summary_content, "bold green"))
        .with_title(Text::styled(" SUCCESS ", "bold bright_white on green"))
        .with_border_style(Style::parse("bold green"))
        .with_box_chars(&HEAVY);
    console.print(&success_panel);
}
