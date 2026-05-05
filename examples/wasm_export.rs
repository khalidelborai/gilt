//! WASM-friendly export demo. Run normally with `cargo run --example wasm_export`,
//! or build for the browser with `cargo build --target wasm32-unknown-unknown`.
//!
//! gilt builds for `wasm32-unknown-unknown` without modification (since v1.2.0)
//! because it has no `libc`, `crossterm`, `termion`, or terminal-syscall
//! dependencies. The path that's actually useful in a browser is
//! `Console::builder().record(true).build()` + `Console::export_html(...)`,
//! which produces a styled HTML fragment ready to drop into a `<div>` or
//! pipe into `xterm.js`'s ANSI parser via `export_text(true, true)`.

use gilt::console::Console;
use gilt::panel::Panel;
use gilt::table::Table;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .record(true)
        .force_terminal(true)
        .build();

    // Build a small dashboard.
    console.print(
        &Panel::from_renderable(&{
            let mut t = Table::new(&["Service", "Status", "Latency"]);
            t.add_row(&["api", "[bold green]healthy[/]", "[dim]43ms[/]"]);
            t.add_row(&["worker", "[bold green]healthy[/]", "[dim]120ms[/]"]);
            t.add_row(&["db", "[bold yellow]degraded[/]", "[bold red]2400ms[/]"]);
            t
        })
        .with_title("Service Status"),
    );

    // Two outputs: ANSI for xterm.js consumers, HTML for direct DOM injection.
    let ansi_for_xtermjs = console.export_text(false, true);
    let html_for_dom = console.export_html(None, false, true);

    println!(
        "--- ANSI ({} bytes, pipe into xterm.js) ---",
        ansi_for_xtermjs.len()
    );
    println!("{}", ansi_for_xtermjs);
    println!(
        "--- HTML ({} bytes, drop into <div>) ---",
        html_for_dom.len()
    );
    // Truncate HTML preview so the demo stays terse.
    let preview: String = html_for_dom.chars().take(200).collect();
    println!("{}…", preview);
}
