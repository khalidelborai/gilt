//! Demonstrates gilt's `Stylize` trait — chain `.bold()`, `.red()`, `.on_blue()` etc.
//! on plain `&str` and `String` values to produce styled output.
//!
//! Run with: `cargo run --example styled_string`

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::styled_str::Stylize;

fn main() {
    let mut console = Console::builder()
        .width(72)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── Basic Attributes ─────────────────────────────────────────────────
    console.print(&Rule::with_title("Basic Attributes"));

    console.print(&"This text is bold".bold());
    console.print(&"This text is italic".italic());
    console.print(&"This text is underline".underline());
    console.print(&"This text is strikethrough".strikethrough());
    console.print(&"This text is dim".dim());

    // ── Foreground Colors ────────────────────────────────────────────────
    console.print(&Rule::with_title("Foreground Colors"));

    console.print(&"Red text".red());
    console.print(&"Green text".green());
    console.print(&"Blue text".blue());
    console.print(&"Cyan text".cyan());
    console.print(&"Magenta text".magenta());
    console.print(&"Bright yellow text".bright_yellow());
    console.print(&"Bright cyan text".bright_cyan());

    // ── Background Colors ────────────────────────────────────────────────
    console.print(&Rule::with_title("Background Colors"));

    console.print(&"Text on blue background".on_blue());
    console.print(&"Text on red background".on_red());
    console.print(&"Text on green background".on_green());

    // ── Chaining Multiple Styles ─────────────────────────────────────────
    console.print(&Rule::with_title("Chaining Styles"));

    console.print(&"Bold + Red + On Blue".bold().red().on_blue());
    console.print(&"Italic + Bright Yellow".italic().bright_yellow());
    console.print(&"Underline + Cyan + On Red".underline().cyan().on_red());
    console.print(&"Dim + Green".dim().green());

    // ── Multiple Styled Strings Together ─────────────────────────────────
    console.print(&Rule::with_title("Styled Strings Printed Together"));

    let greeting = "Hello, ".bold().green();
    let name = "World".bold().bright_yellow();
    let punctuation = "!".bold().red();

    console.print(&greeting);
    console.print(&name);
    console.print(&punctuation);

    // ── Arbitrary Colors with .fg() and .bg() ────────────────────────────
    console.print(&Rule::with_title("Arbitrary Colors via .fg() / .bg()"));

    console.print(&"Custom foreground: #ff6600 (orange)".fg("#ff6600"));
    console.print(&"Custom foreground: rgb(100,149,237) (cornflower)".fg("rgb(100,149,237)"));
    console.print(&"Custom background: #330066 (deep purple)".bg("#330066"));
    console.print(&"Combined: bold + #00ff88 fg + #222222 bg".bold().fg("#00ff88").bg("#222222"));

    // ── Hyperlinks with .link() ──────────────────────────────────────────
    console.print(&Rule::with_title("Hyperlinks"));

    console.print(&"Click me: Rust Homepage".bold().blue().link("https://www.rust-lang.org"));
    console.print(&"Gilt on GitHub".underline().cyan().link("https://github.com/example/gilt"));

    // ── Works on String too ──────────────────────────────────────────────
    console.print(&Rule::with_title("String (owned) values"));

    let owned: String = format!("Dynamically built: 2 + 2 = {}", 2 + 2);
    console.print(&owned.bold().bright_green());
}
