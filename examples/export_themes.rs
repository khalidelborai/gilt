//! Named theme registry + themed HTML export.
//!
//! `ThemeRegistry` ships 5 popular palettes (Dracula, Nord, Gruvbox, Monokai
//! Pro, Solarized Dark). `Console::export_html_with_theme` renders recorded
//! output to a self-contained HTML document using one of them.
//!
//! Run with: `cargo run --example export_themes`

use gilt::console::Console;
use gilt::terminal_theme::ThemeRegistry;

fn main() {
    println!("Built-in themes: {:?}", ThemeRegistry::names());

    // Record some styled output (force_terminal so color is captured).
    let mut console = Console::builder()
        .width(60)
        .record(true)
        .force_terminal(true)
        .build();
    console
        .print_text("[bold magenta]gilt[/] export — [green]ok[/], [red]error[/], [yellow]warn[/]");
    console.print_text("[blue underline]a link[/], [italic]italic[/], [bold]bold[/]");

    for name in ThemeRegistry::names() {
        let html = console.export_html_with_theme(name, false, true);
        println!("  {name:<16} -> {} bytes of HTML", html.len());
    }

    // Write one out so you can open it in a browser.
    let html = console.export_html_with_theme("dracula", false, true);
    std::fs::write("gilt-dracula.html", html).expect("write html");
    println!("wrote gilt-dracula.html (Dracula palette)");
}
