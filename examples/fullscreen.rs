//! Fullscreen terminal rendering with a centered message.
//!
//! Run with: `cargo run --example fullscreen`
//!
//! Demonstrates entering the alternate screen buffer, rendering styled
//! content, and restoring the original screen on exit. Uses the Screen
//! widget to fill the terminal dimensions.

use std::thread;
use std::time::Duration;

use gilt::console::Console;
use gilt::screen::Screen;
use gilt::style::Style;
use gilt::text::Text;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .height(24)
        .force_terminal(true)
        .no_color(false)
        .build();

    // Enter alternate screen and hide cursor.
    console.enter_screen(true);
    console.clear();

    // Build a multi-line message to display fullscreen.
    let mut content = Text::empty();

    // Add some blank lines to push content toward the center.
    for _ in 0..8 {
        content.append_str("\n", None);
    }

    // Centered title.
    let title = format!("{:^80}\n", "Gilt Fullscreen Demo");
    content.append_str(&title, Some(Style::parse("bold magenta").unwrap()));

    content.append_str(&format!("{:^80}\n", ""), None);

    let subtitle = format!("{:^80}\n", "A Rust port of Python's rich library");
    content.append_str(&subtitle, Some(Style::parse("italic cyan").unwrap()));

    content.append_str(&format!("{:^80}\n", ""), None);

    let tagline = format!("{:^80}\n", "Beautiful terminal formatting, blazing fast.");
    content.append_str(&tagline, None);

    content.append_str(&format!("{:^80}\n", ""), None);
    content.append_str(&format!("{:^80}\n", ""), None);

    let exit_msg = format!("{:^80}", "Returning to normal screen in 2 seconds...");
    content.append_str(&exit_msg, Some(Style::parse("dim").unwrap()));

    // Use the Screen widget to fill exactly width x height.
    let screen = Screen::new(content)
        .with_style(Style::parse("on black").unwrap());

    console.print(&screen);

    // Hold the screen so the user can read it.
    thread::sleep(Duration::from_secs(2));

    // Restore the original screen.
    console.exit_screen(true);

    println!("Returned to normal screen.");
}
