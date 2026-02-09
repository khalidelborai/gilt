//! Port of rich's justify2.py — panel justification demo.
//!
//! Creates a small non-expanding panel and prints it four times with
//! different justify settings: default, left, center, right.

use gilt::console::Console;
use gilt::panel::Panel;
use gilt::style::Style;
use gilt::text::{JustifyMethod, Text};

fn main() {
    let mut console = Console::builder()
        .width(20)
        .force_terminal(true)
        .no_color(false)
        .build();

    let style = "bold white on blue";

    // Create a small non-expanding panel with styled content
    let content = Text::new("Gilt", Style::null());
    let panel =
        Panel::fit(content).with_style(Style::parse("on red").unwrap_or_else(|_| Style::null()));

    // Default justify (no explicit justify)
    console.print_styled(&panel, Some(style), None, None, false, true, false);

    // Left justify
    console.print_styled(
        &panel,
        Some(style),
        Some(JustifyMethod::Left),
        None,
        false,
        true,
        false,
    );

    // Center justify
    console.print_styled(
        &panel,
        Some(style),
        Some(JustifyMethod::Center),
        None,
        false,
        true,
        false,
    );

    // Right justify
    console.print_styled(
        &panel,
        Some(style),
        Some(JustifyMethod::Right),
        None,
        false,
        true,
        false,
    );
}
