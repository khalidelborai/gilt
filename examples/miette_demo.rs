//! Demonstrates gilt's miette error reporting integration.
//! Run with: cargo run --example miette_demo --features miette

#[cfg(feature = "miette")]
fn main() {
    use gilt::miette_handler;
    use miette::Diagnostic;

    // Install the gilt handler
    miette_handler::install();

    // Create and display a diagnostic
    #[derive(Debug, thiserror::Error, Diagnostic)]
    #[error("configuration file not found")]
    #[diagnostic(
        code(gilt::config::not_found),
        help("Create a config.toml in the project root directory")
    )]
    struct ConfigError;

    let report = miette::Report::new(ConfigError);
    eprintln!("{report:?}");
}

#[cfg(not(feature = "miette"))]
fn main() {
    eprintln!("This example requires the 'miette' feature: cargo run --example miette_demo --features miette");
}
