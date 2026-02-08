//! Bidirectional conversions between gilt and anstyle types.
//!
//! This module enables interop with the anstyle ecosystem (clap, owo-colors, anstream)
//! by providing `From` implementations for color and style types.

use crate::color::{Color, ColorType};
#[cfg(test)]
use crate::color_triplet::ColorTriplet;
use crate::style::{Style, UnderlineStyle};

// ---------------------------------------------------------------------------
// Color conversions: gilt -> anstyle
// ---------------------------------------------------------------------------

/// Converts a gilt `Color` to an `anstyle::Color`.
///
/// # Mapping
/// - `ColorType::Default` -> `None` (represented as `Option<anstyle::Color>`)
/// - `ColorType::Standard` (0-7) -> `anstyle::AnsiColor`
/// - `ColorType::Standard` (8-15) -> `anstyle::AnsiColor` (bright variants)
/// - `ColorType::EightBit` -> `anstyle::Ansi256Color`
/// - `ColorType::TrueColor` -> `anstyle::RgbColor`
impl From<&Color> for Option<anstyle::Color> {
    fn from(color: &Color) -> Self {
        match color.color_type {
            ColorType::Default => None,
            ColorType::Standard | ColorType::Windows => {
                if let Some(n) = color.number {
                    let ansi = match n {
                        0 => anstyle::AnsiColor::Black,
                        1 => anstyle::AnsiColor::Red,
                        2 => anstyle::AnsiColor::Green,
                        3 => anstyle::AnsiColor::Yellow,
                        4 => anstyle::AnsiColor::Blue,
                        5 => anstyle::AnsiColor::Magenta,
                        6 => anstyle::AnsiColor::Cyan,
                        7 => anstyle::AnsiColor::White,
                        8 => anstyle::AnsiColor::BrightBlack,
                        9 => anstyle::AnsiColor::BrightRed,
                        10 => anstyle::AnsiColor::BrightGreen,
                        11 => anstyle::AnsiColor::BrightYellow,
                        12 => anstyle::AnsiColor::BrightBlue,
                        13 => anstyle::AnsiColor::BrightMagenta,
                        14 => anstyle::AnsiColor::BrightCyan,
                        15 => anstyle::AnsiColor::BrightWhite,
                        _ => return Some(anstyle::Color::Ansi256(anstyle::Ansi256Color(n))),
                    };
                    Some(anstyle::Color::Ansi(ansi))
                } else {
                    None
                }
            }
            ColorType::EightBit => color
                .number
                .map(|n| anstyle::Color::Ansi256(anstyle::Ansi256Color(n))),
            ColorType::TrueColor => color
                .triplet
                .map(|t| anstyle::Color::Rgb(anstyle::RgbColor(t.red, t.green, t.blue))),
        }
    }
}

// ---------------------------------------------------------------------------
// Color conversions: anstyle -> gilt
// ---------------------------------------------------------------------------

/// Converts an `anstyle::Color` to a gilt `Color`.
impl From<anstyle::Color> for Color {
    fn from(color: anstyle::Color) -> Self {
        match color {
            anstyle::Color::Ansi(ansi) => {
                let n = ansi_color_to_number(ansi);
                Color::from_ansi(n)
            }
            anstyle::Color::Ansi256(anstyle::Ansi256Color(n)) => Color::from_ansi(n),
            anstyle::Color::Rgb(anstyle::RgbColor(r, g, b)) => Color::from_rgb(r, g, b),
        }
    }
}

/// Converts an `anstyle::AnsiColor` to its 4-bit color number (0-15).
fn ansi_color_to_number(ansi: anstyle::AnsiColor) -> u8 {
    match ansi {
        anstyle::AnsiColor::Black => 0,
        anstyle::AnsiColor::Red => 1,
        anstyle::AnsiColor::Green => 2,
        anstyle::AnsiColor::Yellow => 3,
        anstyle::AnsiColor::Blue => 4,
        anstyle::AnsiColor::Magenta => 5,
        anstyle::AnsiColor::Cyan => 6,
        anstyle::AnsiColor::White => 7,
        anstyle::AnsiColor::BrightBlack => 8,
        anstyle::AnsiColor::BrightRed => 9,
        anstyle::AnsiColor::BrightGreen => 10,
        anstyle::AnsiColor::BrightYellow => 11,
        anstyle::AnsiColor::BrightBlue => 12,
        anstyle::AnsiColor::BrightMagenta => 13,
        anstyle::AnsiColor::BrightCyan => 14,
        anstyle::AnsiColor::BrightWhite => 15,
    }
}

// ---------------------------------------------------------------------------
// Style conversions: gilt -> anstyle
// ---------------------------------------------------------------------------

/// Converts a gilt `Style` to an `anstyle::Style`.
///
/// # Lossy conversions
/// - gilt's `link` (OSC 8 hyperlinks) is dropped (anstyle has no link support)
/// - gilt's `frame`, `encircle` are dropped (not in anstyle)
/// - gilt's `overline` is dropped (not in anstyle Effects)
impl From<&Style> for anstyle::Style {
    fn from(style: &Style) -> Self {
        let mut result = anstyle::Style::new();

        // Colors
        if let Some(color) = style.color() {
            let opt: Option<anstyle::Color> = color.into();
            if let Some(c) = opt {
                result = result.fg_color(Some(c));
            }
        }
        if let Some(bgcolor) = style.bgcolor() {
            let opt: Option<anstyle::Color> = bgcolor.into();
            if let Some(c) = opt {
                result = result.bg_color(Some(c));
            }
        }

        // Underline color
        if let Some(ul_color) = style.underline_color() {
            let opt: Option<anstyle::Color> = ul_color.into();
            if let Some(c) = opt {
                result = result.underline_color(Some(c));
            }
        }

        // Effects (attributes)
        let mut effects = anstyle::Effects::new();
        if style.bold() == Some(true) {
            effects = effects.insert(anstyle::Effects::BOLD);
        }
        if style.dim() == Some(true) {
            effects = effects.insert(anstyle::Effects::DIMMED);
        }
        if style.italic() == Some(true) {
            effects = effects.insert(anstyle::Effects::ITALIC);
        }
        if style.underline() == Some(true) {
            effects = effects.insert(anstyle::Effects::UNDERLINE);
        }
        if style.blink() == Some(true) {
            effects = effects.insert(anstyle::Effects::BLINK);
        }
        if style.reverse() == Some(true) {
            effects = effects.insert(anstyle::Effects::INVERT);
        }
        if style.conceal() == Some(true) {
            effects = effects.insert(anstyle::Effects::HIDDEN);
        }
        if style.strike() == Some(true) {
            effects = effects.insert(anstyle::Effects::STRIKETHROUGH);
        }

        // Underline style
        if let Some(ul_style) = style.underline_style() {
            match ul_style {
                UnderlineStyle::Curly => {
                    effects = effects.insert(anstyle::Effects::CURLY_UNDERLINE);
                }
                UnderlineStyle::Dotted => {
                    effects = effects.insert(anstyle::Effects::DOTTED_UNDERLINE);
                }
                UnderlineStyle::Dashed => {
                    effects = effects.insert(anstyle::Effects::DASHED_UNDERLINE);
                }
                UnderlineStyle::Double => {
                    effects = effects.insert(anstyle::Effects::DOUBLE_UNDERLINE);
                }
                UnderlineStyle::Single => {
                    effects = effects.insert(anstyle::Effects::UNDERLINE);
                }
            }
        }

        result = result.effects(effects);
        result
    }
}

// ---------------------------------------------------------------------------
// Style conversions: anstyle -> gilt
// ---------------------------------------------------------------------------

/// Converts an `anstyle::Style` to a gilt `Style`.
///
/// # Enhanced conversions
/// - anstyle's underline color -> gilt's `underline_color`
/// - anstyle's curly/dotted/dashed underline -> gilt's `UnderlineStyle`
impl From<anstyle::Style> for Style {
    fn from(style: anstyle::Style) -> Self {
        let color = style.get_fg_color().map(Color::from);
        let bgcolor = style.get_bg_color().map(Color::from);
        let underline_color = style.get_underline_color().map(Color::from);

        let effects = style.get_effects();

        let bold = if effects.contains(anstyle::Effects::BOLD) {
            Some(true)
        } else {
            None
        };
        let dim = if effects.contains(anstyle::Effects::DIMMED) {
            Some(true)
        } else {
            None
        };
        let italic = if effects.contains(anstyle::Effects::ITALIC) {
            Some(true)
        } else {
            None
        };
        let blink = if effects.contains(anstyle::Effects::BLINK) {
            Some(true)
        } else {
            None
        };
        let reverse = if effects.contains(anstyle::Effects::INVERT) {
            Some(true)
        } else {
            None
        };
        let conceal = if effects.contains(anstyle::Effects::HIDDEN) {
            Some(true)
        } else {
            None
        };
        let strike = if effects.contains(anstyle::Effects::STRIKETHROUGH) {
            Some(true)
        } else {
            None
        };

        // Determine underline and underline_style from effects
        let underline;
        let underline_style;

        if effects.contains(anstyle::Effects::CURLY_UNDERLINE) {
            underline = Some(true);
            underline_style = Some(UnderlineStyle::Curly);
        } else if effects.contains(anstyle::Effects::DOTTED_UNDERLINE) {
            underline = Some(true);
            underline_style = Some(UnderlineStyle::Dotted);
        } else if effects.contains(anstyle::Effects::DASHED_UNDERLINE) {
            underline = Some(true);
            underline_style = Some(UnderlineStyle::Dashed);
        } else if effects.contains(anstyle::Effects::DOUBLE_UNDERLINE) {
            underline = Some(true);
            underline_style = Some(UnderlineStyle::Double);
        } else if effects.contains(anstyle::Effects::UNDERLINE) {
            underline = Some(true);
            underline_style = None;
        } else {
            underline = None;
            underline_style = None;
        }

        let mut result = Style::from_color(color, bgcolor);
        // Set attributes via public setters
        result.set_bold(bold);
        result.set_dim(dim);
        result.set_italic(italic);
        result.set_underline(underline);
        result.set_blink(blink);
        result.set_reverse(reverse);
        result.set_conceal(conceal);
        result.set_strike(strike);
        result.set_underline_color(underline_color);
        result.set_underline_style(underline_style);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Color round-trip tests -----------------------------------------

    #[test]
    fn test_standard_color_roundtrip() {
        for n in 0..16u8 {
            let gilt_color = Color::from_ansi(n);
            let anstyle_opt: Option<anstyle::Color> = (&gilt_color).into();
            assert!(anstyle_opt.is_some(), "Standard color {} should convert", n);
            let back = Color::from(anstyle_opt.unwrap());
            assert_eq!(back.number, Some(n), "Round-trip failed for color {}", n);
        }
    }

    #[test]
    fn test_eightbit_color_roundtrip() {
        for n in [16u8, 100, 200, 255] {
            let gilt_color = Color::from_ansi(n);
            let anstyle_opt: Option<anstyle::Color> = (&gilt_color).into();
            assert!(anstyle_opt.is_some());
            let back = Color::from(anstyle_opt.unwrap());
            assert_eq!(back.number, Some(n));
        }
    }

    #[test]
    fn test_truecolor_roundtrip() {
        let gilt_color = Color::from_rgb(128, 64, 32);
        let anstyle_opt: Option<anstyle::Color> = (&gilt_color).into();
        assert!(anstyle_opt.is_some());
        let back = Color::from(anstyle_opt.unwrap());
        assert_eq!(back.triplet, Some(ColorTriplet::new(128, 64, 32)));
    }

    #[test]
    fn test_default_color_to_none() {
        let gilt_color = Color::default_color();
        let anstyle_opt: Option<anstyle::Color> = (&gilt_color).into();
        assert!(anstyle_opt.is_none());
    }

    // -- Style round-trip tests -----------------------------------------

    #[test]
    fn test_style_bold_roundtrip() {
        let gilt_style = Style::parse("bold red on blue").unwrap();
        let anstyle_style: anstyle::Style = (&gilt_style).into();
        let back: Style = anstyle_style.into();
        assert_eq!(back.bold(), Some(true));
        assert!(back.color().is_some());
        assert!(back.bgcolor().is_some());
    }

    #[test]
    fn test_style_lossy_link_dropped() {
        let gilt_style = Style::parse("bold link https://example.com").unwrap();
        let anstyle_style: anstyle::Style = (&gilt_style).into();
        let back: Style = anstyle_style.into();
        // Link should be lost
        assert!(back.link().is_none());
        // Bold should survive
        assert_eq!(back.bold(), Some(true));
    }

    #[test]
    fn test_underline_style_curly_roundtrip() {
        let mut style = Style::null();
        style.set_underline(Some(true));
        style.set_underline_style(Some(UnderlineStyle::Curly));
        let anstyle_style: anstyle::Style = (&style).into();
        let back: Style = anstyle_style.into();
        assert_eq!(back.underline(), Some(true));
        assert_eq!(back.underline_style(), Some(UnderlineStyle::Curly));
    }

    #[test]
    fn test_underline_color_roundtrip() {
        let mut style = Style::null();
        style.set_underline(Some(true));
        style.set_underline_color(Some(Color::from_rgb(255, 0, 0)));
        let anstyle_style: anstyle::Style = (&style).into();
        let back: Style = anstyle_style.into();
        assert!(back.underline_color().is_some());
        assert_eq!(
            back.underline_color().unwrap().triplet,
            Some(ColorTriplet::new(255, 0, 0))
        );
    }

    #[test]
    fn test_effects_mapping() {
        let mut style = Style::null();
        style.set_bold(Some(true));
        style.set_dim(Some(true));
        style.set_italic(Some(true));
        style.set_strike(Some(true));
        style.set_reverse(Some(true));
        style.set_conceal(Some(true));

        let anstyle_style: anstyle::Style = (&style).into();
        let effects = anstyle_style.get_effects();

        assert!(effects.contains(anstyle::Effects::BOLD));
        assert!(effects.contains(anstyle::Effects::DIMMED));
        assert!(effects.contains(anstyle::Effects::ITALIC));
        assert!(effects.contains(anstyle::Effects::STRIKETHROUGH));
        assert!(effects.contains(anstyle::Effects::INVERT));
        assert!(effects.contains(anstyle::Effects::HIDDEN));
    }
}
