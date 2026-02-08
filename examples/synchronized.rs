//! Demonstrates synchronized output (DEC Mode 2026) and OSC 52 clipboard.
//!
//! Synchronized output prevents tearing in terminals by buffering all writes
//! between a begin/end pair, then painting atomically. Clipboard support uses
//! OSC 52 to copy text directly to the system clipboard via escape sequences.
//!
//! Since these features use invisible escape sequences, this example uses
//! capture mode to show the raw sequences being generated.
//!
//! Run with: `cargo run --example synchronized`

use gilt::console::Console;
use gilt::control::Control;
use gilt::rule::Rule;
use gilt::styled_str::Stylize;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("Synchronized Output (DEC Mode 2026)"));

    // ── Explain the concept ──────────────────────────────────────────────
    console.print(&"Synchronized output prevents flickering during rapid updates.".italic());
    console
        .print(&"The terminal buffers everything between begin/end, then paints at once.".italic());
    console.print_text("");

    // ── Method 1: Manual begin/end ───────────────────────────────────────
    console.print(&"Method 1: Manual begin_synchronized / end_synchronized".bold());
    console.print_text("");

    // Use capture to show the escape sequences
    let mut capture_console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(true)
        .build();
    capture_console.begin_capture();
    capture_console.begin_synchronized();
    capture_console.print_text("  Line 1 of synchronized block");
    capture_console.print_text("  Line 2 of synchronized block");
    capture_console.end_synchronized();
    let captured = capture_console.end_capture();

    console.print_text("  Captured output (raw):");
    for line in captured.lines() {
        // Show control sequences as readable text
        let escaped = line
            .replace("\x1b[?2026h", "<BEGIN_SYNC>")
            .replace("\x1b[?2026l", "<END_SYNC>");
        console.print_text(&format!("    {}", escaped));
    }
    console.print_text("");

    // ── Method 2: Closure form ───────────────────────────────────────────
    console.print(&"Method 2: synchronized(|c| { ... }) closure".bold());
    console.print_text("");

    let mut capture_console2 = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(true)
        .build();
    capture_console2.begin_capture();
    capture_console2.synchronized(|c| {
        c.print_text("  Content inside synchronized closure");
        c.print_text("  All painted atomically");
    });
    let captured2 = capture_console2.end_capture();

    console.print_text("  Captured output (raw):");
    for line in captured2.lines() {
        let escaped = line
            .replace("\x1b[?2026h", "<BEGIN_SYNC>")
            .replace("\x1b[?2026l", "<END_SYNC>");
        console.print_text(&format!("    {}", escaped));
    }
    console.print_text("");

    // ── Clipboard via OSC 52 ─────────────────────────────────────────────
    console.print(&Rule::with_title("Clipboard (OSC 52)"));

    console.print(&"OSC 52 sends text to the system clipboard via escape sequences.".italic());
    console.print(&"Supported by: kitty, iTerm2, WezTerm, foot, and others.".italic());
    console.print_text("");

    // Show what the escape sequence looks like
    let clipboard_text = "Hello from gilt!";
    let ctrl = Control::set_clipboard(clipboard_text);
    let raw_seq = ctrl.to_string();
    let escaped = raw_seq.replace('\x1b', "<ESC>").replace('\x07', "<BEL>");

    console.print(&"copy_to_clipboard(\"Hello from gilt!\") produces:".bold());
    console.print_text(&format!("  {}", escaped));
    console.print_text("");

    // Actually send it to the clipboard (will work in supporting terminals)
    let mut live_console = Console::builder().force_terminal(true).build();
    live_console.copy_to_clipboard(clipboard_text);
    console.print_text("  Clipboard command sent! Check your clipboard in a supported terminal.");
    console.print_text("");

    // ── Summary ──────────────────────────────────────────────────────────
    console.print(&Rule::with_title("Escape Sequences Reference"));

    let reference = [
        ("Begin Sync", "ESC[?2026h", "Start buffering output"),
        ("End Sync", "ESC[?2026l", "Flush buffer, paint atomically"),
        (
            "Set Clipboard",
            "ESC]52;c;<base64>BEL",
            "Copy text to clipboard",
        ),
        (
            "Get Clipboard",
            "ESC]52;c;?BEL",
            "Request clipboard contents",
        ),
    ];

    for (name, seq, desc) in &reference {
        console.print_text(&format!("  {:<16} {:<24} {}", name, seq, desc));
    }
}
