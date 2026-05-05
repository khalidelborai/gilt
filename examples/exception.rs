//! Render a fallible operation's error as a styled Traceback widget.
//! Run: `cargo run --example exception`
//!
//! Rust analog of rich/examples/exception.py — Rust uses Result<T, E>
//! instead of try/except, so we show the same loop with .unwrap_or_else
//! routing through the Traceback widget on failure.

use gilt::console::Console;
use gilt::error::traceback::Traceback;

#[derive(Debug)]
struct DivideError(String);
impl std::fmt::Display for DivideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for DivideError {}

fn divide_by(n: f64, d: f64) -> Result<f64, DivideError> {
    if d == 0.0 {
        Err(DivideError(format!("cannot divide {n} by zero")))
    } else {
        Ok(n / d)
    }
}

fn main() {
    let mut c = Console::default();
    for (n, d) in [
        (1000.0, 200.0),
        (10000.0, 500.0),
        (1.0, 0.0),
        (3.1427, 2.0),
        (888.0, 0.0),
    ] {
        c.print_text(&format!("dividing {n} by {d}"));
        match divide_by(n, d) {
            Ok(r) => c.print_text(&format!(" = {r}")),
            Err(e) => c.print(&Traceback::from_error(&e).with_title("ZeroDivisionError")),
        }
    }
}
