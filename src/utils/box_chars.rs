//! Box-drawing character sets for tables.
//!
//! Port of Python `rich/box.py`. Defines 19 built-in box styles for rendering
//! table borders and separators.

use std::sync::LazyLock;

/// Which level of row separator to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowLevel {
    /// Separator below the header row.
    Head,
    /// Separator between body rows.
    Row,
    /// Separator above the footer row.
    Foot,
    /// Mid-level separator (between header and body when there's a distinct mid section).
    Mid,
}

/// A set of box-drawing characters for rendering table borders.
///
/// Parsed from an 8-line definition string where each line has exactly 4 characters:
///
/// ```text
/// Line 1 (top):      top_left, top, top_divider, top_right
/// Line 2 (head):     head_left, _, head_vertical, head_right
/// Line 3 (head_row): head_row_left, head_row_horizontal, head_row_cross, head_row_right
/// Line 4 (mid):      mid_left, _, mid_vertical, mid_right
/// Line 5 (row):      row_left, row_horizontal, row_cross, row_right
/// Line 6 (foot_row): foot_row_left, foot_row_horizontal, foot_row_cross, foot_row_right
/// Line 7 (foot):     foot_left, _, foot_vertical, foot_right
/// Line 8 (bottom):   bottom_left, bottom_char, bottom_divider, bottom_right
/// ```
#[derive(Debug, Clone)]
pub struct BoxChars {
    /// Top-left corner character (e.g. `┌`).
    pub top_left: char,
    /// Top border horizontal fill character (e.g. `─`).
    pub top: char,
    /// Top border column divider character (e.g. `┬`).
    pub top_divider: char,
    /// Top-right corner character (e.g. `┐`).
    pub top_right: char,

    /// Left border character for the header row (e.g. `│`).
    pub head_left: char,
    /// Vertical column divider in the header row (e.g. `│`).
    pub head_vertical: char,
    /// Right border character for the header row (e.g. `│`).
    pub head_right: char,

    /// Left border character for the header separator (e.g. `├`).
    pub head_row_left: char,
    /// Horizontal fill character for the header separator (e.g. `─`).
    pub head_row_horizontal: char,
    /// Cross/intersection character for the header separator (e.g. `┼`).
    pub head_row_cross: char,
    /// Right border character for the header separator (e.g. `┤`).
    pub head_row_right: char,

    /// Left border character for the mid separator (e.g. `│`).
    pub mid_left: char,
    /// Vertical column divider for the mid separator (e.g. `│`).
    pub mid_vertical: char,
    /// Right border character for the mid separator (e.g. `│`).
    pub mid_right: char,

    /// Left border character for body row separators (e.g. `├`).
    pub row_left: char,
    /// Horizontal fill character for body row separators (e.g. `─`).
    pub row_horizontal: char,
    /// Cross/intersection character for body row separators (e.g. `┼`).
    pub row_cross: char,
    /// Right border character for body row separators (e.g. `┤`).
    pub row_right: char,

    /// Left border character for the footer separator (e.g. `├`).
    pub foot_row_left: char,
    /// Horizontal fill character for the footer separator (e.g. `─`).
    pub foot_row_horizontal: char,
    /// Cross/intersection character for the footer separator (e.g. `┼`).
    pub foot_row_cross: char,
    /// Right border character for the footer separator (e.g. `┤`).
    pub foot_row_right: char,

    /// Left border character for the footer row (e.g. `│`).
    pub foot_left: char,
    /// Vertical column divider in the footer row (e.g. `│`).
    pub foot_vertical: char,
    /// Right border character for the footer row (e.g. `│`).
    pub foot_right: char,

    /// Bottom-left corner character (e.g. `└`).
    pub bottom_left: char,
    /// Bottom border horizontal fill character (e.g. `─`).
    pub bottom_char: char,
    /// Bottom border column divider character (e.g. `┴`).
    pub bottom_divider: char,
    /// Bottom-right corner character (e.g. `┘`).
    pub bottom_right: char,

    /// Whether this box uses only ASCII characters.
    pub ascii: bool,
}

impl BoxChars {
    /// Parse a box definition string into a `BoxChars`.
    ///
    /// The string must contain 8 lines separated by `\n`, each with exactly 4 characters.
    ///
    /// # Panics
    ///
    /// Panics if the string does not have exactly 8 lines or any line does not have
    /// exactly 4 characters.
    pub fn new(box_str: &str, ascii: bool) -> Self {
        let lines: Vec<&str> = box_str.split('\n').collect();
        assert_eq!(lines.len(), 8, "Box string must have exactly 8 lines");

        let parse_line = |line: &str| -> Vec<char> {
            let chars: Vec<char> = line.chars().collect();
            assert_eq!(
                chars.len(),
                4,
                "Each box line must have exactly 4 characters, got {} in {:?}",
                chars.len(),
                line,
            );
            chars
        };

        let top = parse_line(lines[0]);
        let head = parse_line(lines[1]);
        let head_row = parse_line(lines[2]);
        let mid = parse_line(lines[3]);
        let row = parse_line(lines[4]);
        let foot_row = parse_line(lines[5]);
        let foot = parse_line(lines[6]);
        let bottom = parse_line(lines[7]);

        Self {
            top_left: top[0],
            top: top[1],
            top_divider: top[2],
            top_right: top[3],

            head_left: head[0],
            head_vertical: head[2],
            head_right: head[3],

            head_row_left: head_row[0],
            head_row_horizontal: head_row[1],
            head_row_cross: head_row[2],
            head_row_right: head_row[3],

            mid_left: mid[0],
            mid_vertical: mid[2],
            mid_right: mid[3],

            row_left: row[0],
            row_horizontal: row[1],
            row_cross: row[2],
            row_right: row[3],

            foot_row_left: foot_row[0],
            foot_row_horizontal: foot_row[1],
            foot_row_cross: foot_row[2],
            foot_row_right: foot_row[3],

            foot_left: foot[0],
            foot_vertical: foot[2],
            foot_right: foot[3],

            bottom_left: bottom[0],
            bottom_char: bottom[1],
            bottom_divider: bottom[2],
            bottom_right: bottom[3],

            ascii,
        }
    }

    /// Build the top border string for columns of given widths.
    ///
    /// Example for widths `[5, 3]` with SQUARE box:
    /// `"┌─────┬───┐"`
    pub fn get_top(&self, widths: &[usize]) -> String {
        // Final size is exactly (sum widths) + (num dividers) + 2 corners.
        // Each char may be up to 4 bytes (CJK box-drawing chars are 3).
        let cap = widths.iter().sum::<usize>() * 3 + widths.len() + 2;
        let mut s = String::with_capacity(cap);
        s.push(self.top_left);
        for (i, &width) in widths.iter().enumerate() {
            for _ in 0..width {
                s.push(self.top);
            }
            if i < widths.len() - 1 {
                s.push(self.top_divider);
            }
        }
        s.push(self.top_right);
        s
    }

    /// Build a row separator string for columns of given widths.
    ///
    /// `level` determines which set of characters to use.
    /// If `edge` is false, the left and right border characters are omitted.
    pub fn get_row(&self, widths: &[usize], level: RowLevel, edge: bool) -> String {
        let (left, horizontal, cross, right) = match level {
            RowLevel::Head => (
                self.head_row_left,
                self.head_row_horizontal,
                self.head_row_cross,
                self.head_row_right,
            ),
            RowLevel::Row => (
                self.row_left,
                self.row_horizontal,
                self.row_cross,
                self.row_right,
            ),
            RowLevel::Foot => (
                self.foot_row_left,
                self.foot_row_horizontal,
                self.foot_row_cross,
                self.foot_row_right,
            ),
            RowLevel::Mid => (self.mid_left, ' ', self.mid_vertical, self.mid_right),
        };

        let cap = widths.iter().sum::<usize>() * 3 + widths.len() + if edge { 2 } else { 0 };
        let mut s = String::with_capacity(cap);
        if edge {
            s.push(left);
        }
        for (i, &width) in widths.iter().enumerate() {
            for _ in 0..width {
                s.push(horizontal);
            }
            if i < widths.len() - 1 {
                s.push(cross);
            }
        }
        if edge {
            s.push(right);
        }
        s
    }

    /// Build the bottom border string for columns of given widths.
    ///
    /// Example for widths `[5, 3]` with SQUARE box:
    /// `"└─────┴───┘"`
    pub fn get_bottom(&self, widths: &[usize]) -> String {
        let cap = widths.iter().sum::<usize>() * 3 + widths.len() + 2;
        let mut s = String::with_capacity(cap);
        s.push(self.bottom_left);
        for (i, &width) in widths.iter().enumerate() {
            for _ in 0..width {
                s.push(self.bottom_char);
            }
            if i < widths.len() - 1 {
                s.push(self.bottom_divider);
            }
        }
        s.push(self.bottom_right);
        s
    }

    /// Return a substituted box for the given terminal capabilities.
    ///
    /// - `ascii_only`: encoding is not UTF-based → always returns `&ASCII` for
    ///   any non-ASCII box (rich parity).
    /// - `legacy_windows`: the console is running on a legacy Windows terminal
    ///   that supports Unicode but not all box styles.  When `true` (and
    ///   `ascii_only` is false), applies the rich legacy-Windows safe
    ///   substitutions: ROUNDED/HEAVY/HEAVY_EDGE/HEAVY_HEAD→SQUARE,
    ///   MINIMAL_HEAVY_HEAD→MINIMAL, SIMPLE_HEAVY→SIMPLE.  Boxes that are
    ///   already "safe" (SQUARE, MINIMAL, SIMPLE, ASCII boxes) are returned
    ///   unchanged.
    ///
    /// **Important**: `legacy_windows` substitution is gated by the caller's
    /// `safe_box` flag (rich parity — rich only applies legacy substitutions
    /// when `safe=True`).  Callers must pass `legacy_windows=true` only when
    /// `safe_box` is also enabled; see Panel::gilt_console and Table::render.
    ///
    /// If neither flag is set, or the box is already ASCII, returns `self`.
    pub fn substitute(&self, ascii_only: bool, legacy_windows: bool) -> &BoxChars {
        if ascii_only {
            // Already ASCII — nothing to do.
            if self.ascii {
                return self;
            }
            // Any non-ASCII box → ASCII (rich parity).
            return &ASCII;
        }
        if legacy_windows && !self.ascii {
            // Apply rich's LEGACY_WINDOWS_SUBSTITUTIONS table exactly
            // (research_doc/12-panel-and-box.md §"Box Substitution").
            // Fingerprint: (top_left, head_row_horizontal[, head_row_cross])
            return match (self.top_left, self.head_row_horizontal) {
                // ROUNDED: ╭─  → SQUARE
                ('\u{256D}', '\u{2500}') => &SQUARE,
                // HEAVY: ┏━ with heavy cross ╋  → SQUARE
                ('\u{250F}', '\u{2501}') if self.head_row_cross == '\u{254B}' => &SQUARE,
                // HEAVY_EDGE: ┏─  → SQUARE
                ('\u{250F}', '\u{2500}') => &SQUARE,
                // HEAVY_HEAD: ┏━ with mixed cross ╇  → SQUARE
                ('\u{250F}', '\u{2501}') if self.head_row_cross == '\u{2547}' => &SQUARE,
                // MINIMAL_HEAVY_HEAD: space, ━, cross=┿  → MINIMAL
                (' ', '\u{2501}') if self.head_row_cross == '\u{253F}' => &MINIMAL,
                // SIMPLE_HEAVY: space, ━, otherwise  → SIMPLE
                (' ', '\u{2501}') => &SIMPLE,
                // All other boxes (DOUBLE, DOUBLE_EDGE, SQUARE, MINIMAL, SIMPLE,
                // HORIZONTALS, etc.) are considered safe on legacy-Windows.
                _ => self,
            };
        }
        self
    }

    /// Return a plain-headed variant of this box style.
    ///
    /// Replaces double/heavy header separators with single-line equivalents:
    /// - HEAVY_HEAD -> SQUARE
    /// - SQUARE_DOUBLE_HEAD -> SQUARE
    /// - MINIMAL_DOUBLE_HEAD -> MINIMAL
    /// - MINIMAL_HEAVY_HEAD -> MINIMAL
    /// - ASCII_DOUBLE_HEAD -> ASCII2
    pub fn get_plain_headed_box(&self) -> &BoxChars {
        match (self.top_left, self.head_row_horizontal) {
            // HEAVY_HEAD: top_left='┏', head_row_horizontal='━', cross='╇'
            ('\u{250F}', '\u{2501}') if self.head_row_cross == '\u{2547}' => &SQUARE,
            // SQUARE_DOUBLE_HEAD: top_left='┌', head_row_horizontal='═'
            ('\u{250C}', '\u{2550}') => &SQUARE,
            // MINIMAL_DOUBLE_HEAD: top_left=' ', head_row_horizontal='═'
            (' ', '\u{2550}') => &MINIMAL,
            // MINIMAL_HEAVY_HEAD: top_left=' ', head_row_horizontal='━', cross='┿'
            (' ', '\u{2501}') if self.head_row_cross == '\u{253F}' => &MINIMAL,
            // ASCII_DOUBLE_HEAD: top_left='+', head_row_horizontal='='
            ('+', '=') => &ASCII2,
            _ => self,
        }
    }
}

// ──────────────────────────────────────────────────────────
// Box constant definitions
// ──────────────────────────────────────────────────────────

/// ASCII box style using `+`, `-`, and `|` characters.
pub static ASCII: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("+--+\n| ||\n|-+|\n| ||\n|-+|\n|-+|\n| ||\n+--+", true));

/// Alternate ASCII box style with `+` at every intersection.
pub static ASCII2: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("+-++\n| ||\n+-++\n| ||\n+-++\n+-++\n| ||\n+-++", true));

/// ASCII box style with `=` for the header separator row.
pub static ASCII_DOUBLE_HEAD: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("+-++\n| ||\n+=++\n| ||\n+-++\n+-++\n| ||\n+-++", true));

/// Standard single-line Unicode box style (`┌─┬┐`, `│`, `└─┴┘`).
pub static SQUARE: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("┌─┬┐\n│ ││\n├─┼┤\n│ ││\n├─┼┤\n├─┼┤\n│ ││\n└─┴┘", false));

/// Single-line Unicode box with a double-line header separator (`╞═╪╡`).
pub static SQUARE_DOUBLE_HEAD: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("┌─┬┐\n│ ││\n╞═╪╡\n│ ││\n├─┼┤\n├─┼┤\n│ ││\n└─┴┘", false));

/// Minimal box style with no outer borders, only column dividers and row separators.
pub static MINIMAL: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("  ╷ \n  │ \n╶─┼╴\n  │ \n╶─┼╴\n╶─┼╴\n  │ \n  ╵ ", false));

/// Minimal box style with a heavy (thick) header separator (`╺━┿╸`).
pub static MINIMAL_HEAVY_HEAD: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("  ╷ \n  │ \n╺━┿╸\n  │ \n╶─┼╴\n╶─┼╴\n  │ \n  ╵ ", false));

/// Minimal box style with a double-line header separator (`═╪`).
pub static MINIMAL_DOUBLE_HEAD: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("  ╷ \n  │ \n ═╪ \n  │ \n ─┼ \n ─┼ \n  │ \n  ╵ ", false));

/// Simple box style with only horizontal rules for header and footer separators.
pub static SIMPLE: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("    \n    \n ── \n    \n    \n ── \n    \n    ", false));

/// Simple box style with only a header separator rule (no footer rule).
pub static SIMPLE_HEAD: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("    \n    \n ── \n    \n    \n    \n    \n    ", false));

/// Simple box style with heavy (thick) horizontal rules (`━`).
pub static SIMPLE_HEAVY: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("    \n    \n ━━ \n    \n    \n ━━ \n    \n    ", false));

/// Box style using only horizontal rules for all borders and separators.
pub static HORIZONTALS: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new(" ── \n    \n ── \n    \n ── \n ── \n    \n ── ", false));

/// Single-line Unicode box with rounded corners (`╭╮╰╯`).
pub static ROUNDED: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("╭─┬╮\n│ ││\n├─┼┤\n│ ││\n├─┼┤\n├─┼┤\n│ ││\n╰─┴╯", false));

/// Heavy (thick) Unicode box style (`┏━┳┓`, `┃`, `┗━┻┛`).
pub static HEAVY: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("┏━┳┓\n┃ ┃┃\n┣━╋┫\n┃ ┃┃\n┣━╋┫\n┣━╋┫\n┃ ┃┃\n┗━┻┛", false));

/// Heavy outer edges with light inner dividers (`┏━┯┓`, `┃│┃`).
pub static HEAVY_EDGE: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("┏━┯┓\n┃ │┃\n┠─┼┨\n┃ │┃\n┠─┼┨\n┠─┼┨\n┃ │┃\n┗━┷┛", false));

/// Heavy header section with light body (`┏━┳┓` header, `├─┼┤` body).
pub static HEAVY_HEAD: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("┏━┳┓\n┃ ┃┃\n┡━╇┩\n│ ││\n├─┼┤\n├─┼┤\n│ ││\n└─┴┘", false));

/// Double-line Unicode box style (`╔═╦╗`, `║`, `╚═╩╝`).
pub static DOUBLE: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("╔═╦╗\n║ ║║\n╠═╬╣\n║ ║║\n╠═╬╣\n╠═╬╣\n║ ║║\n╚═╩╝", false));

/// Double-line outer edges with single-line inner dividers (`╔═╤╗`, `║│║`).
pub static DOUBLE_EDGE: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("╔═╤╗\n║ │║\n╟─┼╢\n║ │║\n╟─┼╢\n╟─┼╢\n║ │║\n╚═╧╝", false));

/// Markdown-compatible table box style using `|` and `-` characters.
pub static MARKDOWN: LazyLock<BoxChars> =
    LazyLock::new(|| BoxChars::new("    \n| ||\n|-||\n| ||\n|-||\n|-||\n| ||\n    ", true));

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Parsing tests ----

    #[test]
    fn test_parse_ascii() {
        let b = &*ASCII;
        assert_eq!(b.top_left, '+');
        assert_eq!(b.top, '-');
        assert_eq!(b.top_divider, '-');
        assert_eq!(b.top_right, '+');
        assert_eq!(b.head_left, '|');
        assert_eq!(b.head_vertical, '|');
        assert_eq!(b.head_right, '|');
        assert_eq!(b.head_row_left, '|');
        assert_eq!(b.head_row_horizontal, '-');
        assert_eq!(b.head_row_cross, '+');
        assert_eq!(b.head_row_right, '|');
        assert_eq!(b.bottom_left, '+');
        assert_eq!(b.bottom_char, '-');
        assert_eq!(b.bottom_divider, '-');
        assert_eq!(b.bottom_right, '+');
        assert!(b.ascii);
    }

    #[test]
    fn test_parse_square() {
        let b = &*SQUARE;
        assert_eq!(b.top_left, '┌');
        assert_eq!(b.top, '─');
        assert_eq!(b.top_divider, '┬');
        assert_eq!(b.top_right, '┐');
        assert_eq!(b.head_left, '│');
        assert_eq!(b.head_vertical, '│');
        assert_eq!(b.head_right, '│');
        assert_eq!(b.head_row_left, '├');
        assert_eq!(b.head_row_horizontal, '─');
        assert_eq!(b.head_row_cross, '┼');
        assert_eq!(b.head_row_right, '┤');
        assert_eq!(b.bottom_left, '└');
        assert_eq!(b.bottom_char, '─');
        assert_eq!(b.bottom_divider, '┴');
        assert_eq!(b.bottom_right, '┘');
        assert!(!b.ascii);
    }

    #[test]
    fn test_parse_heavy() {
        let b = &*HEAVY;
        assert_eq!(b.top_left, '┏');
        assert_eq!(b.top, '━');
        assert_eq!(b.top_divider, '┳');
        assert_eq!(b.top_right, '┓');
        assert_eq!(b.head_row_cross, '╋');
        assert!(!b.ascii);
    }

    #[test]
    fn test_parse_double() {
        let b = &*DOUBLE;
        assert_eq!(b.top_left, '╔');
        assert_eq!(b.top, '═');
        assert_eq!(b.top_divider, '╦');
        assert_eq!(b.top_right, '╗');
        assert_eq!(b.bottom_left, '╚');
        assert_eq!(b.bottom_char, '═');
        assert_eq!(b.bottom_divider, '╩');
        assert_eq!(b.bottom_right, '╝');
    }

    #[test]
    fn test_parse_rounded() {
        let b = &*ROUNDED;
        assert_eq!(b.top_left, '╭');
        assert_eq!(b.top_right, '╮');
        assert_eq!(b.bottom_left, '╰');
        assert_eq!(b.bottom_right, '╯');
    }

    #[test]
    fn test_parse_all_19_boxes() {
        // Just force initialization of all 19 constants to ensure none panic
        let _ = &*ASCII;
        let _ = &*ASCII2;
        let _ = &*ASCII_DOUBLE_HEAD;
        let _ = &*SQUARE;
        let _ = &*SQUARE_DOUBLE_HEAD;
        let _ = &*MINIMAL;
        let _ = &*MINIMAL_HEAVY_HEAD;
        let _ = &*MINIMAL_DOUBLE_HEAD;
        let _ = &*SIMPLE;
        let _ = &*SIMPLE_HEAD;
        let _ = &*SIMPLE_HEAVY;
        let _ = &*HORIZONTALS;
        let _ = &*ROUNDED;
        let _ = &*HEAVY;
        let _ = &*HEAVY_EDGE;
        let _ = &*HEAVY_HEAD;
        let _ = &*DOUBLE;
        let _ = &*DOUBLE_EDGE;
        let _ = &*MARKDOWN;
    }

    #[test]
    fn test_ascii_flag() {
        assert!(ASCII.ascii);
        assert!(ASCII2.ascii);
        assert!(ASCII_DOUBLE_HEAD.ascii);
        assert!(MARKDOWN.ascii);
        assert!(!SQUARE.ascii);
        assert!(!ROUNDED.ascii);
        assert!(!HEAVY.ascii);
        assert!(!DOUBLE.ascii);
    }

    // ---- get_top tests ----

    #[test]
    fn test_get_top_square() {
        let top = SQUARE.get_top(&[5, 3]);
        assert_eq!(top, "┌─────┬───┐");
    }

    #[test]
    fn test_get_top_ascii() {
        // ASCII top_divider is '-', same as top fill char
        let top = ASCII.get_top(&[3, 4]);
        assert_eq!(top, "+--------+");
    }

    #[test]
    fn test_get_top_ascii2() {
        // ASCII2 top_divider is '+'
        let top = ASCII2.get_top(&[3, 4]);
        assert_eq!(top, "+---+----+");
    }

    #[test]
    fn test_get_top_single_column() {
        let top = SQUARE.get_top(&[10]);
        assert_eq!(top, "┌──────────┐");
    }

    #[test]
    fn test_get_top_three_columns() {
        let top = HEAVY.get_top(&[2, 3, 4]);
        assert_eq!(top, "┏━━┳━━━┳━━━━┓");
    }

    // ---- get_bottom tests ----

    #[test]
    fn test_get_bottom_square() {
        let bottom = SQUARE.get_bottom(&[5, 3]);
        assert_eq!(bottom, "└─────┴───┘");
    }

    #[test]
    fn test_get_bottom_heavy() {
        let bottom = HEAVY.get_bottom(&[4, 4]);
        assert_eq!(bottom, "┗━━━━┻━━━━┛");
    }

    #[test]
    fn test_get_bottom_single_column() {
        let bottom = DOUBLE.get_bottom(&[6]);
        assert_eq!(bottom, "╚══════╝");
    }

    // ---- get_row tests ----

    #[test]
    fn test_get_row_head_square() {
        let row = SQUARE.get_row(&[5, 3], RowLevel::Head, true);
        assert_eq!(row, "├─────┼───┤");
    }

    #[test]
    fn test_get_row_head_no_edge() {
        let row = SQUARE.get_row(&[5, 3], RowLevel::Head, false);
        assert_eq!(row, "─────┼───");
    }

    #[test]
    fn test_get_row_body_square() {
        let row = SQUARE.get_row(&[5, 3], RowLevel::Row, true);
        assert_eq!(row, "├─────┼───┤");
    }

    #[test]
    fn test_get_row_foot_square() {
        let row = SQUARE.get_row(&[5, 3], RowLevel::Foot, true);
        assert_eq!(row, "├─────┼───┤");
    }

    #[test]
    fn test_get_row_head_heavy() {
        let row = HEAVY.get_row(&[3, 3], RowLevel::Head, true);
        assert_eq!(row, "┣━━━╋━━━┫");
    }

    #[test]
    fn test_get_row_ascii() {
        let row = ASCII2.get_row(&[4, 4], RowLevel::Head, true);
        assert_eq!(row, "+----+----+");
    }

    // ---- substitute tests ----

    #[test]
    fn test_substitute_not_ascii() {
        let b = SQUARE.substitute(false, false);
        assert_eq!(b.top_left, '┌');
    }

    // Rich parity: ascii_only=true always returns &ASCII for non-ASCII boxes.
    // Old behavior returned SQUARE/MINIMAL/SIMPLE; corrected to ASCII.

    #[test]
    fn test_substitute_rounded_to_ascii() {
        // ascii_only=true: ROUNDED -> ASCII
        let b = ROUNDED.substitute(true, false);
        assert_eq!(b.top_left, '+');
        assert!(b.ascii);
    }

    #[test]
    fn test_substitute_heavy_to_ascii() {
        // ascii_only=true: HEAVY -> ASCII
        let b = HEAVY.substitute(true, false);
        assert_eq!(b.top_left, '+');
        assert!(b.ascii);
    }

    #[test]
    fn test_substitute_heavy_edge_to_ascii() {
        // ascii_only=true: HEAVY_EDGE -> ASCII
        let b = HEAVY_EDGE.substitute(true, false);
        assert_eq!(b.top_left, '+');
        assert!(b.ascii);
    }

    #[test]
    fn test_substitute_simple_heavy_to_ascii() {
        // ascii_only=true: SIMPLE_HEAVY -> ASCII
        let b = SIMPLE_HEAVY.substitute(true, false);
        assert_eq!(b.top_left, '+');
        assert!(b.ascii);
    }

    #[test]
    fn test_substitute_minimal_heavy_head_to_ascii() {
        // ascii_only=true: MINIMAL_HEAVY_HEAD -> ASCII
        let b = MINIMAL_HEAVY_HEAD.substitute(true, false);
        assert_eq!(b.top_left, '+');
        assert!(b.ascii);
    }

    #[test]
    fn test_substitute_square_to_ascii() {
        // SQUARE is not ASCII-flagged; ascii_only -> ASCII
        let b = SQUARE.substitute(true, false);
        assert_eq!(b.top_left, '+');
        assert!(b.ascii);
    }

    #[test]
    fn test_substitute_ascii_unchanged() {
        // Already-ASCII boxes must be returned as-is
        let b = ASCII.substitute(true, false);
        assert_eq!(b.top_left, '+');
        assert!(b.ascii);
    }

    // ── legacy_windows substitution tests ───────────────────────────────────

    #[test]
    fn test_substitute_legacy_windows_rounded_to_square() {
        // legacy_windows=true, not ascii_only: ROUNDED → SQUARE (simpler Unicode)
        let b = ROUNDED.substitute(false, true);
        assert_eq!(
            b.top_left, '┌',
            "ROUNDED should map to SQUARE on legacy-windows"
        );
        assert!(!b.ascii);
    }

    #[test]
    fn test_substitute_legacy_windows_heavy_to_square() {
        // HEAVY → SQUARE
        let b = HEAVY.substitute(false, true);
        assert_eq!(
            b.top_left, '┌',
            "HEAVY should map to SQUARE on legacy-windows"
        );
    }

    #[test]
    fn test_substitute_legacy_windows_double_unchanged() {
        // DOUBLE is NOT in rich's LEGACY_WINDOWS_SUBSTITUTIONS table
        // (research_doc/12-panel-and-box.md) — it should pass through unchanged.
        let b = DOUBLE.substitute(false, true);
        assert_eq!(
            b.top_left, '╔',
            "DOUBLE is safe on legacy-windows and should be returned unchanged"
        );
    }

    #[test]
    fn test_substitute_legacy_windows_minimal_heavy_head_to_minimal() {
        // MINIMAL_HEAVY_HEAD → MINIMAL
        let b = MINIMAL_HEAVY_HEAD.substitute(false, true);
        // MINIMAL top_left is ' '
        assert_eq!(
            b.top_left, ' ',
            "MINIMAL_HEAVY_HEAD should map to MINIMAL on legacy-windows"
        );
        assert_eq!(b.head_row_horizontal, '─');
    }

    #[test]
    fn test_substitute_legacy_windows_simple_heavy_to_simple() {
        // SIMPLE_HEAVY → SIMPLE
        let b = SIMPLE_HEAVY.substitute(false, true);
        assert_eq!(
            b.top_left, ' ',
            "SIMPLE_HEAVY should map to SIMPLE on legacy-windows"
        );
        assert_eq!(b.head_row_horizontal, '─');
    }

    #[test]
    fn test_substitute_legacy_windows_square_unchanged() {
        // SQUARE is already safe on legacy-windows
        let b = SQUARE.substitute(false, true);
        assert_eq!(
            b.top_left, '┌',
            "SQUARE should be unchanged on legacy-windows"
        );
    }

    #[test]
    fn test_substitute_legacy_windows_ascii_unchanged() {
        // ASCII box should pass through untouched
        let b = ASCII.substitute(false, true);
        assert_eq!(b.top_left, '+');
        assert!(b.ascii);
    }

    #[test]
    fn test_substitute_ascii_overrides_legacy_windows() {
        // When both ascii_only=true and legacy_windows=true, ascii_only wins
        let b = ROUNDED.substitute(true, true);
        assert_eq!(b.top_left, '+');
        assert!(b.ascii);
    }

    // ---- get_plain_headed_box tests ----

    #[test]
    fn test_plain_headed_heavy_head() {
        let b = HEAVY_HEAD.get_plain_headed_box();
        assert_eq!(b.top_left, '┌');
        assert_eq!(b.head_row_horizontal, '─');
    }

    #[test]
    fn test_plain_headed_square_double_head() {
        let b = SQUARE_DOUBLE_HEAD.get_plain_headed_box();
        assert_eq!(b.top_left, '┌');
        assert_eq!(b.head_row_horizontal, '─');
    }

    #[test]
    fn test_plain_headed_minimal_double_head() {
        let b = MINIMAL_DOUBLE_HEAD.get_plain_headed_box();
        assert_eq!(b.head_row_horizontal, '─');
        assert_eq!(b.head_row_cross, '┼');
    }

    #[test]
    fn test_plain_headed_ascii_double_head() {
        let b = ASCII_DOUBLE_HEAD.get_plain_headed_box();
        assert_eq!(b.head_row_horizontal, '-');
        assert_eq!(b.head_row_cross, '+');
    }

    #[test]
    fn test_plain_headed_identity() {
        // SQUARE has no special headed variant, returns itself
        let b = SQUARE.get_plain_headed_box();
        assert_eq!(b.top_left, '┌');
        assert_eq!(b.head_row_horizontal, '─');
    }

    // ---- Edge case tests ----

    #[test]
    fn test_get_top_empty_widths() {
        let top = SQUARE.get_top(&[]);
        assert_eq!(top, "┌┐");
    }

    #[test]
    fn test_get_bottom_empty_widths() {
        let bottom = SQUARE.get_bottom(&[]);
        assert_eq!(bottom, "└┘");
    }

    #[test]
    fn test_get_row_empty_widths() {
        let row = SQUARE.get_row(&[], RowLevel::Head, true);
        assert_eq!(row, "├┤");
    }

    #[test]
    fn test_get_top_zero_width_column() {
        let top = SQUARE.get_top(&[0, 3]);
        assert_eq!(top, "┌┬───┐");
    }

    #[test]
    fn test_markdown_box() {
        let b = &*MARKDOWN;
        assert_eq!(b.head_left, '|');
        assert_eq!(b.head_vertical, '|');
        assert_eq!(b.head_right, '|');
        assert_eq!(b.head_row_horizontal, '-');
        assert_eq!(b.head_row_cross, '|');
        assert!(b.ascii);
    }

    /// RowLevel::Mid must use SPACE as the horizontal fill, not `row_horizontal`.
    ///
    /// The `mid` line (line4) in rich's box format is a content/blank row:
    /// its horizontal position is the IGNORED `_` char (always ' ') not a
    /// separator character.  Regression test pinning Item 4 of batch 7.11.
    /// Reference: research_doc/12-panel-and-box.md §"Box Class Structure".
    #[test]
    fn get_row_mid_uses_space_fill_not_horizontal_char() {
        // SQUARE: row_horizontal='─', but Mid fill must be ' '
        let row = SQUARE.get_row(&[5, 3], RowLevel::Mid, true);
        // Should be: │     │   │  (mid_left + spaces + mid_vertical + spaces + mid_right)
        assert!(
            !row.contains('─'),
            "RowLevel::Mid must not use row_horizontal '─'; got: {:?}",
            row
        );
        assert!(
            row.starts_with('│'),
            "RowLevel::Mid must start with mid_left; got: {:?}",
            row
        );
        assert!(
            row.ends_with('│'),
            "RowLevel::Mid must end with mid_right; got: {:?}",
            row
        );
        // All interior chars must be spaces or mid_vertical
        let interior: Vec<char> = row.chars().skip(1).take(row.chars().count() - 2).collect();
        for &ch in &interior {
            assert!(
                ch == ' ' || ch == '│',
                "RowLevel::Mid interior must contain only spaces and mid_vertical; got '{}'",
                ch
            );
        }

        // HEAVY: row_horizontal='━', Mid fill must still be ' '
        let heavy_row = HEAVY.get_row(&[4], RowLevel::Mid, true);
        assert!(
            !heavy_row.contains('━'),
            "RowLevel::Mid must not use row_horizontal '━' for HEAVY box; got: {:?}",
            heavy_row
        );
    }

    #[test]
    #[should_panic(expected = "8 lines")]
    fn test_bad_line_count() {
        BoxChars::new("abc\ndef", false);
    }

    #[test]
    #[should_panic(expected = "4 characters")]
    fn test_bad_char_count() {
        BoxChars::new("ab\nabcd\nabcd\nabcd\nabcd\nabcd\nabcd\nabcd", false);
    }
}
