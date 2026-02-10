//! Demonstrates the Badge widget -- status indicators for modern CLIs.
//!
//! Shows various badge styles: success, error, warning, info, and neutral,
//! with both square and rounded variants, and custom icons.
//!
//! Run with: `cargo run --example badge`

use gilt::badge::{Badge, BadgeStyle};
use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Standard badge styles -----------------------------------------------
    console.print(&Rule::with_title("Standard Badge Styles"));

    // Success badge (green, ✓ icon)
    let success = Badge::success("Build Passed");
    console.print(&success);
    console.print_text(""); // spacer

    // Error badge (red, ✗ icon)
    let error = Badge::error("Tests Failed");
    console.print(&error);
    console.print_text("");

    // Warning badge (yellow, ⚠ icon)
    let warning = Badge::warning("Deprecated");
    console.print(&warning);
    console.print_text("");

    // Info badge (blue, ℹ icon)
    let info = Badge::info("New Feature");
    console.print(&info);
    console.print_text("");

    // Neutral badge (gray, no icon)
    let neutral = Badge::new("Draft");
    console.print(&neutral);
    console.print_text("");

    // -- Rounded badges -------------------------------------------------------
    console.print(&Rule::with_title("Rounded Badges"));

    let rounded_success = Badge::success("Published").rounded(true);
    console.print(&rounded_success);
    console.print_text("");

    let rounded_error = Badge::error("Error").rounded(true);
    console.print(&rounded_error);
    console.print_text("");

    let rounded_info = Badge::info("Info").rounded(true);
    console.print(&rounded_info);
    console.print_text("");

    // -- Custom icons ---------------------------------------------------------
    console.print(&Rule::with_title("Custom Icons"));

    let starred = Badge::new("Starred").icon("★").style(BadgeStyle::Warning);
    console.print(&starred);
    console.print_text("");

    let locked = Badge::new("Locked").icon("🔒").style(BadgeStyle::Info);
    console.print(&locked);
    console.print_text("");

    let rocket = Badge::new("Deployed").icon("🚀").style(BadgeStyle::Success);
    console.print(&rocket);
    console.print_text("");

    // -- Custom styles --------------------------------------------------------
    console.print(&Rule::with_title("Custom Styles"));

    let custom = Badge::new("Experimental")
        .style(BadgeStyle::Custom(
            Style::parse("magenta on black").unwrap(),
        ))
        .icon("🧪");
    console.print(&custom);
    console.print_text("");

    let purple_rounded = Badge::new("Beta")
        .style(BadgeStyle::Custom(
            Style::parse("bold white on purple").unwrap(),
        ))
        .icon("β")
        .rounded(true);
    console.print(&purple_rounded);
    console.print_text("");

    // -- Builder pattern chaining ---------------------------------------------
    console.print(&Rule::with_title("Builder Pattern"));

    let chained = Badge::new("Chained")
        .style(BadgeStyle::Success)
        .icon("→")
        .rounded(true);
    console.print(&chained);
    console.print_text("");

    // -- No icon --------------------------------------------------------------
    console.print(&Rule::with_title("Without Icons"));

    let no_icon_success = Badge::success("Done").icon("");
    console.print(&no_icon_success);
    console.print_text("");

    let plain_neutral = Badge::new("Plain");
    console.print(&plain_neutral);
    console.print_text("");

    // -- Display trait --------------------------------------------------------
    console.print(&Rule::with_title("Display Trait (via println!)"));
    let display_badge = Badge::info("From Display");
    println!("  println! output: {}", display_badge);
}
