//! Auto-theme — detect the terminal background and apply a matching theme.
//!
//! Demonstrates:
//! - `Console::detect_background()` — wraps `query_terminal_background()` on
//!   native + `terminal-query` builds; returns `Unknown` otherwise.
//! - `ConsoleBuilder::auto_theme()` — probes the background and switches to a
//!   light or dark built-in theme automatically.
//! - The three `ConsoleBackground` variants: `Dark`, `Light`, `Unknown`.
//! - `parse_osc11_response` + `is_dark_background` — the dep-free parsing core.
//!
//! Run with: cargo run --example auto_theme --features terminal-query
//! (Falls back gracefully to `Unknown` when run in a non-supporting terminal.)

use gilt::terminal_bg::{is_dark_background, parse_osc11_response, ConsoleBackground};

fn main() {
    use gilt::console::Console;

    // --- Core parsing (always available, no feature needed) ------------------

    let dark_reply = "\x1b]11;rgb:1a1a/1a1a/1a1a\x07";
    let light_reply = "\x1b]11;rgb:ffff/ffff/ffff\x07";

    let dark_triplet = parse_osc11_response(dark_reply).expect("valid dark reply");
    let light_triplet = parse_osc11_response(light_reply).expect("valid light reply");

    println!("=== OSC 11 parsing (dep-free core) ===");
    println!(
        "  dark  reply → {:?}  is_dark={}",
        dark_triplet,
        is_dark_background(dark_triplet)
    );
    println!(
        "  light reply → {:?}  is_dark={}",
        light_triplet,
        is_dark_background(light_triplet)
    );
    assert!(is_dark_background(dark_triplet));
    assert!(!is_dark_background(light_triplet));

    // --- Live terminal probe -------------------------------------------------

    let console = Console::new();
    let bg = console.detect_background();

    println!("\n=== Console::detect_background() ===");
    match bg {
        ConsoleBackground::Dark => {
            println!("  Result: Dark  (terminal uses a dark background)");
        }
        ConsoleBackground::Light => {
            println!("  Result: Light (terminal uses a light background)");
        }
        ConsoleBackground::Unknown => {
            println!("  Result: Unknown (no TTY, terminal unsupported, or timed out)");
            println!("  [expected in CI, pipes, and terminals without OSC 11 support]");
        }
    }

    // --- ConsoleBuilder::auto_theme() ----------------------------------------

    // On terminals that support OSC 11 this applies a light or dark theme;
    // on others it's a safe no-op.
    let _themed_console = Console::builder().width(80).auto_theme().build();

    println!("\n=== ConsoleBuilder::auto_theme() ===");
    println!("  Console built with auto_theme() — theme applied when background was detected.");
    println!("  [tip] Use --features terminal-query to enable the OSC 11 probe.");

    println!("\n[assertions passed]");
}
