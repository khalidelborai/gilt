//! Segment - the atomic unit of terminal rendering.
//!
//! All content flows through segments, which combine text, style, and control codes.

use compact_str::CompactString;
use lru::LruCache;
use std::cell::RefCell;
use std::num::NonZeroUsize;

use crate::cells::{cell_len, get_character_cell_size, is_single_cell_widths, set_cell_size};
use crate::style::Style;

thread_local! {
    /// Per-thread LRU cache for `Segment::split_cells` results, keyed by
    /// `(text, cut)` → `(left_text, right_text)`.
    ///
    /// Mirrors rich's `@lru_cache(maxsize=16_384)` on the `_split_cells`
    /// classmethod (rich/segment.py). The capacity is 16K entries — large
    /// enough to absorb the hot path of repeated re-renders of the same
    /// content (status bars, live displays, etc.) while keeping per-thread
    /// memory bounded.
    ///
    /// Style is intentionally *not* part of the key: `split_cells` only
    /// inspects the cell layout, and a cached result is re-wrapped in the
    /// caller's style at the use site. Treating style as cache-local would
    /// either require an owned `Option<Style>` per value (cloning on every
    /// miss) or risk aliasing if two callers share the same `(text, cut)`.
    static SPLIT_CELLS_CACHE: RefCell<LruCache<(CompactString, usize), (CompactString, CompactString)>> =
        RefCell::new(LruCache::new(NonZeroUsize::new(16_384).unwrap()));
}

/// Look up (or compute) the (left, right) text pair for a `(text, cut)`.
///
/// Extracted so the LRU plumbing lives in one place; the method body itself
/// still owns the segment-cloning logic so the style and control codes are
/// not silently shared.
fn cached_split_texts(text: &str, cut: usize) -> (CompactString, CompactString) {
    let key = (CompactString::from(text), cut);
    SPLIT_CELLS_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(hit) = cache.get(&key) {
            return hit.clone();
        }
        let (left, right) = compute_split_texts(text, cut);
        cache.put(key, (left.clone(), right.clone()));
        (left, right)
    })
}

/// Test-only hook: number of entries currently in the per-thread cache.
///
/// `pub(crate)` so integration tests can assert that the cache is actually
/// being populated for repeated `(text, cut)` lookups.
#[cfg(test)]
pub(crate) fn split_cells_cache_len() -> usize {
    SPLIT_CELLS_CACHE.with(|cache| cache.borrow().len())
}

/// Test-only hook: clear the per-thread cache (useful for test isolation).
#[cfg(test)]
pub(crate) fn split_cells_cache_clear() {
    SPLIT_CELLS_CACHE.with(|cache| cache.borrow_mut().clear());
}

/// Pure split that returns just the two text halves. Used by the cache
/// wrapper above; the public `split_cells` method re-wraps the result with
/// the caller's style and clones the result out.
///
/// This mirrors the body of `Segment::split_cells` — including the
/// double-width straddling space substitution — so the cached text is
/// byte-equal to what the un-cached function would produce.
fn compute_split_texts(text: &str, cut: usize) -> (CompactString, CompactString) {
    let text_len = text.len();
    let cell_length = cell_len(text);

    // Fast path: if cut is beyond text length, return (full, empty)
    if cut >= cell_length {
        return (CompactString::from(text), CompactString::new(""));
    }

    // Fast path: ASCII only. `is_single_cell_widths` is true here, so
    // every char is one cell. On a single-byte string `cut` is also a
    // valid byte boundary; on a multi-byte single-cell string (e.g.
    // "café résumé") the index is `text_len`-clipped above, so the byte
    // slice is well-formed.
    if is_single_cell_widths(text) {
        let byte_pos = cut.min(text_len);
        return (
            CompactString::from(&text[..byte_pos]),
            CompactString::from(&text[byte_pos..]),
        );
    }

    // General case: iterate through characters by cell width.
    let mut cell_pos = 0;
    for (idx, ch) in text.char_indices() {
        let char_width = get_character_cell_size(ch);

        if cell_pos == cut {
            return (
                CompactString::from(&text[..idx]),
                CompactString::from(&text[idx..]),
            );
        } else if cell_pos + char_width > cut {
            // Double-width char straddling the cut — match the
            // un-cached `split_cells` behavior: replace the wide char
            // with a space on each side.
            let before = format!("{} ", &text[..idx]);
            let after = format!(" {}", &text[idx + ch.len_utf8()..]);
            return (CompactString::from(before), CompactString::from(after));
        }

        cell_pos += char_width;
    }

    // Shouldn't reach here for in-range cuts, but handle defensively.
    (CompactString::from(text), CompactString::new(""))
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ControlType {
    /// Emit audible bell (BEL, `\x07`).
    Bell = 1,
    /// Move cursor to the beginning of the current line.
    CarriageReturn = 2,
    /// Move cursor to the top-left corner of the terminal.
    Home = 3,
    /// Clear the entire terminal screen.
    Clear = 4,
    /// Make the terminal cursor visible.
    ShowCursor = 5,
    /// Hide the terminal cursor.
    HideCursor = 6,
    /// Switch to the alternate screen buffer.
    EnableAltScreen = 7,
    /// Return to the primary screen buffer.
    DisableAltScreen = 8,
    /// Move cursor up by a given number of rows.
    CursorUp = 9,
    /// Move cursor down by a given number of rows.
    CursorDown = 10,
    /// Move cursor forward (right) by a given number of columns.
    CursorForward = 11,
    /// Move cursor backward (left) by a given number of columns.
    CursorBackward = 12,
    /// Move cursor to a specific column on the current line.
    CursorMoveToColumn = 13,
    /// Move cursor to an absolute (column, row) position.
    CursorMoveTo = 14,
    /// Erase content on the current line.
    EraseInLine = 15,
    /// Set the terminal window title via an OSC sequence.
    SetWindowTitle = 16,
    /// Begin synchronized output (DEC 2026).
    ///
    /// **gilt extension** — not present in Python rich's `ControlType` enum.
    BeginSync = 17,
    /// End synchronized output (DEC 2026).
    ///
    /// **gilt extension** — not present in Python rich's `ControlType` enum.
    EndSync = 18,
    /// Copy content to the system clipboard via OSC 52.
    ///
    /// **gilt extension** — not present in Python rich's `ControlType` enum.
    SetClipboard = 19,
    /// Request the current clipboard contents via OSC 52.
    ///
    /// **gilt extension** — not present in Python rich's `ControlType` enum.
    RequestClipboard = 20,
    /// Send a desktop notification via OSC 9: `ESC ] 9 ; <message> BEL`.
    ///
    /// Supported by ConEmu, Windows Terminal, and some other terminals.
    ///
    /// **gilt extension** — not present in Python rich's `ControlType` enum.
    DesktopNotification = 21,
    /// Set the taskbar progress indicator via OSC 9;4: ConEmu / Windows Terminal.
    ///
    /// Format: `ESC ] 9 ; 4 ; <state> ; <progress> BEL`
    /// where state ∈ {0=remove, 1=normal, 2=error, 3=indeterminate, 4=paused}
    /// and progress is 0–100.
    ///
    /// **gilt extension** — not present in Python rich's `ControlType` enum.
    SetTaskbarProgress = 22,
}

/// Taskbar progress state for [`ControlType::SetTaskbarProgress`] (OSC 9;4).
///
/// Maps to the integer state codes defined by ConEmu / Windows Terminal:
/// 0 = remove, 1 = normal, 2 = error, 3 = indeterminate, 4 = paused.
///
/// **gilt extension**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskbarState {
    /// Remove / hide the taskbar progress indicator (state 0).
    Remove = 0,
    /// Normal progress (state 1).
    Normal = 1,
    /// Error state (state 2).
    Error = 2,
    /// Indeterminate / busy (state 3).
    Indeterminate = 3,
    /// Paused (state 4).
    Paused = 4,
}

/// Terminal control code with optional parameters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlCode {
    /// A control code with no parameters (e.g., Bell, Clear).
    Simple(ControlType),
    /// A control code with a single integer parameter (e.g., CursorUp with row count).
    WithParam(ControlType, i32),
    /// A control code with a single string parameter (e.g., SetWindowTitle with a title).
    WithParamStr(ControlType, String),
    /// A control code with two integer parameters (e.g., CursorMoveTo with column and row).
    WithTwoParams(ControlType, i32, i32),
}

/// A segment of terminal content with text, style, and optional control codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    /// The text content of this segment.
    pub text: CompactString,
    /// Visual style. **Crate-private** as of v0.11.0 — accessed via
    /// [`Segment::style`], [`Segment::set_style`], etc. Storage type stays
    /// `Option<Style>` in alpha.2; will become `StyleId` in PR3 when the
    /// L2 interner activates. Hiding the field now decouples the storage
    /// swap from the API change.
    pub(crate) style: Option<Style>,
    /// Terminal control codes carried by this segment, or `None` for text-only segments.
    pub control: Option<Vec<ControlCode>>,
}

impl std::fmt::Display for Segment {
    /// Render the segment in a debug-friendly form.
    ///
    /// Mirrors rich's `Segment.__str__` (which falls back to the NamedTuple
    /// repr): `Segment(<text>, <style>, <control>)`. Style and control are
    /// rendered with their `Debug` impls so the output is stable, greppable,
    /// and round-trippable by reading it back as `Debug` if needed.
    ///
    /// `Display` is intentionally distinct from the derived `Debug` so
    /// `format!("{segment}")` (the conventional "show me the text" call)
    /// still works without requiring `{segment:?}` everywhere.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Segment({:?}, {:?}, {:?})",
            self.text, self.style, self.control
        )
    }
}

impl Segment {
    /// Creates a new segment with text, style, and control codes.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    /// use gilt::style::Style;
    ///
    /// let seg = Segment::new("hello", Some(Style::parse("bold")), None);
    /// assert_eq!(seg.text, "hello");
    /// assert!(!seg.is_control());
    /// ```
    pub fn new(text: &str, style: Option<Style>, control: Option<Vec<ControlCode>>) -> Self {
        Segment {
            text: CompactString::from(text),
            style,
            control,
        }
    }

    /// Creates a plain text segment with no style or control codes.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// let seg = Segment::text("hello");
    /// assert_eq!(seg.text, "hello");
    /// assert!(seg.style().is_none());
    /// ```
    pub fn text(text: &str) -> Self {
        Segment {
            text: CompactString::from(text),
            style: None,
            control: None,
        }
    }

    /// Creates a newline segment.
    pub fn line() -> Self {
        Segment::text("\n")
    }

    /// Creates a segment with text and style.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    /// use gilt::style::Style;
    ///
    /// let seg = Segment::styled("warning", Style::parse("bold yellow"));
    /// assert_eq!(seg.text, "warning");
    /// assert!(seg.style().is_some());
    /// ```
    pub fn styled(text: &str, style: Style) -> Self {
        Segment {
            text: CompactString::from(text),
            style: Some(style),
            control: None,
        }
    }

    /// Returns the cell length of this segment (0 for control segments).
    ///
    /// Double-width characters (CJK, emoji) count as 2 cells each.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// assert_eq!(Segment::text("abc").cell_length(), 3);
    /// assert_eq!(Segment::text("\u{1F4A9}").cell_length(), 2); // emoji = 2 cells
    /// ```
    pub fn cell_length(&self) -> usize {
        if self.is_control() {
            0
        } else {
            cell_len(&self.text)
        }
    }

    /// Returns true if this is a control segment.
    pub fn is_control(&self) -> bool {
        self.control.is_some()
    }

    // -- Style accessors (added in v0.11.0-alpha.2) --------------------------

    /// Borrow the segment's style, if any. Replaces direct field access in
    /// v0.11.0; the underlying storage moves to `StyleId` in PR3 without
    /// changing this signature.
    #[inline]
    pub fn style(&self) -> Option<&Style> {
        self.style.as_ref()
    }

    /// Mutable borrow of the optional style. Used by the few in-tree call
    /// sites that mutate a Segment's style after construction. Once PR3
    /// activates the interner this becomes a no-op or is removed.
    #[inline]
    pub fn style_mut(&mut self) -> &mut Option<Style> {
        &mut self.style
    }

    /// Replace the segment's style. Setter form for callers that previously
    /// did `seg.style = Some(...)`.
    #[inline]
    pub fn set_style(&mut self, style: Option<Style>) {
        self.style = style;
    }

    /// Owned-style legacy convenience.
    ///
    /// Returns the style as an owned [`Style`], substituting [`Style::null`]
    /// for the `None` case so callers that previously did
    /// `seg.style.clone().unwrap_or_else(Style::null)` collapse to a single
    /// call.
    ///
    /// **Why this collapses None and Some(null) deliberately:** the L2
    /// interner (PR3) will only have `StyleId::NULL` for both. Tests that
    /// need to distinguish them should use [`Segment::style`] which returns
    /// the borrowed `Option<&Style>` directly.
    pub fn style_owned(&self) -> Style {
        self.style.clone().unwrap_or_else(Style::null)
    }

    /// Returns true if the text is empty (for bool-like checks).
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Splits the segment at a given cell position.
    ///
    /// If the cut position falls in the middle of a double-width character,
    /// it will be replaced with spaces on both sides of the split.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// let seg = Segment::text("Hello");
    /// let (left, right) = seg.split_cells(2);
    /// assert_eq!(left.text, "He");
    /// assert_eq!(right.text, "llo");
    /// ```
    pub fn split_cells(&self, cut: usize) -> (Segment, Segment) {
        let (left_text, right_text) = cached_split_texts(&self.text, cut);
        (
            Segment::new(&left_text, self.style.clone(), None),
            Segment::new(&right_text, self.style.clone(), None),
        )
    }

    /// Applies a base style and/or post style to a list of segments.
    ///
    /// The `style` is applied *underneath* each segment's existing style (as a base),
    /// while `post_style` is applied *on top* of the result.
    /// Control segments are passed through unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    /// use gilt::style::Style;
    ///
    /// let segments = vec![Segment::text("hello")];
    /// let styled = Segment::apply_style(
    ///     &segments,
    ///     Some(Style::parse("bold")),
    ///     None,
    /// );
    /// assert!(styled[0].style().is_some());
    /// ```
    pub fn apply_style(
        segments: &[Segment],
        style: Option<Style>,
        post_style: Option<Style>,
    ) -> Vec<Segment> {
        if style.is_none() && post_style.is_none() {
            return segments.to_vec();
        }

        segments
            .iter()
            .map(|seg| {
                if seg.is_control() {
                    seg.clone()
                } else {
                    let mut new_style = seg.style.clone();

                    if let Some(ref base) = style {
                        new_style = Some(base.clone() + new_style);
                    }

                    if let Some(ref post) = post_style {
                        new_style = Some(new_style.unwrap_or_else(Style::null) + post.clone());
                    }

                    Segment::new(&seg.text, new_style, None)
                }
            })
            .collect()
    }

    /// Filters segments by control flag.
    ///
    /// Pass `true` to keep only control segments, or `false` to keep only text segments.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::{Segment, ControlCode, ControlType};
    ///
    /// let segments = vec![
    ///     Segment::text("hello"),
    ///     Segment::new("", None, Some(vec![ControlCode::Simple(ControlType::Bell)])),
    /// ];
    /// let text_only = Segment::filter_control(&segments, false);
    /// assert_eq!(text_only.len(), 1);
    /// assert_eq!(text_only[0].text, "hello");
    /// ```
    pub fn filter_control(segments: &[Segment], is_control: bool) -> Vec<Segment> {
        segments
            .iter()
            .filter(|seg| seg.is_control() == is_control)
            .cloned()
            .collect()
    }

    /// Splits segments at newline boundaries.
    ///
    /// Each `\n` in the text produces a new line. Control segments are kept
    /// with the line they appear in.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// let segments = vec![Segment::text("Hello\nWorld")];
    /// let lines = Segment::split_lines(&segments);
    /// assert_eq!(lines.len(), 2);
    /// assert_eq!(lines[0][0].text, "Hello");
    /// assert_eq!(lines[1][0].text, "World");
    /// ```
    pub fn split_lines(segments: &[Segment]) -> Vec<Vec<Segment>> {
        let mut lines = Vec::with_capacity(segments.len());
        let mut current_line = Vec::with_capacity(segments.len());

        for segment in segments {
            if segment.is_control() {
                current_line.push(segment.clone());
            } else {
                let parts: Vec<&str> = segment.text.split('\n').collect();

                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        // Commit the completed line and start a fresh one.
                        lines.push(current_line);
                        current_line = Vec::new();
                    }

                    if !part.is_empty() {
                        current_line.push(Segment::new(part, segment.style.clone(), None));
                    }
                }
            }
        }

        // Push the final partial line only when there is actual content to
        // preserve — either the current line is non-empty, or at least one
        // full line was already committed (meaning the input had content).
        // This prevents a spurious empty line for empty input or input that
        // consisted only of a trailing newline.
        if !current_line.is_empty() || !lines.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Adjusts a line to a specific cell length by cropping or padding.
    ///
    /// If the line is shorter than `length` and `pad` is `true`, space characters
    /// with the given `style` are appended. If the line is longer, it is cropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    /// use gilt::style::Style;
    ///
    /// let line = vec![Segment::text("Hi")];
    /// let padded = Segment::adjust_line_length(&line, 5, &Style::null(), true);
    /// assert_eq!(Segment::get_line_length(&padded), 5);
    /// ```
    pub fn adjust_line_length(
        line: &[Segment],
        length: usize,
        style: &Style,
        pad: bool,
    ) -> Vec<Segment> {
        let line_length = Segment::get_line_length(line);

        if line_length == length {
            return line.to_vec();
        }

        if line_length < length {
            if pad {
                let mut result = line.to_vec();
                let spaces = " ".repeat(length - line_length);
                result.push(Segment::styled(&spaces, style.clone()));
                result
            } else {
                line.to_vec()
            }
        } else {
            // Need to crop
            let mut result = Vec::new();
            let mut current_length = 0;

            for segment in line {
                if segment.is_control() {
                    result.push(segment.clone());
                    continue;
                }

                let segment_length = segment.cell_length();

                if current_length + segment_length <= length {
                    result.push(segment.clone());
                    current_length += segment_length;
                } else {
                    // This segment needs cropping
                    let remaining = length - current_length;
                    if remaining > 0 {
                        let cropped_text = set_cell_size(&segment.text, remaining);
                        result.push(Segment::new(&cropped_text, segment.style.clone(), None));
                    }
                    break;
                }
            }

            result
        }
    }

    /// Returns the total cell length of a line of segments.
    ///
    /// Control segments are excluded from the count.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// let line = vec![Segment::text("foo"), Segment::text("bar")];
    /// assert_eq!(Segment::get_line_length(&line), 6);
    /// ```
    pub fn get_line_length(line: &[Segment]) -> usize {
        line.iter()
            .filter(|seg| !seg.is_control())
            .map(|seg| seg.cell_length())
            .sum()
    }

    /// Returns the shape of multiple lines as `(max_width, height)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// let lines = vec![
    ///     vec![Segment::text("Hello")],
    ///     vec![Segment::text("World!")],
    /// ];
    /// assert_eq!(Segment::get_shape(&lines), (6, 2));
    /// ```
    pub fn get_shape(lines: &[Vec<Segment>]) -> (usize, usize) {
        let max_width = lines
            .iter()
            .map(|line| Segment::get_line_length(line))
            .max()
            .unwrap_or(0);
        let height = lines.len();
        (max_width, height)
    }

    /// Adjusts all lines to given dimensions.
    ///
    /// Each line is padded or cropped to `width`. If `height` is provided, extra
    /// blank lines are appended (or excess lines are truncated) to match.
    pub fn set_shape(
        lines: &[Vec<Segment>],
        width: usize,
        height: Option<usize>,
        style: Option<&Style>,
        _new_lines: bool,
    ) -> Vec<Vec<Segment>> {
        let default_style = Style::null();
        let style = style.unwrap_or(&default_style);

        let mut shaped_lines: Vec<Vec<Segment>> = lines
            .iter()
            .map(|line| Segment::adjust_line_length(line, width, style, true))
            .collect();

        if let Some(target_height) = height {
            if shaped_lines.len() < target_height {
                let empty_line = vec![Segment::styled(&" ".repeat(width), style.clone())];
                while shaped_lines.len() < target_height {
                    shaped_lines.push(empty_line.clone());
                }
            } else if shaped_lines.len() > target_height {
                shaped_lines.truncate(target_height);
            }
        }

        shaped_lines
    }

    /// Merges consecutive segments with the same style.
    ///
    /// Adjacent non-control segments that share identical style and control values
    /// are concatenated into a single segment, reducing allocation overhead.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// let segments = vec![
    ///     Segment::text("Hello"),
    ///     Segment::text(" "),
    ///     Segment::text("World!"),
    /// ];
    /// let simplified = Segment::simplify(&segments);
    /// assert_eq!(simplified.len(), 1);
    /// assert_eq!(simplified[0].text, "Hello World!");
    /// ```
    pub fn simplify(segments: &[Segment]) -> Vec<Segment> {
        if segments.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut current = segments[0].clone();

        for segment in &segments[1..] {
            if !current.is_control()
                && !segment.is_control()
                && current.style == segment.style
                && current.control == segment.control
            {
                current.text.push_str(&segment.text);
            } else {
                result.push(current);
                current = segment.clone();
            }
        }

        result.push(current);
        result
    }

    /// Removes hyperlink metadata from segment styles, preserving all other attributes.
    pub fn strip_links(segments: &[Segment]) -> Vec<Segment> {
        segments
            .iter()
            .map(|seg| {
                if let Some(ref style) = seg.style {
                    if style.link().is_some() {
                        let new_style = style.update_link(None);
                        return Segment::new(&seg.text, Some(new_style), seg.control.clone());
                    }
                }
                seg.clone()
            })
            .collect()
    }

    /// Removes all styles from segments, leaving plain text.
    pub fn strip_styles(segments: &[Segment]) -> Vec<Segment> {
        segments
            .iter()
            .map(|seg| Segment::new(&seg.text, None, seg.control.clone()))
            .collect()
    }

    /// Removes foreground and background colors from segment styles while preserving
    /// other attributes such as bold, italic, and underline.
    pub fn remove_color(segments: &[Segment]) -> Vec<Segment> {
        segments
            .iter()
            .map(|seg| {
                if let Some(ref style) = seg.style {
                    let new_style = style.without_color();
                    Segment::new(&seg.text, Some(new_style), seg.control.clone())
                } else {
                    seg.clone()
                }
            })
            .collect()
    }

    /// Divides segments into portions at given cell positions.
    ///
    /// Each value in `cuts` specifies a cumulative cell offset where the segment
    /// list should be split. Returns one `Vec<Segment>` per cut.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// let segments = vec![Segment::text("ABCDE")];
    /// let parts = Segment::divide(&segments, &[2, 5]);
    /// assert_eq!(parts[0][0].text, "AB");
    /// assert_eq!(parts[1][0].text, "CDE");
    /// ```
    pub fn divide(segments: &[Segment], cuts: &[usize]) -> Vec<Vec<Segment>> {
        if cuts.is_empty() {
            return Vec::new();
        }

        if segments.is_empty() {
            return vec![vec![]; cuts.len()];
        }

        let mut result = Vec::with_capacity(cuts.len());
        let mut current_portion = Vec::with_capacity(segments.len());
        let mut cell_position = 0usize;
        let mut cut_index = 0usize;
        let mut seg_idx = 0usize;
        // When a segment must be split at a cut boundary, the "after" half is
        // stored here so we do not need to clone the entire input slice. Only
        // this one owned Segment is ever allocated in the hot path.
        let mut split_remainder: Option<Segment> = None;

        while cut_index < cuts.len() {
            let cut = cuts[cut_index];

            loop {
                // Prefer the remainder from the previous split, then index the
                // original slice — neither requires cloning the whole input.
                let seg: &Segment = if let Some(ref rem) = split_remainder {
                    rem
                } else if seg_idx < segments.len() {
                    &segments[seg_idx]
                } else {
                    break;
                };

                if cell_position >= cut {
                    break;
                }

                if seg.is_control() {
                    current_portion.push(seg.clone());
                    split_remainder = None;
                    seg_idx += 1;
                    continue;
                }

                let segment_length = seg.cell_length();
                let segment_end = cell_position + segment_length;

                if segment_end <= cut {
                    // Entire segment fits in current portion.
                    current_portion.push(seg.clone());
                    cell_position = segment_end;
                    split_remainder = None;
                    seg_idx += 1;
                } else {
                    // Need to split this segment at the cut boundary.
                    let offset = cut - cell_position;
                    let (before, after) = seg.split_cells(offset);

                    if !before.is_empty() {
                        current_portion.push(before);
                    }

                    // Store the "after" half; advance past split_remainder on
                    // the next iteration.
                    if !after.is_empty() {
                        split_remainder = Some(after);
                    } else {
                        split_remainder = None;
                        seg_idx += 1;
                    }

                    cell_position = cut;
                    break;
                }
            }

            result.push(current_portion);
            current_portion = Vec::with_capacity(segments.len().saturating_sub(seg_idx));
            cut_index += 1;
        }

        result
    }

    /// Aligns lines to the top of a given height, padding with blank lines below.
    pub fn align_top(
        lines: &[Vec<Segment>],
        width: usize,
        height: usize,
        style: &Style,
        new_lines: bool,
    ) -> Vec<Vec<Segment>> {
        Segment::set_shape(lines, width, Some(height), Some(style), new_lines)
    }

    /// Aligns lines to the bottom of a given height, padding with blank lines above.
    pub fn align_bottom(
        lines: &[Vec<Segment>],
        width: usize,
        height: usize,
        style: &Style,
        new_lines: bool,
    ) -> Vec<Vec<Segment>> {
        let mut shaped = Segment::set_shape(lines, width, Some(height), Some(style), new_lines);

        if lines.len() < height {
            let padding = height - lines.len();
            let empty_line = vec![Segment::styled(&" ".repeat(width), style.clone())];
            let mut padding_lines = vec![empty_line; padding];
            padding_lines.extend(
                lines
                    .iter()
                    .map(|line| Segment::adjust_line_length(line, width, style, true)),
            );
            shaped = padding_lines;
        }

        shaped
    }

    /// Aligns lines vertically centered within a given height, padding equally above and below.
    pub fn align_middle(
        lines: &[Vec<Segment>],
        width: usize,
        height: usize,
        style: &Style,
        new_lines: bool,
    ) -> Vec<Vec<Segment>> {
        if lines.len() >= height {
            return Segment::set_shape(lines, width, Some(height), Some(style), new_lines);
        }

        let padding = height - lines.len();
        let top_padding = padding / 2;
        let bottom_padding = padding - top_padding;

        let empty_line = vec![Segment::styled(&" ".repeat(width), style.clone())];
        let mut result = vec![empty_line.clone(); top_padding];

        for line in lines {
            result.push(Segment::adjust_line_length(line, width, style, true));
        }

        for _ in 0..bottom_padding {
            result.push(empty_line.clone());
        }

        result
    }

    /// Split segments into lines on newlines, then adjust each line to the given width.
    ///
    /// Port of the `Segment.split_and_crop_lines`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// let segments = vec![Segment::text("Hello\nWorld")];
    /// let lines = Segment::split_and_crop_lines(&segments, 10, None, true, false);
    /// assert_eq!(lines.len(), 2);
    /// assert_eq!(Segment::get_line_length(&lines[0]), 10);
    /// ```
    pub fn split_and_crop_lines(
        segments: &[Segment],
        length: usize,
        style: Option<&Style>,
        pad: bool,
        include_new_lines: bool,
    ) -> Vec<Vec<Segment>> {
        let mut result = Vec::with_capacity(segments.len());
        let mut line: Vec<Segment> = Vec::with_capacity(segments.len());

        for segment in segments {
            if segment.text.contains('\n') && segment.control.is_none() {
                let seg_style = segment.style.clone();
                let mut remaining = segment.text.as_str();
                while !remaining.is_empty() {
                    if let Some(pos) = remaining.find('\n') {
                        let before = &remaining[..pos];
                        if !before.is_empty() {
                            line.push(Segment::new(before, seg_style.clone(), None));
                        }
                        let mut cropped = Segment::adjust_line_length(
                            &line,
                            length,
                            &style.cloned().unwrap_or_else(Style::null),
                            pad,
                        );
                        if include_new_lines {
                            cropped.push(Segment::line());
                        }
                        result.push(cropped);
                        line.clear();
                        remaining = &remaining[pos + 1..];
                    } else {
                        if !remaining.is_empty() {
                            line.push(Segment::new(remaining, seg_style.clone(), None));
                        }
                        break;
                    }
                }
            } else {
                line.push(segment.clone());
            }
        }
        if !line.is_empty() {
            let cropped = Segment::adjust_line_length(
                &line,
                length,
                &style.cloned().unwrap_or_else(Style::null),
                pad,
            );
            result.push(cropped);
        }
        result
    }

    /// Split segments into lines, returning each line with a boolean indicating
    /// whether it was terminated by a newline character.
    ///
    /// Port of the `Segment.split_lines_terminator`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::segment::Segment;
    ///
    /// let segments = vec![Segment::text("Hello\nWorld")];
    /// let lines = Segment::split_lines_terminator(&segments);
    /// assert_eq!(lines[0].1, true);  // "Hello" was followed by \n
    /// assert_eq!(lines[1].1, false); // "World" was not
    /// ```
    pub fn split_lines_terminator(segments: &[Segment]) -> Vec<(Vec<Segment>, bool)> {
        let mut result = Vec::new();
        let mut line: Vec<Segment> = Vec::new();

        for segment in segments {
            if segment.text.contains('\n') && segment.control.is_none() {
                let seg_style = segment.style.clone();
                let mut remaining = segment.text.as_str();
                while !remaining.is_empty() {
                    if let Some(pos) = remaining.find('\n') {
                        let before = &remaining[..pos];
                        if !before.is_empty() {
                            line.push(Segment::new(before, seg_style.clone(), None));
                        }
                        result.push((std::mem::take(&mut line), true));
                        remaining = &remaining[pos + 1..];
                    } else {
                        if !remaining.is_empty() {
                            line.push(Segment::new(remaining, seg_style.clone(), None));
                        }
                        break;
                    }
                }
            } else {
                line.push(segment.clone());
            }
        }
        if !line.is_empty() {
            result.push((line, false));
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Renderable wrapper types (Phase 7 parity)
// ---------------------------------------------------------------------------

/// Renderable wrapper around a flat list of [`Segment`]s.
///
/// Mirrors rich's `Segments` helper class — exposes an existing segment list
/// through the [`Renderable`] trait so it can flow through the same console
/// pipeline as a hand-written renderable.
///
/// [`Renderable`]: crate::console::Renderable
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Segments(pub Vec<Segment>);

/// Renderable wrapper around pre-split lines of [`Segment`]s.
///
/// Mirrors rich's `SegmentLines`. When rendered, each inner line is emitted in
/// order, separated by a single `Segment::line()` (i.e. `\n`) so a downstream
/// consumer gets one continuous segment stream. An empty input renders to an
/// empty output — no stray trailing newline.
///
/// [`Renderable`]: crate::console::Renderable
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SegmentLines(pub Vec<Vec<Segment>>);

impl crate::console::Renderable for Segments {
    fn gilt_console(
        &self,
        _console: &crate::console::Console,
        _options: &crate::console::ConsoleOptions,
    ) -> Vec<Segment> {
        self.0.clone()
    }
}

impl crate::console::Renderable for SegmentLines {
    fn gilt_console(
        &self,
        _console: &crate::console::Console,
        _options: &crate::console::ConsoleOptions,
    ) -> Vec<Segment> {
        let mut out: Vec<Segment> = Vec::with_capacity(self.0.iter().map(|l| l.len() + 1).sum());
        let mut first = true;
        for line in &self.0 {
            if !first {
                out.push(Segment::line());
            }
            out.extend(line.iter().cloned());
            first = false;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;

    #[test]
    fn test_line() {
        assert_eq!(Segment::line(), Segment::text("\n"));
    }

    #[test]
    fn test_apply_style() {
        let segments = vec![
            Segment::text("foo"),
            Segment::styled("bar", Style::parse("bold")),
        ];
        let result = Segment::apply_style(&segments, Some(Style::parse("italic")), None);
        assert_eq!(
            result,
            vec![
                Segment::styled("foo", Style::parse("italic")),
                Segment::styled("bar", Style::parse("italic bold")),
            ]
        );
    }

    #[test]
    fn test_split_lines() {
        let lines = vec![Segment::text("Hello\nWorld")];
        let result = Segment::split_lines(&lines);
        assert_eq!(
            result,
            vec![vec![Segment::text("Hello")], vec![Segment::text("World")]]
        );
    }

    #[test]
    fn test_adjust_line_length_pad() {
        let line = vec![Segment::text("Hello")];
        let style = Style::parse("red");
        let result = Segment::adjust_line_length(&line, 10, &style, true);
        assert_eq!(Segment::get_line_length(&result), 10);
    }

    #[test]
    fn test_adjust_line_length_crop() {
        let line = vec![Segment::text("H"), Segment::text("ello, World!")];
        let result = Segment::adjust_line_length(&line, 5, &Style::null(), true);
        assert_eq!(Segment::get_line_length(&result), 5);
    }

    #[test]
    fn test_get_line_length() {
        assert_eq!(
            Segment::get_line_length(&[Segment::text("foo"), Segment::text("bar")]),
            6
        );
    }

    #[test]
    fn test_get_shape() {
        assert_eq!(Segment::get_shape(&[vec![Segment::text("Hello")]]), (5, 1));
        assert_eq!(
            Segment::get_shape(&[vec![Segment::text("Hello")], vec![Segment::text("World!")]]),
            (6, 2)
        );
    }

    #[test]
    fn test_simplify() {
        let segments = vec![
            Segment::text("Hello"),
            Segment::text(" "),
            Segment::text("World!"),
        ];
        assert_eq!(
            Segment::simplify(&segments),
            vec![Segment::text("Hello World!")]
        );
    }

    #[test]
    fn test_filter_control() {
        let control_code = vec![ControlCode::WithParam(ControlType::Home, 0)];
        let segments = vec![
            Segment::text("foo"),
            Segment::new("bar", None, Some(control_code.clone())),
        ];
        assert_eq!(
            Segment::filter_control(&segments, false),
            vec![Segment::text("foo")]
        );
    }

    #[test]
    fn test_strip_styles() {
        let segments = vec![Segment::styled("foo", Style::parse("bold"))];
        assert_eq!(Segment::strip_styles(&segments), vec![Segment::text("foo")]);
    }

    #[test]
    fn test_strip_links() {
        let segments = vec![Segment::styled(
            "foo",
            Style::parse("bold link https://www.example.org"),
        )];
        let result = Segment::strip_links(&segments);
        assert_eq!(result[0].style.as_ref().unwrap().link(), None);
        assert_eq!(result[0].style.as_ref().unwrap().bold(), Some(true));
    }

    #[test]
    fn test_remove_color() {
        let segments = vec![
            Segment::styled("foo", Style::parse("bold red")),
            Segment::text("bar"),
        ];
        let result = Segment::remove_color(&segments);
        assert_eq!(result[0].style.as_ref().unwrap().color(), None);
        assert_eq!(result[0].style.as_ref().unwrap().bold(), Some(true));
    }

    #[test]
    fn test_is_control() {
        assert!(!Segment::text("foo").is_control());
        assert!(Segment::new("foo", None, Some(vec![])).is_control());
    }

    #[test]
    fn test_divide() {
        let bold = Style::parse("bold");
        let italic = Style::parse("italic");
        let segments = vec![
            Segment::styled("Hello", bold.clone()),
            Segment::styled(" World!", italic.clone()),
        ];
        assert_eq!(Segment::divide(&segments, &[]), Vec::<Vec<Segment>>::new());
        assert_eq!(Segment::divide(&[], &[1]), vec![vec![]]);
        assert_eq!(
            Segment::divide(&segments, &[1]),
            vec![vec![Segment::styled("H", bold.clone())]]
        );
        assert_eq!(
            Segment::divide(&segments, &[4, 20]),
            vec![
                vec![Segment::styled("Hell", bold.clone())],
                vec![
                    Segment::styled("o", bold.clone()),
                    Segment::styled(" World!", italic.clone())
                ],
            ]
        );
    }

    #[test]
    fn test_split_cells_emoji() {
        let segment = Segment::text("💩");
        let (before, after) = segment.split_cells(1);
        assert_eq!(before.text, " ");
        assert_eq!(after.text, " ");
    }

    #[test]
    fn test_split_cells_ascii() {
        let segment = Segment::text("XY");
        let (before, after) = segment.split_cells(1);
        assert_eq!(before.text, "X");
        assert_eq!(after.text, "Y");
    }

    #[test]
    fn test_split_cells_mixed() {
        let segment = Segment::text("X💩Y");
        let (before, after) = segment.split_cells(2);
        assert_eq!(before.text, "X ");
        assert_eq!(after.text, " Y");
    }

    #[test]
    fn test_align_top() {
        let lines = vec![vec![Segment::text("X")]];
        assert_eq!(
            Segment::align_top(&lines, 3, 1, &Style::null(), false),
            Segment::set_shape(&lines, 3, Some(1), Some(&Style::null()), false)
        );
        assert_eq!(
            Segment::align_top(&lines, 3, 3, &Style::null(), false).len(),
            3
        );
    }

    #[test]
    fn test_align_middle() {
        let lines = vec![vec![Segment::text("X")]];
        let result = Segment::align_middle(&lines, 5, 3, &Style::null(), false);
        assert_eq!(result.len(), 3);
        // Middle alignment: 1 padding top, 1 content, 1 padding bottom
        assert_eq!(Segment::get_line_length(&result[0]), 5); // padding
        assert_eq!(Segment::get_line_length(&result[1]), 5); // content padded
        assert_eq!(Segment::get_line_length(&result[2]), 5); // padding
    }

    #[test]
    fn test_align_bottom() {
        let lines = vec![vec![Segment::text("X")]];
        let result = Segment::align_bottom(&lines, 5, 3, &Style::null(), false);
        assert_eq!(result.len(), 3);
        // Bottom alignment: 2 padding, then content
        assert_eq!(Segment::get_line_length(&result[0]), 5); // padding
        assert_eq!(Segment::get_line_length(&result[1]), 5); // padding
        assert_eq!(Segment::get_line_length(&result[2]), 5); // content padded
    }

    #[test]
    fn test_set_shape() {
        let result = Segment::set_shape(&[vec![Segment::text("Hello")]], 10, None, None, false);
        assert_eq!(Segment::get_line_length(&result[0]), 10);
    }

    #[test]
    fn test_cell_length() {
        assert_eq!(Segment::text("abc").cell_length(), 3);
        assert_eq!(Segment::text("💩").cell_length(), 2);
        assert_eq!(
            Segment::new(
                "abc",
                None,
                Some(vec![ControlCode::Simple(ControlType::Bell)])
            )
            .cell_length(),
            0
        );
    }

    #[test]
    fn test_split_lines_multiple_newlines() {
        let segments = vec![Segment::text("Hello\n\nWorld")];
        let result = Segment::split_lines(&segments);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec![Segment::text("Hello")]);
        assert_eq!(result[1], Vec::<Segment>::new());
        assert_eq!(result[2], vec![Segment::text("World")]);
    }

    #[test]
    fn test_split_lines_trailing_newline() {
        let segments = vec![Segment::text("Hello\n")];
        let result = Segment::split_lines(&segments);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec![Segment::text("Hello")]);
        assert_eq!(result[1], Vec::<Segment>::new());
    }

    #[test]
    fn test_simplify_different_styles() {
        let segments = vec![
            Segment::styled("Hello", Style::parse("bold")),
            Segment::styled("World", Style::parse("italic")),
        ];
        let result = Segment::simplify(&segments);
        assert_eq!(result.len(), 2); // Should not merge
    }

    #[test]
    fn test_simplify_with_control() {
        let segments = vec![
            Segment::text("Hello"),
            Segment::new("", None, Some(vec![ControlCode::Simple(ControlType::Bell)])),
            Segment::text("World"),
        ];
        let result = Segment::simplify(&segments);
        assert_eq!(result.len(), 3); // Control segments should not be merged
    }

    #[test]
    fn test_divide_empty_segments() {
        let result = Segment::divide(&[], &[1, 2, 3]);
        assert_eq!(result.len(), 3);
        assert!(result[0].is_empty());
        assert!(result[1].is_empty());
        assert!(result[2].is_empty());
    }

    #[test]
    fn test_split_cells_beyond_length() {
        let segment = Segment::text("Hello");
        let (before, after) = segment.split_cells(10);
        assert_eq!(before.text, "Hello");
        assert_eq!(after.text, "");
    }

    #[test]
    fn test_split_cells_cjk() {
        let segment = Segment::text("あいう"); // 6 cells total
        let (before, after) = segment.split_cells(2);
        assert_eq!(before.text, "あ");
        assert_eq!(after.text, "いう");

        let (before, after) = segment.split_cells(3);
        // Split in middle of い - should get spaces
        assert_eq!(before.text, "あ ");
        assert_eq!(after.text, " う");
    }

    #[test]
    fn test_apply_style_with_control_segments() {
        let control_code = vec![ControlCode::Simple(ControlType::Bell)];
        let segments = vec![
            Segment::text("foo"),
            Segment::new("", None, Some(control_code.clone())),
            Segment::text("bar"),
        ];
        let result = Segment::apply_style(&segments, Some(Style::parse("bold")), None);

        assert_eq!(result[0].style.as_ref().unwrap().bold(), Some(true));
        assert!(result[1].is_control());
        assert_eq!(result[1].style, None);
        assert_eq!(result[2].style.as_ref().unwrap().bold(), Some(true));
    }

    #[test]
    fn test_apply_style_post_style() {
        let segments = vec![Segment::styled("foo", Style::parse("bold"))];
        let result = Segment::apply_style(&segments, None, Some(Style::parse("italic")));
        assert_eq!(result[0].style.as_ref().unwrap().bold(), Some(true));
        assert_eq!(result[0].style.as_ref().unwrap().italic(), Some(true));
    }

    #[test]
    fn test_get_shape_empty() {
        assert_eq!(Segment::get_shape(&[]), (0, 0));
        assert_eq!(Segment::get_shape(&[vec![]]), (0, 1));
    }

    #[test]
    fn test_adjust_line_length_exact() {
        let line = vec![Segment::text("Hello")];
        let result = Segment::adjust_line_length(&line, 5, &Style::null(), true);
        assert_eq!(result, line);
    }

    #[test]
    fn test_adjust_line_length_no_pad() {
        let line = vec![Segment::text("Hi")];
        let result = Segment::adjust_line_length(&line, 10, &Style::null(), false);
        assert_eq!(Segment::get_line_length(&result), 2); // Should not pad
    }

    #[test]
    fn test_divide_with_control_segments() {
        let control_code = vec![ControlCode::Simple(ControlType::Bell)];
        let segments = vec![
            Segment::text("Hello"),
            Segment::new("", None, Some(control_code.clone())),
            Segment::text("World"),
        ];
        let result = Segment::divide(&segments, &[5, 10]);
        assert_eq!(result.len(), 2);
        // First portion should have "Hello" (5 cells)
        assert_eq!(result[0].len(), 1);
        assert_eq!(result[0][0].text, "Hello");
    }

    #[test]
    fn test_split_cells_zero_cut() {
        let segment = Segment::text("Hello");
        let (before, after) = segment.split_cells(0);
        assert_eq!(before.text, "");
        assert_eq!(after.text, "Hello");
    }

    #[test]
    fn test_align_methods_preserve_content() {
        let lines = vec![vec![Segment::text("ABC")]];
        let width = 5;
        let height = 3;

        let top = Segment::align_top(&lines, width, height, &Style::null(), false);
        let middle = Segment::align_middle(&lines, width, height, &Style::null(), false);
        let bottom = Segment::align_bottom(&lines, width, height, &Style::null(), false);

        // All should have same height
        assert_eq!(top.len(), height);
        assert_eq!(middle.len(), height);
        assert_eq!(bottom.len(), height);

        // All should preserve the content somewhere
        assert!(top
            .iter()
            .any(|line| { line.iter().any(|seg| seg.text.contains("ABC")) }));
        assert!(middle
            .iter()
            .any(|line| { line.iter().any(|seg| seg.text.contains("ABC")) }));
        assert!(bottom
            .iter()
            .any(|line| { line.iter().any(|seg| seg.text.contains("ABC")) }));
    }

    #[test]
    fn test_cell_length_with_mixed_content() {
        assert_eq!(Segment::text("a💩b").cell_length(), 4); // 1 + 2 + 1
        assert_eq!(Segment::text("あa").cell_length(), 3); // 2 + 1
    }

    #[test]
    fn test_simplify_empty_segments() {
        let segments = vec![Segment::text(""), Segment::text("Hello"), Segment::text("")];
        let result = Segment::simplify(&segments);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "Hello");
    }

    #[test]
    fn test_apply_style_none_params() {
        let segments = vec![Segment::text("foo")];
        let result = Segment::apply_style(&segments, None, None);
        assert_eq!(result, segments);
    }

    #[test]
    fn test_set_shape_with_height() {
        let lines = vec![vec![Segment::text("A")], vec![Segment::text("B")]];
        let result = Segment::set_shape(&lines, 3, Some(4), Some(&Style::null()), false);
        assert_eq!(result.len(), 4);
        assert_eq!(Segment::get_line_length(&result[0]), 3);
        assert_eq!(Segment::get_line_length(&result[3]), 3);
    }

    #[test]
    fn test_set_shape_truncate() {
        let lines = vec![
            vec![Segment::text("A")],
            vec![Segment::text("B")],
            vec![Segment::text("C")],
        ];
        let result = Segment::set_shape(&lines, 3, Some(2), Some(&Style::null()), false);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_split_lines_with_styled_segments() {
        let bold = Style::parse("bold");
        let segments = vec![Segment::styled("Hello\nWorld", bold.clone())];
        let result = Segment::split_lines(&segments);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0].text, "Hello");
        assert_eq!(result[1][0].text, "World");
        // Style should be preserved
        assert_eq!(result[0][0].style.as_ref().unwrap().bold(), Some(true));
        assert_eq!(result[1][0].style.as_ref().unwrap().bold(), Some(true));
    }

    #[test]
    fn test_divide_exact_boundaries() {
        let segments = vec![Segment::text("ABCDE")];
        let result = Segment::divide(&segments, &[2, 4]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0].text, "AB");
        assert_eq!(result[1][0].text, "CD");
    }

    #[test]
    fn test_is_empty() {
        assert!(Segment::text("").is_empty());
        assert!(!Segment::text("a").is_empty());
    }

    #[test]
    fn test_control_types_coverage() {
        // Test that we can create all control types
        let _ = ControlCode::Simple(ControlType::Bell);
        let _ = ControlCode::WithParam(ControlType::CursorUp, 5);
        let _ = ControlCode::WithParamStr(ControlType::SetWindowTitle, "Test".to_string());
        let _ = ControlCode::WithTwoParams(ControlType::CursorMoveTo, 10, 20);

        // Verify control types are distinct
        assert_ne!(ControlType::Bell as u8, ControlType::Home as u8);
        assert_ne!(ControlType::ShowCursor as u8, ControlType::HideCursor as u8);
    }

    #[test]
    fn test_split_and_crop_lines_basic() {
        let segments = vec![Segment::text("Hello\nWorld")];
        let lines = Segment::split_and_crop_lines(&segments, 10, None, true, false);
        assert_eq!(lines.len(), 2);
        // First line should be "Hello" padded to 10
        let line0_text: String = lines[0].iter().map(|s| s.text.as_str()).collect();
        assert_eq!(line0_text.trim_end(), "Hello");
        assert_eq!(lines[0].iter().map(|s| s.cell_length()).sum::<usize>(), 10);
    }

    #[test]
    fn test_split_and_crop_lines_no_pad() {
        let segments = vec![Segment::text("Hi\nWorld")];
        let lines = Segment::split_and_crop_lines(&segments, 10, None, false, false);
        assert_eq!(lines.len(), 2);
        let line0_text: String = lines[0].iter().map(|s| s.text.as_str()).collect();
        assert_eq!(line0_text, "Hi");
    }

    #[test]
    fn test_split_and_crop_lines_with_newline_segments() {
        let segments = vec![Segment::text("Hello\nWorld")];
        let lines = Segment::split_and_crop_lines(&segments, 10, None, false, true);
        assert_eq!(lines.len(), 2);
        // Each line should end with a newline segment
        assert_eq!(lines[0].last().unwrap().text, "\n");
    }

    #[test]
    fn test_split_and_crop_lines_crop() {
        let segments = vec![Segment::text("Hello, World!")];
        let lines = Segment::split_and_crop_lines(&segments, 5, None, false, false);
        assert_eq!(lines.len(), 1);
        let line_text: String = lines[0].iter().map(|s| s.text.as_str()).collect();
        assert_eq!(line_text, "Hello");
    }

    #[test]
    fn test_split_lines_terminator_basic() {
        let segments = vec![Segment::text("Hello\nWorld")];
        let lines = Segment::split_lines_terminator(&segments);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].1); // first line has terminator
        assert!(!lines[1].1); // last line doesn\'t
        let text0: String = lines[0].0.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(text0, "Hello");
    }

    #[test]
    fn test_split_lines_terminator_no_newline() {
        let segments = vec![Segment::text("Hello")];
        let lines = Segment::split_lines_terminator(&segments);
        assert_eq!(lines.len(), 1);
        assert!(!lines[0].1);
    }

    #[test]
    fn test_split_lines_terminator_trailing_newline() {
        let segments = vec![Segment::text("Hello\n")];
        let lines = Segment::split_lines_terminator(&segments);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].1);
    }
    // -- Segments / SegmentLines renderable wrappers ------------------------

    #[test]
    fn test_segments_renders_as_is() {
        use crate::console::Renderable;

        let console = Console::new();
        let options = console.options();
        let inner = vec![Segment::new("hi", None, None)];
        let wrapper = Segments(inner.clone());
        let rendered = wrapper.gilt_console(&console, &options);
        assert_eq!(rendered, inner);
    }

    #[test]
    fn test_segment_lines_flattens_with_newlines() {
        use crate::console::Renderable;

        let console = Console::new();
        let options = console.options();
        let a = Segment::new("a", None, None);
        let b = Segment::new("b", None, None);
        let wrapper = SegmentLines(vec![vec![a.clone()], vec![b.clone()]]);
        let rendered = wrapper.gilt_console(&console, &options);
        assert_eq!(rendered, vec![a, Segment::line(), b],);
    }

    #[test]
    fn test_segment_lines_empty() {
        use crate::console::Renderable;

        let console = Console::new();
        let options = console.options();
        let wrapper = SegmentLines(Vec::new());
        let rendered = wrapper.gilt_console(&console, &options);
        assert!(rendered.is_empty());
    }

    // -- split_cells cache (Phase 7) -----------------------------------------

    #[test]
    fn test_split_cells_cache_repeat_same_input() {
        // Cache correctness: calling the split fn twice with the same input
        // must return equal results. The un-cached function already satisfies
        // this; the LRU cache must not break it.
        let seg = Segment::text("Hello, World!");
        let (a1, a2) = seg.split_cells(5);
        let (b1, b2) = seg.split_cells(5);
        assert_eq!(a1.text, b1.text);
        assert_eq!(a2.text, b2.text);
    }

    #[test]
    fn test_split_cells_cache_third_distinct_input() {
        // A third distinct input must also be correct after the cache has
        // been populated by prior lookups.
        let seg_a = Segment::text("Hello, World!");
        let _ = seg_a.split_cells(5);
        let seg_b = Segment::text("foo bar baz");
        let _ = seg_b.split_cells(3);
        let seg_c = Segment::text("ABCDE");
        let (c1, c2) = seg_c.split_cells(3);
        assert_eq!(c1.text, "ABC");
        assert_eq!(c2.text, "DE");
    }

    #[test]
    fn test_split_cells_cache_double_width_boundary() {
        // Exercise the double-width boundary path (CJK) with repeated calls.
        // '中' is two cells; splitting at 2 must keep it on the left.
        let seg = Segment::text("中文A");
        let (a1, a2) = seg.split_cells(2);
        let (b1, b2) = seg.split_cells(2);
        assert_eq!(a1.text, b1.text);
        assert_eq!(a2.text, b2.text);
        assert_eq!(a1.text, "中");
        assert_eq!(a2.text, "文A");
    }

    #[test]
    fn test_split_cells_cache_populates() {
        // Verify the LRU cache is actually populated by `split_cells`. The
        // first call must add an entry; a second call with the same key
        // must not grow the cache.
        split_cells_cache_clear();
        let seg = Segment::text("cache populate test");
        let _ = seg.split_cells(4);
        assert!(
            split_cells_cache_len() >= 1,
            "cache should have at least one entry after a split_cells call"
        );
        let before = split_cells_cache_len();
        let _ = seg.split_cells(4);
        assert_eq!(
            split_cells_cache_len(),
            before,
            "repeat call must hit, not grow, the cache"
        );
    }

    // -- Display impl for Segment (Phase 7) ----------------------------------

    #[test]
    fn test_segment_display_contains_text() {
        let s = format!("{}", Segment::new("x", None, None));
        assert!(
            s.contains('x'),
            "Display must contain the segment text; got: {s:?}"
        );
    }

    #[test]
    fn test_segment_display_empty_text() {
        let s = format!("{}", Segment::new("", None, None));
        // An empty-text segment still must produce a Display representation
        // that is recognizably a Segment — the format string must include
        // the "Segment(" prefix even when there is no text.
        assert!(
            s.contains("Segment("),
            "Display must include the Segment() prefix; got: {s:?}"
        );
    }

    #[test]
    fn test_segment_display_with_style() {
        let s = format!("{}", Segment::styled("hi", Style::parse("bold")));
        // Should include the text and indicate this is a Segment.
        assert!(s.contains("hi"));
        assert!(s.contains("Segment("));
    }
}
