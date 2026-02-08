//! Screen context demo — "Don't Panic!" centered on screen.
//!
//! Run with: `cargo run --example screen`
//!
//! Port of Python rich's screen.py demo. Uses the Screen widget to
//! fill the entire terminal with a centered message, similar to the
//! Hitchhiker's Guide to the Galaxy.

use std::thread;
use std::time::Duration;

use gilt::console::Console;
use gilt::screen::Screen;
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let width = 80;
    let height = 24;

    let mut console = Console::builder()
        .width(width)
        .height(height)
        .force_terminal(true)
        .no_color(false)
        .build();

    // Enter the alternate screen buffer.
    console.enter_screen(true);
    console.clear();

    // Build the centered "Don't Panic!" message.
    let mut content = Text::empty();

    // Vertical centering: push content down to roughly the middle.
    let vertical_pad = height / 2 - 2;
    for _ in 0..vertical_pad {
        content.append_str("\n", None);
    }

    // The famous message in large, friendly letters.
    let message = format!("{:^width$}\n", "DON'T PANIC!", width = width);
    content.append_str(
        &message,
        Some(Style::parse("bold yellow on blue").unwrap()),
    );

    content.append_str(&format!("{:^width$}\n", "", width = width), None);

    let sub = format!(
        "{:^width$}\n",
        "-- The Hitchhiker's Guide to the Galaxy",
        width = width
    );
    content.append_str(&sub, Some(Style::parse("italic white").unwrap()));

    content.append_str(&format!("{:^width$}\n", "", width = width), None);

    let hint = format!("{:^width$}", "Exiting in 3 seconds...", width = width);
    content.append_str(&hint, Some(Style::parse("dim").unwrap()));

    // Render as a full screen.
    let screen = Screen::new(content);
    console.print(&screen);

    thread::sleep(Duration::from_secs(3));

    // Restore.
    console.exit_screen(true);
    println!("Back to reality.");
}
