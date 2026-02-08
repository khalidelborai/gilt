//! Word wrapping utilities for terminal text.
//!
//! This module provides functions for splitting text into words and determining
//! line break positions for word wrapping, respecting cell widths for CJK and
//! other double-width characters.
//!
//! Port of Python rich's `_wrap.py`.

use once_cell::sync::Lazy;
use regex::Regex;

use crate::cells::{cell_len, chop_cells};

/// Regex matching a "word" — optional leading whitespace, then non-whitespace,
/// then optional trailing whitespace.
static RE_WORD: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\S+\s*").unwrap());

/// Yields each word from the text as `(char_start, char_end, word_slice)`.
///
/// A "word" is defined by the regex `\s*\S+\s*`, matching optional leading
/// whitespace, one or more non-whitespace characters, and optional trailing
/// whitespace. Positions are **character** (char) indices, not byte indices.
///
/// # Examples
///
/// ```
/// use gilt::wrap::words;
///
/// let result: Vec<_> = words("foo bar baz");
/// assert_eq!(result, vec![
///     (0, 4, "foo "),
///     (4, 8, "bar "),
///     (8, 11, "baz"),
/// ]);
/// ```
pub fn words(text: &str) -> Vec<(usize, usize, &str)> {
    // The regex crate works with byte offsets, but we need char offsets.
    // Build a byte-to-char mapping.
    let byte_to_char = build_byte_to_char_map(text);

    let mut result = Vec::new();
    for m in RE_WORD.find_iter(text) {
        let byte_start = m.start();
        let byte_end = m.end();
        let char_start = byte_to_char[byte_start];
        let char_end = byte_to_char[byte_end];
        result.push((char_start, char_end, m.as_str()));
    }
    result
}

/// Build a mapping from byte offset to char index for the given string.
///
/// Returns a vector of length `text.len() + 1` where `v[byte_offset]` is the
/// char index at that byte position. Only positions that are valid char
/// boundaries have meaningful values; we populate all of them by carrying
/// forward.
fn build_byte_to_char_map(text: &str) -> Vec<usize> {
    let mut map = vec![0usize; text.len() + 1];
    let mut char_idx = 0;
    for (byte_idx, _ch) in text.char_indices() {
        map[byte_idx] = char_idx;
        char_idx += 1;
    }
    // The position one past the last byte corresponds to the total char count.
    map[text.len()] = char_idx;
    map
}

/// Given text and a cell width, return char-index offsets where the string
/// should be split for word wrapping.
///
/// `fold` controls whether words longer than `width` are folded (broken across
/// multiple lines) or left as single overlong lines.
///
/// The returned positions are **character** (char) indices suitable for use
/// with `Text.divide`.
///
/// # Examples
///
/// ```
/// use gilt::wrap::divide_line;
///
/// assert_eq!(divide_line("foo bar baz", 3, true), vec![4, 8]);
/// assert_eq!(divide_line("abracadabra", 4, true), vec![4, 8]);
/// ```
pub fn divide_line(text: &str, width: usize, fold: bool) -> Vec<usize> {
    if width == 0 {
        return vec![];
    }

    let mut break_positions: Vec<usize> = Vec::new();
    let mut cell_offset: usize = 0;

    for (start, _end, word) in words(text) {
        let word_length = cell_len(word.trim_end());
        let remaining_space = width.saturating_sub(cell_offset);
        let word_fits_remaining_space = remaining_space >= word_length;

        if word_fits_remaining_space {
            cell_offset += cell_len(word);
        } else if word_length > width {
            if fold {
                let folded_word = chop_cells(word, width);
                let num_pieces = folded_word.len();
                let mut current_start = start;

                for (i, line) in folded_word.iter().enumerate() {
                    let is_last = i == num_pieces - 1;
                    if is_last {
                        if current_start > 0 {
                            break_positions.push(current_start);
                        }
                        cell_offset = cell_len(line);
                    } else {
                        if current_start > 0 {
                            break_positions.push(current_start);
                        }
                        current_start += line.chars().count();
                    }
                }
            } else {
                if start > 0 {
                    break_positions.push(start);
                }
                cell_offset = cell_len(word);
            }
        } else {
            // Word fits on a fresh line but not remaining space.
            if cell_offset > 0 && start > 0 {
                break_positions.push(start);
            }
            cell_offset = cell_len(word);
        }
    }

    break_positions
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // words() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_words_basic() {
        let result = words("foo bar baz");
        assert_eq!(
            result,
            vec![(0, 4, "foo "), (4, 8, "bar "), (8, 11, "baz"),]
        );
    }

    #[test]
    fn test_words_leading_whitespace() {
        let result = words("  hello world");
        assert_eq!(result, vec![(0, 8, "  hello "), (8, 13, "world"),]);
    }

    #[test]
    fn test_words_trailing_whitespace() {
        let result = words("hello world  ");
        assert_eq!(result, vec![(0, 6, "hello "), (6, 13, "world  "),]);
    }

    #[test]
    fn test_words_single_word() {
        let result = words("hello");
        assert_eq!(result, vec![(0, 5, "hello")]);
    }

    #[test]
    fn test_words_empty_string() {
        let result = words("");
        assert_eq!(result, Vec::<(usize, usize, &str)>::new());
    }

    #[test]
    fn test_words_only_whitespace() {
        // The regex \s*\S+\s* requires at least one \S, so pure whitespace yields nothing.
        let result = words("   ");
        assert_eq!(result, Vec::<(usize, usize, &str)>::new());
    }

    #[test]
    fn test_words_multiple_spaces_between() {
        let result = words("foo   bar");
        // Regex is greedy: at pos 0, \s*="" \S+="foo" \s*="   " -> "foo   "
        // Then at pos 6: \s*="" \S+="bar" -> "bar"
        assert_eq!(result, vec![(0, 6, "foo   "), (6, 9, "bar"),]);
    }

    #[test]
    fn test_words_cjk() {
        // CJK chars are \S, whitespace is \s — regex should still work.
        let result = words("あ い");
        assert_eq!(result, vec![(0, 2, "あ "), (2, 3, "い"),]);
    }

    #[test]
    fn test_words_no_trailing_space() {
        let result = words("abracadabra");
        assert_eq!(result, vec![(0, 11, "abracadabra")]);
    }

    // -----------------------------------------------------------------------
    // divide_line() tests — simple cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_divide_line_simple_width_3() {
        assert_eq!(divide_line("foo bar baz", 3, true), vec![4, 8]);
    }

    #[test]
    fn test_divide_line_simple_width_4() {
        assert_eq!(divide_line("foo bar baz", 4, true), vec![4, 8]);
    }

    #[test]
    fn test_divide_line_simple_width_7() {
        // "foo ": cell_len=4, fits in 7 -> offset=4
        // "bar ": rstrip="bar" len=3, remaining=7-4=3, fits -> offset=8
        // "baz": rstrip="baz" len=3, remaining=7-8=0 < 3 -> break at 8
        assert_eq!(divide_line("foo bar baz", 7, true), vec![8]);
    }

    #[test]
    fn test_divide_line_fits_on_one_line() {
        assert_eq!(divide_line("foo bar baz", 20, true), Vec::<usize>::new());
    }

    #[test]
    fn test_divide_line_exact_fit() {
        assert_eq!(divide_line("foo bar baz", 11, true), Vec::<usize>::new());
    }

    // -----------------------------------------------------------------------
    // divide_line() tests — fold long words
    // -----------------------------------------------------------------------

    #[test]
    fn test_divide_line_fold_long_word() {
        // "abracadabra" width=4
        // Single word (0,11), word_length=11 > 4, fold=true
        // chop_cells = ["abra","cada","bra"]
        // Piece 0 "abra" (not last): start=0, no append. start += 4 -> 4
        // Piece 1 "cada" (not last): start=4 > 0 -> append(4). start += 4 -> 8
        // Piece 2 "bra" (last): start=8 > 0 -> append(8). cell_offset=3
        assert_eq!(divide_line("abracadabra", 4, true), vec![4, 8]);
    }

    #[test]
    fn test_divide_line_fold_long_word_after_short() {
        // "XX 12345678912" width=4
        // "XX ": fits -> cell_offset=3
        // "12345678912": len=11>4, fold
        //   chop = ["1234","5678","912"]
        //   "1234": start=3>0 -> append(3). start=7
        //   "5678": start=7>0 -> append(7). start=11
        //   "912": start=11>0 -> append(11). cell_offset=3
        assert_eq!(divide_line("XX 12345678912", 4, true), vec![3, 7, 11]);
    }

    #[test]
    fn test_divide_line_fold_single_char_width() {
        // "abcd" width=1 -> each char on its own line
        // chop = ["a","b","c","d"]
        // "a": start=0, no append. start=1
        // "b": start=1, append(1). start=2
        // "c": start=2, append(2). start=3
        // "d" (last): start=3, append(3). cell_offset=1
        assert_eq!(divide_line("abcd", 1, true), vec![1, 2, 3]);
    }

    // -----------------------------------------------------------------------
    // divide_line() tests — no fold
    // -----------------------------------------------------------------------

    #[test]
    fn test_divide_line_no_fold() {
        // "abracadabra" width=4, fold=false
        // word_length=11 > 4, fold=false
        // start=0, no append. cell_offset=11
        assert_eq!(divide_line("abracadabra", 4, false), Vec::<usize>::new());
    }

    #[test]
    fn test_divide_line_no_fold_long_word_after_short() {
        // "XX 12345678912" width=4, fold=false
        // "XX ": fits -> cell_offset=3
        // "12345678912": len=11>4, fold=false. start=3>0 -> append(3). cell_offset=11
        assert_eq!(divide_line("XX 12345678912", 4, false), vec![3]);
    }

    // -----------------------------------------------------------------------
    // divide_line() tests — CJK characters
    // -----------------------------------------------------------------------

    #[test]
    fn test_divide_line_cjk_width_4() {
        // "ああああ" -> each char 2 cells wide -> total 8 cells
        // Single word, word_length=8 > 4, fold=true
        // chop_cells("ああああ", 4) = ["ああ", "ああ"]
        // Piece 0 (not last): start=0, no append. start += 2 -> 2
        // Piece 1 (last): start=2 > 0, append(2). cell_offset=4
        assert_eq!(divide_line("ああああ", 4, true), vec![2]);
    }

    #[test]
    fn test_divide_line_cjk_with_ascii() {
        // "aあ bい" width=3
        // words: "aあ "(0,3), "bい"(3,5)
        // "aあ ": rstrip="aあ", cell_len=3, remaining=3, fits -> cell_offset=cell_len("aあ ")=4
        // "bい": rstrip="bい", cell_len=3, remaining=3-4=0 < 3. 3<=3. cell_offset>0 && start(3)>0 -> append(3)
        assert_eq!(divide_line("aあ bい", 3, true), vec![3]);
    }

    #[test]
    fn test_divide_line_cjk_fold() {
        // "ああああああ" -> 6 CJK chars, 12 cells, width=5
        // word_length=12 > 5, fold=true
        // chop_cells with width=5: each CJK is 2 cells, so 2 chars fit in 4 cells (can't fit 3rd at 6>5)
        // chop = ["ああ", "ああ", "ああ"]
        // Piece 0: start=0, no append. start += 2 -> 2
        // Piece 1: start=2, append(2). start += 2 -> 4
        // Piece 2 (last): start=4, append(4). cell_offset=4
        assert_eq!(divide_line("ああああああ", 5, true), vec![2, 4]);
    }

    // -----------------------------------------------------------------------
    // divide_line() edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_divide_line_empty_string() {
        assert_eq!(divide_line("", 10, true), Vec::<usize>::new());
    }

    #[test]
    fn test_divide_line_width_zero() {
        assert_eq!(divide_line("hello", 0, true), Vec::<usize>::new());
    }

    #[test]
    fn test_divide_line_single_word_fits() {
        assert_eq!(divide_line("hello", 10, true), Vec::<usize>::new());
    }

    #[test]
    fn test_divide_line_single_char() {
        assert_eq!(divide_line("a", 1, true), Vec::<usize>::new());
    }

    #[test]
    fn test_divide_line_all_spaces_yields_nothing() {
        assert_eq!(divide_line("     ", 3, true), Vec::<usize>::new());
    }

    #[test]
    fn test_divide_line_word_exactly_width() {
        assert_eq!(divide_line("abcd", 4, true), Vec::<usize>::new());
    }

    #[test]
    fn test_divide_line_two_words_each_exactly_width() {
        // "abcd efgh" width=4
        // "abcd ": rstrip="abcd", len=4, fits in 4 -> cell_offset=5
        // "efgh": rstrip="efgh", len=4, remaining=4-5=0 < 4. 4<=4. cell_offset>0 && start>0 -> append(5)
        assert_eq!(divide_line("abcd efgh", 4, true), vec![5]);
    }

    #[test]
    fn test_divide_line_word_with_leading_spaces() {
        // "  hello  world" width=5
        // words: "  hello  "(0,9), "world"(9,14)
        // "  hello  ": rstrip="  hello", cell_len=7, remaining=5, doesn't fit
        //   word_length=7>5, fold=true
        //   chop_cells("  hello  ", 5) = ["  hel", "lo  "]
        //   Piece 0 (not last): start=0, no append. start += 5 -> 5
        //   Piece 1 (last): start=5, append(5). cell_offset=cell_len("lo  ")=4
        // "world": rstrip="world", cell_len=5, remaining=5-4=1 < 5. 5<=5.
        //   cell_offset(4)>0 && start(9)>0 -> append(9)
        assert_eq!(divide_line("  hello  world", 5, true), vec![5, 9]);
    }

    #[test]
    fn test_divide_line_many_short_words() {
        // "a b c d e" width=1
        // words: "a "(0,2), "b "(2,4), "c "(4,6), "d "(6,8), "e"(8,9)
        // "a ": rstrip="a", len=1, fits in 1 -> cell_offset=2
        // "b ": rstrip="b", len=1, remaining=1-2=0 < 1. 1<=1. cell_offset>0 && start>0 -> append(2). cell_offset=2
        // "c ": same -> append(4). cell_offset=2
        // "d ": same -> append(6). cell_offset=2
        // "e": same -> append(8). cell_offset=1
        assert_eq!(divide_line("a b c d e", 1, true), vec![2, 4, 6, 8]);
    }

    // -----------------------------------------------------------------------
    // build_byte_to_char_map() tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_byte_to_char_map_ascii() {
        let map = build_byte_to_char_map("abc");
        assert_eq!(map, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_byte_to_char_map_multibyte() {
        // "あ" is 3 bytes (UTF-8: E3 81 82)
        let map = build_byte_to_char_map("あ");
        assert_eq!(map[0], 0);
        assert_eq!(map[3], 1);
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn test_byte_to_char_map_mixed() {
        // "aあb" -> 'a'=1 byte, 'あ'=3 bytes, 'b'=1 byte -> 5 bytes total
        let map = build_byte_to_char_map("aあb");
        assert_eq!(map[0], 0); // 'a' at char 0
        assert_eq!(map[1], 1); // 'あ' at char 1
        assert_eq!(map[4], 2); // 'b' at char 2
        assert_eq!(map[5], 3); // past end
    }

    #[test]
    fn test_byte_to_char_map_empty() {
        let map = build_byte_to_char_map("");
        assert_eq!(map, vec![0]);
    }
}
