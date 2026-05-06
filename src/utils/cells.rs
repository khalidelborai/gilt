//! Cell width calculation for terminal display.
//!
//! This module provides utilities for calculating the visual width of text in terminal cells,
//! handling single-width (ASCII, box drawing) and double-width (CJK, emoji) characters.

use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Iterate `text` as extended grapheme clusters (Unicode UAX #29).
///
/// Use this when slicing or wrapping needs to keep ZWJ sequences
/// (`👨‍👩‍👧`), flag emoji (`🇺🇸`), and combining-mark sequences
/// (`café`) intact. For pure-ASCII input, prefer `text.chars()` —
/// the segmentation pass adds overhead without changing the result.
#[allow(dead_code)] // wired up by PRs 2-3; landed here in PR 1 as plumbing.
pub(crate) fn graphemes(text: &str) -> impl Iterator<Item = &str> {
    UnicodeSegmentation::graphemes(text, true)
}

/// Get the cell width of a string (how many terminal columns it occupies).
///
/// # Examples
///
/// ```
/// use gilt::cells::cell_len;
///
/// assert_eq!(cell_len("abc"), 3);
/// assert_eq!(cell_len("💩"), 2);
/// assert_eq!(cell_len("わさび"), 6);  // 3 CJK chars × 2
/// ```
pub fn cell_len(text: &str) -> usize {
    // Fast path for pure ASCII (the common case in terminal output) — every
    // ASCII byte is a single cell, so byte length equals cell length and we
    // skip the Unicode width-table lookup entirely.
    if text.is_ascii() {
        return text.len();
    }
    text.width()
}

/// Get the cell width of a single character (0, 1, or 2).
///
/// Returns:
/// - 0 for control characters and zero-width characters
/// - 1 for single-width characters (ASCII, box drawing, etc.)
/// - 2 for double-width characters (CJK, emoji, etc.)
///
/// # Examples
///
/// ```
/// use gilt::cells::get_character_cell_size;
///
/// assert_eq!(get_character_cell_size('\0'), 0);
/// assert_eq!(get_character_cell_size('a'), 1);
/// assert_eq!(get_character_cell_size('💩'), 2);
/// ```
pub fn get_character_cell_size(c: char) -> usize {
    // Fast path: printable ASCII = 1 cell; control chars (< 0x20) = 0.
    if (c as u32) < 0x80 {
        return if (c as u32) >= 0x20 { 1 } else { 0 };
    }
    c.width().unwrap_or(0)
}

/// Crop or pad a string to fit in exactly `total` cells.
///
/// If the string is too long, it will be cropped. If a crop would split a double-width
/// character, it will be replaced with a space. If the string is too short, it will be
/// padded with spaces.
///
/// # Examples
///
/// ```
/// use gilt::cells::set_cell_size;
///
/// assert_eq!(set_cell_size("foo", 0), "");
/// assert_eq!(set_cell_size("foo", 2), "fo");
/// assert_eq!(set_cell_size("foo", 3), "foo");
/// assert_eq!(set_cell_size("foo", 4), "foo ");
/// assert_eq!(set_cell_size("😽😽", 4), "😽😽");
/// assert_eq!(set_cell_size("😽😽", 3), "😽 ");  // crop in middle of emoji → space
/// ```
pub fn set_cell_size(text: &str, total: usize) -> Cow<'_, str> {
    let current_len = cell_len(text);

    if current_len == total {
        return Cow::Borrowed(text);
    }

    if current_len < total {
        // Pad with spaces
        let mut result = String::with_capacity(text.len() + (total - current_len));
        result.push_str(text);
        result.push_str(&" ".repeat(total - current_len));
        return Cow::Owned(result);
    }

    if total == 0 {
        return Cow::Borrowed("");
    }

    // Need to crop
    let mut result = String::with_capacity(text.len());
    let mut cell_position = 0;

    for c in text.chars() {
        let char_width = get_character_cell_size(c);

        if cell_position + char_width <= total {
            result.push(c);
            cell_position += char_width;
        } else if cell_position < total {
            // We have space left but the character doesn't fit
            // Replace with space(s) to fill remaining cells
            result.push_str(&" ".repeat(total - cell_position));
            break;
        } else {
            // Already at target width
            break;
        }
    }

    Cow::Owned(result)
}

/// Split text into lines where each line fits within `width` cells.
///
/// If a double-width character would overflow the width, it starts a new line.
///
/// # Examples
///
/// ```
/// use gilt::cells::chop_cells;
///
/// assert_eq!(chop_cells("abcdefghijk", 3), vec!["abc", "def", "ghi", "jk"]);
/// assert_eq!(chop_cells("ありがとう", 3), vec!["あ", "り", "が", "と", "う"]);
/// ```
pub fn chop_cells(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for c in text.chars() {
        let char_width = get_character_cell_size(c);

        if current_width + char_width <= width {
            current_line.push(c);
            current_width += char_width;
        } else {
            // Start a new line
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
            current_line.push(c);
            current_width = char_width;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Fast check: are all characters single-cell width?
///
/// Returns `true` if all characters in the string occupy exactly 1 cell,
/// `false` if any character is double-width or zero-width.
///
/// # Examples
///
/// ```
/// use gilt::cells::is_single_cell_widths;
///
/// assert!(is_single_cell_widths("hello world"));
/// assert!(is_single_cell_widths("┌─┬┐│ ││"));  // box drawing = single width
/// assert!(!is_single_cell_widths("💩"));
/// assert!(!is_single_cell_widths("わさび"));
/// ```
pub fn is_single_cell_widths(text: &str) -> bool {
    text.chars().all(|c| get_character_cell_size(c) == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_character_cell_size() {
        // Control characters - unicode-width returns Some(0) for C0 and C1 control codes
        // Note: \x00 (NUL) returns None which becomes 0
        assert_eq!(get_character_cell_size('\0'), 0);

        // Most C0 control characters (\x01-\x1f) return Some(0)
        // But some may return None (becomes 0)
        let x01_width = get_character_cell_size('\x01');
        let x1f_width = get_character_cell_size('\x1f');
        // These should be 0, but if unicode-width changes, we accept 0 or 1
        assert!(
            x01_width <= 1,
            "\\x01 width should be 0 or 1, got {}",
            x01_width
        );
        assert!(
            x1f_width <= 1,
            "\\x1f width should be 0 or 1, got {}",
            x1f_width
        );

        // Single-width: ASCII
        assert_eq!(get_character_cell_size('a'), 1);
        assert_eq!(get_character_cell_size('A'), 1);
        assert_eq!(get_character_cell_size('0'), 1);
        assert_eq!(get_character_cell_size(' '), 1);

        // Double-width: emoji
        assert_eq!(get_character_cell_size('💩'), 2);
        assert_eq!(get_character_cell_size('😽'), 2);

        // Double-width: CJK
        assert_eq!(get_character_cell_size('あ'), 2);
        assert_eq!(get_character_cell_size('わ'), 2);
        assert_eq!(get_character_cell_size('さ'), 2);
        assert_eq!(get_character_cell_size('び'), 2);
    }

    #[test]
    fn test_cell_len() {
        // Empty string
        assert_eq!(cell_len(""), 0);

        // ASCII
        assert_eq!(cell_len("abc"), 3);
        assert_eq!(cell_len("hello world"), 11);

        // Emoji
        assert_eq!(cell_len("💩"), 2);
        assert_eq!(cell_len("😽😽"), 4);

        // CJK
        assert_eq!(cell_len("わさび"), 6); // 3 CJK chars × 2
        assert_eq!(cell_len("あ"), 2);
        assert_eq!(cell_len("ありがとう"), 10); // 5 CJK chars × 2

        // Mixed ASCII + CJK
        assert_eq!(cell_len("aあb"), 4); // 1+2+1

        // Control characters
        // Note: unicode-width may treat some control characters as having width 1
        let x01_len = cell_len("\x01");
        assert!(x01_len <= 1, "Expected \\x01 width 0 or 1, got {}", x01_len);

        let x1f_len = cell_len("\x1f");
        assert!(x1f_len <= 1, "Expected \\x1f width 0 or 1, got {}", x1f_len);

        // Control char in middle - may have width
        let a_x01_b_len = cell_len("a\x01b");
        assert!(
            (2..=3).contains(&a_x01_b_len),
            "Expected a\\x01b width 2-3, got {}",
            a_x01_b_len
        );

        // Box drawing characters (single-width)
        assert_eq!(cell_len("┌─┬┐"), 4);
        assert_eq!(cell_len("│ ││"), 4);
    }

    #[test]
    fn test_set_cell_size_exact_match() {
        assert_eq!(set_cell_size("foo", 3), "foo");
        assert_eq!(set_cell_size("😽😽", 4), "😽😽");
    }

    #[test]
    fn test_set_cell_size_padding() {
        assert_eq!(set_cell_size("foo", 4), "foo ");
        assert_eq!(set_cell_size("foo", 5), "foo  ");
        assert_eq!(set_cell_size("😽😽", 5), "😽😽 ");
        assert_eq!(set_cell_size("a", 10), "a         ");
    }

    #[test]
    fn test_set_cell_size_cropping() {
        assert_eq!(set_cell_size("foo", 0), "");
        assert_eq!(set_cell_size("foo", 1), "f");
        assert_eq!(set_cell_size("foo", 2), "fo");
        assert_eq!(set_cell_size("abcdefgh", 5), "abcde");
    }

    #[test]
    fn test_set_cell_size_crop_double_width() {
        // Exact fit for double-width
        assert_eq!(set_cell_size("😽😽", 4), "😽😽");
        assert_eq!(set_cell_size("😽😽", 2), "😽");

        // Crop in middle of emoji → space
        assert_eq!(set_cell_size("😽😽", 3), "😽 ");
        assert_eq!(set_cell_size("😽😽", 1), " "); // emoji is 2-wide, can't fit → space

        // CJK cropping
        // "あり" = 2+2 = 4 cells, "ありが" = 2+2+2 = 6 cells
        let result = set_cell_size("ありがとう", 6);
        assert_eq!(
            result,
            "ありが",
            "Expected 'ありが' (6 cells), got '{}' ({} cells)",
            result,
            cell_len(&result)
        );

        assert_eq!(set_cell_size("ありがとう", 5), "あり "); // can't fit 3rd char, add space
        assert_eq!(set_cell_size("ありがとう", 4), "あり");
        assert_eq!(set_cell_size("ありがとう", 3), "あ ");
    }

    #[test]
    fn test_set_cell_size_mixed_width() {
        // Mixed ASCII + emoji
        assert_eq!(set_cell_size("a😽b", 4), "a😽b");
        assert_eq!(set_cell_size("a😽b", 3), "a😽");
        assert_eq!(set_cell_size("a😽b", 2), "a "); // 'a' fits (1), emoji doesn't (2), pad with space

        // Mixed ASCII + CJK
        assert_eq!(set_cell_size("aあb", 4), "aあb");
        assert_eq!(set_cell_size("aあb", 3), "aあ");
        assert_eq!(set_cell_size("aあb", 2), "a ");
    }

    #[test]
    fn test_chop_cells_single_width() {
        assert_eq!(
            chop_cells("abcdefghijk", 3),
            vec!["abc", "def", "ghi", "jk"]
        );
        assert_eq!(chop_cells("hello", 3), vec!["hel", "lo"]);
        assert_eq!(chop_cells("abc", 3), vec!["abc"]);
        assert_eq!(chop_cells("abc", 10), vec!["abc"]);
    }

    #[test]
    fn test_chop_cells_double_width() {
        // Each CJK char is 2-wide, so with width=3, only one char fits per line
        // (would need width=4 to fit 2 chars)
        assert_eq!(
            chop_cells("ありがとう", 3),
            vec!["あ", "り", "が", "と", "う"]
        );
        assert_eq!(chop_cells("ありがとう", 4), vec!["あり", "がと", "う"]);
        assert_eq!(chop_cells("ありがとう", 6), vec!["ありが", "とう"]);

        // Emoji
        assert_eq!(chop_cells("😽😽😽", 4), vec!["😽😽", "😽"]);
        assert_eq!(chop_cells("😽😽😽", 5), vec!["😽😽", "😽"]); // can't fit 3rd emoji
    }

    #[test]
    fn test_chop_cells_mixed_width() {
        // Mixed single and double width: "あ1り234が5と6う78"
        // あ=2, 1=1, り=2, 2=1, 3=1, 4=1, が=2, 5=1, と=2, 6=1, う=2, 7=1, 8=1
        let text = "あ1り234が5と6う78";
        let result = chop_cells(text, 3);
        // あ=2, 1=1 => 3 cells: "あ1"
        // り=2, 2=1 => 3 cells: "り2"
        // 3=1, 4=1, が=2 => can't fit が, so "34", then "が5"
        // と=2, 6=1 => 3 cells: "と6"
        // う=2, 7=1 => 3 cells: "う7"
        // 8=1 => "8"
        assert_eq!(result, vec!["あ1", "り2", "34", "が5", "と6", "う7", "8"]);
    }

    #[test]
    fn test_chop_cells_empty() {
        assert_eq!(chop_cells("", 3), Vec::<String>::new());
        assert_eq!(chop_cells("abc", 0), Vec::<String>::new());
    }

    #[test]
    fn test_is_single_cell_widths() {
        // ASCII text
        assert!(is_single_cell_widths("hello world"));
        assert!(is_single_cell_widths("abc123"));
        assert!(is_single_cell_widths("The quick brown fox"));

        // Box drawing characters (single width)
        assert!(is_single_cell_widths("┌─┬┐│ ││"));
        assert!(is_single_cell_widths("├─┼─┤"));

        // Empty string
        assert!(is_single_cell_widths(""));

        // Emoji (double width)
        assert!(!is_single_cell_widths("💩"));
        assert!(!is_single_cell_widths("😽"));
        assert!(!is_single_cell_widths("hello 💩"));

        // CJK (double width)
        assert!(!is_single_cell_widths("わさび"));
        assert!(!is_single_cell_widths("ありがとう"));
        assert!(!is_single_cell_widths("hello あ"));

        // Control characters (zero width)
        assert!(!is_single_cell_widths("\x01"));
        assert!(!is_single_cell_widths("a\x01b"));
    }

    #[test]
    fn test_long_strings() {
        // Long ASCII string (512+ chars)
        let long_ascii = "a".repeat(600);
        assert_eq!(cell_len(&long_ascii), 600);
        assert_eq!(set_cell_size(&long_ascii, 500).len(), 500);
        assert!(is_single_cell_widths(&long_ascii));

        // Long CJK string
        let long_cjk = "あ".repeat(300);
        assert_eq!(cell_len(&long_cjk), 600); // 300 chars × 2
        assert!(!is_single_cell_widths(&long_cjk));
    }

    #[test]
    fn test_edge_cases() {
        // Single character
        assert_eq!(cell_len("a"), 1);
        assert_eq!(set_cell_size("a", 1), "a");
        assert_eq!(chop_cells("a", 1), vec!["a"]);

        // NUL followed by printable
        // Note: unicode-width may count \x00 as width 0 or 1 depending on version
        let nul_a_len = cell_len("\x00a");
        assert!(
            (1..=2).contains(&nul_a_len),
            "Expected \\x00a width 1-2, got {}",
            nul_a_len
        );

        // Multiple spaces
        assert_eq!(cell_len("   "), 3);
        assert_eq!(set_cell_size("   ", 5), "     ");

        // Newlines and tabs
        // Note: unicode-width may treat these differently than other control chars
        let tab_width = get_character_cell_size('\t');
        let newline_width = get_character_cell_size('\n');
        // Tab is often treated as width 2-4, newline as 0-1
        // Just verify they return reasonable values
        assert!(
            tab_width <= 4,
            "Tab width should be <= 4, got {}",
            tab_width
        );
        assert!(
            newline_width <= 1,
            "Newline width should be <= 1, got {}",
            newline_width
        );
    }
}

#[cfg(test)]
mod tests_v1_4_width_fixes {
    use super::cell_len;

    /// Codepoint-as-width sites (accordion icons, log time alignment,
    /// bar prefix/body lengths, gradient justification padding) all
    /// route through `cell_len`. These assertions document the
    /// correct visible-width values for the inputs that previously
    /// returned codepoint counts.

    #[test]
    fn family_zwj_emoji_is_2_cells_not_5_codepoints() {
        // 👨‍👩‍👧 = U+1F468 ZWJ U+1F469 ZWJ U+1F467 (5 codepoints) → 2 cells
        let s = "👨\u{200d}👩\u{200d}👧";
        assert_eq!(s.chars().count(), 5);
        assert_eq!(cell_len(s), 2);
    }

    #[test]
    fn flag_emoji_is_2_cells_not_2_codepoints_misread_as_1_each() {
        // 🇺🇸 = U+1F1FA U+1F1F8 (2 regional indicators) → 2 cells
        let s = "\u{1F1FA}\u{1F1F8}";
        assert_eq!(cell_len(s), 2);
    }

    #[test]
    fn combining_acute_zero_width() {
        // "café" with combining acute = 5 codepoints, 4 cells
        let s = "cafe\u{0301}";
        assert_eq!(s.chars().count(), 5);
        assert_eq!(cell_len(s), 4);
    }

    #[test]
    fn ascii_fast_path_unchanged() {
        assert_eq!(cell_len("hello"), 5);
        assert_eq!(cell_len(""), 0);
    }
}
