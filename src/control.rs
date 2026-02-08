//! Control codes and sequences for terminal manipulation.
//!
//! This module provides utilities for working with terminal control codes,
//! including stripping and escaping control characters, and generating
//! ANSI control sequences for cursor movement, screen clearing, etc.

use std::fmt;

use crate::segment::{ControlCode, ControlType, Segment};

/// Base64 encoding alphabet (RFC 4648).
const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encode bytes to a base64 string (RFC 4648, with padding).
///
/// This is a minimal implementation to avoid adding an external dependency.
pub(crate) fn base64_encode(input: &[u8]) -> String {
    if input.is_empty() {
        return String::new();
    }
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        output.push(BASE64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        output.push(BASE64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            output.push(BASE64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(BASE64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

/// Characters considered "control" for stripping/escaping purposes.
/// Bell(7), Backspace(8), Vertical Tab(11), Form Feed(12), Carriage Return(13).
const CONTROL_CHARS: &[char] = &[
    '\x07', // Bell
    '\x08', // Backspace
    '\x0B', // Vertical Tab
    '\x0C', // Form Feed
    '\r',   // Carriage Return
];

/// Remove control characters (Bell, Backspace, VT, FF, CR) from text.
///
/// This is the canonical implementation. `text.rs` should delegate to this.
///
/// # Examples
/// ```
/// use gilt::control::strip_control_codes;
/// assert_eq!(strip_control_codes("foo\rbar"), "foobar");
/// assert_eq!(strip_control_codes("hello"), "hello");
/// ```
pub fn strip_control_codes(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    text.chars()
        .filter(|c| !CONTROL_CHARS.contains(c))
        .collect()
}

/// Replace control characters with their escape sequence representations.
///
/// - Bell (0x07) → `\a`
/// - Backspace (0x08) → `\b`
/// - Vertical Tab (0x0B) → `\v`
/// - Form Feed (0x0C) → `\f`
/// - Carriage Return (0x0D) → `\r`
///
/// # Examples
/// ```
/// use gilt::control::escape_control_codes;
/// assert_eq!(escape_control_codes("foo\rbar"), r"foo\rbar");
/// ```
pub fn escape_control_codes(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\x07' => result.push_str(r"\a"),
            '\x08' => result.push_str(r"\b"),
            '\x0B' => result.push_str(r"\v"),
            '\x0C' => result.push_str(r"\f"),
            '\r' => result.push_str(r"\r"),
            _ => result.push(c),
        }
    }
    result
}

/// Render a single `ControlCode` to its ANSI escape sequence string.
fn render_code(code: &ControlCode) -> String {
    match code {
        ControlCode::Simple(ct) => match ct {
            ControlType::Bell => "\x07".to_string(),
            ControlType::CarriageReturn => "\r".to_string(),
            ControlType::Home => "\x1b[H".to_string(),
            ControlType::Clear => "\x1b[2J".to_string(),
            ControlType::ShowCursor => "\x1b[?25h".to_string(),
            ControlType::HideCursor => "\x1b[?25l".to_string(),
            ControlType::EnableAltScreen => "\x1b[?1049h".to_string(),
            ControlType::DisableAltScreen => "\x1b[?1049l".to_string(),
            // Simple variants for parameterized types default to 0/empty
            ControlType::CursorUp => "\x1b[0A".to_string(),
            ControlType::CursorDown => "\x1b[0B".to_string(),
            ControlType::CursorForward => "\x1b[0C".to_string(),
            ControlType::CursorBackward => "\x1b[0D".to_string(),
            ControlType::CursorMoveToColumn => "\x1b[1G".to_string(),
            ControlType::CursorMoveTo => "\x1b[1;1H".to_string(),
            ControlType::EraseInLine => "\x1b[0K".to_string(),
            ControlType::SetWindowTitle => String::new(),
            ControlType::BeginSync => "\x1b[?2026h".to_string(),
            ControlType::EndSync => "\x1b[?2026l".to_string(),
            ControlType::SetClipboard => String::new(),
            ControlType::RequestClipboard => "\x1b]52;c;?\x07".to_string(),
        },
        ControlCode::WithParam(ct, n) => match ct {
            ControlType::CursorUp => format!("\x1b[{}A", n),
            ControlType::CursorDown => format!("\x1b[{}B", n),
            ControlType::CursorForward => format!("\x1b[{}C", n),
            ControlType::CursorBackward => format!("\x1b[{}D", n),
            ControlType::CursorMoveToColumn => format!("\x1b[{}G", n + 1), // 0-indexed to 1-indexed
            ControlType::EraseInLine => format!("\x1b[{}K", n),
            // For other types with a single param, render as best we can
            _ => render_code(&ControlCode::Simple(*ct)),
        },
        ControlCode::WithParamStr(ct, s) => match ct {
            ControlType::SetWindowTitle => format!("\x1b]0;{}\x07", s),
            ControlType::SetClipboard => format!("\x1b]52;c;{}\x07", s),
            _ => render_code(&ControlCode::Simple(*ct)),
        },
        ControlCode::WithTwoParams(ct, x, y) => match ct {
            ControlType::CursorMoveTo => format!("\x1b[{};{}H", y + 1, x + 1), // 0-indexed to 1-indexed
            _ => render_code(&ControlCode::Simple(*ct)),
        },
    }
}

/// A renderable that generates ANSI control sequences.
///
/// `Control` wraps one or more `ControlCode` values, renders them to ANSI
/// escape sequences, and stores them in a `Segment`.
pub struct Control {
    /// The segment containing the rendered ANSI escape sequence text and control metadata.
    pub segment: Segment,
}

impl Control {
    /// Create a new `Control` from a list of control codes.
    ///
    /// Each code is rendered to its ANSI escape sequence and the results
    /// are concatenated into a single `Segment`.
    pub fn new(codes: Vec<ControlCode>) -> Self {
        let rendered: String = codes.iter().map(render_code).collect();
        let segment = Segment::new(&rendered, None, Some(codes));
        Control { segment }
    }

    /// Produce a bell (BEL) control.
    pub fn bell() -> Self {
        Self::new(vec![ControlCode::Simple(ControlType::Bell)])
    }

    /// Move cursor to home position (top-left).
    pub fn home() -> Self {
        Self::new(vec![ControlCode::Simple(ControlType::Home)])
    }

    /// Clear the entire screen.
    pub fn clear() -> Self {
        Self::new(vec![
            ControlCode::Simple(ControlType::Home),
            ControlCode::Simple(ControlType::Clear),
        ])
    }

    /// Show or hide the cursor.
    pub fn show_cursor(show: bool) -> Self {
        if show {
            Self::new(vec![ControlCode::Simple(ControlType::ShowCursor)])
        } else {
            Self::new(vec![ControlCode::Simple(ControlType::HideCursor)])
        }
    }

    /// Enable or disable the alternate screen buffer.
    ///
    /// When enabling, also moves cursor to home position.
    pub fn alt_screen(enable: bool) -> Self {
        if enable {
            Self::new(vec![
                ControlCode::Simple(ControlType::EnableAltScreen),
                ControlCode::Simple(ControlType::Home),
            ])
        } else {
            Self::new(vec![ControlCode::Simple(ControlType::DisableAltScreen)])
        }
    }

    /// Move cursor by a relative offset (x=columns, y=rows).
    ///
    /// Positive x moves right, negative moves left.
    /// Positive y moves down, negative moves up.
    /// Zero offsets produce no output for that axis.
    pub fn cursor_move(x: i32, y: i32) -> Self {
        let mut codes = Vec::new();
        if x > 0 {
            codes.push(ControlCode::WithParam(ControlType::CursorForward, x));
        } else if x < 0 {
            codes.push(ControlCode::WithParam(ControlType::CursorBackward, x.abs()));
        }
        if y > 0 {
            codes.push(ControlCode::WithParam(ControlType::CursorDown, y));
        } else if y < 0 {
            codes.push(ControlCode::WithParam(ControlType::CursorUp, y.abs()));
        }
        Self::new(codes)
    }

    /// Move cursor to an absolute position (0-indexed).
    ///
    /// ANSI sequences use 1-indexed positions, so the conversion is handled
    /// internally.
    pub fn move_to(x: i32, y: i32) -> Self {
        Self::new(vec![ControlCode::WithTwoParams(
            ControlType::CursorMoveTo,
            x,
            y,
        )])
    }

    /// Move cursor to a specific column, with an optional row offset.
    ///
    /// `x` is the 0-indexed column. `y` is a relative row offset
    /// (positive = down, negative = up, 0 = no row movement).
    pub fn move_to_column(x: i32, y: i32) -> Self {
        let mut codes = vec![ControlCode::WithParam(ControlType::CursorMoveToColumn, x)];
        if y > 0 {
            codes.push(ControlCode::WithParam(ControlType::CursorDown, y));
        } else if y < 0 {
            codes.push(ControlCode::WithParam(ControlType::CursorUp, y.abs()));
        }
        Self::new(codes)
    }

    /// Set the terminal window title.
    pub fn title(title: &str) -> Self {
        Self::new(vec![ControlCode::WithParamStr(
            ControlType::SetWindowTitle,
            title.to_string(),
        )])
    }

    /// Begin synchronized output (DEC Mode 2026).
    ///
    /// The terminal buffers all subsequent output until [`end_sync`](Control::end_sync)
    /// is called, then paints atomically to prevent tearing.
    pub fn begin_sync() -> Self {
        Self::new(vec![ControlCode::Simple(ControlType::BeginSync)])
    }

    /// End synchronized output (DEC Mode 2026).
    ///
    /// The terminal flushes all buffered content and renders it at once.
    pub fn end_sync() -> Self {
        Self::new(vec![ControlCode::Simple(ControlType::EndSync)])
    }

    /// Copy text to the system clipboard via OSC 52.
    ///
    /// The text is base64-encoded and wrapped in an OSC 52 escape sequence.
    /// This works in terminals that support OSC 52 (kitty, iTerm2, WezTerm, etc.).
    pub fn set_clipboard(text: &str) -> Self {
        let encoded = base64_encode(text.as_bytes());
        Self::new(vec![ControlCode::WithParamStr(
            ControlType::SetClipboard,
            encoded,
        )])
    }

    /// Request clipboard contents via OSC 52.
    ///
    /// Most terminals require explicit opt-in for clipboard reading.
    pub fn request_clipboard() -> Self {
        Self::new(vec![ControlCode::Simple(ControlType::RequestClipboard)])
    }
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.segment.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_bell() {
        let ctrl = Control::new(vec![ControlCode::Simple(ControlType::Bell)]);
        assert_eq!(ctrl.to_string(), "\x07");
    }

    #[test]
    fn test_strip_control_codes_empty() {
        assert_eq!(strip_control_codes(""), "");
    }

    #[test]
    fn test_strip_control_codes_cr() {
        assert_eq!(strip_control_codes("foo\rbar"), "foobar");
    }

    #[test]
    fn test_strip_control_codes_normal() {
        assert_eq!(strip_control_codes("hello world"), "hello world");
    }

    #[test]
    fn test_strip_control_codes_all() {
        // Bell, Backspace, VT, FF, CR
        assert_eq!(strip_control_codes("a\x07b\x08c\x0Bd\x0Ce\rf"), "abcdef");
    }

    #[test]
    fn test_escape_control_codes_empty() {
        assert_eq!(escape_control_codes(""), "");
    }

    #[test]
    fn test_escape_control_codes_cr() {
        assert_eq!(escape_control_codes("foo\rbar"), r"foo\rbar");
    }

    #[test]
    fn test_escape_control_codes_normal() {
        assert_eq!(escape_control_codes("hello world"), "hello world");
    }

    #[test]
    fn test_escape_control_codes_all() {
        assert_eq!(
            escape_control_codes("a\x07b\x08c\x0Bd\x0Ce\rf"),
            r"a\ab\bc\vd\fe\rf"
        );
    }

    #[test]
    fn test_control_move_to() {
        let ctrl = Control::move_to(5, 10);
        assert_eq!(ctrl.to_string(), "\x1b[11;6H");
        // Verify the control codes stored in the segment
        assert_eq!(
            ctrl.segment.control,
            Some(vec![ControlCode::WithTwoParams(
                ControlType::CursorMoveTo,
                5,
                10
            )])
        );
    }

    #[test]
    fn test_control_cursor_move_zero() {
        let ctrl = Control::cursor_move(0, 0);
        assert_eq!(ctrl.to_string(), "");
    }

    #[test]
    fn test_control_cursor_move() {
        let ctrl = Control::cursor_move(3, 4);
        assert_eq!(ctrl.to_string(), "\x1b[3C\x1b[4B");
    }

    #[test]
    fn test_control_cursor_move_negative() {
        let ctrl = Control::cursor_move(-3, -4);
        assert_eq!(ctrl.to_string(), "\x1b[3D\x1b[4A");
    }

    #[test]
    fn test_move_to_column() {
        let ctrl = Control::move_to_column(10, 20);
        assert_eq!(ctrl.to_string(), "\x1b[11G\x1b[20B");
    }

    #[test]
    fn test_move_to_column_no_row() {
        let ctrl = Control::move_to_column(5, 0);
        assert_eq!(ctrl.to_string(), "\x1b[6G");
    }

    #[test]
    fn test_title() {
        let ctrl = Control::title("hello");
        assert_eq!(ctrl.to_string(), "\x1b]0;hello\x07");
    }

    #[test]
    fn test_bell() {
        assert_eq!(Control::bell().to_string(), "\x07");
    }

    #[test]
    fn test_home() {
        assert_eq!(Control::home().to_string(), "\x1b[H");
    }

    #[test]
    fn test_clear() {
        assert_eq!(Control::clear().to_string(), "\x1b[H\x1b[2J");
    }

    #[test]
    fn test_show_cursor() {
        assert_eq!(Control::show_cursor(true).to_string(), "\x1b[?25h");
        assert_eq!(Control::show_cursor(false).to_string(), "\x1b[?25l");
    }

    #[test]
    fn test_alt_screen() {
        assert_eq!(Control::alt_screen(true).to_string(), "\x1b[?1049h\x1b[H");
        assert_eq!(Control::alt_screen(false).to_string(), "\x1b[?1049l");
    }

    #[test]
    fn test_display() {
        let ctrl = Control::bell();
        assert_eq!(format!("{}", ctrl), "\x07");
    }

    // -- Base64 encoding ----------------------------------------------------

    #[test]
    fn test_base64_encode_empty() {
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_base64_encode_rfc4648_vectors() {
        // Standard test vectors from RFC 4648 section 10
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_base64_encode_hello_world() {
        assert_eq!(base64_encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_base64_encode_unicode() {
        // Unicode characters encoded as UTF-8 bytes
        let text = "Hello \u{1F600}"; // "Hello" + grinning face emoji
        assert_eq!(
            base64_encode(text.as_bytes()),
            base64_encode(text.as_bytes()) // self-consistent check
        );
        // Known value: "Hello \xf0\x9f\x98\x80"
        assert_eq!(
            base64_encode("Hello \u{1F600}".as_bytes()),
            "SGVsbG8g8J+YgA=="
        );
    }

    // -- Synchronized output ------------------------------------------------

    #[test]
    fn test_begin_sync() {
        let ctrl = Control::begin_sync();
        assert_eq!(ctrl.to_string(), "\x1b[?2026h");
    }

    #[test]
    fn test_end_sync() {
        let ctrl = Control::end_sync();
        assert_eq!(ctrl.to_string(), "\x1b[?2026l");
    }

    #[test]
    fn test_begin_sync_segment_is_control() {
        let ctrl = Control::begin_sync();
        assert!(ctrl.segment.is_control());
    }

    #[test]
    fn test_end_sync_segment_is_control() {
        let ctrl = Control::end_sync();
        assert!(ctrl.segment.is_control());
    }

    // -- Clipboard (OSC 52) -------------------------------------------------

    #[test]
    fn test_set_clipboard() {
        let ctrl = Control::set_clipboard("hello");
        // "hello" in base64 is "aGVsbG8="
        assert_eq!(ctrl.to_string(), "\x1b]52;c;aGVsbG8=\x07");
    }

    #[test]
    fn test_set_clipboard_empty() {
        let ctrl = Control::set_clipboard("");
        assert_eq!(ctrl.to_string(), "\x1b]52;c;\x07");
    }

    #[test]
    fn test_set_clipboard_unicode() {
        let ctrl = Control::set_clipboard("\u{00e9}"); // e-acute
                                                       // \xc3\xa9 in base64 is "w6k="
        assert_eq!(ctrl.to_string(), "\x1b]52;c;w6k=\x07");
    }

    #[test]
    fn test_request_clipboard() {
        let ctrl = Control::request_clipboard();
        assert_eq!(ctrl.to_string(), "\x1b]52;c;?\x07");
    }

    #[test]
    fn test_request_clipboard_segment_is_control() {
        let ctrl = Control::request_clipboard();
        assert!(ctrl.segment.is_control());
    }
}
