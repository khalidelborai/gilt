//! Demonstrates gilt's tracing subscriber integration for styled log output.
//!
//! The `GiltLayer` formats tracing events with color-coded levels, timestamps,
//! structured fields, and module paths — all rendered through gilt's Console.
//!
//! This example requires the `tracing` feature flag:
//!
//!   cargo run --example tracing_demo --features tracing
//!
//! Without the feature flag, it prints a helpful message and exits.

fn main() {
    #[cfg(not(feature = "tracing"))]
    {
        eprintln!("The tracing_demo example requires the 'tracing' feature.");
        eprintln!();
        eprintln!("  cargo run --example tracing_demo --features tracing");
        eprintln!();
        eprintln!("This enables gilt's GiltLayer, which formats tracing events");
        eprintln!("with color-coded levels, timestamps, and structured fields.");
        std::process::exit(0);
    }

    #[cfg(feature = "tracing")]
    run_tracing_demo();
}

#[cfg(feature = "tracing")]
fn run_tracing_demo() {
    use gilt::console::Console;
    use gilt::rule::Rule;
    use gilt::styled_str::Stylize;
    use gilt::tracing_layer::GiltLayer;
    use tracing_subscriber::prelude::*;

    // ── Set up the tracing subscriber with GiltLayer ─────────────────────
    let console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    let layer = GiltLayer::new()
        .with_console(console)
        .with_show_time(true)
        .with_show_level(true)
        .with_show_target(true)
        .with_show_span_path(true);

    tracing_subscriber::registry()
        .with(layer)
        .init();

    // ── Print a header (using a separate console) ────────────────────────
    let mut header_console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    header_console.print(&Rule::with_title("Tracing Events at Every Level"));
    header_console.print(&"Each level gets a distinct color: ERROR=red, WARN=yellow, INFO=blue, DEBUG=green, TRACE=dim".italic());
    header_console.print_text("");

    // ── Emit events at every level ───────────────────────────────────────
    tracing::error!("something went wrong");
    tracing::warn!("disk usage above 90%");
    tracing::info!("server started successfully");
    tracing::debug!("connection pool initialized");
    tracing::trace!("entering hot loop iteration");

    // ── Structured fields ────────────────────────────────────────────────
    header_console.print_text("");
    header_console.print(&Rule::with_title("Structured Fields"));
    header_console.print(&"Events can carry typed key=value fields alongside the message.".italic());
    header_console.print_text("");

    tracing::info!(user = "alice", action = "login", "user authenticated");
    tracing::info!(
        method = "GET",
        path = "/api/users",
        status = 200u64,
        latency_ms = 42u64,
        "request handled"
    );
    tracing::warn!(
        queue_depth = 1500u64,
        threshold = 1000u64,
        "message queue backing up"
    );
    tracing::error!(
        code = "E_TIMEOUT",
        retries = 3u64,
        "database connection failed"
    );

    // ── Span context ─────────────────────────────────────────────────────
    header_console.print_text("");
    header_console.print(&Rule::with_title("Span Context"));
    header_console.print(&"Events inside spans show the span path for context.".italic());
    header_console.print_text("");

    {
        let _server = tracing::info_span!("server").entered();
        tracing::info!(port = 8080u64, "listening");

        {
            let _request = tracing::info_span!("request").entered();
            tracing::debug!(method = "POST", path = "/api/data", "processing");
            tracing::info!("request completed");
        }
    }

    // ── Summary ──────────────────────────────────────────────────────────
    header_console.print_text("");
    header_console.print(&Rule::with_title("GiltLayer Features"));

    let features = [
        "Color-coded levels (ERROR=red, WARN=yellow, INFO=blue, DEBUG=green, TRACE=dim)",
        "HH:MM:SS timestamps (toggleable with .with_show_time())",
        "Module path / target column (toggleable with .with_show_target())",
        "Span path context (toggleable with .with_show_span_path())",
        "Structured key=value fields in dim italic",
        "Full gilt Console rendering with theme support",
    ];

    for feature in &features {
        header_console.print(&format!("  * {}", feature).green());
    }
}
