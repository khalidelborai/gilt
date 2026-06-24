//! RAII guards demonstration for Console
//!
//! Demonstrates RAII guard patterns — theme, capture, and screen management.
//!
//! Run: cargo run --example raii_guards

use gilt::prelude::*;
use gilt::theme::Theme;
use std::collections::HashMap;

fn main() {
    let mut console = Console::new();

    console.rule(Some("RAII Guards Demo"));
    console.print_text("Demonstrating RAII patterns for Console\n");

    // ========================================================================
    // 1. use_theme — RAII ThemeGuard
    //
    // use_theme pushes a theme and returns a ThemeGuard.  When the guard drops
    // (end of scope), pop_theme is called automatically.  Because the guard
    // holds an exclusive `&mut Console`, you cannot call console methods inside
    // the same scope while the guard is alive — hold the guard in a block and
    // do your themed rendering before or after, or use push_theme/pop_theme for
    // interleaved rendering (see section 4 below).
    // ========================================================================
    console.rule(Some("use_theme (ThemeGuard)"));

    let mut custom_styles = HashMap::new();
    custom_styles.insert("info".to_string(), Style::parse("bold magenta on grey15"));
    let custom_theme = Theme::new(Some(custom_styles), true);

    // Push via RAII guard — theme is automatically popped when guard drops.
    let _guard = console.use_theme(custom_theme, true);
    drop(_guard); // explicit drop; in real code let the block scope handle it

    console.print_text("Theme guard was active and has now been automatically popped.");
    console.print_text("  [info]'info' is back to the default style[/info]");

    // ========================================================================
    // 2. Screen Management
    // ========================================================================
    console.rule(Some("Screen Management Pattern"));

    console.print_text("The console supports alternate screen mode:");
    console.print_text("  - console.enter_screen() - Enter alternate screen");
    console.print_text("  - console.exit_screen() - Exit alternate screen");
    console.print_text("\n(Not demonstrating to avoid clearing terminal)");

    // ========================================================================
    // 3. Capture Management
    // ========================================================================
    console.rule(Some("Capture Pattern"));

    console.print_text("The console supports capture mode:");

    console.begin_capture();
    console.print_text("  This text is being captured...");
    console.print_text("  It won't appear on screen!");
    let captured = console.end_capture();

    console.print_text("\nCaptured output:");
    console.print(&Text::new(&captured, Style::null()));

    // ========================================================================
    // 4. Manual push_theme / pop_theme (for interleaved rendering)
    // ========================================================================
    console.rule(Some("Manual push_theme / pop_theme"));

    console.print_text("Before theme push:");
    console.print_text("  [info]This uses the default 'info' style[/info]");

    let mut manual_styles = HashMap::new();
    manual_styles.insert("info".to_string(), Style::parse("bold magenta on grey15"));
    let manual_theme = Theme::new(Some(manual_styles), true);

    console.push_theme(manual_theme, true);
    console.print_text("\nAfter theme push:");
    console.print_text("  [info]This uses the custom magenta 'info' style[/info]");

    console.pop_theme();
    console.print_text("\nAfter theme pop:");
    console.print_text("  [info]This reverts to default 'info' style[/info]");

    console.line(1);
    console.print_text("[green]✓[/green] RAII guards demo complete!");
}
