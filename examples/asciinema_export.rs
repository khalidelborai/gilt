//! Asciinema v2 `.cast` export — timed recording with a custom clock.
//!
//! Demonstrates:
//! - `Console::builder().record(true)` + `.with_asciinema_clock(...)` for
//!   deterministic, monotonic timestamps.
//! - `console.begin_asciinema_record()` — start a timed recording session.
//! - Printing a few styled lines; each call becomes one timed event.
//! - `console.export_asciinema(Some("demo"))` — produce asciinema v2 NDJSON.
//! - `console.save_asciinema("demo.cast", Some("demo"))` — write to a file.
//!
//! Run with: cargo run --example asciinema_export --features asciinema

#[cfg(feature = "asciinema")]
fn main() {
    use gilt::console::Console;
    use std::sync::{Arc, Mutex};

    // A simple counter-based clock that advances by 0.5 s per call.
    // This makes the output fully deterministic and independent of wall time.
    let counter = Arc::new(Mutex::new(0u32));
    let counter_clone = Arc::clone(&counter);

    let mut console = Console::builder()
        .width(80)
        .height(24)
        .force_terminal(true)
        .record(true)
        .build()
        .with_asciinema_clock(move || {
            let mut c = counter_clone.lock().unwrap();
            let t = *c;
            *c += 1;
            t as f64 * 0.5 // 0.0, 0.5, 1.0, …
        });

    // Begin the timed recording session.
    console.begin_asciinema_record();

    // Each print call appends one timed event.
    console.print_text("[bold green]Step 1:[/] Initialising…");
    console.print_text("[bold yellow]Step 2:[/] Processing data…");
    console.print_text("[bold cyan]Step 3:[/] [italic]All done![/]");

    // Export to an in-memory string and print its metadata.
    let cast = console.export_asciinema(Some("demo"));
    let line_count = cast.lines().count();
    println!("=== asciinema cast ({} lines) ===", line_count);
    for line in cast.lines() {
        println!("{}", line);
    }

    // Also save to a file (native only — excluded from wasm builds).
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = std::env::temp_dir().join("demo.cast");
        console.save_asciinema(&path, Some("demo")).unwrap();
        let metadata = std::fs::metadata(&path).unwrap();
        println!(
            "\nSaved {:?} ({} bytes)",
            path.file_name().unwrap(),
            metadata.len()
        );
        // Clean up the temp file.
        let _ = std::fs::remove_file(&path);
    }
}

#[cfg(not(feature = "asciinema"))]
fn main() {
    eprintln!("This example requires --features asciinema");
}
