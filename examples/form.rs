//! Form — multi-step chained prompts (Input, Confirm, Select).
//!
//! Demonstrates:
//! - `Form::new()` — create an empty form.
//! - `FormField::input("key", "prompt")` — free-text input field.
//! - `FormField::confirm("key", "prompt", default)` — yes/no confirmation.
//! - `FormField::select("key", "prompt", options)` — pick one from a list.
//! - `.validate(|s| …)` — attach a validator to an Input field.
//! - `form.run()` — read from stdin interactively.
//! - `form.run_with_input(&mut reader)` — testable variant (used here).
//!
//! The example reads from stdin when run interactively. When stdin is a pipe
//! the form receives the answers from the `Cursor`-backed reader below so it
//! completes without blocking.
//!
//! Run with: cargo run --example form --features interactive

use gilt::form::{Form, FormField};
use std::io::Cursor;

fn main() {
    let form = Form::new()
        // Free-text input with a non-empty validator.
        .field(
            FormField::input("username", "GitHub username").validate(|s| {
                if s.is_empty() {
                    Err("Username cannot be blank".to_string())
                } else {
                    Ok(())
                }
            }),
        )
        // Yes/no confirmation — default = true (Enter → "true").
        .field(FormField::confirm("push", "Push to remote?", true))
        // Select one of three options (by value or by 1-based index).
        .field(FormField::select(
            "branch",
            "Target branch",
            vec!["main".to_string(), "dev".to_string(), "staging".to_string()],
        ))
        // Accessible mode off (plain-text fallback disabled for this demo).
        .accessible(false);

    // Simulated answers: "alice\n" for username, "\n" for confirm (accepts default=true),
    // "2\n" for select (picks "dev" by 1-based index).
    let input_bytes = b"alice\n\n2\n".as_ref();
    let mut cursor = Cursor::new(input_bytes);

    let result = form
        .run_with_input(&mut cursor)
        .expect("form completed successfully");

    println!("=== Collected form values ===");
    let mut keys: Vec<&str> = result.keys().map(|s| s.as_str()).collect();
    keys.sort_unstable();
    for key in &keys {
        println!("  {}: {}", key, result[*key]);
    }

    // Verify collected values.
    assert_eq!(result["username"], "alice");
    assert_eq!(result["push"], "true"); // default=true + Enter
    assert_eq!(result["branch"], "dev"); // option index 2

    println!("\n[assertions passed]");
    println!("[tip] Call form.run() to read answers from stdin interactively.");
}
