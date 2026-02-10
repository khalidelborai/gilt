//! Toast notification examples — demonstrating status messages for CLI apps.
//!
//! This example shows the various ways to create and display toast notifications,
//! including different types, custom styling, progress bars, and the ToastManager.

use std::time::Duration;

use gilt::console::Console;
use gilt::style::Style;
use gilt::toast::{
    toast_error, toast_info, toast_success, toast_warning, Toast, ToastManager, ToastType,
};

fn main() {
    let mut console = Console::builder()
        .width(60)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Section 1: Basic Toast Types -----------------------------------------

    println!("=== Basic Toast Types ===\n");

    Toast::success("Operation completed successfully").show(&mut console);

    Toast::error("Failed to save file").show(&mut console);

    Toast::warning("Disk space low").show(&mut console);

    Toast::info("3 new notifications").show(&mut console);

    // -- Section 2: Custom Icons ----------------------------------------------

    println!("\n=== Custom Icons ===\n");

    Toast::success("Build completed!")
        .icon("🚀")
        .show(&mut console);

    Toast::error("Connection failed")
        .icon("🔌")
        .show(&mut console);

    Toast::warning("Battery low").icon("🔋").show(&mut console);

    Toast::info("New email").icon("📧").show(&mut console);

    // -- Section 3: Custom Styling --------------------------------------------

    println!("\n=== Custom Styling ===\n");

    // Custom style with magenta color
    let custom_style = Style::parse("magenta bold").unwrap();
    Toast::new("Custom notification")
        .toast_type(ToastType::Custom(custom_style))
        .icon("🎉")
        .show(&mut console);

    // Custom style with dimmed appearance
    let subtle_style = Style::parse("dim italic").unwrap();
    Toast::new("Background task completed")
        .toast_type(ToastType::Custom(subtle_style))
        .icon("◌")
        .show(&mut console);

    // -- Section 4: Toast Manager (Multiple Toasts) ----------------------------

    println!("\n=== Toast Manager ===\n");

    let mut manager = ToastManager::new();
    manager.push(Toast::success("File uploaded"));
    manager.push(Toast::info("Processing image..."));
    manager.push(Toast::warning("Slow connection detected"));
    manager.push(Toast::error("Backup failed"));
    manager.push(Toast::info("5 tasks remaining"));

    // Only show up to 3 at a time
    manager.show_all(&mut console);

    // -- Section 5: Progress Bar Toasts ---------------------------------------

    println!("\n=== Progress Bar Toasts (non-blocking display) ===\n");

    Toast::info("Upload in progress...")
        .duration(Duration::from_secs(5))
        .show_progress(true)
        .show(&mut console);

    Toast::success("Installing packages")
        .duration(Duration::from_secs(10))
        .show_progress(true)
        .show(&mut console);

    // -- Section 6: Global Convenience Functions ------------------------------

    println!("\n=== Global Convenience Functions ===\n");

    // These use the default global console
    toast_success("Using global console!");
    toast_info("No need to create a Console instance");
    toast_warning("Quick and easy");
    toast_error("Error handling made simple");

    // -- Section 7: Builder Pattern Chaining ----------------------------------

    println!("\n=== Builder Pattern Chaining ===\n");

    Toast::new("Complex configuration")
        .toast_type(ToastType::Success)
        .duration(Duration::from_secs(8))
        .icon("✨")
        .show_progress(true)
        .width(50)
        .show(&mut console);

    // -- Section 8: Compact/Minimal Toasts ------------------------------------

    println!("\n=== Compact Toasts ===\n");

    // Very short messages
    Toast::success("Done!").show(&mut console);
    Toast::error("Oops!").show(&mut console);
    Toast::info("OK").show(&mut console);

    // -- Summary --------------------------------------------------------------

    println!("\n=== Summary ===");
    println!("Toast notifications provide:");
    println!("  • Four built-in types (success, error, warning, info)");
    println!("  • Custom icons and styling");
    println!("  • Optional progress bars for time-based feedback");
    println!("  • ToastManager for handling multiple notifications");
    println!("  • Global convenience functions for quick usage");
}
