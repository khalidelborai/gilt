//! Demonstrates gilt's eyre error reporting integration.
//! Run with: cargo run --example eyre_demo --features eyre

#[cfg(feature = "eyre")]
fn main() -> Result<(), eyre::Report> {
    use gilt::eyre_handler;

    // Install the gilt handler
    eyre_handler::install().expect("failed to install eyre handler");

    // Simulate a chained error
    #[derive(Debug, thiserror::Error)]
    #[error("failed to connect to database")]
    struct DbError;

    #[derive(Debug, thiserror::Error)]
    #[error("application startup failed")]
    struct AppError(#[from] DbError);

    let result: Result<(), AppError> = Err(AppError(DbError));
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            let report = eyre::Report::new(e);
            eprintln!("{report:?}");
            Ok(())
        }
    }
}

#[cfg(not(feature = "eyre"))]
fn main() {
    eprintln!("This example requires the 'eyre' feature: cargo run --example eyre_demo --features eyre");
}
