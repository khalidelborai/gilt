//! Fuzzy/substring select — opt-in interactive single-select with filter.
//!
//! # Core (always available, dep-free)
//!
//! [`FuzzySelectState<T>`] is a pure-logic state machine: no terminal I/O, no
//! feature gates.  It holds a list of items, a current filter string, the
//! filtered+ranked indices, and the highlighted index.  All the unit tests run
//! against this type.
//!
//! # Interactive driver (opt-in via `tty-select` feature)
//!
//! When the `tty-select` feature is enabled, [`FuzzySelect`] wraps the state
//! and adds crossterm raw-mode I/O.
//!
//! # Example (core, dep-free)
//!
//! ```
//! use gilt::fuzzy_select::FuzzySelectState;
//!
//! let mut state = FuzzySelectState::new(vec!["apple", "banana", "grape"]);
//! state.set_filter("ap");
//! assert!(state.filtered().len() >= 2); // apple, grape
//! state.move_down();
//! let sel = state.selection();
//! assert!(sel.is_some());
//! ```

// ---------------------------------------------------------------------------
// FuzzySelectState — dep-free core (NOT feature-gated)
// ---------------------------------------------------------------------------

/// Pure-logic state for a fuzzy/substring filter + arrow-key selection UI.
///
/// `T` is the item type; items must implement `AsRef<str>` for matching.
/// This type is NOT feature-gated — it is always compiled and unit-testable.
pub struct FuzzySelectState<T: AsRef<str>> {
    items: Vec<T>,
    filter: String,
    /// Indices into `items` that match the current filter, in match-quality order.
    filtered: Vec<usize>,
    /// Highlighted index within `filtered` (not into `items`).
    highlight: usize,
}

impl<T: AsRef<str>> FuzzySelectState<T> {
    /// Create a new `FuzzySelectState` from a list of items.
    ///
    /// The initial filter is empty (all items shown), highlight at 0.
    pub fn new(items: Vec<T>) -> Self {
        let n = items.len();
        let mut state = FuzzySelectState {
            items,
            filter: String::new(),
            filtered: (0..n).collect(),
            highlight: 0,
        };
        // Sort/rank with the empty filter to ensure deterministic order
        state.recompute_filtered();
        state
    }

    /// Update the filter string and re-rank the filtered list.
    ///
    /// The matching algorithm is case-insensitive substring containment.
    /// Items are ranked by match position (earlier = higher rank), then by
    /// original index for a stable, deterministic ordering.
    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_string();
        self.recompute_filtered();
        // Reset highlight whenever the filter changes so it's always in-bounds.
        self.highlight = 0;
    }

    /// Move the highlight one step toward the end of the list (down).
    ///
    /// Clamped at the last item — does not wrap.
    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() {
            let last = self.filtered.len() - 1;
            if self.highlight < last {
                self.highlight += 1;
            }
        }
    }

    /// Move the highlight one step toward the beginning of the list (up).
    ///
    /// Clamped at 0 — does not wrap.
    pub fn move_up(&mut self) {
        if self.highlight > 0 {
            self.highlight -= 1;
        }
    }

    /// Return the currently highlighted item, or `None` when the list is empty.
    pub fn selection(&self) -> Option<&T> {
        let idx = self.filtered.get(self.highlight)?;
        self.items.get(*idx)
    }

    /// Return the filtered+ranked indices into the original `items` slice.
    ///
    /// An empty filter returns all items in their original order.
    pub fn filtered(&self) -> &[usize] {
        &self.filtered
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    fn recompute_filtered(&mut self) {
        let filter_lower = self.filter.to_lowercase();

        if filter_lower.is_empty() {
            // No filter — all items, original order.
            self.filtered = (0..self.items.len()).collect();
            return;
        }

        // Case-insensitive substring match, ranked by match position then index.
        let mut matches: Vec<(usize, usize)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let haystack = item.as_ref().to_lowercase();
                haystack.find(&filter_lower).map(|pos| (pos, i))
            })
            .collect();

        // Sort by (match_position, original_index) for stability.
        matches.sort_unstable_by_key(|&(pos, idx)| (pos, idx));

        self.filtered = matches.into_iter().map(|(_, idx)| idx).collect();
    }
}

// ---------------------------------------------------------------------------
// FuzzySelect — interactive driver (opt-in: tty-select feature)
// ---------------------------------------------------------------------------

/// Interactive single-select with a fuzzy filter and arrow-key navigation.
///
/// Requires the `tty-select` feature.  The pure-logic core is in
/// [`FuzzySelectState`] and is always available.
///
/// ```text
/// // Example usage (requires tty-select feature):
/// // use gilt::fuzzy_select::FuzzySelect;
/// // let item = FuzzySelect::new(vec!["apple", "banana"]).ask()?;
/// ```
#[cfg(feature = "tty-select")]
pub struct FuzzySelect<T: AsRef<str> + Clone> {
    items: Vec<T>,
    prompt: String,
}

#[cfg(feature = "tty-select")]
impl<T: AsRef<str> + Clone> FuzzySelect<T> {
    /// Create a new `FuzzySelect` with the given items.
    pub fn new(items: Vec<T>) -> Self {
        FuzzySelect {
            items,
            prompt: "Select:".to_string(),
        }
    }

    /// Set the prompt text.
    #[must_use]
    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.prompt = prompt.to_string();
        self
    }

    /// Enter raw mode, show the interactive UI, and return the selected item.
    ///
    /// Returns `Ok(Some(item))` on Enter, `Ok(None)` on Esc/empty list.
    /// Restores the terminal via a RAII guard even on panic or `?`.
    pub fn ask(self) -> std::io::Result<Option<T>> {
        use crossterm::{
            cursor,
            event::{self, Event, KeyCode},
            execute,
            terminal::{self, ClearType},
        };
        use std::io::{stdout, Write};

        // RAII guard: disables raw mode when dropped (even on panic).
        struct RawModeGuard;
        impl Drop for RawModeGuard {
            fn drop(&mut self) {
                let _ = terminal::disable_raw_mode();
            }
        }

        if self.items.is_empty() {
            return Ok(None);
        }

        terminal::enable_raw_mode()?;
        let _guard = RawModeGuard;

        let mut state = FuzzySelectState::new(self.items.clone());
        let mut filter = String::new();

        loop {
            // Redraw
            execute!(
                stdout(),
                terminal::Clear(ClearType::CurrentLine),
                cursor::MoveToColumn(0)
            )?;
            print!("{} [{}] ", self.prompt, filter);
            stdout().flush()?;

            for (display_idx, &item_idx) in state.filtered().iter().enumerate() {
                let item = self.items[item_idx].as_ref();
                if display_idx == state.highlight {
                    print!("\r\n> {}", item);
                } else {
                    print!("\r\n  {}", item);
                }
            }
            // Move cursor back up to prompt line
            let rows = state.filtered().len();
            if rows > 0 {
                execute!(stdout(), cursor::MoveUp(rows as u16))?;
            }
            stdout().flush()?;

            // Read next key event
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Enter => {
                        // Clear the candidate lines before returning
                        for _ in 0..=state.filtered().len() {
                            execute!(
                                stdout(),
                                terminal::Clear(ClearType::CurrentLine),
                                cursor::MoveDown(1)
                            )?;
                        }
                        execute!(stdout(), cursor::MoveUp(state.filtered().len() as u16 + 1))?;
                        return Ok(state.selection().cloned());
                    }
                    KeyCode::Esc => {
                        return Ok(None);
                    }
                    KeyCode::Up => state.move_up(),
                    KeyCode::Down => state.move_down(),
                    KeyCode::Backspace => {
                        filter.pop();
                        state.set_filter(&filter);
                    }
                    KeyCode::Char(c) => {
                        filter.push(c);
                        state.set_filter(&filter);
                    }
                    _ => {}
                }
            }
        }
    }
}
