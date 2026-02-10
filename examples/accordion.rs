//! Accordion widget examples -- collapsible content panels.
//!
//! This example demonstrates:
//! - Single accordions with various configurations
//! - Accordion groups with multiple sections
//! - Custom icons and styling
//! - Mutual exclusion (only one open at a time)

use gilt::prelude::*;
use gilt::accordion::{Accordion, AccordionGroup};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                    Accordion Widget Examples                     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // ── Example 1: Basic expanded accordion ────────────────────────────────
    println!("1. Basic Expanded Accordion (default)");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let mut console = Console::builder()
        .width(80)
        .no_color(false)
        .build();
    
    let basic = Accordion::new(
        "Getting Started",
        Text::new(
            "Welcome to the accordion widget! This content is visible because \
             the accordion is expanded by default. You can collapse it by calling \
             .collapsed(true) or using the .collapse() method.",
            Style::null()
        )
    );
    
    console.print(&basic);
    console.line(1);

    // ── Example 2: Collapsed accordion ─────────────────────────────────────
    println!("2. Collapsed Accordion");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let collapsed = Accordion::new(
        "Hidden Details",
        Text::new(
            "You shouldn't see this content because the accordion is collapsed. \
             The title is still visible with a ▶ icon indicating it can be expanded.",
            Style::null()
        )
    ).collapsed(true);
    
    console.print(&collapsed);
    console.line(1);

    // ── Example 3: Styled accordion ────────────────────────────────────────
    println!("3. Styled Accordion");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let styled = Accordion::new(
        "Important Information",
        Text::new(
            "This accordion uses custom styling. The title is bold and yellow, \
             while the icons are cyan. You can style each part independently.",
            Style::null()
        )
    )
    .title_style(Style::parse("bold yellow").unwrap())
    .icon_style(Style::parse("cyan").unwrap());
    
    console.print(&styled);
    console.line(1);

    // ── Example 4: Custom icons ────────────────────────────────────────────
    println!("4. Custom Icons (+/−)");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let custom_icons = Accordion::new(
        "Plus/Minus Style",
        Text::new(
            "This accordion uses + and − symbols instead of the default arrows. \
             You can use any characters you prefer for the expand/collapse icons.",
            Style::null()
        )
    )
    .icons("+", "−")
    .icon_style(Style::parse("green").unwrap());
    
    console.print(&custom_icons);
    console.line(1);

    // ── Example 5: Custom indentation ──────────────────────────────────────
    println!("5. Custom Indentation (4 spaces)");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let indented = Accordion::new(
        "Indented Content",
        Text::new(
            "This content is indented with 4 spaces instead of the default 2. \
             You can adjust the indentation to match your preferred style.",
            Style::null()
        )
    )
    .indent(4);
    
    console.print(&indented);
    console.line(1);

    // ── Example 6: Accordion Group (multiple open) ─────────────────────────
    println!("6. Accordion Group (multiple can be open)");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let multi_open_group = AccordionGroup::new(vec![
        Accordion::new(
            "Configuration",
            Text::new(
                "Settings and configuration options go here. This could include \
                 API keys, endpoints, timeouts, and other tunable parameters.",
                Style::null()
            )
        ),
        Accordion::new(
            "Advanced Options",
            Text::new(
                "Advanced configuration for power users. Enable experimental \
                 features, adjust performance settings, and customize behavior.",
                Style::null()
            )
        )
        .collapsed(true),
        Accordion::new(
            "Troubleshooting",
            Text::new(
                "Common issues and their solutions. Check logs, enable debug \
                 mode, and contact support if problems persist.",
                Style::null()
            )
        )
        .collapsed(true),
    ]);
    
    console.print(&multi_open_group);
    console.line(1);

    // ── Example 7: Accordion Group (mutual exclusion) ──────────────────────
    println!("7. Accordion Group (only one open at a time)");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let single_open_group = AccordionGroup::new(vec![
        Accordion::new(
            "Step 1: Preparation",
            Text::new(
                "Gather all necessary materials and ensure you have the required \
                 tools. Read through the entire process before starting.",
                Style::null()
            )
        ),
        Accordion::new(
            "Step 2: Installation",
            Text::new(
                "Install the software package using your system's package manager. \
                 Verify the installation completed successfully.",
                Style::null()
            )
        )
        .collapsed(true),
        Accordion::new(
            "Step 3: Configuration",
            Text::new(
                "Edit the configuration files to match your environment. Set up \
                 environment variables and test the connection.",
                Style::null()
            )
        )
        .collapsed(true),
        Accordion::new(
            "Step 4: Deployment",
            Text::new(
                "Deploy to your production environment. Monitor logs and verify \
                 everything is working as expected.",
                Style::null()
            )
        )
        .collapsed(true),
    ])
    .allow_multiple_open(false);
    
    console.print(&single_open_group);
    console.line(1);

    // ── Example 8: Nested content with Text styling ────────────────────────
    println!("8. Rich Text Content");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let rich_text = Text::from_markup(
        "[bold]Key Features:[/bold]\n\
         • [green]Fast[/green] - Optimized for performance\n\
         • [blue]Safe[/blue] - Memory safety guaranteed\n\
         • [magenta]Easy[/magenta] - Simple and intuitive API"
    ).unwrap();
    
    let rich_content = Accordion::new(
        "Feature Highlights",
        rich_text
    )
    .title_style(Style::parse("bold cyan").unwrap());
    
    console.print(&rich_content);
    console.line(1);

    // ── Example 9: Programmatic control demo ───────────────────────────────
    println!("9. Programmatic Control Demo");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let mut dynamic = Accordion::new(
        "Dynamic Accordion",
        Text::new(
            "This accordion's state can be changed programmatically. \
             Try using .toggle(), .expand(), or .collapse() methods.",
            Style::null()
        )
    );
    
    println!("Initial state (expanded):");
    console.print(&dynamic);
    
    dynamic.collapse();
    println!("\nAfter calling .collapse():");
    console.print(&dynamic);
    
    dynamic.expand();
    println!("\nAfter calling .expand():");
    console.print(&dynamic);
    
    dynamic.toggle();
    println!("\nAfter calling .toggle():");
    console.print(&dynamic);

    // ── Example 10: Using Display trait ────────────────────────────────────
    println!("\n10. Using Display Trait (format! macro)");
    println!("───────────────────────────────────────────────────────────────────\n");
    
    let display_accordion = Accordion::new(
        "Display Output",
        Text::new("This accordion is rendered via the Display trait.", Style::null())
    );
    
    // Using Display trait
    let as_string = format!("{}", display_accordion);
    println!("As string:\n{}", as_string);
    
    println!("\n✓ All examples completed!");
}
