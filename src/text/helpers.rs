//! Helper functions for text manipulation.

use std::borrow::Cow;

/// Strip control codes from text (Bell, Backspace, VT, FF, CR).
pub fn strip_control_codes(text: &str) -> Cow<'_, str> {
    if !text
        .chars()
        .any(|c| matches!(c as u32, 7 | 8 | 11 | 12 | 13))
    {
        return Cow::Borrowed(text);
    }
    Cow::Owned(
        text.chars()
            .filter(|c| !matches!(*c as u32, 7 | 8 | 11 | 12 | 13))
            .collect(),
    )
}

/// Convert a char index to a byte index within a string.
pub fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Get a substring by char indices `[start..end)`.
pub fn char_slice(s: &str, start: usize, end: usize) -> &str {
    let byte_start = char_to_byte_index(s, start);
    let byte_end = char_to_byte_index(s, end);
    &s[byte_start..byte_end]
}

/// Compute GCD of two numbers (iterative).
pub fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}
