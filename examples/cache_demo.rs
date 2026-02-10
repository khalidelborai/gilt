//! LRU Cache demonstration for Style and Color parsing
//!
//! This example shows how the LRU cache improves performance for repeated
//! style and color parsing operations.
//!
//! Run: cargo run --example cache_demo --release

use gilt::prelude::*;
use std::time::Instant;

fn main() {
    let mut console = Console::new();

    console.rule(Some("LRU Cache Demo"));
    console.print_text("Demonstrating Style and Color parsing cache\n");

    // ========================================================================
    // 1. Style Parsing Cache
    // ========================================================================
    console.rule(Some("Style Parsing Cache"));

    let styles_to_parse = [
        "bold red",
        "italic green on blue",
        "dim cyan",
        "underline magenta",
        "strike yellow",
        "bold red",             // Duplicate
        "italic green on blue", // Duplicate
        "dim cyan",             // Duplicate
    ];

    // First pass - populate cache
    console.print_text("First pass (populating cache):");
    let start = Instant::now();
    for style_str in &styles_to_parse {
        let _ = Style::parse(style_str);
    }
    let first_pass = start.elapsed();
    console.print_text(&format!("  Time: {:?}", first_pass));

    // Second pass - should use cache
    console.print_text("\nSecond pass (using cache):");
    let start = Instant::now();
    for style_str in &styles_to_parse {
        let _ = Style::parse(style_str);
    }
    let second_pass = start.elapsed();
    console.print_text(&format!("  Time: {:?}", second_pass));

    // Show improvement
    if first_pass > second_pass {
        let ratio = first_pass.as_nanos() as f64 / second_pass.as_nanos() as f64;
        console.print_text(&format!(
            "\n[green]✓[/green] Cache speedup: {:.1}x faster",
            ratio
        ));
    }

    // ========================================================================
    // 2. Color Parsing Cache
    // ========================================================================
    console.rule(Some("Color Parsing Cache"));

    let colors_to_parse = [
        "red",
        "#FF5733",
        "rgb(100, 150, 200)",
        "blue",
        "#00FF00",
        "red",     // Duplicate
        "#FF5733", // Duplicate
        "blue",    // Duplicate
    ];

    // First pass
    console.print_text("First pass (populating cache):");
    let start = Instant::now();
    for color_str in &colors_to_parse {
        let _ = Color::parse(color_str);
    }
    let first_pass = start.elapsed();
    console.print_text(&format!("  Time: {:?}", first_pass));

    // Second pass
    console.print_text("\nSecond pass (using cache):");
    let start = Instant::now();
    for color_str in &colors_to_parse {
        let _ = Color::parse(color_str);
    }
    let second_pass = start.elapsed();
    console.print_text(&format!("  Time: {:?}", second_pass));

    if first_pass > second_pass {
        let ratio = first_pass.as_nanos() as f64 / second_pass.as_nanos() as f64;
        console.print_text(&format!(
            "\n[green]✓[/green] Cache speedup: {:.1}x faster",
            ratio
        ));
    }

    // ========================================================================
    // 3. Cache Statistics
    // ========================================================================
    console.rule(Some("Cache Statistics"));

    console.print_text(&format!(
        "Style cache size: {} entries",
        gilt::style::style_cache_size()
    ));
    console.print_text(&format!(
        "Color cache size: {} entries",
        gilt::color::color_cache_size()
    ));

    // ========================================================================
    // 4. Cache Clearing
    // ========================================================================
    console.rule(Some("Cache Clearing"));

    console.print_text("Clearing caches...");
    gilt::style::clear_style_cache();
    gilt::color::clear_color_cache();

    console.print_text(&format!(
        "Style cache size after clear: {}",
        gilt::style::style_cache_size()
    ));
    console.print_text(&format!(
        "Color cache size after clear: {}",
        gilt::color::color_cache_size()
    ));

    // ========================================================================
    // 5. Real-world Scenario: Theming
    // ========================================================================
    console.rule(Some("Real-World: Dynamic Theming"));

    console.print_text("Simulating dynamic theme application...\n");

    // Simulate applying a theme with many styled elements
    let theme_styles = [
        "bold",
        "dim",
        "italic",
        "underline",
        "red",
        "green",
        "blue",
        "yellow",
        "on_red",
        "on_green",
        "on_blue",
        "bold red",
        "italic green",
        "underline blue",
    ];

    let start = Instant::now();
    for _ in 0..100 {
        for style_str in &theme_styles {
            let _ = Style::parse(style_str);
        }
    }
    let themed = start.elapsed();

    console.print_text(&format!(
        "Applied {} styles x 100 iterations in {:?}",
        theme_styles.len(),
        themed
    ));
    console.print_text(&format!(
        "Style cache size: {} (max 256)",
        gilt::style::style_cache_size()
    ));

    console.line(1);
    console.print_text("[green]✓[/green] Cache demo complete!");
}
