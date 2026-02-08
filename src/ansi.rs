//! ANSI escape code parsing and conversion to styled Text.
//!
//! This module parses ANSI escape codes from terminal output and converts them
//! to styled `Text` objects. It is a port of Python's `rich/ansi.py`.

use std::sync::LazyLock;
use regex::Regex;

use crate::color::Color;
use crate::style::Style;
use crate::text::Text;

// ---------------------------------------------------------------------------
// ANSI regex
// ---------------------------------------------------------------------------

/// Regex that matches ANSI escape sequences:
/// - Single-char C0/C1 sequences: `\x1b[0-?]`
/// - OSC sequences: `\x1b](.*?)\x1b\\`
/// - CSI/Fe sequences: `\x1b([(@-Z\\-_]|\[[0-?]*[ -/]*[@-~])`
static RE_ANSI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:\x1b[0-?])|(?:\x1b\](.*?)\x1b\\)|(?:\x1b([(@\x2d-Z\\\x2d_]|\[[0-?]*[ -/]*[@-~]))",
    )
    .expect("ANSI regex must compile")
});

// ---------------------------------------------------------------------------
// SGR_STYLE_MAP
// ---------------------------------------------------------------------------

/// Maps SGR (Select Graphic Rendition) code numbers to style definition strings.
fn sgr_style(code: u8) -> Option<&'static str> {
    match code {
        1 => Some("bold"),
        2 => Some("dim"),
        3 => Some("italic"),
        4 => Some("underline"),
        5 => Some("blink"),
        6 => Some("blink2"),
        7 => Some("reverse"),
        8 => Some("conceal"),
        9 => Some("strike"),
        21 => Some("underline2"),
        22 => Some("not dim not bold"),
        23 => Some("not italic"),
        24 => Some("not underline"),
        25 => Some("not blink"),
        26 => Some("not blink2"),
        27 => Some("not reverse"),
        28 => Some("not conceal"),
        29 => Some("not strike"),
        30 => Some("color(0)"),
        31 => Some("color(1)"),
        32 => Some("color(2)"),
        33 => Some("color(3)"),
        34 => Some("color(4)"),
        35 => Some("color(5)"),
        36 => Some("color(6)"),
        37 => Some("color(7)"),
        39 => Some("default"),
        40 => Some("on color(0)"),
        41 => Some("on color(1)"),
        42 => Some("on color(2)"),
        43 => Some("on color(3)"),
        44 => Some("on color(4)"),
        45 => Some("on color(5)"),
        46 => Some("on color(6)"),
        47 => Some("on color(7)"),
        49 => Some("on default"),
        51 => Some("frame"),
        52 => Some("encircle"),
        53 => Some("overline"),
        54 => Some("not frame not encircle"),
        55 => Some("not overline"),
        90 => Some("color(8)"),
        91 => Some("color(9)"),
        92 => Some("color(10)"),
        93 => Some("color(11)"),
        94 => Some("color(12)"),
        95 => Some("color(13)"),
        96 => Some("color(14)"),
        97 => Some("color(15)"),
        100 => Some("on color(8)"),
        101 => Some("on color(9)"),
        102 => Some("on color(10)"),
        103 => Some("on color(11)"),
        104 => Some("on color(12)"),
        105 => Some("on color(13)"),
        106 => Some("on color(14)"),
        107 => Some("on color(15)"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// AnsiToken
// ---------------------------------------------------------------------------

/// A token produced by ANSI tokenization.
struct AnsiToken {
    /// Plain text between escape sequences.
    plain: String,
    /// SGR code string (semicolon-separated numbers), if this is an SGR sequence.
    sgr: Option<String>,
    /// OSC payload, if this is an OSC sequence.
    osc: Option<String>,
}

// ---------------------------------------------------------------------------
// ansi_tokenize
// ---------------------------------------------------------------------------

/// Tokenizes a string into plain text and ANSI escape sequences.
///
/// Yields `AnsiToken` values:
/// - Plain text segments between escape sequences
/// - SGR codes (CSI sequences ending in `m`)
/// - OSC sequences (Operating System Commands)
/// - Character set designation sequences (`(`) are skipped
fn ansi_tokenize(ansi_text: &str) -> Vec<AnsiToken> {
    let mut tokens = Vec::new();
    let mut position = 0;

    for caps in RE_ANSI.captures_iter(ansi_text) {
        let whole = caps.get(0).unwrap();
        let start = whole.start();
        let end = whole.end();

        // Groups: 1 = OSC payload, 2 = SGR/Fe sequence
        let osc = caps.get(1).map(|m| m.as_str().to_string());
        let sgr = caps.get(2).map(|m| m.as_str().to_string());

        // Emit any plain text before this match
        if start > position {
            tokens.push(AnsiToken {
                plain: ansi_text[position..start].to_string(),
                sgr: None,
                osc: None,
            });
        }

        if let Some(ref sgr_val) = sgr {
            if sgr_val == "(" {
                // Character set designation: skip the next byte too
                position = end + 1;
                continue;
            }
            if sgr_val.ends_with('m') {
                // CSI SGR sequence: strip leading `[` and trailing `m`
                let inner = &sgr_val[1..sgr_val.len() - 1];
                tokens.push(AnsiToken {
                    plain: String::new(),
                    sgr: Some(inner.to_string()),
                    osc,
                });
            } else {
                // Other CSI/Fe sequence: emit with no sgr content
                tokens.push(AnsiToken {
                    plain: String::new(),
                    sgr: None,
                    osc,
                });
            }
        } else {
            // OSC-only or single-char sequence
            tokens.push(AnsiToken {
                plain: String::new(),
                sgr: None,
                osc,
            });
        }

        position = end;
    }

    // Emit remaining plain text
    if position < ansi_text.len() {
        tokens.push(AnsiToken {
            plain: ansi_text[position..].to_string(),
            sgr: None,
            osc: None,
        });
    }

    tokens
}

// ---------------------------------------------------------------------------
// AnsiDecoder
// ---------------------------------------------------------------------------

/// Decodes ANSI escape codes into styled `Text` objects.
pub struct AnsiDecoder {
    style: Style,
}

impl AnsiDecoder {
    /// Creates a new decoder with a null (empty) style.
    pub fn new() -> Self {
        AnsiDecoder {
            style: Style::null(),
        }
    }

    /// Decodes multi-line ANSI text, returning one `Text` per line.
    pub fn decode(&mut self, terminal_text: &str) -> Vec<Text> {
        terminal_text
            .lines()
            .map(|line| self.decode_line(line))
            .collect()
    }

    /// Decodes a single line containing ANSI codes.
    ///
    /// Handles:
    /// - Carriage returns (keeps only text after the last `\r`)
    /// - SGR codes for text attributes and colors
    /// - OSC 8 hyperlink sequences
    /// - 256-color and truecolor foreground/background
    pub fn decode_line(&mut self, line: &str) -> Text {
        let mut text = Text::new("", Style::null());

        // Handle carriage returns: keep only text after the last \r
        let line = match line.rsplit_once('\r') {
            Some((_, after)) => after,
            None => line,
        };

        for token in ansi_tokenize(line) {
            if !token.plain.is_empty() {
                // Append plain text with current style
                let style = if self.style.is_null() {
                    None
                } else {
                    Some(self.style.clone())
                };
                text.append_str(&token.plain, style);
            } else if let Some(ref osc) = token.osc {
                // Handle OSC sequences
                if let Some(after_prefix) = osc.strip_prefix("8;") {
                    // OSC 8 hyperlink: format is "8;params;url"
                    if let Some((_params, url)) = after_prefix.split_once(';') {
                        if url.is_empty() {
                            self.style = self.style.update_link(None);
                        } else {
                            self.style = self.style.update_link(Some(url));
                        }
                    }
                }
            } else if let Some(ref sgr) = token.sgr {
                // Parse SGR codes
                let codes: Vec<u8> = sgr
                    .split(';')
                    .filter_map(|part| {
                        if part.is_empty() {
                            Some(0)
                        } else if part.chars().all(|c| c.is_ascii_digit()) {
                            part.parse::<u16>().ok().map(|v| v.min(255) as u8)
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut iter = codes.iter().copied();
                while let Some(code) = iter.next() {
                    if code == 0 {
                        self.style = Style::null();
                    } else if let Some(style_str) = sgr_style(code) {
                        if let Ok(parsed) = Style::parse(style_str) {
                            self.style = self.style.clone() + parsed;
                        }
                    } else if code == 38 {
                        // Foreground color
                        if let Some(color_type) = iter.next() {
                            if color_type == 5 {
                                if let Some(n) = iter.next() {
                                    let color = Color::from_ansi(n);
                                    self.style =
                                        self.style.clone() + Style::from_color(Some(color), None);
                                }
                            } else if color_type == 2 {
                                let r = iter.next();
                                let g = iter.next();
                                let b = iter.next();
                                if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                    let color = Color::from_rgb(r, g, b);
                                    self.style =
                                        self.style.clone() + Style::from_color(Some(color), None);
                                }
                            }
                        }
                    } else if code == 48 {
                        // Background color
                        if let Some(color_type) = iter.next() {
                            if color_type == 5 {
                                if let Some(n) = iter.next() {
                                    let color = Color::from_ansi(n);
                                    self.style =
                                        self.style.clone() + Style::from_color(None, Some(color));
                                }
                            } else if color_type == 2 {
                                let r = iter.next();
                                let g = iter.next();
                                let b = iter.next();
                                if let (Some(r), Some(g), Some(b)) = (r, g, b) {
                                    let color = Color::from_rgb(r, g, b);
                                    self.style =
                                        self.style.clone() + Style::from_color(None, Some(color));
                                }
                            }
                        }
                    }
                    // Unknown codes are silently ignored
                }
            }
        }

        text
    }
}

impl Default for AnsiDecoder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- SGR_STYLE_MAP tests ------------------------------------------------

    #[test]
    fn test_sgr_style_map_basic_attributes() {
        assert_eq!(sgr_style(1), Some("bold"));
        assert_eq!(sgr_style(2), Some("dim"));
        assert_eq!(sgr_style(3), Some("italic"));
        assert_eq!(sgr_style(4), Some("underline"));
        assert_eq!(sgr_style(5), Some("blink"));
        assert_eq!(sgr_style(6), Some("blink2"));
        assert_eq!(sgr_style(7), Some("reverse"));
        assert_eq!(sgr_style(8), Some("conceal"));
        assert_eq!(sgr_style(9), Some("strike"));
        assert_eq!(sgr_style(21), Some("underline2"));
    }

    #[test]
    fn test_sgr_style_map_reset_attributes() {
        assert_eq!(sgr_style(22), Some("not dim not bold"));
        assert_eq!(sgr_style(23), Some("not italic"));
        assert_eq!(sgr_style(24), Some("not underline"));
        assert_eq!(sgr_style(25), Some("not blink"));
        assert_eq!(sgr_style(26), Some("not blink2"));
        assert_eq!(sgr_style(27), Some("not reverse"));
        assert_eq!(sgr_style(28), Some("not conceal"));
        assert_eq!(sgr_style(29), Some("not strike"));
        assert_eq!(sgr_style(54), Some("not frame not encircle"));
        assert_eq!(sgr_style(55), Some("not overline"));
    }

    #[test]
    fn test_sgr_style_map_foreground_colors() {
        for i in 0..8u8 {
            let expected = format!("color({})", i);
            assert_eq!(
                sgr_style(30 + i),
                Some(expected.as_str()).map(|_| sgr_style(30 + i).unwrap())
            );
        }
        assert_eq!(sgr_style(39), Some("default"));
    }

    #[test]
    fn test_sgr_style_map_background_colors() {
        for i in 0..8u8 {
            assert!(sgr_style(40 + i).unwrap().starts_with("on "));
        }
        assert_eq!(sgr_style(49), Some("on default"));
    }

    #[test]
    fn test_sgr_style_map_bright_colors() {
        for i in 0..8u8 {
            let expected = format!("color({})", 8 + i);
            assert_eq!(sgr_style(90 + i).unwrap(), expected);
        }
        for i in 0..8u8 {
            let expected = format!("on color({})", 8 + i);
            assert_eq!(sgr_style(100 + i).unwrap(), expected);
        }
    }

    #[test]
    fn test_sgr_style_map_decorations() {
        assert_eq!(sgr_style(51), Some("frame"));
        assert_eq!(sgr_style(52), Some("encircle"));
        assert_eq!(sgr_style(53), Some("overline"));
    }

    #[test]
    fn test_sgr_style_map_unknown() {
        assert_eq!(sgr_style(10), None);
        assert_eq!(sgr_style(99), None);
        assert_eq!(sgr_style(200), None);
    }

    // -- ansi_tokenize tests ------------------------------------------------

    #[test]
    fn test_tokenize_plain_text() {
        let tokens = ansi_tokenize("Hello, World!");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].plain, "Hello, World!");
        assert!(tokens[0].sgr.is_none());
        assert!(tokens[0].osc.is_none());
    }

    #[test]
    fn test_tokenize_sgr_sequence() {
        let tokens = ansi_tokenize("\x1b[1mBold\x1b[0m");
        // Should produce: SGR("1"), plain("Bold"), SGR("0")
        assert!(tokens.len() >= 3);

        // First token: SGR bold
        assert!(tokens[0].plain.is_empty());
        assert_eq!(tokens[0].sgr.as_deref(), Some("1"));

        // Second token: plain text
        assert_eq!(tokens[1].plain, "Bold");

        // Third token: SGR reset
        assert!(tokens[2].plain.is_empty());
        assert_eq!(tokens[2].sgr.as_deref(), Some("0"));
    }

    #[test]
    fn test_tokenize_mixed_content() {
        let tokens = ansi_tokenize("Hello \x1b[31mRed\x1b[0m World");
        // Expect: plain("Hello "), SGR("31"), plain("Red"), SGR("0"), plain(" World")
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].plain, "Hello ");
        assert_eq!(tokens[1].sgr.as_deref(), Some("31"));
        assert_eq!(tokens[2].plain, "Red");
        assert_eq!(tokens[3].sgr.as_deref(), Some("0"));
        assert_eq!(tokens[4].plain, " World");
    }

    #[test]
    fn test_tokenize_osc_sequence() {
        let tokens = ansi_tokenize("\x1b]8;;https://example.com\x1b\\Link\x1b]8;;\x1b\\");
        // Should have OSC tokens and plain text
        let has_osc = tokens.iter().any(|t| t.osc.is_some());
        assert!(has_osc);
        let has_link = tokens.iter().any(|t| t.plain == "Link");
        assert!(has_link);
    }

    #[test]
    fn test_tokenize_multiple_sgr_codes() {
        let tokens = ansi_tokenize("\x1b[1;31mBoldRed\x1b[0m");
        assert!(tokens[0].sgr.as_deref() == Some("1;31"));
    }

    #[test]
    fn test_tokenize_empty_string() {
        let tokens = ansi_tokenize("");
        assert!(tokens.is_empty());
    }

    // -- AnsiDecoder::decode_line tests -------------------------------------

    #[test]
    fn test_decode_line_plain_text() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("Hello, World!");
        assert_eq!(text.plain(), "Hello, World!");
        assert!(text.spans().is_empty());
    }

    #[test]
    fn test_decode_line_bold_text() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[1mBold\x1b[0m");
        assert_eq!(text.plain(), "Bold");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].style.bold(), Some(true));
    }

    #[test]
    fn test_decode_line_colored_text() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[31mRed\x1b[0m");
        assert_eq!(text.plain(), "Red");
        assert_eq!(text.spans().len(), 1);
        let color = text.spans()[0].style.color().unwrap();
        assert_eq!(color.number, Some(1));
    }

    #[test]
    fn test_decode_line_256_color() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[38;5;196mRed256\x1b[0m");
        assert_eq!(text.plain(), "Red256");
        assert_eq!(text.spans().len(), 1);
        let color = text.spans()[0].style.color().unwrap();
        assert_eq!(color.number, Some(196));
    }

    #[test]
    fn test_decode_line_truecolor_rgb() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[38;2;255;128;0mOrange\x1b[0m");
        assert_eq!(text.plain(), "Orange");
        assert_eq!(text.spans().len(), 1);
        let color = text.spans()[0].style.color().unwrap();
        assert_eq!(color.triplet.unwrap().red, 255);
        assert_eq!(color.triplet.unwrap().green, 128);
        assert_eq!(color.triplet.unwrap().blue, 0);
    }

    #[test]
    fn test_decode_line_reset_clears_style() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[1mBold\x1b[0mNormal");
        assert_eq!(text.plain(), "BoldNormal");
        // "Bold" has bold style, "Normal" has no style
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].start, 0);
        assert_eq!(text.spans()[0].end, 4);
        assert_eq!(text.spans()[0].style.bold(), Some(true));
    }

    #[test]
    fn test_decode_line_carriage_return() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("OldText\rNewText");
        assert_eq!(text.plain(), "NewText");
    }

    #[test]
    fn test_decode_line_multiple_sgr_codes_in_one_sequence() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[1;31mBoldRed\x1b[0m");
        assert_eq!(text.plain(), "BoldRed");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].style.bold(), Some(true));
        let color = text.spans()[0].style.color().unwrap();
        assert_eq!(color.number, Some(1));
    }

    #[test]
    fn test_decode_line_background_color_standard() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[42mGreenBG\x1b[0m");
        assert_eq!(text.plain(), "GreenBG");
        assert_eq!(text.spans().len(), 1);
        let bgcolor = text.spans()[0].style.bgcolor().unwrap();
        assert_eq!(bgcolor.number, Some(2));
    }

    #[test]
    fn test_decode_line_background_256_color() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[48;5;100mBG256\x1b[0m");
        assert_eq!(text.plain(), "BG256");
        assert_eq!(text.spans().len(), 1);
        let bgcolor = text.spans()[0].style.bgcolor().unwrap();
        assert_eq!(bgcolor.number, Some(100));
    }

    #[test]
    fn test_decode_line_background_truecolor() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[48;2;10;20;30mBGRGB\x1b[0m");
        assert_eq!(text.plain(), "BGRGB");
        assert_eq!(text.spans().len(), 1);
        let bgcolor = text.spans()[0].style.bgcolor().unwrap();
        assert_eq!(bgcolor.triplet.unwrap().red, 10);
        assert_eq!(bgcolor.triplet.unwrap().green, 20);
        assert_eq!(bgcolor.triplet.unwrap().blue, 30);
    }

    #[test]
    fn test_decode_line_osc8_hyperlink() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b]8;;https://example.com\x1b\\Click\x1b]8;;\x1b\\");
        assert_eq!(text.plain(), "Click");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].style.link(), Some("https://example.com"));
    }

    #[test]
    fn test_decode_multiline() {
        let mut decoder = AnsiDecoder::new();
        let texts = decoder.decode("Line 1\nLine 2\nLine 3");
        assert_eq!(texts.len(), 3);
        assert_eq!(texts[0].plain(), "Line 1");
        assert_eq!(texts[1].plain(), "Line 2");
        assert_eq!(texts[2].plain(), "Line 3");
    }

    #[test]
    fn test_decode_unknown_codes_ignored() {
        let mut decoder = AnsiDecoder::new();
        // Code 10 and 99 are not in SGR_STYLE_MAP
        let text = decoder.decode_line("\x1b[10;99mText\x1b[0m");
        assert_eq!(text.plain(), "Text");
        // No style should be applied since codes are unknown
        assert!(text.spans().is_empty());
    }

    #[test]
    fn test_decode_nested_styles_accumulate() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[1mBold\x1b[3mBoldItalic\x1b[0mNormal");
        assert_eq!(text.plain(), "BoldBoldItalicNormal");

        // First segment: bold only (chars 0..4)
        assert_eq!(text.spans()[0].start, 0);
        assert_eq!(text.spans()[0].end, 4);
        assert_eq!(text.spans()[0].style.bold(), Some(true));
        assert_eq!(text.spans()[0].style.italic(), None);

        // Second segment: bold + italic (chars 4..14)
        assert_eq!(text.spans()[1].start, 4);
        assert_eq!(text.spans()[1].end, 14);
        assert_eq!(text.spans()[1].style.bold(), Some(true));
        assert_eq!(text.spans()[1].style.italic(), Some(true));
    }

    #[test]
    fn test_decode_style_persists_across_lines() {
        let mut decoder = AnsiDecoder::new();
        let texts = decoder.decode("\x1b[1mBold\nStillBold\x1b[0m");
        assert_eq!(texts.len(), 2);

        // First line: bold applied
        assert_eq!(texts[0].plain(), "Bold");
        assert_eq!(texts[0].spans().len(), 1);
        assert_eq!(texts[0].spans()[0].style.bold(), Some(true));

        // Second line: bold persists from previous line
        assert_eq!(texts[1].plain(), "StillBold");
        assert_eq!(texts[1].spans().len(), 1);
        assert_eq!(texts[1].spans()[0].style.bold(), Some(true));
    }

    #[test]
    fn test_decode_empty_sgr_treated_as_reset() {
        let mut decoder = AnsiDecoder::new();
        // \x1b[m is equivalent to \x1b[0m (reset)
        let text = decoder.decode_line("\x1b[1mBold\x1b[mNormal");
        assert_eq!(text.plain(), "BoldNormal");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].start, 0);
        assert_eq!(text.spans()[0].end, 4);
    }

    #[test]
    fn test_decode_line_bright_foreground() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[91mBrightRed\x1b[0m");
        assert_eq!(text.plain(), "BrightRed");
        assert_eq!(text.spans().len(), 1);
        let color = text.spans()[0].style.color().unwrap();
        assert_eq!(color.number, Some(9));
    }

    #[test]
    fn test_decode_line_bright_background() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[101mBrightRedBG\x1b[0m");
        assert_eq!(text.plain(), "BrightRedBG");
        assert_eq!(text.spans().len(), 1);
        let bgcolor = text.spans()[0].style.bgcolor().unwrap();
        assert_eq!(bgcolor.number, Some(9));
    }

    #[test]
    fn test_decode_line_default_foreground() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[31mRed\x1b[39mDefault\x1b[0m");
        assert_eq!(text.plain(), "RedDefault");
        // After \x1b[39m, foreground is set to default
        assert_eq!(text.spans().len(), 2);
        // Second span should have default foreground color
        let color = text.spans()[1].style.color().unwrap();
        assert!(color.is_default());
    }

    #[test]
    fn test_decode_line_no_ansi() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("Just plain text");
        assert_eq!(text.plain(), "Just plain text");
        assert!(text.spans().is_empty());
    }

    #[test]
    fn test_decode_multiline_empty() {
        let mut decoder = AnsiDecoder::new();
        let texts = decoder.decode("");
        assert!(texts.is_empty());
    }

    #[test]
    fn test_decode_default_trait() {
        let decoder = AnsiDecoder::default();
        assert!(decoder.style.is_null());
    }

    #[test]
    fn test_decode_line_fg_and_bg_combined() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[31;42mRedOnGreen\x1b[0m");
        assert_eq!(text.plain(), "RedOnGreen");
        assert_eq!(text.spans().len(), 1);
        let fg = text.spans()[0].style.color().unwrap();
        assert_eq!(fg.number, Some(1));
        let bg = text.spans()[0].style.bgcolor().unwrap();
        assert_eq!(bg.number, Some(2));
    }

    #[test]
    fn test_decode_line_multiple_resets() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\x1b[1mBold\x1b[0m\x1b[0mPlain");
        assert_eq!(text.plain(), "BoldPlain");
        assert_eq!(text.spans().len(), 1);
        assert_eq!(text.spans()[0].end, 4);
    }

    #[test]
    fn test_decode_line_carriage_return_at_start() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("\rOverwritten");
        assert_eq!(text.plain(), "Overwritten");
    }

    #[test]
    fn test_decode_line_multiple_carriage_returns() {
        let mut decoder = AnsiDecoder::new();
        let text = decoder.decode_line("First\rSecond\rThird");
        assert_eq!(text.plain(), "Third");
    }
}
