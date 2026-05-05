//! Terminal hyperlink demo. Run: `cargo run --example link`

fn main() {
    let mut c = gilt::console::Console::default();
    c.print_text("If your terminal supports links, the following text should be clickable:");
    c.print_text("[link=https://github.com/khalidelborai/gilt][i]Visit [red]gilt[/red][/i] [yellow]on GitHub[/]");
}
