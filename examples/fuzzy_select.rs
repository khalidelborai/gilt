//! Fuzzy select state — dep-free filter + ranking core.
//!
//! Demonstrates:
//! - `FuzzySelectState::new(items)` — initial state, all items shown.
//! - `state.set_filter("...")` — case-insensitive substring filter; resets
//!   highlight to 0 and re-ranks results.
//! - `state.filtered()` — indices into the original item slice.
//! - `state.selection()` — currently highlighted item.
//! - `state.move_down()` / `state.move_up()` — arrow-key navigation.
//!
//! Note: `--features tty-select` enables `FuzzySelect::ask()`, which adds
//! an interactive arrow-key driven TUI using crossterm raw mode.
//!
//! Run with: cargo run --example fuzzy_select

use gilt::fuzzy_select::FuzzySelectState;

fn main() {
    let words = vec![
        "apple",
        "avocado",
        "banana",
        "blueberry",
        "cherry",
        "grape",
        "grapefruit",
        "lemon",
        "lime",
        "mango",
        "papaya",
        "peach",
        "pear",
        "pineapple",
        "plum",
        "raspberry",
        "strawberry",
        "watermelon",
    ];

    let mut state = FuzzySelectState::new(words.clone());

    println!("=== No filter (all {} items) ===", words.len());
    for &idx in state.filtered() {
        println!("  {}", words[idx]);
    }

    // --- Set a filter --------------------------------------------------------

    state.set_filter("ap");
    println!(
        "\n=== Filter = \"ap\" ({} matches) ===",
        state.filtered().len()
    );
    for &idx in state.filtered() {
        let marker = if state.selection() == Some(&words[idx]) {
            " ◀ (selected)"
        } else {
            ""
        };
        println!("  {}{}", words[idx], marker);
    }

    // --- Navigate with arrow keys (logical) ----------------------------------

    state.move_down();
    println!("\nAfter move_down → selection = {:?}", state.selection());
    state.move_down();
    println!("After move_down → selection = {:?}", state.selection());
    state.move_up();
    println!("After move_up  → selection = {:?}", state.selection());

    // --- Exhaustive filter ---------------------------------------------------

    state.set_filter("zzz_no_match");
    println!(
        "\n=== Filter = \"zzz_no_match\" → {} matches ===",
        state.filtered().len()
    );
    assert_eq!(state.filtered().len(), 0);
    assert_eq!(state.selection(), None, "empty list → no selection");

    // --- Reset to empty filter -----------------------------------------------

    state.set_filter("");
    println!(
        "\n=== Filter cleared → {} items (all back) ===",
        state.filtered().len()
    );
    assert_eq!(state.filtered().len(), words.len());

    println!("\n[assertions passed]");
    println!("[tip] Add --features tty-select for the interactive FuzzySelect::ask() driver.");
}
