//! Demonstrates gilt's terminal hyperlink support via markup.

use gilt::console::Console;
use gilt::rule::Rule;

fn main() {
    let mut console = Console::builder()
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("Terminal Hyperlinks"));

    console.print_text("If your terminal supports links, the following text should be clickable:");
    console.print_text("");
    console.print_text(
        "[link=https://github.com/Textualize/rich]Visit [bold]Rich[/bold] on GitHub[/link]",
    );
    console.print_text("[link=https://www.rust-lang.org][italic]The Rust[/italic] [yellow]Programming Language[/yellow][/link]");
    console.print_text("");
    console.print_text("Links are embedded using OSC 8 escape sequences — invisible in terminals");
    console.print_text("that don't support them, clickable in those that do.");
}
