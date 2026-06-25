//! Console capabilities — print all detected terminal capability flags.
//!
//! Demonstrates:
//! - `console.capabilities()` — returns a `&ConsoleCapabilities` struct.
//! - Fields: `color_system`, `truecolor`, `is_terminal`, `kitty`, `sixel`,
//!   `iterm`, `synchronized_output`, `unicode_version`.
//! - `ConsoleCapabilities::from_env_parts(...)` — the pure, testable helper.
//!
//! Run with: cargo run --example capabilities

use gilt::console::Console;
use gilt::console_caps::ConsoleCapabilities;

fn main() {
    let console = Console::new();
    let caps = console.capabilities();

    println!("=== ConsoleCapabilities (from environment) ===");
    println!("  color_system        : {:?}", caps.color_system);
    println!("  truecolor           : {}", caps.truecolor);
    println!("  is_terminal         : {}", caps.is_terminal);
    println!("  kitty               : {}", caps.kitty);
    println!("  sixel               : {}", caps.sixel);
    println!("  iterm               : {}", caps.iterm);
    println!("  synchronized_output : {}", caps.synchronized_output);
    println!(
        "  unicode_version     : {}",
        caps.unicode_version
            .map(|v| v.to_string())
            .unwrap_or_else(|| "not set".to_string())
    );

    // --- Show that from_env_parts is fully testable without a real terminal ---

    // Simulate a Ghostty terminal with Unicode 16 support.
    let simulated = ConsoleCapabilities::from_env_parts(
        Some("truecolor"),      // COLORTERM
        Some("xterm-256color"), // TERM
        true,                   // is_terminal
        Some("16"),             // UNICODE_VERSION
        None,                   // KITTY_WINDOW_ID
        Some("ghostty"),        // TERM_PROGRAM
        None,                   // GILT_IMAGE_PROTOCOL override
    );

    println!("\n=== Simulated Ghostty capabilities ===");
    println!("  color_system  : {:?}", simulated.color_system);
    println!("  truecolor     : {}", simulated.truecolor);
    println!("  kitty         : {}", simulated.kitty); // true: ghostty → kitty
    println!("  unicode_version: {:?}", simulated.unicode_version);

    assert!(
        simulated.kitty,
        "ghostty TERM_PROGRAM implies kitty support"
    );
    assert!(simulated.truecolor, "COLORTERM=truecolor implies truecolor");
    assert_eq!(simulated.unicode_version, Some(16));

    println!("\n[assertions passed]");
}
