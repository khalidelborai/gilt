//! Rich text module - the core text manipulation type.
//!
//! This module provides the `Text` type which represents styled terminal text,
//! along with supporting types `Span`, `Lines`, and related enums.
//! Port of Python's rich/text.py.

// Re-export enums
pub use enums::{JustifyMethod, OverflowMethod};

// Re-export helper functions
pub use helpers::strip_control_codes;

// Re-export core types
pub use core::{Text, TextOrStr, TextPart};
pub use lines::Lines;
pub use span::Span;

mod core;
pub mod enums;
mod helpers;
mod lines;
mod span;

// ---------------------------------------------------------------------------
// Tests (kept here temporarily - will be moved to tests/ in Phase 3)
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    // Tests are in tests/unit/text_tests.rs for organization
}
