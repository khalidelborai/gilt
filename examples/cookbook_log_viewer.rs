//! Cookbook: Colored Log Viewer
//!
//! A rich log viewer that renders structured log entries with
//! level-appropriate coloring, IP/path highlighting, and a summary
//! panel. Demonstrates tables with pre-styled Text cells,
//! highlighters, and panels.
//!
//! Run with: `cargo run --example cookbook_log_viewer`

use gilt::box_chars::ROUNDED;
use gilt::highlighter::{Highlighter, ReprHighlighter};
use gilt::prelude::*;

/// A single log entry with timestamp, level, source, and message.
struct LogEntry {
    timestamp: &'static str,
    level: &'static str,
    source: &'static str,
    message: &'static str,
}

/// Return a (style_string, badge) pair for a given log level.
fn level_style(level: &str) -> (&str, &str) {
    match level {
        "INFO" => ("bold green", " INFO  "),
        "WARN" => ("bold yellow", " WARN  "),
        "ERROR" => ("bold red", " ERROR "),
        "DEBUG" => ("dim", " DEBUG "),
        _ => ("", " ????? "),
    }
}

fn main() {
    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── Header ──────────────────────────────────────────────────────────
    let header = Panel::new(Text::styled(
        "Application Log Viewer\nShowing last 10 entries from myapp.log",
        Style::parse("bold").unwrap(),
    ))
    .title(Text::styled(
        "Log Viewer",
        Style::parse("bold bright_white").unwrap(),
    ))
    .border_style(Style::parse("bright_green").unwrap());
    console.print(&header);

    // ── Sample log data ─────────────────────────────────────────────────
    let entries = vec![
        LogEntry {
            timestamp: "2026-02-09 08:12:01",
            level: "INFO",
            source: "server",
            message: "Listening on 0.0.0.0:8080",
        },
        LogEntry {
            timestamp: "2026-02-09 08:12:03",
            level: "INFO",
            source: "auth",
            message: "Connected to auth service at 10.0.1.5:6379",
        },
        LogEntry {
            timestamp: "2026-02-09 08:13:15",
            level: "DEBUG",
            source: "router",
            message: "GET /api/v1/users from 192.168.1.42",
        },
        LogEntry {
            timestamp: "2026-02-09 08:13:16",
            level: "INFO",
            source: "handler",
            message: "Served 200 OK for /api/v1/users in 12ms",
        },
        LogEntry {
            timestamp: "2026-02-09 08:14:02",
            level: "WARN",
            source: "cache",
            message: "Cache miss for key user:1337 on 10.0.2.10:11211",
        },
        LogEntry {
            timestamp: "2026-02-09 08:14:05",
            level: "DEBUG",
            source: "pool",
            message: "Connection pool at 8/20 active connections",
        },
        LogEntry {
            timestamp: "2026-02-09 08:15:30",
            level: "ERROR",
            source: "database",
            message: "Query timeout after 30s on 10.0.3.1:5432",
        },
        LogEntry {
            timestamp: "2026-02-09 08:15:31",
            level: "ERROR",
            source: "handler",
            message: "500 Internal Server Error for /api/v1/reports",
        },
        LogEntry {
            timestamp: "2026-02-09 08:16:00",
            level: "WARN",
            source: "monitor",
            message: "Memory usage at 87% on host web-03 (10.0.1.3)",
        },
        LogEntry {
            timestamp: "2026-02-09 08:16:45",
            level: "INFO",
            source: "deploy",
            message: "Health check passed for /healthz endpoint",
        },
    ];

    // ── Log Table ───────────────────────────────────────────────────────
    console.print(&Rule::with_title("Log Entries").style(Style::parse("bright_green").unwrap()));

    let mut table = Table::new(&["Timestamp", "Level", "Source", "Message"]);
    table.header_style = "bold bright_white on grey23".to_string();
    table.border_style = "dim".to_string();
    table.box_chars = Some(&ROUNDED);
    table.show_lines = true;

    let highlighter = ReprHighlighter::new();

    for entry in &entries {
        let (lstyle, badge) = level_style(entry.level);

        let ts = Text::styled(entry.timestamp, Style::parse("cyan").unwrap());
        let level_text = Text::styled(badge, Style::parse(lstyle).unwrap());
        let source = Text::styled(entry.source, Style::parse("bold bright_white").unwrap());

        // Apply highlighter to the message to color IPs, paths, numbers
        let message = highlighter.apply(entry.message);

        table.add_row_text(&[ts, level_text, source, message]);
    }

    console.print(&table);

    // ── Summary Panel ───────────────────────────────────────────────────
    console.print(&Rule::with_title("Summary").style(Style::parse("bright_green").unwrap()));

    let mut info_count = 0u32;
    let mut warn_count = 0u32;
    let mut error_count = 0u32;
    let mut debug_count = 0u32;

    for entry in &entries {
        match entry.level {
            "INFO" => info_count += 1,
            "WARN" => warn_count += 1,
            "ERROR" => error_count += 1,
            "DEBUG" => debug_count += 1,
            _ => {}
        }
    }

    let summary = format!(
        "[bold green]INFO[/bold green]:  {info_count}    \
         [bold yellow]WARN[/bold yellow]:  {warn_count}    \
         [bold red]ERROR[/bold red]: {error_count}    \
         [dim]DEBUG[/dim]: {debug_count}    \
         [bold]TOTAL[/bold]: {}",
        entries.len()
    );

    let summary_text = console.render_str(&summary, None, None, None);
    let summary_panel = Panel::fit(summary_text)
        .title(Text::styled("Level Counts", Style::parse("bold").unwrap()))
        .border_style(Style::parse("bright_green").unwrap());
    console.print(&summary_panel);
}
