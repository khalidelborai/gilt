//! Demonstrates the Breadcrumbs widget for navigation path display.
//!
//! Shows various breadcrumb styles and use cases: file paths, navigation flows,
//! settings hierarchies, and wizard steps.
//!
//! Run with: `cargo run --example breadcrumbs`

use gilt::breadcrumbs::Breadcrumbs;
use gilt::console::Console;
use gilt::rule::Rule;
use gilt::style::Style;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // -- Default separator -----------------------------------------------------
    console.print(&Rule::with_title("Default Separator ( > )"));

    let default_crumbs = Breadcrumbs::new(vec!["Home".into(), "Settings".into(), "Profile".into()]);
    console.print(&default_crumbs);
    console.print_text("");

    // -- Slash separator (file paths) ------------------------------------------
    console.print(&Rule::with_title("Slash Separator (File Paths)"));

    let file_path = Breadcrumbs::slash(vec![
        "home".into(),
        "user".into(),
        "projects".into(),
        "myapp".into(),
        "src".into(),
        "main.rs".into(),
    ]);
    console.print(&file_path);
    console.print_text("");

    // -- Arrow separator (workflow) --------------------------------------------
    console.print(&Rule::with_title("Arrow Separator (Workflow Steps)"));

    let workflow = Breadcrumbs::arrow(vec![
        "Input".into(),
        "Process".into(),
        "Validate".into(),
        "Output".into(),
    ]);
    console.print(&workflow);
    console.print_text("");

    // -- Chevron separator -----------------------------------------------------
    console.print(&Rule::with_title("Chevron Separator"));

    let chevron_crumbs = Breadcrumbs::chevron(vec![
        "Store".into(),
        "Electronics".into(),
        "Laptops".into(),
        "Gaming".into(),
    ]);
    console.print(&chevron_crumbs);
    console.print_text("");

    // -- From path string ------------------------------------------------------
    console.print(&Rule::with_title("From Path String"));

    let from_path = Breadcrumbs::from_path("api/v1/users/123/posts");
    console.print(&from_path);
    console.print_text("");

    // -- With active (last item) styling ---------------------------------------
    console.print(&Rule::with_title("With Active Item Styling"));

    let styled_active = Breadcrumbs::new(vec![
        "Dashboard".into(),
        "Users".into(),
        "User Details".into(),
    ])
    .style(Style::parse("dim").unwrap())
    .separator_style(Style::parse("dim").unwrap())
    .active_style(Style::parse("bold green").unwrap());
    console.print(&styled_active);
    console.print_text("");

    // -- Custom separator ------------------------------------------------------
    console.print(&Rule::with_title("Custom Separator"));

    let custom_sep =
        Breadcrumbs::new(vec!["First".into(), "Second".into(), "Third".into()]).separator(" | ");
    console.print(&custom_sep);
    console.print_text("");

    // -- Navigation flows ------------------------------------------------------
    console.print(&Rule::with_title("Navigation Flows"));

    // E-commerce navigation
    let ecommerce = Breadcrumbs::new(vec![
        "Home".into(),
        "Category".into(),
        "Subcategory".into(),
        "Product".into(),
    ]);
    console.print(&ecommerce);
    console.print_text("");

    // Documentation navigation
    let docs = Breadcrumbs::slash(vec![
        "docs".into(),
        "getting-started".into(),
        "installation".into(),
    ]);
    console.print(&docs);
    console.print_text("");

    // -- Wizard steps ----------------------------------------------------------
    console.print(&Rule::with_title("Wizard / Multi-step Form"));

    let wizard = Breadcrumbs::arrow(vec![
        "Welcome".into(),
        "Personal Info".into(),
        "Preferences".into(),
        "Review".into(),
        "Complete".into(),
    ]);
    console.print(&wizard);
    console.print_text("");

    // -- Dynamic building (push/pop) -------------------------------------------
    console.print(&Rule::with_title("Dynamic Building with Push/Pop"));

    let mut dynamic = Breadcrumbs::new(vec!["Root".into()]);
    console.print(&dynamic);
    console.print_text("");

    dynamic.push("Level 1");
    console.print(&dynamic);
    console.print_text("");

    dynamic.push("Level 2");
    dynamic.push("Level 3");
    console.print(&dynamic);
    console.print_text("");

    // Pop the last item
    dynamic.pop();
    console.print_text("After pop:");
    console.print(&dynamic);
    console.print_text("");

    // -- Display trait ---------------------------------------------------------
    console.print(&Rule::with_title("Display Trait (via println!)"));
    let display_crumbs = Breadcrumbs::new(vec![
        "Section A".into(),
        "Section B".into(),
        "Section C".into(),
    ]);
    println!("  println! output: {}", display_crumbs);
    console.print_text("");

    // -- Single item -----------------------------------------------------------
    console.print(&Rule::with_title("Single Item (No Separator)"));
    let single = Breadcrumbs::new(vec!["Home".into()]);
    console.print(&single);
    console.print_text("");

    // -- Empty breadcrumbs -----------------------------------------------------
    console.print(&Rule::with_title("Empty Breadcrumbs"));
    let empty = Breadcrumbs::new(vec![]);
    console.print(&empty);
    console.print_text("(nothing displayed above)");
}
