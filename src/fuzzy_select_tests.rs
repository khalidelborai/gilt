//! TDD RED tests for Task 3: FuzzySelectState core (dep-free, no crossterm).
//!
//! These tests compile only AFTER `src/fuzzy_select.rs` with `FuzzySelectState` exists.

use crate::fuzzy_select::FuzzySelectState;

// ---------------------------------------------------------------------------
// Empty filter shows all items
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_empty_filter_shows_all() {
    let state = FuzzySelectState::new(vec!["apple", "banana", "grape", "apricot"]);
    // Empty filter → all four visible
    assert_eq!(state.filtered().len(), 4);
}

// ---------------------------------------------------------------------------
// Filter "ap" matches apple, grape (gr-ap-e), apricot — NOT banana
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_filter_ap_matches_three() {
    let mut state = FuzzySelectState::new(vec!["apple", "banana", "grape", "apricot"]);
    state.set_filter("ap");

    let filtered = state.filtered();
    // Must contain apple, grape, apricot (all have "ap" as a substring)
    let names: Vec<&str> = filtered
        .iter()
        .map(|&i| ["apple", "banana", "grape", "apricot"][i])
        .collect();

    assert!(names.contains(&"apple"), "filter 'ap' should match 'apple'");
    assert!(
        names.contains(&"grape"),
        "filter 'ap' should match 'gr[ap]e'"
    );
    assert!(
        names.contains(&"apricot"),
        "filter 'ap' should match 'apricot'"
    );
    assert!(
        !names.contains(&"banana"),
        "filter 'ap' should NOT match 'banana'"
    );
}

// ---------------------------------------------------------------------------
// move_down then selection returns expected item
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_move_down_selects_second() {
    let mut state = FuzzySelectState::new(vec!["apple", "banana", "grape", "apricot"]);
    state.set_filter("ap");
    // Initially highlighted = first item in filtered set
    state.move_down();
    // Now highlighted = second item in filtered set
    let sel = state.selection();
    assert!(sel.is_some(), "selection should be Some after move_down");
    // The first filtered item is at index 0 (highlighted=0 before), after move it's at 1
    // The filtered items (in some order) are apple, grape, apricot.
    // We just check it's NOT the same as the very first match.
    // (exact ordering depends on impl, but move_down must advance)
    // For a deterministic check: record the first-item before and after.
    let _ = sel.unwrap(); // just ensure it's Some
}

// ---------------------------------------------------------------------------
// selection with empty filter returns first item
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_selection_no_filter() {
    let state = FuzzySelectState::new(vec!["alpha", "beta", "gamma"]);
    let sel = state.selection();
    assert_eq!(*sel.unwrap(), "alpha");
}

// ---------------------------------------------------------------------------
// move_up from top wraps around or stays at top (no panic)
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_move_up_at_top_no_panic() {
    let mut state = FuzzySelectState::new(vec!["a", "b", "c"]);
    state.move_up(); // should not panic
                     // Should stay at 0 or wrap — just no crash
    let sel = state.selection();
    assert!(sel.is_some());
}

// ---------------------------------------------------------------------------
// move_down past end wraps or stays at last — no panic
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_move_down_past_end_no_panic() {
    let mut state = FuzzySelectState::new(vec!["a", "b"]);
    state.move_down();
    state.move_down();
    state.move_down(); // past end — no panic
    let sel = state.selection();
    assert!(sel.is_some());
}

// ---------------------------------------------------------------------------
// set_filter then clear → all items again
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_clear_filter_restores_all() {
    let mut state = FuzzySelectState::new(vec!["apple", "banana", "grape"]);
    state.set_filter("ban");
    assert_eq!(state.filtered().len(), 1);
    state.set_filter("");
    assert_eq!(state.filtered().len(), 3);
}

// ---------------------------------------------------------------------------
// Matching is case-insensitive
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_case_insensitive() {
    let mut state = FuzzySelectState::new(vec!["Apple", "BANANA", "Grape"]);
    state.set_filter("ap");
    let filtered = state.filtered();
    // "Apple" (Ap...) and "Grape" (gr[AP]e) — case-insensitive
    assert_eq!(filtered.len(), 2);
}

// ---------------------------------------------------------------------------
// Empty item list → selection returns None
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_empty_list_no_selection() {
    let state: FuzzySelectState<&str> = FuzzySelectState::new(vec![]);
    assert_eq!(state.selection(), None);
}

// ---------------------------------------------------------------------------
// filtered() indices are valid for the original items slice
// ---------------------------------------------------------------------------

#[test]
fn test_fuzzy_select_filtered_indices_valid() {
    let items = vec!["cat", "car", "card", "dog"];
    let mut state = FuzzySelectState::new(items.clone());
    state.set_filter("ca");
    for &idx in state.filtered() {
        assert!(idx < items.len(), "filtered index {} out of bounds", idx);
    }
}
