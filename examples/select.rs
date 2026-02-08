//! Demonstrates Select and MultiSelect interactive prompts.
//!
//! Select lets the user pick one option from a numbered list.
//! MultiSelect lets the user pick multiple options (comma-separated).
//!
//! This example is interactive — it reads from stdin.
//!
//! Run with: `cargo run --example select`

use gilt::console::Console;
use gilt::prompt::{MultiSelect, Select};
use gilt::rule::Rule;
use gilt::styled_str::Stylize;

fn main() {
    let mut console = Console::builder()
        .width(72)
        .force_terminal(true)
        .no_color(false)
        .build();

    console.print(&Rule::with_title("Select Prompt"));
    console.print(&"Pick one option from the list by entering its number.".italic());
    console.print_text("");

    // ── Select: Choose a programming language ────────────────────────────
    let languages = vec![
        "Rust".into(),
        "Python".into(),
        "TypeScript".into(),
        "Go".into(),
        "Zig".into(),
    ];

    let select = Select::new("Choose your favorite language", languages.clone());

    match select.ask(&mut console) {
        Ok(index) => {
            console.print_text("");
            console.print(
                &format!("You selected: {} (index {})", languages[index], index)
                    .bold()
                    .green(),
            );
        }
        Err(e) => {
            console.print_text(&format!("Error: {}", e));
        }
    }

    console.print_text("");

    // ── Select with default ──────────────────────────────────────────────
    console.print(&Rule::with_title("Select with Default"));
    console.print(&"Press Enter without typing to accept the default.".italic());
    console.print_text("");

    let editors = vec![
        "Neovim".into(),
        "VS Code".into(),
        "Helix".into(),
        "Emacs".into(),
    ];

    let select_default = Select::new("Choose an editor", editors.clone())
        .with_default(0); // Default to Neovim (index 0)

    match select_default.ask(&mut console) {
        Ok(index) => {
            console.print_text("");
            console.print(
                &format!("You selected: {}", editors[index])
                    .bold()
                    .cyan(),
            );
        }
        Err(e) => {
            console.print_text(&format!("Error: {}", e));
        }
    }

    console.print_text("");

    // ── MultiSelect: Choose toppings ─────────────────────────────────────
    console.print(&Rule::with_title("MultiSelect Prompt"));
    console.print(&"Enter comma-separated numbers, or type 'all'.".italic());
    console.print_text("");

    let toppings = vec![
        "Pepperoni".into(),
        "Mushrooms".into(),
        "Olives".into(),
        "Onions".into(),
        "Bell Peppers".into(),
        "Extra Cheese".into(),
    ];

    let multi = MultiSelect::new("Select your pizza toppings", toppings.clone())
        .with_min(1); // At least one topping required

    match multi.ask(&mut console) {
        Ok(indices) => {
            console.print_text("");
            let selected: Vec<&str> = indices.iter().map(|&i| toppings[i].as_str()).collect();
            console.print(
                &format!("Your toppings: {}", selected.join(", "))
                    .bold()
                    .bright_yellow(),
            );
        }
        Err(e) => {
            console.print_text(&format!("Error: {}", e));
        }
    }

    console.print_text("");
    console.print(&Rule::with_title("Done"));
}
