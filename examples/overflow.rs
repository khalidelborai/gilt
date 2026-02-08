//! Demonstrates gilt's text overflow methods — Fold, Crop, and Ellipsis.

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;
use gilt::text::{OverflowMethod, Text};

fn main() {
    let mut console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(false)
        .build();

    let long_word = "Supercalifragilisticexpialidocious";

    console.print(&Rule::with_title("Overflow Methods"));
    console.print_text(&format!(
        "[dim]Original ({} chars):[/dim] {long_word}",
        long_word.len()
    ));
    console.print_text("[dim]Truncating to 20 cells...[/dim]");
    console.print_text("");

    // -- 1. Fold — crops to width (same as crop for single-line truncate) -----

    console.print_text("[bold]Fold:[/bold]");
    let mut fold_text = Text::new(long_word, Style::parse("bold blue").unwrap());
    fold_text.truncate(20, Some(OverflowMethod::Fold), false);
    console.print(&fold_text);
    console.print_text("");

    // -- 2. Crop — truncates silently -----------------------------------------

    console.print_text("[bold]Crop:[/bold]");
    let mut crop_text = Text::new(long_word, Style::parse("bold red").unwrap());
    crop_text.truncate(20, Some(OverflowMethod::Crop), false);
    console.print(&crop_text);
    console.print_text("");

    // -- 3. Ellipsis — truncates with an ellipsis character -------------------

    console.print_text("[bold]Ellipsis:[/bold]");
    let mut ellipsis_text = Text::new(long_word, Style::parse("bold green").unwrap());
    ellipsis_text.truncate(20, Some(OverflowMethod::Ellipsis), false);
    console.print(&ellipsis_text);
    console.print_text("");

    // -- 4. Side-by-side comparison with padding ------------------------------

    console.print(&Rule::with_title("Padded to 20 cells"));

    for (name, method) in [
        ("Fold    ", OverflowMethod::Fold),
        ("Crop    ", OverflowMethod::Crop),
        ("Ellipsis", OverflowMethod::Ellipsis),
    ] {
        let mut text = Text::new(long_word, Style::null());
        text.truncate(20, Some(method), true); // pad=true for alignment
        let mut line = Text::new(&format!("{name} |"), Style::parse("dim").unwrap());
        line.append_str(text.plain(), None);
        line.append_str("|", Some(Style::parse("dim").unwrap()));
        console.print(&line);
    }
}
