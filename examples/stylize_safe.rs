//! Safe stylize methods demonstration
//!
//! This example demonstrates the fallible/safe alternatives to panicking
//! style methods. Use these when working with dynamic or user-provided
//! style/color values.
//!
//! Run: cargo run --example stylize_safe

use gilt::errors::{ColorParseError, StyleError};
use gilt::prelude::*;
use gilt::styled_str::Stylize;
use gilt::text::Text;

/// Configuration for user-defined styling
struct StyleConfig {
    text_color: String,
    background_color: String,
    attributes: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut console = Console::new();

    console.print(&Text::new("Safe Stylize Demo", Style::parse("bold underline")?));

    // =======================================================================
    // PART 1: The Problem - Panicking Methods
    // =======================================================================
    console.print(&Text::new("", Style::null()));
    console.print(&Text::new(
        "1. The Problem: Panicking Methods",
        Style::parse("bold")?,
    ));
    console.print(&Text::new(
        "When styling text with user input, panicking methods like .fg() and .bg() can crash your program.",
        Style::parse("dim")?,
    ));

    // These work fine with known-good values:
    let safe = "Valid styling".bold().red().on_blue();
    console.print(&safe);

    // But these would panic with invalid input:
    // let bad = "Crash!".fg("not_a_color");  // PANICS!
    // let bad = "Crash!".bg("#gggggg");      // PANICS!

    console.print(&Text::new(
        "See the source code for examples of what NOT to do.",
        Style::parse("dim")?,
    ));

    // =======================================================================
    // PART 2: Safe Alternatives with ? Operator
    // =======================================================================
    console.print(&Text::new("", Style::null()));
    console.print(&Text::new(
        "2. Safe Alternatives with ? Operator",
        Style::parse("bold")?,
    ));
    console.print(&Text::new(
        "Use try_fg(), try_bg(), try_styled(), and try_attr() for safe styling.",
        Style::parse("dim")?,
    ));

    // Using the ? operator for concise error propagation
    let user_color = "#ff6600";
    let styled = "Orange text (valid hex)".try_fg(user_color)?;
    console.print(&styled);

    let user_bg = "blue";
    let styled = "Text on blue background".try_bg(user_bg)?;
    console.print(&styled);

    let user_style = "bold italic red";
    let styled = "Bold italic red text".try_styled(user_style)?;
    console.print(&styled);

    // Chain multiple safe operations using ?
    let styled = "Fancy styled text"
        .try_attr("bold")?
        .try_fg("#00ff88")?
        .try_bg("#222222")?;
    console.print(&styled);

    // =======================================================================
    // PART 3: Error Handling with match
    // =======================================================================
    console.print(&Text::new("", Style::null()));
    console.print(&Text::new(
        "3. Error Handling with match",
        Style::parse("bold")?,
    ));
    console.print(&Text::new(
        "Handle different error cases explicitly.",
        Style::parse("dim")?,
    ));

    // Demonstrate handling different error types
    let invalid_colors = vec!["#ff6600", "not_a_color", "#gggggg", "rgb(999,0,0)"];

    for color in invalid_colors {
        match "Sample text".try_fg(color) {
            Ok(styled) => {
                let msg = format!("✓ '{}' is valid: ", color);
                console.print(&msg.bold().green());
                console.print(&styled);
            }
            Err(ColorParseError::InvalidHexFormat(msg)) => {
                console.print(&format!("✗ '{}' has invalid hex format: {}", color, msg).red().dim());
            }
            Err(ColorParseError::UnknownColorName(msg)) => {
                console.print(&format!("✗ '{}' is an unknown color name", msg).red().dim());
            }
            Err(ColorParseError::ComponentOutOfRange(msg)) => {
                console.print(
                    &format!("✗ '{}' has out-of-range component: {}", color, msg)
                        .red()
                        .dim(),
                );
            }
            Err(e) => {
                console.print(&format!("✗ '{}' parse error: {}", color, e).red().dim());
            }
        }
    }

    // =======================================================================
    // PART 4: Fallback Styles
    // =======================================================================
    console.print(&Text::new("", Style::null()));
    console.print(&Text::new(
        "4. Providing Fallback Styles",
        Style::parse("bold")?,
    ));
    console.print(&Text::new(
        "Gracefully handle errors by falling back to default styles.",
        Style::parse("dim")?,
    ));

    // Valid color - uses it
    let styled = style_with_fallback("This uses the user's color (purple)", "purple");
    console.print(&styled);

    // Invalid color - falls back to gray
    console.print(&"Warning: 'invalid_color_name' is not a valid color, using grey50".dim().yellow());
    let styled = style_with_fallback("This falls back to gray", "invalid_color_name");
    console.print(&styled);

    // =======================================================================
    // PART 5: Real-World Use Case - Styling User Input
    // =======================================================================
    console.print(&Text::new("", Style::null()));
    console.print(&Text::new(
        "5. Real-World: Styling User Input",
        Style::parse("bold")?,
    ));
    console.print(&Text::new(
        "Example: A CLI tool that accepts style preferences from config or CLI args.",
        Style::parse("dim")?,
    ));

    let user_config = StyleConfig {
        text_color: "#e06c75".to_string(), // Valid color (VSCode red)
        background_color: "not_a_color".to_string(), // Invalid - will use fallback
        attributes: vec!["bold".to_string(), "unknown_attr".to_string()], // Mixed validity
    };

    let result = apply_user_style("User-styled text (bold red)", &user_config);
    console.print(&result);

    // =======================================================================
    // PART 6: Combining Safe Methods
    // =======================================================================
    console.print(&Text::new("", Style::null()));
    console.print(&Text::new(
        "6. Combining Safe Methods",
        Style::parse("bold")?,
    ));
    console.print(&Text::new(
        "Build complex styles safely by chaining fallible operations.",
        Style::parse("dim")?,
    ));

    // Successful combination
    match build_complex_style("Complex styling", "cyan", "#1e1e1e", "bold underline") {
        Ok(styled) => console.print(&styled),
        Err(e) => console.print(&format!("Style error: {}", e).red()),
    }

    // Failed combination (bad background color)
    match build_complex_style("Won't work", "green", "invalid_bg", "italic") {
        Ok(styled) => console.print(&styled),
        Err(e) => console.print(&format!("Caught error: {}", e).red().dim()),
    }

    // =======================================================================
    // PART 7: Working with StyleError
    // =======================================================================
    console.print(&Text::new("", Style::null()));
    console.print(&Text::new(
        "7. Handling StyleError Variants",
        Style::parse("bold")?,
    ));
    console.print(&Text::new(
        "StyleError provides detailed information about what went wrong.",
        Style::parse("dim")?,
    ));

    let style_tests = vec![
        "bold red",      // Valid
        "invalid_attr",  // Unknown attribute
        "bold on",       // Missing background color
        "not",           // Missing attribute after 'not'
    ];

    for style_str in style_tests {
        match "Test".try_styled(style_str) {
            Ok(_) => console.print(&format!("✓ '{}' parsed successfully", style_str).green()),
            Err(StyleError::UnknownAttribute(attr)) => {
                console.print(&format!("✗ '{}' - unknown attribute: '{}'", style_str, attr).red());
            }
            Err(StyleError::InvalidSyntax(msg)) => {
                console.print(&format!("✗ '{}' - invalid syntax: {}", style_str, msg).red());
            }
            Err(e) => console.print(&format!("✗ '{}' - error: {}", style_str, e).red()),
        }
    }

    console.print(&Text::new("", Style::null()));
    console.print(&Text::new(
        "Demo Complete",
        Style::parse("bold underline")?,
    ));
    console.print(&Text::new(
        "Use try_fg(), try_bg(), try_styled(), and try_attr() for robust error handling!",
        Style::parse("dim italic")?,
    ));

    Ok(())
}

/// Helper function: Apply style with fallback to grey50 if color is invalid
fn style_with_fallback(text: &str, user_color: &str) -> StyledStr {
    // Try the user's color first
    match text.try_fg(user_color) {
        Ok(styled) => styled,
        Err(_) => {
            // Fall back to a safe default
            text.fg("grey50")
        }
    }
}

/// Helper function: Apply user configuration with error handling
fn apply_user_style(text: &str, config: &StyleConfig) -> StyledStr {
    let mut styled: StyledStr = text.styled(Style::null());

    // Apply foreground color with fallback
    styled = match styled.clone().try_fg(&config.text_color) {
        Ok(s) => s,
        Err(_) => {
            println!("Warning: Invalid text color '{}'", config.text_color);
            styled // Keep original (unstyled)
        }
    };

    // Apply background color with fallback
    styled = match styled.clone().try_bg(&config.background_color) {
        Ok(s) => s,
        Err(_) => {
            println!("Warning: Invalid background color '{}'", config.background_color);
            styled // Skip background
        }
    };

    // Apply attributes, skipping invalid ones
    for attr in &config.attributes {
        styled = match styled.clone().try_attr(attr) {
            Ok(s) => s,
            Err(_) => {
                println!("Warning: Unknown attribute '{}'", attr);
                styled // Keep current on error
            }
        };
    }

    styled
}

/// Helper function: Build complex style safely
fn build_complex_style(
    text: &str,
    fg: &str,
    bg: &str,
    style_def: &str,
) -> Result<StyledStr, Box<dyn std::error::Error>> {
    // Start with the text and apply styles step by step
    let styled = text
        .try_styled(style_def)? // Apply base style
        .try_fg(fg)? // Override/add foreground
        .try_bg(bg)?; // Add background

    Ok(styled)
}
