//! Segment - the atomic unit of terminal rendering.
//!
//! All content flows through segments, which combine text, style, and control codes.

use crate::cells::{cell_len, get_character_cell_size, is_single_cell_widths, set_cell_size};
use crate::style::Style;

/// Terminal control code types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ControlType {
    Bell = 1,
    CarriageReturn = 2,
    Home = 3,
    Clear = 4,
    ShowCursor = 5,
    HideCursor = 6,
    EnableAltScreen = 7,
    DisableAltScreen = 8,
    CursorUp = 9,
    CursorDown = 10,
    CursorForward = 11,
    CursorBackward = 12,
    CursorMoveToColumn = 13,
    CursorMoveTo = 14,
    EraseInLine = 15,
    SetWindowTitle = 16,
    BeginSync = 17,
    EndSync = 18,
    SetClipboard = 19,
    RequestClipboard = 20,
}

/// Terminal control code with optional parameters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ControlCode {
    Simple(ControlType),
    WithParam(ControlType, i32),
    WithParamStr(ControlType, String),
    WithTwoParams(ControlType, i32, i32),
}

/// A segment of terminal content with text, style, and optional control codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub style: Option<Style>,
    pub control: Option<Vec<ControlCode>>,
}

impl Segment {
    /// Creates a new segment with text, style, and control codes.
    pub fn new(text: &str, style: Option<Style>, control: Option<Vec<ControlCode>>) -> Self {
        Segment {
            text: text.to_string(),
            style,
            control,
        }
    }

    /// Creates a plain text segment with no style or control codes.
    pub fn text(text: &str) -> Self {
        Segment {
            text: text.to_string(),
            style: None,
            control: None,
        }
    }

    /// Creates a newline segment.
    pub fn line() -> Self {
        Segment::text("\n")
    }

    /// Creates a segment with text and style.
    pub fn styled(text: &str, style: Style) -> Self {
        Segment {
            text: text.to_string(),
            style: Some(style),
            control: None,
        }
    }

    /// Returns the cell length of this segment (0 for control segments).
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

    /// Returns true if the text is empty (for bool-like checks).
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Splits the segment at a given cell position.
    ///
    /// If the cut position falls in the middle of a double-width character,
    /// it will be replaced with spaces on both sides of the split.
    pub fn split_cells(&self, cut: usize) -> (Segment, Segment) {
        let text_len = self.text.len();
        let cell_length = cell_len(&self.text);

        // Fast path: if cut is beyond text length, return (self, empty)
        if cut >= cell_length {
            return (self.clone(), Segment::new("", self.style.clone(), None));
        }

        // Fast path: ASCII only
        if is_single_cell_widths(&self.text) {
            let byte_pos = cut.min(text_len);
            return (
                Segment::new(&self.text[..byte_pos], self.style.clone(), None),
                Segment::new(&self.text[byte_pos..], self.style.clone(), None),
            );
        }

        // General case: iterate through characters
        let mut cell_pos = 0;

        for (idx, ch) in self.text.char_indices() {
            let char_width = get_character_cell_size(ch);

            if cell_pos == cut {
                // Exact match
                return (
                    Segment::new(&self.text[..idx], self.style.clone(), None),
                    Segment::new(&self.text[idx..], self.style.clone(), None),
                );
            } else if cell_pos + char_width > cut {
                // Would overflow: double-width char straddling the cut
                // Replace with spaces
                let before = format!("{} ", &self.text[..idx]);
                let after = format!(" {}", &self.text[idx + ch.len_utf8()..]);
                return (
                    Segment::new(&before, self.style.clone(), None),
                    Segment::new(&after, self.style.clone(), None),
                );
            }

            cell_pos += char_width;
        }

        // Shouldn't reach here, but handle edge case
        (self.clone(), Segment::new("", self.style.clone(), None))
    }

    /// Applies a base style and/or post style to a list of segments.
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
    pub fn filter_control(segments: &[Segment], is_control: bool) -> Vec<Segment> {
        segments
            .iter()
            .filter(|seg| seg.is_control() == is_control)
            .cloned()
            .collect()
    }

    /// Splits segments at newline boundaries.
    pub fn split_lines(segments: &[Segment]) -> Vec<Vec<Segment>> {
        let mut lines = Vec::new();
        let mut current_line = Vec::new();

        for segment in segments {
            if segment.is_control() {
                current_line.push(segment.clone());
            } else {
                let parts: Vec<&str> = segment.text.split('\n').collect();

                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        lines.push(current_line);
                        current_line = Vec::new();
                    }

                    if !part.is_empty() {
                        current_line.push(Segment::new(part, segment.style.clone(), None));
                    }
                }

                // Handle trailing newline
                if segment.text.ends_with('\n') && !parts.is_empty() {
                    lines.push(current_line);
                    current_line = Vec::new();
                }
            }
        }

        if !current_line.is_empty() || lines.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Adjusts a line to a specific cell length by cropping or padding.
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
    pub fn get_line_length(line: &[Segment]) -> usize {
        line.iter()
            .filter(|seg| !seg.is_control())
            .map(|seg| seg.cell_length())
            .sum()
    }

    /// Returns the shape of multiple lines (max_width, height).
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

    /// Removes links from segment styles.
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

    /// Removes all styles from segments.
    pub fn strip_styles(segments: &[Segment]) -> Vec<Segment> {
        segments
            .iter()
            .map(|seg| Segment::new(&seg.text, None, seg.control.clone()))
            .collect()
    }

    /// Removes colors from segment styles but keeps other attributes.
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
    pub fn divide(segments: &[Segment], cuts: &[usize]) -> Vec<Vec<Segment>> {
        if cuts.is_empty() {
            return Vec::new();
        }

        if segments.is_empty() {
            return vec![vec![]; cuts.len()];
        }

        let mut result = Vec::new();
        let mut current_portion = Vec::new();
        let mut cell_position = 0;
        let mut cut_index = 0;

        // Track remaining segments to process
        let mut remaining_segments: Vec<Segment> = segments.to_vec();
        let mut seg_idx = 0;

        while cut_index < cuts.len() && seg_idx < remaining_segments.len() {
            let cut = cuts[cut_index];

            while seg_idx < remaining_segments.len() && cell_position < cut {
                let segment = &remaining_segments[seg_idx];

                if segment.is_control() {
                    current_portion.push(segment.clone());
                    seg_idx += 1;
                    continue;
                }

                let segment_length = segment.cell_length();
                let segment_end = cell_position + segment_length;

                if segment_end <= cut {
                    // Entire segment fits in current portion
                    current_portion.push(segment.clone());
                    cell_position = segment_end;
                    seg_idx += 1;
                } else {
                    // Need to split this segment
                    let offset = cut - cell_position;
                    let (before, after) = segment.split_cells(offset);

                    if !before.is_empty() {
                        current_portion.push(before);
                    }

                    // Replace current segment with the remainder
                    if !after.is_empty() {
                        remaining_segments[seg_idx] = after;
                    } else {
                        seg_idx += 1;
                    }

                    cell_position = cut;
                    break;
                }
            }

            result.push(current_portion);
            current_portion = Vec::new();
            cut_index += 1;
        }

        result
    }

    /// Aligns lines to the top of a given height.
    pub fn align_top(
        lines: &[Vec<Segment>],
        width: usize,
        height: usize,
        style: &Style,
        new_lines: bool,
    ) -> Vec<Vec<Segment>> {
        Segment::set_shape(lines, width, Some(height), Some(style), new_lines)
    }

    /// Aligns lines to the bottom of a given height.
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

    /// Aligns lines to the middle of a given height.
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
    /// Port of Python rich's `Segment.split_and_crop_lines`.
    pub fn split_and_crop_lines(
        segments: &[Segment],
        length: usize,
        style: Option<&Style>,
        pad: bool,
        include_new_lines: bool,
    ) -> Vec<Vec<Segment>> {
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
    /// Port of Python rich's `Segment.split_lines_terminator`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line() {
        assert_eq!(Segment::line(), Segment::text("\n"));
    }

    #[test]
    fn test_apply_style() {
        let segments = vec![
            Segment::text("foo"),
            Segment::styled("bar", Style::parse("bold").unwrap()),
        ];
        let result = Segment::apply_style(&segments, Some(Style::parse("italic").unwrap()), None);
        assert_eq!(
            result,
            vec![
                Segment::styled("foo", Style::parse("italic").unwrap()),
                Segment::styled("bar", Style::parse("italic bold").unwrap()),
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
        let style = Style::parse("red").unwrap();
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
        let segments = vec![Segment::styled("foo", Style::parse("bold").unwrap())];
        assert_eq!(Segment::strip_styles(&segments), vec![Segment::text("foo")]);
    }

    #[test]
    fn test_strip_links() {
        let segments = vec![Segment::styled(
            "foo",
            Style::parse("bold link https://www.example.org").unwrap(),
        )];
        let result = Segment::strip_links(&segments);
        assert_eq!(result[0].style.as_ref().unwrap().link(), None);
        assert_eq!(result[0].style.as_ref().unwrap().bold(), Some(true));
    }

    #[test]
    fn test_remove_color() {
        let segments = vec![
            Segment::styled("foo", Style::parse("bold red").unwrap()),
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
        let bold = Style::parse("bold").unwrap();
        let italic = Style::parse("italic").unwrap();
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
            Segment::styled("Hello", Style::parse("bold").unwrap()),
            Segment::styled("World", Style::parse("italic").unwrap()),
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
        let result = Segment::apply_style(&segments, Some(Style::parse("bold").unwrap()), None);

        assert_eq!(result[0].style.as_ref().unwrap().bold(), Some(true));
        assert!(result[1].is_control());
        assert_eq!(result[1].style, None);
        assert_eq!(result[2].style.as_ref().unwrap().bold(), Some(true));
    }

    #[test]
    fn test_apply_style_post_style() {
        let segments = vec![Segment::styled("foo", Style::parse("bold").unwrap())];
        let result = Segment::apply_style(&segments, None, Some(Style::parse("italic").unwrap()));
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
        let bold = Style::parse("bold").unwrap();
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
        assert_eq!(lines[0].1, true); // first line has terminator
        assert_eq!(lines[1].1, false); // last line doesn\'t
        let text0: String = lines[0].0.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(text0, "Hello");
    }

    #[test]
    fn test_split_lines_terminator_no_newline() {
        let segments = vec![Segment::text("Hello")];
        let lines = Segment::split_lines_terminator(&segments);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].1, false);
    }

    #[test]
    fn test_split_lines_terminator_trailing_newline() {
        let segments = vec![Segment::text("Hello\n")];
        let lines = Segment::split_lines_terminator(&segments);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].1, true);
    }
}
