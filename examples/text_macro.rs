//! Compile-time markup with the `text!` macro (feature `derive`).
//!
//! `text!` validates gilt markup at **compile time** — a typo'd, unknown, or
//! unclosed tag is a `cargo build` error, not a runtime surprise. It expands to
//! a [`Text`](gilt::text::Text). Try uncommenting one of the broken lines at the
//! bottom and re-compiling.
//!
//! Run with: `cargo run --example text_macro --features derive`

use gilt::console::Console;

fn main() {
    let mut console = Console::new();

    // Each of these is validated at compile time and expands to a `Text`:
    console.print(&gilt::text!(
        "[bold magenta]gilt[/] · the [green]text![/] macro"
    ));
    console.print(&gilt::text!(
        "[bold red]Error:[/] [italic]file not found[/]"
    ));
    console.print(&gilt::text!(
        "nested [bold]bold [underline]+ underline[/underline][/bold] done"
    ));
    console.print(&gilt::text!(
        "hyperlink: [blue underline link=https://docs.rs/gilt]docs.rs/gilt[/]"
    ));

    // These would each FAIL to compile (uncomment to see the error):
    //   gilt::text!("[bold]unclosed");       // unclosed tag
    //   gilt::text!("[blod]typo[/]");        // unknown style token `blod`
    //   gilt::text!("[bold]x[/italic]");     // mismatched close
}
