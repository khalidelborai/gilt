//! Diagnostic tool example - Reports terminal capabilities.
//!
//! This example demonstrates the diagnostic module which reports
//! terminal capabilities similar to Python Rich's diagnose.py.
//!
//! Run with:
//!   cargo run --example diagnose
//!
//! Try with different environment variables:
//!   NO_COLOR=1 cargo run --example diagnose
//!   FORCE_COLOR=3 cargo run --example diagnose
//!   TERM=xterm-256color cargo run --example diagnose

use gilt::diagnose;

fn main() {
    // Print the diagnostic report directly
    diagnose::print_report();

    // You can also get the report as a string
    let _report_string = diagnose::report();

    // Or access the structured data
    let report = diagnose::DiagnosticReport::generate();
    
    println!("\n");
    println!("Additional structured information:");
    println!("  Terminal type: {}", report.terminal.term);
    println!("  Is terminal:   {}", report.terminal.is_terminal);
    println!("  Color support: {:?}", report.color_support);
    println!("  Platform:      {} on {}", report.platform.os, report.platform.arch);
}
