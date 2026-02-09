//! Demonstrates the full Prompt API with simulated input.
//!
//! Run with: `cargo run --example prompt_demo`

use std::io::Cursor;

use gilt::console::Console;
use gilt::prompt::{
    ask_float_with_input, ask_int_with_input, confirm_with_input, MultiSelect, Prompt, Select,
};
use gilt::rule::Rule;

fn main() {
    let mut console = Console::builder()
        .width(80)
        .force_terminal(true)
        .no_color(false)
        .build();

    // ── 1. Basic prompt ─────────────────────────────────────────────────
    console.print(&Rule::with_title("1. Basic Prompt"));

    let mut input = Cursor::new(b"Alice\n" as &[u8]);
    let result = Prompt::new("What is your name?").ask_with_input(&mut input);
    console.print_text(&format!("  You entered: {}\n", result));

    // ── 2. Prompt with default ──────────────────────────────────────────
    console.print(&Rule::with_title("2. Prompt with Default"));

    let mut input = Cursor::new(b"\n" as &[u8]); // empty input -> uses default
    let result = Prompt::new("Favorite color?")
        .with_default("blue")
        .ask_with_input(&mut input);
    console.print_text(&format!(
        "  Empty input with default=\"blue\": {}\n",
        result
    ));

    // ── 3. Prompt with choices ──────────────────────────────────────────
    console.print(&Rule::with_title("3. Prompt with Choices"));

    let mut input = Cursor::new(b"green\n" as &[u8]);
    let result = Prompt::new("Pick a color")
        .with_choices(vec!["red".into(), "green".into(), "blue".into()])
        .ask_with_input(&mut input);
    console.print_text(&format!("  Selected: {}\n", result));

    // ── 4. Confirm prompt ───────────────────────────────────────────────
    console.print(&Rule::with_title("4. Confirm Prompt"));

    let mut input = Cursor::new(b"y\n" as &[u8]);
    let result = confirm_with_input("Continue?", &mut input);
    console.print_text(&format!("  Confirmed: {}\n", result));

    let mut input = Cursor::new(b"no\n" as &[u8]);
    let result = confirm_with_input("Delete everything?", &mut input);
    console.print_text(&format!("  Confirmed: {}\n", result));

    // ── 5. Integer prompt ───────────────────────────────────────────────
    console.print(&Rule::with_title("5. Integer Prompt"));

    let mut input = Cursor::new(b"42\n" as &[u8]);
    let result = ask_int_with_input("Enter your age", &mut input);
    console.print_text(&format!("  Age: {}\n", result));

    // ── 6. Float prompt ─────────────────────────────────────────────────
    console.print(&Rule::with_title("6. Float Prompt"));

    let mut input = Cursor::new(b"3.14\n" as &[u8]);
    let result = ask_float_with_input("Enter pi", &mut input);
    console.print_text(&format!("  Pi: {:.2}\n", result));

    // ── 7. Select (single choice from numbered list) ────────────────────
    console.print(&Rule::with_title("7. Select Prompt"));

    let mut input = Cursor::new(b"1\n" as &[u8]);
    let select = Select::new(
        "Choose a language",
        vec!["Rust".into(), "Python".into(), "Go".into()],
    );
    let index = select.ask_with_input(&mut console, &mut input).unwrap();
    let choices = &["Rust", "Python", "Go"];
    console.print_text(&format!("  Selected index {}: {}\n", index, choices[index]));

    // ── 8. MultiSelect (multiple choices from numbered list) ────────────
    console.print(&Rule::with_title("8. MultiSelect Prompt"));

    let mut input = Cursor::new(b"1,3\n" as &[u8]);
    let multi = MultiSelect::new(
        "Choose toppings",
        vec!["cheese".into(), "pepperoni".into(), "mushrooms".into()],
    )
    .with_min(1);
    let values = multi
        .ask_values_with_input(&mut console, &mut input)
        .unwrap();
    console.print_text(&format!("  Selected: {}\n", values.join(", ")));

    // ── 9. Case-insensitive choices ─────────────────────────────────────
    console.print(&Rule::with_title("9. Case-Insensitive Choices"));

    let mut input = Cursor::new(b"rust\n" as &[u8]);
    let result = Prompt::new("Favorite language?")
        .with_choices(vec!["Rust".into(), "Python".into(), "Go".into()])
        .with_case_sensitive(false)
        .ask_with_input(&mut input);
    console.print_text(&format!("  Typed \"rust\", resolved to: {}\n", result));

    // ── Done ────────────────────────────────────────────────────────────
    console.print(&Rule::new());
    console.print_text("All prompt demos complete.");
}
