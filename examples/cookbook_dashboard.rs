//! Cookbook: CLI Server Dashboard
//!
//! A polished server monitoring dashboard combining panels, tables,
//! sparklines, rules, and styled text to create a real-world
//! ops-style terminal UI.
//!
//! Run with: `cargo run --example cookbook_dashboard`

use gilt::bar::Bar;
use gilt::box_chars::{DOUBLE, HEAVY, ROUNDED};
use gilt::color::Color;
use gilt::prelude::*;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── Header ──────────────────────────────────────────────────────────
    let header = Panel::new(Text::new(
        "GILT OPS DASHBOARD  --  myapp v3.2.1\nMonitoring 6 servers across 3 regions",
        Style::null(),
    ))
    .with_title(Text::styled(
        "Server Monitor",
        Style::parse("bold white").unwrap(),
    ))
    .with_subtitle(Text::styled(
        "refreshed just now",
        Style::parse("dim").unwrap(),
    ))
    .with_border_style(Style::parse("bright_cyan").unwrap())
    .with_box_chars(&DOUBLE);
    console.print(&header);

    // ── Server Status Table ─────────────────────────────────────────────
    console
        .print(&Rule::with_title("Server Fleet").with_style(Style::parse("bright_blue").unwrap()));

    let mut table = Table::new(&["Server", "Region", "CPU %", "Mem %", "Uptime", "Status"]);
    table.title = Some("Fleet Overview".to_string());
    table.title_style = "bold".to_string();
    table.header_style = "bold bright_white on grey23".to_string();
    table.border_style = "bright_blue".to_string();
    table.box_chars = Some(&ROUNDED);

    // Server data: (name, region, cpu, mem, uptime, status)
    let servers: &[(&str, &str, &str, &str, &str, &str)] = &[
        (
            "web-01",
            "us-east",
            "23",
            "41",
            "45d 12h",
            "[bold green]OK[/bold green]",
        ),
        (
            "web-02",
            "us-east",
            "67",
            "58",
            "45d 12h",
            "[bold green]OK[/bold green]",
        ),
        (
            "web-03",
            "eu-west",
            "82",
            "71",
            "12d  3h",
            "[bold yellow]WARN[/bold yellow]",
        ),
        (
            "api-01",
            "us-east",
            "45",
            "33",
            "30d  8h",
            "[bold green]OK[/bold green]",
        ),
        (
            "api-02",
            "eu-west",
            "91",
            "85",
            " 2d  1h",
            "[bold red]CRIT[/bold red]",
        ),
        (
            "db-01",
            "ap-south",
            "38",
            "62",
            "90d  0h",
            "[bold green]OK[/bold green]",
        ),
    ];

    for &(name, region, cpu, mem, uptime, status) in servers {
        table.add_row(&[name, region, cpu, mem, uptime, status]);
    }
    console.print(&table);

    // ── Request Rate Sparkline ──────────────────────────────────────────
    console.print(
        &Rule::with_title("Request Rate (last 60s)")
            .with_style(Style::parse("bright_blue").unwrap()),
    );

    // Simulated requests-per-second over the last 60 seconds
    let rps: Vec<f64> = vec![
        120.0, 135.0, 142.0, 138.0, 155.0, 160.0, 148.0, 170.0, 185.0, 190.0, 178.0, 165.0, 172.0,
        180.0, 195.0, 210.0, 225.0, 218.0, 205.0, 198.0, 215.0, 230.0, 245.0, 260.0, 255.0, 240.0,
        235.0, 250.0, 265.0, 270.0, 258.0, 245.0, 238.0, 242.0, 255.0, 268.0, 275.0, 280.0, 272.0,
        265.0, 258.0, 270.0, 282.0, 290.0, 285.0, 278.0, 268.0, 260.0, 255.0, 248.0, 240.0, 235.0,
        245.0, 258.0, 270.0, 278.0, 285.0, 290.0, 295.0, 300.0,
    ];

    let sparkline = Sparkline::new(&rps)
        .with_width(70)
        .with_min(100.0)
        .with_max(320.0)
        .with_style(Style::parse("bright_green").unwrap());

    let spark_label = Text::styled("  req/s  ", Style::parse("dim").unwrap());
    console.print(&spark_label);
    console.print(&sparkline);

    // Show min/max labels
    let min_val = rps.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = rps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg_val: f64 = rps.iter().sum::<f64>() / rps.len() as f64;
    let stats_line = format!(
        "  min: {:.0} req/s    avg: {:.0} req/s    max: {:.0} req/s    current: {:.0} req/s",
        min_val,
        avg_val,
        max_val,
        rps.last().unwrap()
    );
    console.print(&Text::styled(&stats_line, Style::parse("dim").unwrap()));

    // ── Resource Bars ───────────────────────────────────────────────────
    console.print(
        &Rule::with_title("Cluster Resources").with_style(Style::parse("bright_blue").unwrap()),
    );

    let resources: &[(&str, f64, &str)] = &[
        ("  CPU  (avg)", 57.7, "yellow"),
        ("  Memory    ", 58.3, "bright_magenta"),
        ("  Disk      ", 34.0, "bright_cyan"),
        ("  Network   ", 22.0, "bright_green"),
    ];

    for &(label, pct, color) in resources {
        let label_text = Text::styled(label, Style::parse("bold").unwrap());
        console.print(&label_text);
        let bar = Bar::new(100.0, 0.0, pct)
            .with_width(50)
            .with_color(Color::parse(color).unwrap());
        console.print(&bar);
        let pct_text = Text::styled(&format!("  {pct:>5.1}%"), Style::parse("bold").unwrap());
        console.print(&pct_text);
    }

    // ── Footer ──────────────────────────────────────────────────────────
    console.print(
        &Rule::new()
            .with_characters("\u{2550}")
            .with_style(Style::parse("bright_cyan").unwrap()),
    );

    let footer = Panel::fit(Text::styled(
        "4 OK  |  1 WARN  |  1 CRIT  |  6 total",
        Style::parse("bold").unwrap(),
    ))
    .with_border_style(Style::parse("dim").unwrap())
    .with_box_chars(&HEAVY);
    console.print(&footer);
}
