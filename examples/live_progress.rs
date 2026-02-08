//! Live display with a progress-like table updated each tick.
//!
//! Run with: `cargo run --example live_progress`
//!
//! Demonstrates combining the Live display widget with a manually
//! built progress table, showing how `Live::update` can render
//! arbitrary Text content with a live-updating display.

use std::thread;
use std::time::Duration;

use gilt::console::Console;
use gilt::live::Live;
use gilt::style::Style;
use gilt::text::Text;

/// Format a simple text-based progress bar for embedding in a Text.
fn bar_text(completed: f64, total: f64, width: usize) -> String {
    let pct = (completed / total).min(1.0);
    let filled = (pct * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}] {:>5.1}%",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        pct * 100.0
    )
}

/// Build a styled Text renderable showing multiple task rows.
fn build_display(tasks: &[(&str, f64, f64)]) -> Text {
    let header_style = Style::parse("bold cyan").unwrap();
    let mut text = Text::styled("  Task Progress Dashboard\n\n", header_style);

    for (name, completed, total) in tasks {
        let bar = bar_text(*completed, *total, 20);
        let line = format!("  {:<24} {}\n", name, bar);

        let style = if *completed >= *total {
            Style::parse("green").unwrap()
        } else {
            Style::null()
        };

        text.append_str(&line, Some(style));
    }

    text.append_str(
        "\n  Press Ctrl-C to cancel",
        Some(Style::parse("dim").unwrap()),
    );

    text
}

fn main() {
    let console = Console::builder()
        .force_terminal(true)
        .no_color(false)
        .build();

    let mut tasks: Vec<(&str, f64, f64)> = vec![
        ("Indexing files", 0.0, 500.0),
        ("Building cache", 0.0, 300.0),
        ("Optimizing", 0.0, 800.0),
        ("Verifying", 0.0, 200.0),
    ];

    let speeds = [12.0, 8.0, 15.0, 5.0];

    let initial = build_display(&tasks);
    let mut live = Live::new(initial)
        .with_console(console)
        .with_auto_refresh(false);

    live.start();

    loop {
        // Advance each task.
        for (i, task) in tasks.iter_mut().enumerate() {
            task.1 = (task.1 + speeds[i]).min(task.2);
        }

        let display = build_display(&tasks);
        live.update(display, true);

        // Check if all tasks are complete.
        let all_done = tasks
            .iter()
            .all(|(_, completed, total)| *completed >= *total);
        if all_done {
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }

    live.stop();

    println!("\nAll tasks finished!");
}
