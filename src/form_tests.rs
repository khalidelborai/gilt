//! TDD RED tests for Task 2: Form builder (huh pattern).
//!
//! These tests compile only AFTER `src/form.rs` (with Form + FormField) is implemented.

use std::collections::HashMap;
use std::io::Cursor;

use crate::form::{Form, FormField};

// ---------------------------------------------------------------------------
// Basic Form: Input + Confirm, happy path
// ---------------------------------------------------------------------------

#[test]
fn test_form_input_and_confirm_happy_path() {
    // Input: "Alice", Confirm: "y"
    let input_bytes: &[u8] = b"Alice\ny\n";
    let mut cursor = Cursor::new(input_bytes);

    let result = Form::new()
        .field(FormField::input("name", "What is your name?"))
        .field(FormField::confirm("ok", "Are you sure?", false))
        .run_with_input(&mut cursor)
        .expect("form should succeed");

    assert_eq!(result.get("name").map(|s| s.as_str()), Some("Alice"));
    assert_eq!(result.get("ok").map(|s| s.as_str()), Some("true"));
}

// ---------------------------------------------------------------------------
// Input validation: reject empty, accept non-empty
// ---------------------------------------------------------------------------

#[test]
fn test_form_input_validation_re_prompts_on_empty() {
    // First line is empty (rejected), second is "Bob" (accepted)
    let input_bytes: &[u8] = b"\nBob\n";
    let mut cursor = Cursor::new(input_bytes);

    let result = Form::new()
        .field(FormField::input("name", "Name?").validate(|s| {
            if s.trim().is_empty() {
                Err("Name cannot be empty".to_string())
            } else {
                Ok(())
            }
        }))
        .run_with_input(&mut cursor)
        .expect("form should succeed after re-prompt");

    assert_eq!(result.get("name").map(|s| s.as_str()), Some("Bob"));
}

// ---------------------------------------------------------------------------
// Confirm field: default false, enter pressed → "false"
// ---------------------------------------------------------------------------

#[test]
fn test_form_confirm_default_false() {
    let input_bytes: &[u8] = b"\n"; // blank → use default (false)
    let mut cursor = Cursor::new(input_bytes);

    let result = Form::new()
        .field(FormField::confirm("proceed", "Continue?", false))
        .run_with_input(&mut cursor)
        .expect("form should succeed");

    assert_eq!(result.get("proceed").map(|s| s.as_str()), Some("false"));
}

// ---------------------------------------------------------------------------
// Select field: pick option 2 (0-indexed 1)
// ---------------------------------------------------------------------------

#[test]
fn test_form_select_field() {
    // "banana" is the second option
    let input_bytes: &[u8] = b"banana\n";
    let mut cursor = Cursor::new(input_bytes);

    let result = Form::new()
        .field(FormField::select(
            "fruit",
            "Pick a fruit",
            vec![
                "apple".to_string(),
                "banana".to_string(),
                "cherry".to_string(),
            ],
        ))
        .run_with_input(&mut cursor)
        .expect("form should succeed");

    assert_eq!(result.get("fruit").map(|s| s.as_str()), Some("banana"));
}

// ---------------------------------------------------------------------------
// run_with_input returns correct HashMap keys
// ---------------------------------------------------------------------------

#[test]
fn test_form_returns_all_keys() {
    let input_bytes: &[u8] = b"Alice\ny\n";
    let mut cursor = Cursor::new(input_bytes);

    let result: HashMap<String, String> = Form::new()
        .field(FormField::input("username", "Username:"))
        .field(FormField::confirm("agree", "Agree to terms?", true))
        .run_with_input(&mut cursor)
        .expect("form should succeed");

    assert!(
        result.contains_key("username"),
        "result must contain 'username'"
    );
    assert!(result.contains_key("agree"), "result must contain 'agree'");
}

// ---------------------------------------------------------------------------
// Accessible mode: force plain output, same result
// ---------------------------------------------------------------------------

#[test]
fn test_form_accessible_mode_same_result() {
    let input_bytes: &[u8] = b"Carol\n";
    let mut cursor = Cursor::new(input_bytes);

    // accessible(true) forces plain text prompts (no ANSI styling)
    let result = Form::new()
        .accessible(true)
        .field(FormField::input("name", "Your name?"))
        .run_with_input(&mut cursor)
        .expect("accessible form should succeed");

    assert_eq!(result.get("name").map(|s| s.as_str()), Some("Carol"));
}

// ---------------------------------------------------------------------------
// Auto-detect accessible: NO_COLOR set → same flow (just no styling)
// ---------------------------------------------------------------------------

#[test]
fn test_form_auto_accessible_no_color_env() {
    // We test the auto-detection logic here; the result should still be correct.
    // We set NO_COLOR in env before construction, then clear it after.
    // (In CI NO_COLOR may already be set; this is fine.)
    let input_bytes: &[u8] = b"Dave\n";
    let mut cursor = Cursor::new(input_bytes);

    std::env::set_var("NO_COLOR", "1");
    let result = Form::new()
        .field(FormField::input("name", "Your name?"))
        .run_with_input(&mut cursor);
    std::env::remove_var("NO_COLOR");

    let result = result.expect("form should succeed with NO_COLOR set");
    assert_eq!(result.get("name").map(|s| s.as_str()), Some("Dave"));
}
