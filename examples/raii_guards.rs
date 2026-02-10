//! RAII guards demonstration for Console
//!
//! This example demonstrates the RAII guard patterns for Console
//! (Note: Full RAII guard implementation is in progress)
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
    // 1. Manual Theme Management (until ThemeGuard is implemented)
    // ========================================================================
    console.rule(Some("Theme Push/Pop Pattern"));

    console.print_text("Before theme push:");
    console.print_text("  [info]This uses the default 'info' style[/info]");

    // Create custom theme
    let mut custom_styles = HashMap::new();
    custom_styles.insert(
        "info".to_string(),
        Style::parse("bold magenta on grey15").unwrap(),
    );
    let custom_theme = Theme::new(Some(custom_styles), true);

    // Push theme
    console.push_theme(custom_theme);
    console.print_text("\nAfter theme push:");
    console.print_text("  [info]This uses the custom magenta 'info' style[/info]");

    // Pop theme
    let _ = console.pop_theme();
    console.print_text("\nAfter theme pop:");
    console.print_text("  [info]This reverts to default 'info' style[/info]");

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
    // 4. RAII Guard Pattern (Conceptual)
    // ========================================================================
    console.rule(Some("RAII Guard Pattern (Conceptual)"));

    console.print_text("Future implementation will support:");
    console.print_text("");
    console.print_text("  {");
    console.print_text("      let _guard = console.use_theme(custom_theme);");
    console.print_text("      // Theme active here");
    console.print_text("  } // Theme automatically popped");
    console.print_text("");
    console.print_text("  {");
    console.print_text("      let _guard = console.screen(true)?;");
    console.print_text("      // Alternate screen active");
    console.print_text("  } // Automatically exits screen");

    // ========================================================================
    // 5. Current Best Practice
    // ========================================================================
    console.rule(Some("Current Best Practice"));

    console.print_text("For now, use explicit push/pop with defer pattern:");
    console.print_text("");
    console.print_text("  console.push_theme(theme);");
    console.print_text("  // ... do work ...");
    console.print_text("  console.pop_theme(); // Always pop!");

    console.line(1);
    console.print_text("[green]✓[/green] RAII guards demo complete!");
}
