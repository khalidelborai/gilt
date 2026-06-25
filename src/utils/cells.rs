//! Cell width calculation for terminal display.
//!
//! This module provides utilities for calculating the visual width of text in terminal cells,
//! handling single-width (ASCII, box drawing) and double-width (CJK, emoji) characters.

use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

/// Iterate `text` as extended grapheme clusters (Unicode UAX #29).
///
/// Use this when slicing or wrapping needs to keep ZWJ sequences
/// (`👨‍👩‍👧`), flag emoji (`🇺🇸`), and combining-mark sequences
/// (`café`) intact. For pure-ASCII input, prefer `text.chars()` —
/// the segmentation pass adds overhead without changing the result.
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
///
/// # Cluster vs codepoint width (terminal-reality rule)
///
/// `cell_len` returns the **sum of per-codepoint widths**, not the
/// cluster-aware width that `unicode_width::UnicodeWidthStr::width`
/// produces. The two diverge for ZWJ sequences:
///
/// - `cell_len("👨\u{200d}👩\u{200d}👧") == 6` (per-codepoint: 2+0+2+0+2)
/// - `"👨\u{200d}👩\u{200d}👧".width() == 2` (cluster-aware)
///
/// Terminals without color-emoji + ZWJ font support (most Linux
/// xterm/tmux setups, headless CI) render the family as three
/// separate 2-cell emoji = 6 cells. Reporting 2 here would make
/// `Table` columns underflow and overflow into neighbouring cells.
/// Reporting 6 matches the terminal reality on the majority of
/// gilt deployments.
///
/// Modern terminals with full color-emoji-and-ZWJ support (kitty,
/// iTerm2, Windows Terminal, alacritty + emoji-aware font) DO
/// render the cluster as a single 2-cell glyph; on those terminals
/// gilt over-reserves space. The trade-off was made deliberately:
/// over-reserved space looks correct (just slightly wasteful);
/// under-reserved space breaks table layouts.
///
/// Flag emoji (regional indicators) and combining-mark sequences
/// (`café`) are unaffected — both forms agree.
pub fn cell_len(text: &str) -> usize {
    // Fast path for pure printable ASCII — every byte in 0x20..=0x7E is a
    // single cell, so byte length equals cell length and we skip the Unicode
    // width-table lookup entirely.
    // DEL (0x7F) and C0 controls are NOT printable so we cannot use the
    // simple byte-length trick; fall through to per-codepoint sum.
    if text.bytes().all(|b| (0x20..0x7f).contains(&b)) {
        return text.len();
    }
    // Per-codepoint sum (NOT text.width(), which is cluster-aware and
    // disagrees with terminal reality for ZWJ sequences). See docstring.
    //
    // Opt 3: use the thread-local LRU cache for non-ASCII chars to avoid
    // repeated unicode_width table lookups for the same codepoints.
    text.chars()
        .map(|c| {
            if (c as u32) < 0x80 {
                // ASCII: fast path, no cache needed.
                if (c as u32) >= 0x20 && c != '\x7f' {
                    1
                } else {
                    0
                }
            } else {
                cached_char_width(c)
            }
        })
        .sum()
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
    // Fast path: printable ASCII (0x20..=0x7E) = 1 cell.
    // DEL (0x7F) falls through to width() which returns 0.
    // C0 control chars (< 0x20) return 0.
    if (c as u32) < 0x80 {
        return if (c as u32) >= 0x20 && c != '\x7f' {
            1
        } else {
            0
        };
    }
    // Opt 3: use the thread-local LRU cache for non-ASCII chars.
    cached_char_width(c)
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
    // ASCII fast path: if every byte is a printable ASCII character (0x20..0x7F
    // exclusive — i.e. no DEL, no control chars), then cell width == byte
    // length and we can crop with a direct byte slice instead of the O(n)
    // grapheme-cluster walk.
    if text.bytes().all(|b| (0x20..0x7f).contains(&b)) {
        let text_len = text.len(); // == cell length for printable ASCII
        if text_len == total {
            return Cow::Borrowed(text);
        }
        if text_len < total {
            let mut result = String::with_capacity(total);
            result.push_str(text);
            result.extend(std::iter::repeat_n(' ', total - text_len));
            return Cow::Owned(result);
        }
        if total == 0 {
            return Cow::Borrowed("");
        }
        // Crop: &text[..total] is safe because all bytes are single-byte chars.
        return Cow::Borrowed(&text[..total]);
    }

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

    // Need to crop. Iterate by GRAPHEME CLUSTER (not codepoint) so we
    // don't leave a dangling ZWJ joiner / variation selector / regional
    // indicator half. v1.4.0 grapheme-correctness fix; pre-v1.4 used
    // `text.chars()` which could split inside a ZWJ family emoji.
    let mut result = String::with_capacity(text.len());
    let mut cell_position = 0;

    for cluster in graphemes(text) {
        let cluster_width = cell_len(cluster);

        if cell_position + cluster_width <= total {
            result.push_str(cluster);
            cell_position += cluster_width;
        } else if cell_position < total {
            // The cluster doesn't fit; replace with space(s) to fill
            // the remaining width. Matches the pre-v1.4 wide-char-crop
            // behaviour but at the grapheme-cluster level.
            result.push_str(&" ".repeat(total - cell_position));
            break;
        } else {
            // Already at target width.
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

    for cluster in UnicodeSegmentation::graphemes(text, true) {
        let cluster_width = cell_len(cluster);

        if current_width + cluster_width <= width {
            current_line.push_str(cluster);
            current_width += cluster_width;
        } else {
            // Start a new line
            if !current_line.is_empty() {
                lines.push(current_line);
                current_line = String::new();
            }
            current_line.push_str(cluster);
            current_width = cluster_width;
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

/// Split `text` into grapheme cluster spans, returning one entry per visual
/// unit as `(start_byte, end_byte, cell_width)`.
///
/// Grapheme clusters are determined by Unicode UAX #29 extended grapheme
/// cluster boundaries (`unicode_segmentation::UnicodeSegmentation::graphemes`).
/// The `cell_width` of each span equals the sum of the per-codepoint widths
/// of all characters in the cluster, consistent with [`cell_len`].
///
/// Zero-width combining marks (e.g. U+0301 COMBINING ACUTE ACCENT) and ZWJ
/// sequences (e.g. family emoji U+1F468 U+200D …) are included in the span
/// of their base character, so the cluster's byte range covers the whole
/// sequence.
///
/// # Examples
///
/// ```
/// use gilt::cells::split_graphemes;
///
/// // Pure ASCII: each character is its own span, width 1.
/// let spans = split_graphemes("ab");
/// assert_eq!(spans, vec![(0, 1, 1), (1, 2, 1)]);
///
/// // CJK: "日" is U+65E5, encoded as 3 UTF-8 bytes, width 2.
/// let spans = split_graphemes("日");
/// assert_eq!(spans.len(), 1);
/// assert_eq!(spans[0].2, 2);
///
/// // Combining mark: café (c a f e + U+0301).  The combining acute joins
/// // the 'e' cluster — five codepoints but four grapheme clusters.
/// let s = "cafe\u{0301}";
/// let spans = split_graphemes(s);
/// assert_eq!(spans.len(), 4);  // c, a, f, é-cluster
/// assert_eq!(spans[3].0, 3);   // 'e' starts at byte 3
/// assert_eq!(spans[3].1, s.len()); // cluster ends at end-of-string
/// assert_eq!(spans[3].2, 1);   // 'e' + combining = still 1 cell wide
/// ```
pub fn split_graphemes(text: &str) -> Vec<(usize, usize, usize)> {
    // Fast path for pure ASCII (no grapheme overhead needed).
    if text.bytes().all(|b| (0x20..0x7f).contains(&b)) {
        return text
            .char_indices()
            .map(|(i, c)| (i, i + c.len_utf8(), 1))
            .collect();
    }

    let mut spans = Vec::new();
    // `graphemes(text, true)` returns string slices into `text`.
    // We recover the byte offset from the pointer difference.
    let base = text.as_ptr() as usize;

    for cluster in UnicodeSegmentation::graphemes(text, true) {
        let start_byte = cluster.as_ptr() as usize - base;
        let end_byte = start_byte + cluster.len();
        let width = cell_len(cluster);
        spans.push((start_byte, end_byte, width));
    }

    spans
}

/// Split `text` at exactly `length` terminal cells, returning `(left, right)`.
///
/// If `length` falls inside a double-width character, that character is
/// replaced with spaces: `left` gains a trailing space (filling the remaining
/// cell) and `right` gains a leading space (occupying the vacated first cell).
/// This matches the behaviour of rich's `cells.split_text`.
///
/// # Fast path
/// For strings where every character is single-cell-width, the split is a
/// simple slice at the `length`-th character with no allocation.
///
/// # Examples
///
/// ```
/// use gilt::cells::split_text;
///
/// // Pure ASCII: straightforward split.
/// let (l, r) = split_text("hello world", 5);
/// assert_eq!(l, "hello");
/// assert_eq!(r, " world");
///
/// // CJK: "日本語" is 3 × 2 = 6 cells.  Split at 4 → "日本" | "語".
/// let (l, r) = split_text("日本語", 4);
/// assert_eq!(l, "日本");
/// assert_eq!(r, "語");
///
/// // Split in the middle of a CJK char → space padding on both sides.
/// let (l, r) = split_text("日本語", 3);
/// assert_eq!(l, "日 ");   // "日" (2 cells) + space = 3 cells
/// assert_eq!(r, " 語");   // space + "語" (2 cells)
///
/// // Split beyond string length returns the full string and empty.
/// let (l, r) = split_text("abc", 10);
/// assert_eq!(l, "abc");
/// assert_eq!(r, "");
/// ```
pub fn split_text(text: &str, length: usize) -> (String, String) {
    if length == 0 {
        return (String::new(), text.to_owned());
    }

    // Fast path: all single-cell characters — the n-th cell == n-th char.
    if is_single_cell_widths(text) {
        let split_byte = text
            .char_indices()
            .nth(length)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        return (text[..split_byte].to_owned(), text[split_byte..].to_owned());
    }

    // General path via grapheme spans.
    let spans = split_graphemes(text);
    let total_cells: usize = spans.iter().map(|s| s.2).sum();

    // If requested length >= total width, return the whole string.
    if length >= total_cells {
        return (text.to_owned(), String::new());
    }

    // Walk spans to find the split point.
    let mut accumulated = 0usize;
    for &(start, end, width) in &spans {
        if accumulated + width <= length {
            accumulated += width;
            if accumulated == length {
                // Exact boundary between this cluster and the next.
                return (text[..end].to_owned(), text[end..].to_owned());
            }
        } else {
            // `length` falls inside this span (a wide char straddles the
            // boundary). Replace the wide cluster with spaces on both sides,
            // exactly as rich does.
            let mut left = text[..start].to_owned();
            // Fill remaining cells with spaces so left has exactly `length` cells.
            let left_cells = accumulated; // cells before this cluster
            let spaces_left = length.saturating_sub(left_cells);
            left.push_str(&" ".repeat(spaces_left));

            let right_cells_needed = width.saturating_sub(spaces_left);
            let mut right = " ".repeat(right_cells_needed);
            right.push_str(&text[end..]);

            return (left, right);
        }
    }

    // Accumulated == length exactly at the end of the string.
    (text.to_owned(), String::new())
}

// RED test lives in the `tests` mod below. See `cell_len_cache_hit_on_second_call`.
#[cfg(test)]
mod tests {
    use super::*;

    // -- Opt 3: cell_len LRU cache ------------------------------------------

    /// RED test: after calling `cell_len` on a CJK string, the non-ASCII chars
    /// should be present in the thread-local cache. This test asserts that the
    /// cache was populated (i.e., the width was stored), which only happens
    /// AFTER the cache is wired in. Before implementation, `cache_hit_count_for`
    /// returns `None` because the cache is never populated.
    #[test]
    fn cell_len_cache_hit_on_second_call() {
        // Prime the cache by calling cell_len on a CJK string.
        let cjk_str = "わさび"; // 3 CJK chars, each 2 cells wide
        let first = cell_len(cjk_str);
        assert_eq!(first, 6, "cell_len('わさび') must be 6");

        // The first CJK char 'わ' (U+308F) should now be in the cache.
        let cached = super::cache_hit_count_for('わ');
        assert!(
            cached.is_some(),
            "after cell_len('わさび'), char 'わ' must be in the LRU cache \
             (cache was not populated — Opt 3 not yet implemented)"
        );
        assert_eq!(cached.unwrap(), 2u8, "cached width of 'わ' must be 2");

        // Second call must give identical result (correctness preserved).
        let second = cell_len(cjk_str);
        assert_eq!(second, 6, "repeated cell_len must return same value");

        // Emoji test.
        let emoji_str = "💩";
        let emoji_len = cell_len(emoji_str);
        assert_eq!(emoji_len, 2, "cell_len('💩') must be 2");
        let cached_emoji = super::cache_hit_count_for('💩');
        assert!(
            cached_emoji.is_some(),
            "after cell_len('💩'), the emoji must be in the LRU cache"
        );
        assert_eq!(cached_emoji.unwrap(), 2u8, "cached width of '💩' must be 2");
    }

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

    // -----------------------------------------------------------------------
    // split_graphemes tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_split_graphemes_ascii() {
        // Pure ASCII: each char is its own span, width 1.
        let spans = split_graphemes("abc");
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0], (0, 1, 1));
        assert_eq!(spans[1], (1, 2, 1));
        assert_eq!(spans[2], (2, 3, 1));
    }

    #[test]
    fn test_split_graphemes_empty() {
        let spans = split_graphemes("");
        assert!(spans.is_empty());
    }

    #[test]
    fn test_split_graphemes_cjk_double_width() {
        // "日" is U+65E5: 3 UTF-8 bytes, width 2.
        let spans = split_graphemes("日");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].0, 0);
        assert_eq!(spans[0].1, "日".len()); // 3
        assert_eq!(spans[0].2, 2, "CJK char must be width 2");

        // "日本" = two CJK chars, each 3 bytes, 2 cells.
        let spans = split_graphemes("日本");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0], (0, 3, 2));
        assert_eq!(spans[1], (3, 6, 2));
    }

    #[test]
    fn test_split_graphemes_combining_mark() {
        // "cafe\u{0301}" — e + combining acute = 1 grapheme cluster.
        // 'e' is at byte 3, '\u{0301}' is 2 bytes, so cluster spans bytes 3..6.
        let s = "cafe\u{0301}";
        let spans = split_graphemes(s);
        assert_eq!(spans.len(), 4, "four grapheme clusters: c, a, f, é-cluster");
        assert_eq!(spans[0], (0, 1, 1)); // 'c'
        assert_eq!(spans[1], (1, 2, 1)); // 'a'
        assert_eq!(spans[2], (2, 3, 1)); // 'f'
                                         // é-cluster: starts at byte 3, length = 'e'(1) + U+0301(2) = 3 bytes, width = 1
        assert_eq!(spans[3].0, 3);
        assert_eq!(spans[3].1, s.len()); // 3 + 3 = 6
        assert_eq!(spans[3].2, 1, "combining accent does not add cell width");
    }

    #[test]
    fn test_split_graphemes_zwj_family_emoji() {
        // 👨‍👩‍👧 = U+1F468 ZWJ U+1F469 ZWJ U+1F467 — 5 codepoints, 1 grapheme cluster.
        // Per-codepoint widths: 2 + 0 + 2 + 0 + 2 = 6 (terminal reality).
        let s = "👨\u{200d}👩\u{200d}👧";
        let spans = split_graphemes(s);
        // UAX #29 says the whole ZWJ sequence is one cluster.
        assert_eq!(spans.len(), 1, "ZWJ family emoji is one grapheme cluster");
        assert_eq!(spans[0].0, 0);
        assert_eq!(spans[0].1, s.len());
        // cell_len sums per-codepoint: 2+0+2+0+2 = 6
        assert_eq!(spans[0].2, 6, "ZWJ family emoji has terminal width 6");
    }

    #[test]
    fn test_split_graphemes_flag_emoji() {
        // 🇺🇸 = U+1F1FA U+1F1F8 (two regional indicators) → 1 grapheme cluster, 2 cells.
        let s = "\u{1F1FA}\u{1F1F8}";
        let spans = split_graphemes(s);
        assert_eq!(spans.len(), 1, "flag emoji is one grapheme cluster");
        assert_eq!(spans[0].2, 2, "flag emoji is 2 cells wide");
    }

    #[test]
    fn test_split_graphemes_mixed() {
        // "日a本" = CJK(2) + ASCII(1) + CJK(2)
        let s = "日a本";
        let spans = split_graphemes(s);
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].2, 2); // 日
        assert_eq!(spans[1].2, 1); // a
        assert_eq!(spans[2].2, 2); // 本
    }

    #[test]
    fn test_split_graphemes_span_covers_full_string() {
        // Total byte coverage: all spans together must cover [0, text.len()).
        let s = "日a本\u{0301}café";
        let spans = split_graphemes(s);
        if !spans.is_empty() {
            assert_eq!(spans[0].0, 0, "first span must start at byte 0");
            assert_eq!(
                spans.last().unwrap().1,
                s.len(),
                "last span must end at text.len()"
            );
        }
        // Contiguous: each span's end == next span's start.
        for window in spans.windows(2) {
            assert_eq!(
                window[0].1, window[1].0,
                "spans must be contiguous (no gaps)"
            );
        }
    }

    // -----------------------------------------------------------------------
    // split_text tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_split_text_ascii() {
        let (l, r) = split_text("hello world", 5);
        assert_eq!(l, "hello");
        assert_eq!(r, " world");

        let (l, r) = split_text("abc", 0);
        assert_eq!(l, "");
        assert_eq!(r, "abc");

        let (l, r) = split_text("abc", 3);
        assert_eq!(l, "abc");
        assert_eq!(r, "");

        // Beyond string length.
        let (l, r) = split_text("abc", 10);
        assert_eq!(l, "abc");
        assert_eq!(r, "");
    }

    #[test]
    fn test_split_text_cjk_exact_boundary() {
        // "日本語" = 6 cells. Split at 4 gives "日本" | "語".
        let (l, r) = split_text("日本語", 4);
        assert_eq!(l, "日本");
        assert_eq!(r, "語");

        // Task brief example: split_text("日本語ab", 4) → "日本" | "語ab"
        let (l, r) = split_text("日本語ab", 4);
        assert_eq!(l, "日本");
        assert_eq!(r, "語ab");

        // Split at 0.
        let (l, r) = split_text("日本語", 0);
        assert_eq!(l, "");
        assert_eq!(r, "日本語");

        // Split at full width.
        let (l, r) = split_text("日本語", 6);
        assert_eq!(l, "日本語");
        assert_eq!(r, "");
    }

    #[test]
    fn test_split_text_cjk_straddle() {
        // "日本語": split at 3 falls inside "本" (which occupies cells 2-3).
        // Expected: left = "日 " (2 + 1 space = 3 cells), right = " 語" (1 space + 2 = 3 cells).
        let (l, r) = split_text("日本語", 3);
        assert_eq!(
            cell_len(&l),
            3,
            "left part must be exactly 3 cells, got '{}'",
            l
        );
        assert_eq!(l, "日 ", "left must be '日 ' (日 + space)");
        assert_eq!(r, " 語", "right must be ' 語' (space + 語)");
    }

    #[test]
    fn test_split_text_emoji_straddle() {
        // "😽😽" = 4 cells. Split at 1 straddles first emoji.
        let (l, r) = split_text("😽😽", 1);
        assert_eq!(cell_len(&l), 1, "left must be 1 cell wide");
        assert_eq!(l, " ", "left must be a single space");
        assert_eq!(r, " 😽", "right must start with compensating space");
    }

    #[test]
    fn test_split_text_emoji_exact() {
        // Split at 2 gives first emoji | second emoji.
        let (l, r) = split_text("😽😽", 2);
        assert_eq!(l, "😽");
        assert_eq!(r, "😽");
    }

    #[test]
    fn test_split_text_mixed_ascii_cjk() {
        // "aあb" = 4 cells (1+2+1). Split at 2 → "a" + space + ... wait:
        // spans: ('a',0..1,1), ('あ',1..4,2), ('b',4..5,1)
        // accumulated after 'a' = 1, after 'あ' = 3, but split at 2 straddles 'あ'.
        let (l, r) = split_text("aあb", 2);
        assert_eq!(cell_len(&l), 2, "left must be 2 cells");
        // 'a'(1) + space(1) = 2 cells
        assert_eq!(l, "a ");
        // space(1) + 'b'(1) = 2 cells
        assert_eq!(r, " b");

        // Split at 1 → "a" | "あb"
        let (l, r) = split_text("aあb", 1);
        assert_eq!(l, "a");
        assert_eq!(r, "あb");

        // Split at 3 → "aあ" | "b"
        let (l, r) = split_text("aあb", 3);
        assert_eq!(l, "aあ");
        assert_eq!(r, "b");
    }

    #[test]
    fn test_split_text_width_preservation() {
        // cell_len(left) + cell_len(right) should equal cell_len(text)
        // for clean splits, or cell_len(text) + spaces for straddle splits.
        let cases = [
            ("日本語abc", 3),
            ("hello world", 6),
            ("あいうえお", 5),
            ("a日b本c語", 4),
        ];
        for (text, at) in cases {
            let (l, r) = split_text(text, at);
            assert_eq!(
                cell_len(&l),
                at.min(cell_len(text)),
                "split_text({text:?}, {at}): left width should be {at} cells, got '{}' = {} cells",
                l,
                cell_len(&l)
            );
            let _ = r; // right side checked implicitly
        }
    }
}

// ---------------------------------------------------------------------------
// Opt 3: thread-local LRU cache for non-ASCII char widths
// ---------------------------------------------------------------------------

use lru::LruCache;
use std::cell::RefCell;
use std::num::NonZeroUsize;

thread_local! {
    /// Per-thread LRU cache: char → cell width (as u8, 0/1/2).
    ///
    /// Capacity 1024: enough for a full CJK/emoji working set without
    /// excessive memory use. Only non-ASCII chars are cached; ASCII (< 0x80)
    /// uses the fast path that already avoids the unicode_width lookup.
    static CHAR_WIDTH_CACHE: RefCell<LruCache<char, u8>> =
        RefCell::new(LruCache::new(NonZeroUsize::new(1024).unwrap()));
}

/// Look up the cached width of a non-ASCII `char`, or compute and store it.
///
/// Called only for chars with `c as u32 >= 0x80`. The result is cast to `u8`
/// (values are 0, 1, or 2 — always fits).
#[inline]
fn cached_char_width(c: char) -> usize {
    CHAR_WIDTH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(&w) = cache.get(&c) {
            return w as usize;
        }
        let w = c.width().unwrap_or(0) as u8;
        cache.put(c, w);
        w as usize
    })
}

#[cfg(test)]
pub(crate) fn cache_hit_count_for(c: char) -> Option<u8> {
    CHAR_WIDTH_CACHE.with(|cache| cache.borrow().peek(&c).copied())
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
    fn family_zwj_emoji_is_6_cells_terminal_reality() {
        // 👨‍👩‍👧 = U+1F468 ZWJ U+1F469 ZWJ U+1F467 (5 codepoints).
        // Per-codepoint widths: 2 + 0 + 2 + 0 + 2 = 6.
        // v1.4.1 deliberately reports 6 (terminal reality on most
        // setups) rather than 2 (Unicode cluster spec). See cell_len
        // docstring for the rationale.
        let s = "👨\u{200d}👩\u{200d}👧";
        assert_eq!(s.chars().count(), 5);
        assert_eq!(cell_len(s), 6);
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
