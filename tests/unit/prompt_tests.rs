//! Prompt module tests
//!
//! Extracted from src/prompt.rs

use super::*;
use crate::console::Console;
use crate::prompt::*;
use std::io::Cursor;

// -- Simple prompt returns input ----------------------------------------

#[test]
fn test_simple_prompt_returns_input() {
    let p = Prompt::new("Enter name");
    let mut input = Cursor::new(b"Alice\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "Alice");
}

// -- Prompt with default (empty input returns default) ------------------

#[test]
fn test_prompt_with_default_empty_returns_default() {
    let p = Prompt::new("Enter name").with_default("Bob");
    let mut input = Cursor::new(b"\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "Bob");
}

#[test]
fn test_prompt_with_default_non_empty_returns_input() {
    let p = Prompt::new("Enter name").with_default("Bob");
    let mut input = Cursor::new(b"Charlie\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "Charlie");
}

// -- Prompt with choices (valid choice accepted) ------------------------

#[test]
fn test_prompt_with_choices_valid() {
    let p = Prompt::new("Pick fruit").with_choices(vec![
        "apple".into(),
        "orange".into(),
        "pear".into(),
    ]);
    let mut input = Cursor::new(b"apple\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "apple");
}

// -- Prompt with choices (invalid choice rejected, loops) ---------------

#[test]
fn test_prompt_with_choices_invalid_then_valid() {
    let p = Prompt::new("Pick fruit").with_choices(vec![
        "apple".into(),
        "orange".into(),
        "pear".into(),
    ]);
    // First "banana" is invalid, then "orange" is valid
    let mut input = Cursor::new(b"banana\norange\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "orange");
}

// -- Case insensitive choices -------------------------------------------

#[test]
fn test_case_insensitive_choices() {
    let p = Prompt::new("Pick")
        .with_choices(vec!["Apple".into(), "Orange".into()])
        .with_case_sensitive(false);
    let mut input = Cursor::new(b"apple\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    // Should return the original-cased version
    assert_eq!(result, "Apple");
}

#[test]
fn test_case_sensitive_choices_reject_wrong_case() {
    let p = Prompt::new("Pick")
        .with_choices(vec!["Apple".into(), "Orange".into()])
        .with_case_sensitive(true);
    // "apple" is wrong case; then "Apple" is correct
    let mut input = Cursor::new(b"apple\nApple\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "Apple");
}

// -- confirm() returns true/false ---------------------------------------

#[test]
fn test_confirm_yes() {
    let mut input = Cursor::new(b"y\n" as &[u8]);
    let result = confirm_with_input("Continue?", &mut input);
    assert!(result);
}

#[test]
fn test_confirm_yes_full() {
    let mut input = Cursor::new(b"yes\n" as &[u8]);
    let result = confirm_with_input("Continue?", &mut input);
    assert!(result);
}

#[test]
fn test_confirm_no() {
    let mut input = Cursor::new(b"n\n" as &[u8]);
    let result = confirm_with_input("Continue?", &mut input);
    assert!(!result);
}

#[test]
fn test_confirm_no_full() {
    let mut input = Cursor::new(b"no\n" as &[u8]);
    let result = confirm_with_input("Continue?", &mut input);
    assert!(!result);
}

#[test]
fn test_confirm_case_insensitive() {
    let mut input = Cursor::new(b"Y\n" as &[u8]);
    let result = confirm_with_input("Continue?", &mut input);
    assert!(result);
}

#[test]
fn test_confirm_invalid_then_valid() {
    let mut input = Cursor::new(b"maybe\ny\n" as &[u8]);
    let result = confirm_with_input("Continue?", &mut input);
    assert!(result);
}

// -- ask_int() valid integer --------------------------------------------

#[test]
fn test_ask_int_valid() {
    let mut input = Cursor::new(b"42\n" as &[u8]);
    let result = ask_int_with_input("Enter number", &mut input);
    assert_eq!(result, 42);
}

#[test]
fn test_ask_int_negative() {
    let mut input = Cursor::new(b"-7\n" as &[u8]);
    let result = ask_int_with_input("Enter number", &mut input);
    assert_eq!(result, -7);
}

// -- ask_int() invalid input loops --------------------------------------

#[test]
fn test_ask_int_invalid_then_valid() {
    let mut input = Cursor::new(b"abc\n42\n" as &[u8]);
    let result = ask_int_with_input("Enter number", &mut input);
    assert_eq!(result, 42);
}

// -- ask_float() valid float --------------------------------------------

#[test]
fn test_ask_float_valid() {
    let mut input = Cursor::new(b"3.14\n" as &[u8]);
    let result = ask_float_with_input("Enter number", &mut input);
    assert!((result - 3.14).abs() < f64::EPSILON);
}

#[test]
fn test_ask_float_integer_input() {
    let mut input = Cursor::new(b"7\n" as &[u8]);
    let result = ask_float_with_input("Enter number", &mut input);
    assert!((result - 7.0).abs() < f64::EPSILON);
}

// -- ask_float() invalid input loops ------------------------------------

#[test]
fn test_ask_float_invalid_then_valid() {
    let mut input = Cursor::new(b"xyz\n2.718\n" as &[u8]);
    let result = ask_float_with_input("Enter number", &mut input);
    assert!((result - 2.718).abs() < f64::EPSILON);
}

// -- Prompt text includes choices when show_choices is true --------------

#[test]
fn test_prompt_text_includes_choices() {
    let p = Prompt::new("Pick fruit")
        .with_choices(vec!["apple".into(), "orange".into()])
        .with_show_choices(true);
    let text = p.make_prompt();
    let plain = text.plain().to_string();
    assert!(plain.contains("[apple/orange]"));
}

#[test]
fn test_prompt_text_hides_choices_when_disabled() {
    let p = Prompt::new("Pick fruit")
        .with_choices(vec!["apple".into(), "orange".into()])
        .with_show_choices(false);
    let text = p.make_prompt();
    let plain = text.plain().to_string();
    assert!(!plain.contains("apple"));
    assert!(!plain.contains("orange"));
}

// -- Prompt text includes default when show_default is true --------------

#[test]
fn test_prompt_text_includes_default() {
    let p = Prompt::new("Enter name")
        .with_default("World")
        .with_show_default(true);
    let text = p.make_prompt();
    let plain = text.plain().to_string();
    assert!(plain.contains("(World)"));
}

#[test]
fn test_prompt_text_hides_default_when_disabled() {
    let p = Prompt::new("Enter name")
        .with_default("World")
        .with_show_default(false);
    let text = p.make_prompt();
    let plain = text.plain().to_string();
    assert!(!plain.contains("(World)"));
}

// -- Password flag (verify it's stored) ---------------------------------

#[test]
fn test_password_flag() {
    let p = Prompt::new("Password").with_password(true);
    assert!(p.password);

    let p2 = Prompt::new("Password").with_password(false);
    assert!(!p2.password);
}

// -- Builder methods ----------------------------------------------------

#[test]
fn test_builder_with_choices() {
    let p = Prompt::new("test").with_choices(vec!["a".into(), "b".into()]);
    assert_eq!(p.choices, Some(vec!["a".to_string(), "b".to_string()]));
}

#[test]
fn test_builder_with_default() {
    let p = Prompt::new("test").with_default("val");
    assert_eq!(p.default, Some("val".to_string()));
}

#[test]
fn test_builder_with_case_sensitive() {
    let p = Prompt::new("test").with_case_sensitive(false);
    assert!(!p.case_sensitive);
}

#[test]
fn test_builder_with_show_default() {
    let p = Prompt::new("test").with_show_default(false);
    assert!(!p.show_default);
}

#[test]
fn test_builder_with_show_choices() {
    let p = Prompt::new("test").with_show_choices(false);
    assert!(!p.show_choices);
}

#[test]
fn test_builder_with_password() {
    let p = Prompt::new("test").with_password(true);
    assert!(p.password);
}

#[test]
fn test_builder_with_completions() {
    let p = Prompt::new("test").with_completions(vec!["foo".into(), "bar".into()]);
    assert_eq!(
        p.completions,
        Some(vec!["foo".to_string(), "bar".to_string()])
    );
}

#[test]
fn test_builder_completions_default_none() {
    let p = Prompt::new("test");
    assert!(p.completions.is_none());
}

// -- Prompt suffix is ": " ----------------------------------------------

#[test]
fn test_prompt_suffix() {
    let p = Prompt::new("Enter value");
    let text = p.make_prompt();
    let plain = text.plain().to_string();
    assert!(plain.ends_with(": "));
}

// -- Default on EOF -----------------------------------------------------

#[test]
fn test_default_on_eof() {
    let p = Prompt::new("Enter").with_default("fallback");
    let mut input = Cursor::new(b"" as &[u8]); // EOF immediately
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "fallback");
}

// -- No default, no choices, empty input --------------------------------

#[test]
fn test_no_default_empty_returns_empty() {
    let p = Prompt::new("Enter");
    let mut input = Cursor::new(b"\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "");
}

// -- Prompt with both choices and default --------------------------------

#[test]
fn test_prompt_text_choices_and_default() {
    let p = Prompt::new("Pick")
        .with_choices(vec!["a".into(), "b".into()])
        .with_default("a")
        .with_show_choices(true)
        .with_show_default(true);
    let text = p.make_prompt();
    let plain = text.plain().to_string();
    assert!(plain.contains("[a/b]"));
    assert!(plain.contains("(a)"));
    assert!(plain.ends_with(": "));
}

// -- Choices with default on empty input ---------------------------------

#[test]
fn test_choices_default_on_empty() {
    let p = Prompt::new("Pick")
        .with_choices(vec!["a".into(), "b".into()])
        .with_default("a");
    let mut input = Cursor::new(b"\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "a");
}

// -- Styled spans in prompt text ----------------------------------------

#[test]
fn test_prompt_has_styled_choices_span() {
    let p = Prompt::new("Pick")
        .with_choices(vec!["x".into(), "y".into()])
        .with_show_choices(true);
    let text = p.make_prompt();
    // Should have at least one span for the choices styling
    assert!(!text.spans().is_empty());
}

#[test]
fn test_prompt_has_styled_default_span() {
    let p = Prompt::new("Pick")
        .with_default("z")
        .with_show_default(true);
    let text = p.make_prompt();
    // Should have at least one span for the default styling
    assert!(!text.spans().is_empty());
}

// -- InvalidResponse ----------------------------------------------------

#[test]
fn test_invalid_response_display() {
    let err = InvalidResponse {
        message: "bad input".to_string(),
    };
    assert_eq!(format!("{}", err), "bad input");
}

#[test]
fn test_invalid_response_debug() {
    let err = InvalidResponse {
        message: "bad".to_string(),
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("InvalidResponse"));
    assert!(debug.contains("bad"));
}

// -- Password mode (ask_with_input reads normally — password only affects ask()) --

#[test]
fn test_password_ask_with_input_reads_normally() {
    // ask_with_input always reads from the BufRead regardless of password flag.
    // This verifies the password flag doesn't break BufRead-based input.
    let p = Prompt::new("Password").with_password(true);
    let mut input = Cursor::new(b"secret123\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "secret123");
}

#[test]
fn test_password_with_default_on_empty() {
    let p = Prompt::new("Password")
        .with_password(true)
        .with_default("default_pass");
    let mut input = Cursor::new(b"\n" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "default_pass");
}

#[test]
fn test_password_with_default_on_eof() {
    let p = Prompt::new("Password")
        .with_password(true)
        .with_default("fallback");
    let mut input = Cursor::new(b"" as &[u8]);
    let result = p.ask_with_input(&mut input);
    assert_eq!(result, "fallback");
}

#[test]
fn test_password_prompt_text_unchanged() {
    // Password mode should NOT affect the rendered prompt text
    let p1 = Prompt::new("Enter password").with_password(true);
    let p2 = Prompt::new("Enter password").with_password(false);
    let text1 = p1.make_prompt().plain().to_string();
    let text2 = p2.make_prompt().plain().to_string();
    assert_eq!(text1, text2);
}

// ===================================================================
// Select tests
// ===================================================================

// -- Select: parse single number input ----------------------------------

#[test]
fn test_select_parse_single_number() {
    let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    assert_eq!(s.parse_input("2"), Ok(1)); // 1-based "2" -> 0-based 1
}

#[test]
fn test_select_parse_first_choice() {
    let s = Select::new("Pick", vec!["A".into(), "B".into()]);
    assert_eq!(s.parse_input("1"), Ok(0));
}

#[test]
fn test_select_parse_last_choice() {
    let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into(), "D".into()]);
    assert_eq!(s.parse_input("4"), Ok(3));
}

// -- Select: validate number in range -----------------------------------

#[test]
fn test_select_validate_in_range() {
    let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    assert!(s.parse_input("1").is_ok());
    assert!(s.parse_input("2").is_ok());
    assert!(s.parse_input("3").is_ok());
}

// -- Select: validate number out of range -------------------------------

#[test]
fn test_select_validate_out_of_range_high() {
    let s = Select::new("Pick", vec!["A".into(), "B".into()]);
    let result = s.parse_input("3");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("between 1 and 2"));
}

#[test]
fn test_select_validate_out_of_range_zero() {
    let s = Select::new("Pick", vec!["A".into(), "B".into()]);
    let result = s.parse_input("0");
    assert!(result.is_err());
}

#[test]
fn test_select_validate_not_a_number() {
    let s = Select::new("Pick", vec!["A".into(), "B".into()]);
    let result = s.parse_input("abc");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("not a valid number"));
}

// -- Select: default selection when input is empty ----------------------

#[test]
fn test_select_default_on_empty() {
    let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_default(1);
    assert_eq!(s.parse_input(""), Ok(1));
}

#[test]
fn test_select_no_default_on_empty() {
    let s = Select::new("Pick", vec!["A".into(), "B".into()]);
    let result = s.parse_input("");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("enter a number"));
}

// -- Select: handle whitespace in input ---------------------------------

#[test]
fn test_select_whitespace_input() {
    let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    assert_eq!(s.parse_input("  2  "), Ok(1));
}

// -- Select: empty choices list error -----------------------------------

#[test]
fn test_select_empty_choices() {
    let s = Select::new("Pick", vec![]);
    let mut console = Console::builder().quiet(true).build();
    let mut input = Cursor::new(b"1\n" as &[u8]);
    let result = s.ask_with_input(&mut console, &mut input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message, "No choices provided");
}

// -- Select: display formatting -----------------------------------------

#[test]
fn test_select_format_choices() {
    let s = Select::new(
        "Select a color",
        vec!["Red".into(), "Green".into(), "Blue".into()],
    );
    let output = s.format_choices();
    assert!(output.contains("? Select a color:"));
    assert!(output.contains("  1) Red"));
    assert!(output.contains("  2) Green"));
    assert!(output.contains("  3) Blue"));
}

#[test]
fn test_select_format_input_prompt_no_default() {
    let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    let prompt = s.format_input_prompt();
    assert_eq!(prompt, "Enter choice [1-3]: ");
}

#[test]
fn test_select_format_input_prompt_with_default() {
    let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_default(1);
    let prompt = s.format_input_prompt();
    assert_eq!(prompt, "Enter choice [1-3] (2): ");
}

// -- Select: ask_with_input valid interaction ---------------------------

#[test]
fn test_select_ask_with_input_valid() {
    let s = Select::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    let mut console = Console::builder().quiet(true).build();
    let mut input = Cursor::new(b"2\n" as &[u8]);
    let result = s.ask_with_input(&mut console, &mut input);
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_select_ask_with_input_invalid_then_valid() {
    let s = Select::new("Pick", vec!["A".into(), "B".into()]);
    let mut console = Console::builder().quiet(true).build();
    let mut input = Cursor::new(b"5\n1\n" as &[u8]);
    let result = s.ask_with_input(&mut console, &mut input);
    assert_eq!(result.unwrap(), 0);
}

// -- Select: ask_value --------------------------------------------------

#[test]
fn test_select_ask_value_with_input() {
    let s = Select::new("Pick", vec!["Red".into(), "Green".into(), "Blue".into()]);
    let mut console = Console::builder().quiet(true).build();
    let mut input = Cursor::new(b"3\n" as &[u8]);
    let result = s.ask_value_with_input(&mut console, &mut input);
    assert_eq!(result.unwrap(), "Blue");
}

// -- Select: default on EOF ---------------------------------------------

#[test]
fn test_select_default_on_eof() {
    let s = Select::new("Pick", vec!["A".into(), "B".into()]).with_default(0);
    let mut console = Console::builder().quiet(true).build();
    let mut input = Cursor::new(b"" as &[u8]);
    let result = s.ask_with_input(&mut console, &mut input);
    assert_eq!(result.unwrap(), 0);
}

// -- Select: builder methods --------------------------------------------

#[test]
fn test_select_builder_with_default() {
    let s = Select::new("Pick", vec!["A".into()]).with_default(0);
    assert_eq!(s.default, Some(0));
}

#[test]
fn test_select_builder_with_style() {
    let style = Style::parse("red bold").unwrap();
    let s = Select::new("Pick", vec!["A".into()]).with_style(style);
    assert_eq!(s.style.bold(), Some(true));
}

#[test]
fn test_select_builder_with_highlight_style() {
    let style = Style::parse("green").unwrap();
    let s = Select::new("Pick", vec!["A".into()]).with_highlight_style(style);
    assert!(s.highlight_style.color().is_some());
}

// ===================================================================
// MultiSelect tests
// ===================================================================

// -- MultiSelect: parse comma-separated input ---------------------------

#[test]
fn test_multiselect_parse_comma_separated() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into(), "D".into()]);
    assert_eq!(ms.parse_input("1,3"), Ok(vec![0, 2]));
}

#[test]
fn test_multiselect_parse_single() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
    assert_eq!(ms.parse_input("2"), Ok(vec![1]));
}

// -- MultiSelect: parse "all" keyword -----------------------------------

#[test]
fn test_multiselect_parse_all() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    assert_eq!(ms.parse_input("all"), Ok(vec![0, 1, 2]));
}

#[test]
fn test_multiselect_parse_all_case_insensitive() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
    assert_eq!(ms.parse_input("ALL"), Ok(vec![0, 1]));
    assert_eq!(ms.parse_input("All"), Ok(vec![0, 1]));
}

// -- MultiSelect: handle whitespace in input ----------------------------

#[test]
fn test_multiselect_whitespace_input() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    assert_eq!(ms.parse_input("  1 , 3 "), Ok(vec![0, 2]));
}

#[test]
fn test_multiselect_trailing_comma() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
    assert_eq!(ms.parse_input("1,2,"), Ok(vec![0, 1]));
}

// -- MultiSelect: default selection when input is empty -----------------

#[test]
fn test_multiselect_default_on_empty() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()])
        .with_defaults(vec![0, 2]);
    assert_eq!(ms.parse_input(""), Ok(vec![0, 2]));
}

// -- MultiSelect: min/max selection validation --------------------------

#[test]
fn test_multiselect_min_validation() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_min(2);
    let result = ms.parse_input("1");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("at least 2"));
}

#[test]
fn test_multiselect_min_satisfied() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_min(2);
    assert_eq!(ms.parse_input("1,2"), Ok(vec![0, 1]));
}

#[test]
fn test_multiselect_max_validation() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_max(1);
    let result = ms.parse_input("1,2");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("at most 1"));
}

#[test]
fn test_multiselect_max_satisfied() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_max(2);
    assert_eq!(ms.parse_input("1,3"), Ok(vec![0, 2]));
}

#[test]
fn test_multiselect_min_max_combined() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into(), "D".into()])
        .with_min(1)
        .with_max(3);
    assert!(ms.parse_input("").is_err()); // 0 < min
    assert!(ms.parse_input("1").is_ok()); // 1 >= min
    assert!(ms.parse_input("1,2,3").is_ok()); // 3 <= max
    assert!(ms.parse_input("1,2,3,4").is_err()); // 4 > max
}

// -- MultiSelect: empty choices list error ------------------------------

#[test]
fn test_multiselect_empty_choices() {
    let ms = MultiSelect::new("Pick", vec![]);
    let mut console = Console::builder().quiet(true).build();
    let mut input = Cursor::new(b"1\n" as &[u8]);
    let result = ms.ask_with_input(&mut console, &mut input);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().message, "No choices provided");
}

// -- MultiSelect: validate out of range ---------------------------------

#[test]
fn test_multiselect_out_of_range() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
    let result = ms.parse_input("3");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("out of range"));
}

#[test]
fn test_multiselect_zero() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
    let result = ms.parse_input("0");
    assert!(result.is_err());
}

// -- MultiSelect: display formatting ------------------------------------

#[test]
fn test_multiselect_format_choices() {
    let ms = MultiSelect::new(
        "Select colors",
        vec!["Red".into(), "Green".into(), "Blue".into()],
    );
    let output = ms.format_choices();
    assert!(output.contains("? Select colors (comma-separated):"));
    assert!(output.contains("  1) Red"));
    assert!(output.contains("  2) Green"));
    assert!(output.contains("  3) Blue"));
}

#[test]
fn test_multiselect_format_input_prompt_no_defaults() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    let prompt = ms.format_input_prompt();
    assert_eq!(prompt, "Enter choices [1-3, e.g. 1,3]: ");
}

#[test]
fn test_multiselect_format_input_prompt_with_defaults() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()])
        .with_defaults(vec![0, 2]);
    let prompt = ms.format_input_prompt();
    assert_eq!(prompt, "Enter choices [1-3, e.g. 1,3] (1,3): ");
}

// -- MultiSelect: ask_with_input valid interaction ----------------------

#[test]
fn test_multiselect_ask_with_input_valid() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    let mut console = Console::builder().quiet(true).build();
    let mut input = Cursor::new(b"1,3\n" as &[u8]);
    let result = ms.ask_with_input(&mut console, &mut input);
    assert_eq!(result.unwrap(), vec![0, 2]);
}

#[test]
fn test_multiselect_ask_with_input_invalid_then_valid() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]).with_min(1);
    let mut console = Console::builder().quiet(true).build();
    // First line is empty (fails min), then "2" succeeds
    let mut input = Cursor::new(b"\n2\n" as &[u8]);
    let result = ms.ask_with_input(&mut console, &mut input);
    assert_eq!(result.unwrap(), vec![1]);
}

// -- MultiSelect: ask_values --------------------------------------------

#[test]
fn test_multiselect_ask_values_with_input() {
    let ms = MultiSelect::new("Pick", vec!["Red".into(), "Green".into(), "Blue".into()]);
    let mut console = Console::builder().quiet(true).build();
    let mut input = Cursor::new(b"1,3\n" as &[u8]);
    let result = ms.ask_values_with_input(&mut console, &mut input);
    assert_eq!(result.unwrap(), vec!["Red".to_string(), "Blue".to_string()]);
}

// -- MultiSelect: deduplicate input -------------------------------------

#[test]
fn test_multiselect_deduplicate() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]);
    assert_eq!(ms.parse_input("1,1,2"), Ok(vec![0, 1]));
}

// -- MultiSelect: empty input with min=0 succeeds -----------------------

#[test]
fn test_multiselect_empty_input_min_zero() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
    assert_eq!(ms.parse_input(""), Ok(vec![]));
}

// -- MultiSelect: builder methods ---------------------------------------

#[test]
fn test_multiselect_builder_with_defaults() {
    let ms = MultiSelect::new("Pick", vec!["A".into()]).with_defaults(vec![0]);
    assert_eq!(ms.defaults, vec![0]);
}

#[test]
fn test_multiselect_builder_with_min() {
    let ms = MultiSelect::new("Pick", vec!["A".into()]).with_min(1);
    assert_eq!(ms.min_selections, 1);
}

#[test]
fn test_multiselect_builder_with_max() {
    let ms = MultiSelect::new("Pick", vec!["A".into()]).with_max(3);
    assert_eq!(ms.max_selections, Some(3));
}

#[test]
fn test_multiselect_builder_with_style() {
    let style = Style::parse("red bold").unwrap();
    let ms = MultiSelect::new("Pick", vec!["A".into()]).with_style(style);
    assert_eq!(ms.style.bold(), Some(true));
}

#[test]
fn test_multiselect_builder_with_highlight_style() {
    let style = Style::parse("green").unwrap();
    let ms = MultiSelect::new("Pick", vec!["A".into()]).with_highlight_style(style);
    assert!(ms.highlight_style.color().is_some());
}

// -- Select: display via console capture --------------------------------

#[test]
fn test_select_display_via_capture() {
    let s = Select::new(
        "Pick a fruit",
        vec!["Apple".into(), "Banana".into(), "Cherry".into()],
    );
    let mut console = Console::builder().width(80).force_terminal(true).build();
    console.begin_capture();
    console.print_text(&s.format_choices());
    let captured = console.end_capture();
    assert!(captured.contains("? Pick a fruit:"));
    assert!(captured.contains("1) Apple"));
    assert!(captured.contains("2) Banana"));
    assert!(captured.contains("3) Cherry"));
}

// -- MultiSelect: display via console capture ---------------------------

#[test]
fn test_multiselect_display_via_capture() {
    let ms = MultiSelect::new(
        "Pick colors",
        vec!["Red".into(), "Green".into(), "Blue".into(), "Yellow".into()],
    );
    let mut console = Console::builder().width(80).force_terminal(true).build();
    console.begin_capture();
    console.print_text(&ms.format_choices());
    let captured = console.end_capture();
    assert!(captured.contains("? Pick colors (comma-separated):"));
    assert!(captured.contains("1) Red"));
    assert!(captured.contains("2) Green"));
    assert!(captured.contains("3) Blue"));
    assert!(captured.contains("4) Yellow"));
}

// -- MultiSelect: "all" with max constraint -----------------------------

#[test]
fn test_multiselect_all_with_max() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into(), "C".into()]).with_max(2);
    let result = ms.parse_input("all");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("at most 2"));
}

// -- Select: negative number input --------------------------------------

#[test]
fn test_select_negative_number() {
    let s = Select::new("Pick", vec!["A".into(), "B".into()]);
    let result = s.parse_input("-1");
    assert!(result.is_err());
}

// -- MultiSelect: not a number in list ----------------------------------

#[test]
fn test_multiselect_invalid_number_in_list() {
    let ms = MultiSelect::new("Pick", vec!["A".into(), "B".into()]);
    let result = ms.parse_input("1,abc");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("not a valid number"));
}
