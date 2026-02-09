//! Demonstrates gilt's RichHandler integration with the `log` crate.
//!
//! Run with: `cargo run --example logging_demo`

#[cfg(feature = "logging")]
fn main() {
    use gilt::console::Console;
    use gilt::logging_handler::RichHandler;

    // Build a console with forced color output for the demo.
    let console = Console::builder()
        .width(100)
        .force_terminal(true)
        .no_color(false)
        .build();

    // Create a RichHandler with all columns enabled and install it
    // as the global logger.
    let handler = RichHandler::new()
        .with_console(console)
        .with_show_time(true)
        .with_show_level(true)
        .with_show_path(true)
        .with_markup(true);

    log::set_boxed_logger(Box::new(handler)).expect("Failed to set logger");
    log::set_max_level(log::LevelFilter::Trace);

    // Demonstrate each log level.
    log::error!("Database connection failed: timeout after 30s");
    log::warn!("Disk usage is at 89%, consider cleanup");
    log::info!("Server started on http://0.0.0.0:8080");
    log::debug!("Loaded 42 configuration entries from config.toml");
    log::trace!("Entering request handler for GET /api/health");

    // The RichHandler also highlights HTTP keywords by default.
    log::info!("GET /index.html => 200 OK");
    log::info!("POST /api/users => 201 Created");
    log::warn!("DELETE /api/users/7 => 403 Forbidden");
}

#[cfg(not(feature = "logging"))]
fn main() {
    eprintln!(
        "This example requires the 'logging' feature.\n\
         Run with: cargo run --example logging_demo --features logging"
    );
}
