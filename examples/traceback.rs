//! Traceback rendering demo — shows gilt's styled error display.
//!
//! Demonstrates rendering of error chains, backtraces, and custom tracebacks
//! with syntax highlighting and source context.

use gilt::console::Console;
use gilt::rule::Rule;
use gilt::traceback::{Frame, Traceback};
use std::io;

fn main() {
    let mut console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── 1. Traceback from an error chain ───────────────────────────────────
    console.print(&Rule::with_title("Error Chain"));

    let inner = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
    let outer = io::Error::other(format!("failed to connect to database: {}", inner));
    let tb = Traceback::from_error(&outer);
    console.print(&tb);

    // ── 2. Traceback from a synthetic backtrace string ─────────────────────
    console.print(&Rule::with_title("Parsed Backtrace"));

    let bt_string = "\
   0: std::backtrace::Backtrace::force_capture
             at /rustc/abc123/library/std/src/backtrace.rs:331:18
   1: gilt::traceback::Traceback::from_backtrace
             at ./src/traceback.rs:136:9
   2: examples::traceback::main
             at ./examples/traceback.rs:25:5
   3: std::rt::lang_start::{{closure}}
             at /rustc/abc123/library/std/src/rt.rs:177:18";

    let tb = Traceback::from_backtrace(bt_string);
    console.print(&tb);

    // ── 3. Traceback from a panic ──────────────────────────────────────────
    console.print(&Rule::with_title("Panic Traceback"));

    let panic_bt = "\
   0: std::backtrace::Backtrace::force_capture
             at /rustc/abc123/library/std/src/backtrace.rs:331:18
   1: core::panicking::panic_fmt
             at /rustc/abc123/library/core/src/panicking.rs:75:14
   2: myapp::process_data
             at ./src/main.rs:42:9
   3: myapp::main
             at ./src/main.rs:10:5";

    let tb = Traceback::from_panic(
        "index out of bounds: the len is 3 but the index is 5",
        panic_bt,
    );
    console.print(&tb);

    // ── 4. Manually constructed traceback with source lines ────────────────
    console.print(&Rule::with_title("Custom Traceback with Source Lines"));

    let frames = vec![
        Frame::new("src/database.rs", Some(87), "Database::connect")
            .with_source_line("    let conn = TcpStream::connect(&self.addr)?;"),
        Frame::new("src/pool.rs", Some(42), "ConnectionPool::get")
            .with_source_line("    let conn = self.db.connect().await?;"),
        Frame::new("src/handlers.rs", Some(15), "handle_request")
            .with_source_line("    let db = pool.get().await?;"),
        Frame::new("src/main.rs", Some(28), "main")
            .with_source_line("    server.run(handle_request).await?;"),
    ];

    let tb = Traceback {
        title: "ConnectionError".to_string(),
        message: "failed to establish database connection\n\
                  Caused by:\n  \
                  Connection refused (os error 111)"
            .to_string(),
        frames,
        ..Traceback::new()
    };
    console.print(&tb);

    // ── 5. Customised width and extra context lines ────────────────────────
    console.print(&Rule::with_title("Narrow Width (60 cols)"));

    let tb = Traceback::from_panic(
        "called `Option::unwrap()` on a `None` value",
        "\
   0: core::panicking::panic_fmt
             at /rustc/abc123/library/core/src/panicking.rs:75:14
   1: myapp::config::load
             at ./src/config.rs:33:22",
    )
    .with_width(60)
    .with_word_wrap(true);
    console.print(&tb);
}
