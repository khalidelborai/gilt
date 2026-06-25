//! Terminal color representation and manipulation.
//!
//! This module provides the Color type that represents terminal colors,
//! supporting different color systems (standard 16, 8-bit 256, truecolor).

pub mod accessibility;
pub mod color_env;
pub mod color_triplet;
pub mod palette;
pub mod terminal_theme;
pub mod theme;

use crate::error::ColorParseError;

use self::color_triplet::ColorTriplet;
use self::palette::{EIGHT_BIT_PALETTE, STANDARD_PALETTE, WINDOWS_PALETTE};
use self::terminal_theme::{TerminalTheme, DEFAULT_TERMINAL_THEME};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Write as _;

/// Color system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ColorSystem {
    /// Standard 16-color palette (ANSI colors 0-15).
    Standard = 1,
    /// Extended 256-color palette (8-bit ANSI).
    EightBit = 2,
    /// 24-bit true-color (16 million colors).
    TrueColor = 3,
    /// Windows console legacy color palette.
    Windows = 4,
}

/// Color type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ColorType {
    /// Terminal default color (foreground or background).
    Default = 0,
    /// Standard 16-color ANSI palette.
    Standard = 1,
    /// Extended 256-color (8-bit) palette.
    EightBit = 2,
    /// 24-bit true-color RGB.
    TrueColor = 3,
    /// Windows console legacy color.
    Windows = 4,
}

/// A terminal color.
///
/// **v0.11.0 break:** `Color` was a struct with `name`, `color_type`, `number`,
/// `triplet` fields. It is now a 5-variant enum, shrinking from ~40 B + heap
/// String to **~4 B inline**. The `name` field becomes [`Color::name`] (a
/// `Cow<'_, str>` method) and `color_type` becomes [`Color::kind`].
///
/// # Migration
///
/// ```ignore
/// // Before:
/// Color { name: "default".into(), color_type: ColorType::Default,
///         number: None, triplet: None }
/// // After:
/// Color::Default
///
/// // Before:
/// Color { name: "color(42)".into(), color_type: ColorType::EightBit,
///         number: Some(42), triplet: None }
/// // After:
/// Color::EightBit(42)
///
/// // Before:
/// Color { name: "#ff0000".into(), color_type: ColorType::TrueColor,
///         number: None, triplet: Some(ColorTriplet::new(255, 0, 0)) }
/// // After:
/// Color::TrueColor(ColorTriplet::new(255, 0, 0))
///
/// // Before:                       // After:
/// color.name                       color.name()
/// color.color_type                 color.kind()
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// Terminal default color (foreground or background).
    Default,
    /// Standard 16-color ANSI palette (`number < 16`).
    Standard(u8),
    /// Extended 256-color (8-bit) palette (`number 16..=255`).
    EightBit(u8),
    /// 24-bit true-color RGB.
    TrueColor(ColorTriplet),
    /// Windows console legacy 16-color palette. Behaves identically to
    /// [`Color::Standard`] except resolution uses the Windows palette.
    Windows(u8),
}

impl Color {
    /// Parses a color string into a Color.
    ///
    /// Supports:
    /// - "default" - terminal default color
    /// - Named colors: "red", "bright_red", "yellow4", etc.
    /// - Hex: "#ff0000"
    /// - color(N): "color(100)"
    /// - RGB: "rgb(255,0,0)"
    pub fn parse(color: &str) -> Result<Color, ColorParseError> {
        // Check cache first — colors are heavily reused during render.
        {
            let mut guard = get_color_cache();
            if let Some(cache) = guard.as_mut() {
                if let Some(hit) = cache.get(color) {
                    return Ok(*hit);
                }
            }
        }

        let parsed = Self::parse_uncached(color)?;

        let mut guard = get_color_cache();
        if let Some(cache) = guard.as_mut() {
            cache.put(color.to_string(), parsed);
        }
        Ok(parsed)
    }

    fn parse_uncached(color: &str) -> Result<Color, ColorParseError> {
        let color_lower = color.to_lowercase();
        let color_trimmed = color_lower.trim();

        // Handle default
        if color_trimmed == "default" {
            return Ok(Color::default_color());
        }

        // Handle hex colors
        if let Some(hex) = color_trimmed.strip_prefix('#') {
            if hex.len() != 6 {
                return Err(ColorParseError::InvalidHexFormat(color.to_string()));
            }
            let triplet = parse_rgb_hex(hex)?;
            return Ok(Color::from_triplet(triplet));
        }

        // Handle color(N) format
        if color_trimmed.starts_with("color(") && color_trimmed.ends_with(')') {
            let number_str = &color_trimmed[6..color_trimmed.len() - 1];
            let number = number_str
                .parse::<u8>()
                .map_err(|_| ColorParseError::InvalidColorSpec(color.to_string()))?;
            return Ok(Color::from_ansi(number));
        }

        // Handle rgb(R,G,B) format
        if color_trimmed.starts_with("rgb(") && color_trimmed.ends_with(')') {
            let rgb_str = &color_trimmed[4..color_trimmed.len() - 1];
            let parts: Vec<&str> = rgb_str.split(',').collect();
            if parts.len() != 3 {
                return Err(ColorParseError::InvalidRgbFormat(color.to_string()));
            }

            let red = parts[0]
                .trim()
                .parse::<u8>()
                .map_err(|_| ColorParseError::ComponentOutOfRange(color.to_string()))?;
            let green = parts[1]
                .trim()
                .parse::<u8>()
                .map_err(|_| ColorParseError::ComponentOutOfRange(color.to_string()))?;
            let blue = parts[2]
                .trim()
                .parse::<u8>()
                .map_err(|_| ColorParseError::ComponentOutOfRange(color.to_string()))?;

            return Ok(Color::from_rgb(red, green, blue));
        }

        // Try to parse as a named color
        if let Some(number) = get_ansi_color_number(color_trimmed) {
            return Ok(Color::from_ansi(number));
        }

        Err(ColorParseError::UnknownColorName(color.to_string()))
    }

    /// Creates a Color from an 8-bit ANSI color number. Numbers `0..16` map
    /// to [`Color::Standard`]; `16..=255` map to [`Color::EightBit`].
    pub fn from_ansi(number: u8) -> Color {
        if number < 16 {
            Color::Standard(number)
        } else {
            Color::EightBit(number)
        }
    }

    /// Creates a Color from an RGB triplet.
    pub fn from_triplet(triplet: ColorTriplet) -> Color {
        Color::TrueColor(triplet)
    }

    /// Creates a Color from RGB components.
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Color {
        Color::TrueColor(ColorTriplet::new(red, green, blue))
    }

    /// Returns the default terminal color.
    pub fn default_color() -> Color {
        Color::Default
    }

    /// Human-readable name. Borrowed for known palette colors (no alloc),
    /// owned `format!("color({n})")` or hex for numbered/RGB colors.
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            Color::Default => Cow::Borrowed("default"),
            Color::Standard(n) | Color::Windows(n) => match ansi_color_name(*n) {
                Some(name) => Cow::Borrowed(name),
                None => Cow::Owned(format!("color({})", n)),
            },
            Color::EightBit(n) => Cow::Owned(format!("color({})", n)),
            Color::TrueColor(t) => Cow::Owned(t.hex()),
        }
    }

    /// Classification matching the legacy [`ColorType`] for callers that
    /// need to switch on the variant tag without exposing the inner data.
    pub fn kind(&self) -> ColorType {
        match self {
            Color::Default => ColorType::Default,
            Color::Standard(_) => ColorType::Standard,
            Color::EightBit(_) => ColorType::EightBit,
            Color::TrueColor(_) => ColorType::TrueColor,
            Color::Windows(_) => ColorType::Windows,
        }
    }

    /// The palette number, if applicable. `None` for `Default` and `TrueColor`.
    pub fn number(&self) -> Option<u8> {
        match self {
            Color::Standard(n) | Color::EightBit(n) | Color::Windows(n) => Some(*n),
            Color::Default | Color::TrueColor(_) => None,
        }
    }

    /// The RGB triplet, if this is a [`Color::TrueColor`].
    pub fn triplet(&self) -> Option<ColorTriplet> {
        match self {
            Color::TrueColor(t) => Some(*t),
            _ => None,
        }
    }

    /// Returns the native color system for this color.
    pub fn system(&self) -> ColorSystem {
        match self {
            Color::Default | Color::Standard(_) => ColorSystem::Standard,
            Color::EightBit(_) => ColorSystem::EightBit,
            Color::TrueColor(_) => ColorSystem::TrueColor,
            Color::Windows(_) => ColorSystem::Windows,
        }
    }

    /// Returns true if the color is system-defined (not 8-bit/truecolor).
    pub fn is_system_defined(&self) -> bool {
        matches!(
            self,
            Color::Default | Color::Standard(_) | Color::Windows(_)
        )
    }

    /// Returns true if this is the default color.
    pub fn is_default(&self) -> bool {
        matches!(self, Color::Default)
    }

    /// Resolves the color to an RGB triplet.
    ///
    /// # Arguments
    /// * `theme` - Optional theme to use for resolving system colors. If None, uses DEFAULT_TERMINAL_THEME.
    /// * `foreground` - Whether this is a foreground color (affects default color resolution).
    pub fn get_truecolor(&self, theme: Option<&TerminalTheme>, foreground: bool) -> ColorTriplet {
        let theme = theme.unwrap_or(&DEFAULT_TERMINAL_THEME);

        match self {
            Color::Default => {
                if foreground {
                    theme.foreground_color
                } else {
                    theme.background_color
                }
            }
            Color::Standard(n) | Color::Windows(n) => theme.ansi_colors.get(*n as usize),
            Color::EightBit(n) => EIGHT_BIT_PALETTE.get(*n as usize),
            Color::TrueColor(t) => *t,
        }
    }

    /// Gets the ANSI escape codes for this color as a `Vec<String>`.
    ///
    /// **Note:** the production render path uses [`Color::write_ansi_codes`]
    /// which writes directly into a pre-allocated buffer (no allocation).
    /// Prefer that method when calling per-segment.
    ///
    /// # Arguments
    /// * `foreground` - If true, returns foreground codes; otherwise background codes.
    pub fn get_ansi_codes(&self, foreground: bool) -> Vec<String> {
        let default = if foreground { "39" } else { "49" };
        match self {
            Color::Default => vec![default.to_string()],
            Color::Standard(n) | Color::Windows(n) => {
                let base = if foreground { 30u16 } else { 40 };
                let n = *n as u16;
                if n < 8 {
                    vec![format!("{}", base + n)]
                } else {
                    vec![format!("{}", base + 60 + (n - 8))]
                }
            }
            Color::EightBit(n) => {
                let prefix = if foreground { "38" } else { "48" };
                vec![prefix.to_string(), "5".to_string(), format!("{}", n)]
            }
            Color::TrueColor(t) => {
                let prefix = if foreground { "38" } else { "48" };
                vec![
                    prefix.to_string(),
                    "2".to_string(),
                    format!("{}", t.red),
                    format!("{}", t.green),
                    format!("{}", t.blue),
                ]
            }
        }
    }

    /// Writes ANSI escape codes for this color directly into a semicolon-separated
    /// SGR buffer, avoiding per-code String allocations.
    ///
    /// If `buf` is non-empty, a leading `;` separator is written first.
    pub fn write_ansi_codes(&self, foreground: bool, buf: &mut String) {
        // Helper: append separator if buffer is non-empty
        macro_rules! sep {
            ($b:expr) => {
                if !$b.is_empty() {
                    $b.push(';');
                }
            };
        }

        let default = if foreground { "39" } else { "49" };
        match self {
            Color::Default => {
                sep!(buf);
                buf.push_str(default);
            }
            Color::Standard(n) | Color::Windows(n) => {
                let base: u16 = if foreground { 30 } else { 40 };
                let n = *n as u16;
                sep!(buf);
                if n < 8 {
                    write!(buf, "{}", base + n).unwrap();
                } else {
                    write!(buf, "{}", base + 60 + (n - 8)).unwrap();
                }
            }
            Color::EightBit(n) => {
                sep!(buf);
                buf.push_str(if foreground { "38;5;" } else { "48;5;" });
                write!(buf, "{}", n).unwrap();
            }
            Color::TrueColor(t) => {
                sep!(buf);
                let prefix = if foreground { "38;2;" } else { "48;2;" };
                write!(buf, "{}{};{};{}", prefix, t.red, t.green, t.blue).unwrap();
            }
        }
    }

    /// Writes underline color codes (SGR 58) directly into a semicolon-separated
    /// SGR buffer. Converts foreground codes to underline-color equivalents.
    pub fn write_underline_color_codes(&self, buf: &mut String) {
        macro_rules! sep {
            ($b:expr) => {
                if !$b.is_empty() {
                    $b.push(';');
                }
            };
        }

        match self {
            Color::TrueColor(t) => {
                sep!(buf);
                write!(buf, "58;2;{};{};{}", t.red, t.green, t.blue).unwrap();
            }
            Color::EightBit(n) => {
                sep!(buf);
                write!(buf, "58;5;{}", n).unwrap();
            }
            Color::Standard(n) | Color::Windows(n) => {
                if *n < 16 {
                    sep!(buf);
                    write!(buf, "58;5;{}", n).unwrap();
                }
            }
            Color::Default => {}
        }
    }

    /// Downgrades the color to a lower color system.
    pub fn downgrade(&self, system: ColorSystem) -> Color {
        if matches!(self, Color::Default) {
            return *self;
        }

        // Fidelity order: Standard ≈ Windows (1) < EightBit (2) < TrueColor (3).
        // Windows shares fidelity level with Standard.
        fn fidelity(s: ColorSystem) -> u8 {
            match s {
                ColorSystem::Standard | ColorSystem::Windows => 1,
                ColorSystem::EightBit => 2,
                ColorSystem::TrueColor => 3,
            }
        }

        // Early-return: color is already at or below the target fidelity —
        // returning *self avoids a round-trip through the palette that can
        // silently change the index (audit #39).
        if fidelity(self.system()) <= fidelity(system) {
            return *self;
        }

        match system {
            ColorSystem::TrueColor => *self,
            ColorSystem::EightBit => match self {
                Color::TrueColor(triplet) => {
                    let (_h, l, s) = rgb_to_hls(triplet.normalized());
                    let color_number = if s < 0.15 {
                        // Grayscale
                        let gray = (l * 25.0).round() as u8;
                        if gray == 0 {
                            16
                        } else if gray == 25 {
                            231
                        } else {
                            231 + gray
                        }
                    } else {
                        // 6×6×6 cube
                        let red = triplet.red;
                        let green = triplet.green;
                        let blue = triplet.blue;

                        let six_red = if red < 95 {
                            red as f64 / 95.0
                        } else {
                            1.0 + (red - 95) as f64 / 40.0
                        };
                        let six_green = if green < 95 {
                            green as f64 / 95.0
                        } else {
                            1.0 + (green - 95) as f64 / 40.0
                        };
                        let six_blue = if blue < 95 {
                            blue as f64 / 95.0
                        } else {
                            1.0 + (blue - 95) as f64 / 40.0
                        };

                        16 + 36 * six_red.round() as u8
                            + 6 * six_green.round() as u8
                            + six_blue.round() as u8
                    };
                    Color::from_ansi(color_number)
                }
                _ => *self,
            },
            ColorSystem::Standard => {
                // Rich's `Color.downgrade(Standard)` always resolves the color
                // to its RGB triplet and nearest-matches against the 16-entry
                // STANDARD_PALETTE — it never short-circuits by index. For
                // EightBit colors the RGB comes from EIGHT_BIT_PALETTE[n];
                // because EIGHT_BIT_PALETTE[0-15] RGBs differ from
                // STANDARD_PALETTE[0-15] RGBs, the bright colors 8-15 do NOT
                // map to themselves (e.g. EightBit(8)=(128,128,128) ->
                // Standard(7)=(170,170,170)). Index-passthrough here would
                // produce Standard(8-15) which, while representable (bright
                // ANSI codes 90-97), is NOT what rich emits. (deep-review C1)
                let triplet = self.get_truecolor(None, true);
                let index = STANDARD_PALETTE.match_color(&triplet);
                Color::Standard(index as u8)
            }
            ColorSystem::Windows => {
                // EightBit(0-15) IS the Windows palette — pass through to
                // avoid an RGB round-trip that can silently change the index
                // (audit #39 residual, Phase 7).
                if let Color::EightBit(n) = self {
                    if *n < 16 {
                        return Color::Windows(*n);
                    }
                }
                let triplet = self.get_truecolor(None, true);
                let index = WINDOWS_PALETTE.match_color(&triplet);
                Color::Windows(index as u8)
            }
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Color('{}', ColorType::{:?}, number={})",
            self.name(),
            self.kind(),
            match self.number() {
                Some(n) => n.to_string(),
                None => "None".to_string(),
            }
        )
    }
}

// ---------------------------------------------------------------------------
// serde Serialize / Deserialize for Color (gated on `json` feature)
// ---------------------------------------------------------------------------
//
// `Color` serializes as its canonical string form (e.g. `"red"`, `"#ff8800"`,
// `"color(123)"`, `"default"`) and deserializes by calling `Color::parse`.
// A custom impl is needed because the enum variants have no stable 1-to-1
// mapping with serde's derive (EightBit and Standard both serialize as the
// same string form and must not be conflated).

#[cfg(feature = "json")]
mod color_serde {
    use super::Color;
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S: Serializer>(color: &Color, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&color.name())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
        struct ColorVisitor;
        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = Color;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "a color string like \"red\", \"#ff8800\", or \"color(42)\""
                )
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Color, E> {
                Color::parse(v).map_err(|e| de::Error::custom(e.to_string()))
            }
        }
        d.deserialize_str(ColorVisitor)
    }
}

#[cfg(feature = "json")]
impl serde::Serialize for Color {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        color_serde::serialize(self, s)
    }
}

#[cfg(feature = "json")]
impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        color_serde::deserialize(d)
    }
}

/// Parses a 6-character hex string into an RGB triplet.
pub fn parse_rgb_hex(hex: &str) -> Result<ColorTriplet, ColorParseError> {
    if hex.len() != 6 {
        return Err(ColorParseError::InvalidHexFormat(hex.to_string()));
    }

    let red = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| ColorParseError::InvalidHexFormat(hex.to_string()))?;
    let green = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| ColorParseError::InvalidHexFormat(hex.to_string()))?;
    let blue = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| ColorParseError::InvalidHexFormat(hex.to_string()))?;

    Ok(ColorTriplet::new(red, green, blue))
}

/// Blends two RGB colors using linear interpolation.
///
/// # Arguments
/// * `color1` - First color
/// * `color2` - Second color
/// * `cross_fade` - Blend factor (0.0 = color1, 1.0 = color2)
pub fn blend_rgb(color1: ColorTriplet, color2: ColorTriplet, cross_fade: f64) -> ColorTriplet {
    let r = (color1.red as f64 * (1.0 - cross_fade) + color2.red as f64 * cross_fade) as u8;
    let g = (color1.green as f64 * (1.0 - cross_fade) + color2.green as f64 * cross_fade) as u8;
    let b = (color1.blue as f64 * (1.0 - cross_fade) + color2.blue as f64 * cross_fade) as u8;
    ColorTriplet::new(r, g, b)
}

/// Converts RGB (normalized 0.0-1.0) to HLS.
///
/// Returns (hue, lightness, saturation) where:
/// - hue: 0.0-1.0
/// - lightness: 0.0-1.0
/// - saturation: 0.0-1.0
fn rgb_to_hls(rgb: (f64, f64, f64)) -> (f64, f64, f64) {
    let (r, g, b) = rgb;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        return (0.0, l, 0.0); // achromatic
    }

    let delta = max - min;
    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };

    let h = if max == r {
        (g - b) / delta + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };

    (h / 6.0, l, s)
}

/// Inverse of [`get_ansi_color_number`] for the 16 standard colors. Returns
/// the canonical name so `Color::name()` round-trips through `Color::parse`.
/// Numbers ≥ 16 don't have unique canonical names — use `format!("color({n})")`.
pub fn ansi_color_name(n: u8) -> Option<&'static str> {
    Some(match n {
        0 => "black",
        1 => "red",
        2 => "green",
        3 => "yellow",
        4 => "blue",
        5 => "magenta",
        6 => "cyan",
        7 => "white",
        8 => "bright_black",
        9 => "bright_red",
        10 => "bright_green",
        11 => "bright_yellow",
        12 => "bright_blue",
        13 => "bright_magenta",
        14 => "bright_cyan",
        15 => "bright_white",
        _ => return None,
    })
}

/// Gets the ANSI color number for a named color.
pub fn get_ansi_color_number(name: &str) -> Option<u8> {
    match name {
        "black" => Some(0),
        "red" => Some(1),
        "green" => Some(2),
        "yellow" => Some(3),
        "blue" => Some(4),
        "magenta" => Some(5),
        "cyan" => Some(6),
        "white" => Some(7),
        "bright_black" => Some(8),
        "bright_red" => Some(9),
        "bright_green" => Some(10),
        "bright_yellow" => Some(11),
        "bright_blue" => Some(12),
        "bright_magenta" => Some(13),
        "bright_cyan" => Some(14),
        "bright_white" => Some(15),
        "grey0" | "gray0" => Some(16),
        "navy_blue" => Some(17),
        "dark_blue" => Some(18),
        "blue3" => Some(20),
        "blue1" => Some(21),
        "dark_green" => Some(22),
        "deep_sky_blue4" => Some(25),
        "dodger_blue3" => Some(26),
        "dodger_blue2" => Some(27),
        "green4" => Some(28),
        "spring_green4" => Some(29),
        "turquoise4" => Some(30),
        "deep_sky_blue3" => Some(32),
        "dodger_blue1" => Some(33),
        "green3" => Some(40),
        "spring_green3" => Some(41),
        "dark_cyan" => Some(36),
        "light_sea_green" => Some(37),
        "deep_sky_blue2" => Some(38),
        "deep_sky_blue1" => Some(39),
        "spring_green2" => Some(47),
        "cyan3" => Some(43),
        "dark_turquoise" => Some(44),
        "turquoise2" => Some(45),
        "green1" => Some(46),
        "spring_green1" => Some(48),
        "medium_spring_green" => Some(49),
        "cyan2" => Some(50),
        "cyan1" => Some(51),
        "dark_red" => Some(88),
        "deep_pink4" => Some(125),
        "purple4" => Some(55),
        "purple3" => Some(56),
        "blue_violet" => Some(57),
        "orange4" => Some(94),
        "grey37" | "gray37" => Some(59),
        "medium_purple4" => Some(60),
        "slate_blue3" => Some(62),
        "royal_blue1" => Some(63),
        "chartreuse4" => Some(64),
        "dark_sea_green4" => Some(71),
        "pale_turquoise4" => Some(66),
        "steel_blue" => Some(67),
        "steel_blue3" => Some(68),
        "cornflower_blue" => Some(69),
        "chartreuse3" => Some(76),
        "cadet_blue" => Some(73),
        "sky_blue3" => Some(74),
        "steel_blue1" => Some(81),
        "pale_green3" => Some(114),
        "sea_green3" => Some(78),
        "aquamarine3" => Some(79),
        "medium_turquoise" => Some(80),
        "chartreuse2" => Some(112),
        "sea_green2" => Some(83),
        "sea_green1" => Some(85),
        "aquamarine1" => Some(122),
        "dark_slate_gray2" => Some(87),
        "dark_magenta" => Some(91),
        "dark_violet" => Some(128),
        "purple" => Some(129),
        "light_pink4" => Some(95),
        "plum4" => Some(96),
        "medium_purple3" => Some(98),
        "slate_blue1" => Some(99),
        "yellow4" => Some(106),
        "wheat4" => Some(101),
        "grey53" | "gray53" => Some(102),
        "light_slate_grey" | "light_slate_gray" => Some(103),
        "medium_purple" => Some(104),
        "light_slate_blue" => Some(105),
        "dark_olive_green3" => Some(149),
        "dark_sea_green" => Some(108),
        "light_sky_blue3" => Some(110),
        "sky_blue2" => Some(111),
        "dark_sea_green3" => Some(150),
        "dark_slate_gray3" => Some(116),
        "sky_blue1" => Some(117),
        "chartreuse1" => Some(118),
        "light_green" => Some(120),
        "pale_green1" => Some(156),
        "dark_slate_gray1" => Some(123),
        "red3" => Some(160),
        "medium_violet_red" => Some(126),
        "magenta3" => Some(164),
        "dark_orange3" => Some(166),
        "indian_red" => Some(167),
        "hot_pink3" => Some(168),
        "medium_orchid3" => Some(133),
        "medium_orchid" => Some(134),
        "medium_purple2" => Some(140),
        "dark_goldenrod" => Some(136),
        "light_salmon3" => Some(173),
        "rosy_brown" => Some(138),
        "grey63" | "gray63" => Some(139),
        "medium_purple1" => Some(141),
        "gold3" => Some(178),
        "dark_khaki" => Some(143),
        "navajo_white3" => Some(144),
        "grey69" | "gray69" => Some(145),
        "light_steel_blue3" => Some(146),
        "light_steel_blue" => Some(147),
        "yellow3" => Some(184),
        "dark_sea_green2" => Some(157),
        "light_cyan3" => Some(152),
        "light_sky_blue1" => Some(153),
        "green_yellow" => Some(154),
        "dark_olive_green2" => Some(155),
        "dark_sea_green1" => Some(193),
        "pale_turquoise1" => Some(159),
        "deep_pink3" => Some(162),
        "magenta2" => Some(200),
        "hot_pink2" => Some(169),
        "orchid" => Some(170),
        "medium_orchid1" => Some(207),
        "orange3" => Some(172),
        "light_pink3" => Some(174),
        "pink3" => Some(175),
        "plum3" => Some(176),
        "violet" => Some(177),
        "light_goldenrod3" => Some(179),
        "tan" => Some(180),
        "misty_rose3" => Some(181),
        "thistle3" => Some(182),
        "plum2" => Some(183),
        "khaki3" => Some(185),
        "light_goldenrod2" => Some(222),
        "light_yellow3" => Some(187),
        "grey84" | "gray84" => Some(188),
        "light_steel_blue1" => Some(189),
        "yellow2" => Some(190),
        "dark_olive_green1" => Some(192),
        "honeydew2" => Some(194),
        "light_cyan1" => Some(195),
        "red1" => Some(196),
        "deep_pink2" => Some(197),
        "deep_pink1" => Some(199),
        "magenta1" => Some(201),
        "orange_red1" => Some(202),
        "indian_red1" => Some(204),
        "hot_pink" => Some(206),
        "dark_orange" => Some(208),
        "salmon1" => Some(209),
        "light_coral" => Some(210),
        "pale_violet_red1" => Some(211),
        "orchid2" => Some(212),
        "orchid1" => Some(213),
        "orange1" => Some(214),
        "sandy_brown" => Some(215),
        "light_salmon1" => Some(216),
        "light_pink1" => Some(217),
        "pink1" => Some(218),
        "plum1" => Some(219),
        "gold1" => Some(220),
        "navajo_white1" => Some(223),
        "misty_rose1" => Some(224),
        "thistle1" => Some(225),
        "yellow1" => Some(226),
        "light_goldenrod1" => Some(227),
        "khaki1" => Some(228),
        "wheat1" => Some(229),
        "cornsilk1" => Some(230),
        "grey100" | "gray100" => Some(231),
        "grey3" | "gray3" => Some(232),
        "grey7" | "gray7" => Some(233),
        "grey11" | "gray11" => Some(234),
        "grey15" | "gray15" => Some(235),
        "grey19" | "gray19" => Some(236),
        "grey23" | "gray23" => Some(237),
        "grey27" | "gray27" => Some(238),
        "grey30" | "gray30" => Some(239),
        "grey35" | "gray35" => Some(240),
        "grey39" | "gray39" => Some(241),
        "grey42" | "gray42" => Some(242),
        "grey46" | "gray46" => Some(243),
        "grey50" | "gray50" => Some(244),
        "grey54" | "gray54" => Some(245),
        "grey58" | "gray58" => Some(246),
        "grey62" | "gray62" => Some(247),
        "grey66" | "gray66" => Some(248),
        "grey70" | "gray70" => Some(249),
        "grey74" | "gray74" => Some(250),
        "grey78" | "gray78" => Some(251),
        "grey82" | "gray82" => Some(252),
        "grey85" | "gray85" => Some(253),
        "grey89" | "gray89" => Some(254),
        "grey93" | "gray93" => Some(255),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Parse tests
    #[test]
    fn test_parse_default() {
        let color = Color::parse("default").unwrap();
        assert_eq!(color.name(), "default");
        assert_eq!(color.kind(), ColorType::Default);
        assert_eq!(color.number(), None);
        assert_eq!(color.triplet(), None);
    }

    #[test]
    fn test_parse_red() {
        let color = Color::parse("red").unwrap();
        assert_eq!(color.name(), "red");
        assert_eq!(color.kind(), ColorType::Standard);
        assert_eq!(color.number(), Some(1));
        assert_eq!(color.triplet(), None);
    }

    #[test]
    fn test_parse_bright_red() {
        let color = Color::parse("bright_red").unwrap();
        assert_eq!(color.name(), "bright_red");
        assert_eq!(color.kind(), ColorType::Standard);
        assert_eq!(color.number(), Some(9));
        assert_eq!(color.triplet(), None);
    }

    #[test]
    fn test_parse_yellow4() {
        let color = Color::parse("yellow4").unwrap();
        // L1 enum break (v0.11.0): EightBit colors no longer round-trip
        // their named form ("yellow4" → "color(106)"). Only the 16
        // standard colors get canonical names from `ansi_color_name`.
        assert_eq!(color.name(), "color(106)");
        assert_eq!(color.kind(), ColorType::EightBit);
        assert_eq!(color.number(), Some(106));
        assert_eq!(color.triplet(), None);
    }

    #[test]
    fn test_parse_color_100() {
        let color = Color::parse("color(100)").unwrap();
        assert_eq!(color.name(), "color(100)");
        assert_eq!(color.kind(), ColorType::EightBit);
        assert_eq!(color.number(), Some(100));
        assert_eq!(color.triplet(), None);
    }

    #[test]
    fn test_parse_hex() {
        let color = Color::parse("#112233").unwrap();
        assert_eq!(color.name(), "#112233");
        assert_eq!(color.kind(), ColorType::TrueColor);
        assert_eq!(color.number(), None);
        assert_eq!(color.triplet(), Some(ColorTriplet::new(0x11, 0x22, 0x33)));
    }

    #[test]
    fn test_parse_rgb() {
        let color = Color::parse("rgb(90,100,110)").unwrap();
        assert_eq!(color.name(), "#5a646e");
        assert_eq!(color.kind(), ColorType::TrueColor);
        assert_eq!(color.number(), None);
        assert_eq!(color.triplet(), Some(ColorTriplet::new(90, 100, 110)));
    }

    // Parse error tests
    #[test]
    fn test_parse_error_color_256() {
        let result = Color::parse("color(256)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_invalid_rgb() {
        let result = Color::parse("rgb(999,0,0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_rgb_missing_component() {
        let result = Color::parse("rgb(0,0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unknown_color() {
        let result = Color::parse("nosuchcolor");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_invalid_hex() {
        let result = Color::parse("#xxyyzz");
        assert!(result.is_err());
    }

    // from_triplet tests
    #[test]
    fn test_from_triplet() {
        let color = Color::from_triplet(ColorTriplet::new(0x10, 0x20, 0x30));
        assert_eq!(color.name(), "#102030");
        assert_eq!(color.kind(), ColorType::TrueColor);
    }

    // from_ansi tests
    #[test]
    fn test_from_ansi_standard() {
        let color = Color::from_ansi(1);
        assert_eq!(color.kind(), ColorType::Standard);
        assert_eq!(color.number(), Some(1));
    }

    #[test]
    fn test_from_ansi_eightbit() {
        let color = Color::from_ansi(100);
        assert_eq!(color.kind(), ColorType::EightBit);
        assert_eq!(color.number(), Some(100));
    }

    // get_ansi_codes tests
    #[test]
    fn test_get_ansi_codes_default_foreground() {
        let color = Color::parse("default").unwrap();
        assert_eq!(color.get_ansi_codes(true), vec!["39"]);
    }

    #[test]
    fn test_get_ansi_codes_default_background() {
        let color = Color::parse("default").unwrap();
        assert_eq!(color.get_ansi_codes(false), vec!["49"]);
    }

    #[test]
    fn test_get_ansi_codes_red_foreground() {
        let color = Color::parse("red").unwrap();
        assert_eq!(color.get_ansi_codes(true), vec!["31"]);
    }

    #[test]
    fn test_get_ansi_codes_red_background() {
        let color = Color::parse("red").unwrap();
        assert_eq!(color.get_ansi_codes(false), vec!["41"]);
    }

    #[test]
    fn test_get_ansi_codes_bright_red_foreground() {
        let color = Color::parse("bright_red").unwrap();
        assert_eq!(color.get_ansi_codes(true), vec!["91"]);
    }

    #[test]
    fn test_get_ansi_codes_truecolor_foreground() {
        let color = Color::parse("#ff0000").unwrap();
        assert_eq!(color.get_ansi_codes(true), vec!["38", "2", "255", "0", "0"]);
    }

    #[test]
    fn test_get_ansi_codes_truecolor_background() {
        let color = Color::parse("#ff0000").unwrap();
        assert_eq!(
            color.get_ansi_codes(false),
            vec!["48", "2", "255", "0", "0"]
        );
    }

    #[test]
    fn test_get_ansi_codes_eightbit_foreground() {
        let color = Color::parse("color(100)").unwrap();
        assert_eq!(color.get_ansi_codes(true), vec!["38", "5", "100"]);
    }

    // get_truecolor tests
    #[test]
    fn test_get_truecolor_hex() {
        let color = Color::parse("#ff0000").unwrap();
        assert_eq!(
            color.get_truecolor(None, true),
            ColorTriplet::new(255, 0, 0)
        );
    }

    #[test]
    fn test_get_truecolor_red() {
        let color = Color::parse("red").unwrap();
        assert_eq!(
            color.get_truecolor(None, true),
            ColorTriplet::new(128, 0, 0)
        );
    }

    #[test]
    fn test_get_truecolor_default_foreground() {
        let color = Color::parse("default").unwrap();
        assert_eq!(color.get_truecolor(None, true), ColorTriplet::new(0, 0, 0));
    }

    #[test]
    fn test_get_truecolor_default_background() {
        let color = Color::parse("default").unwrap();
        assert_eq!(
            color.get_truecolor(None, false),
            ColorTriplet::new(255, 255, 255)
        );
    }

    // downgrade tests
    #[test]
    fn test_downgrade_black_to_eightbit() {
        let color = Color::parse("#000000").unwrap();
        let downgraded = color.downgrade(ColorSystem::EightBit);
        assert_eq!(downgraded.number(), Some(16));
    }

    #[test]
    fn test_downgrade_white_to_eightbit() {
        let color = Color::parse("#ffffff").unwrap();
        let downgraded = color.downgrade(ColorSystem::EightBit);
        assert_eq!(downgraded.number(), Some(231));
    }

    #[test]
    fn test_downgrade_red_to_eightbit() {
        let color = Color::parse("#ff0000").unwrap();
        let downgraded = color.downgrade(ColorSystem::EightBit);
        assert_eq!(downgraded.number(), Some(196));
    }

    #[test]
    fn test_downgrade_red_to_standard() {
        let color = Color::parse("#ff0000").unwrap();
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.number(), Some(1));
    }

    #[test]
    fn test_downgrade_green_to_standard() {
        let color = Color::parse("#00ff00").unwrap();
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.number(), Some(2));
    }

    #[test]
    fn test_downgrade_color_20_to_standard() {
        let color = Color::parse("color(20)").unwrap();
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.number(), Some(4));
    }

    // blend_rgb tests
    #[test]
    fn test_blend_rgb() {
        let result = blend_rgb(
            ColorTriplet::new(10, 20, 30),
            ColorTriplet::new(30, 40, 50),
            0.5,
        );
        assert_eq!(result, ColorTriplet::new(20, 30, 40));
    }

    /// Python rich uses `int()` (truncation, not rounding) for blend.
    /// A 50% blend of black (0) and white (255) must yield 127 (truncated),
    /// not 128 (rounded).
    #[test]
    fn test_blend_rgb_truncates_not_rounds() {
        let black = ColorTriplet::new(0, 0, 0);
        let white = ColorTriplet::new(255, 255, 255);
        let result = blend_rgb(black, white, 0.5);
        assert_eq!(result, ColorTriplet::new(127, 127, 127));
    }

    #[test]
    fn test_blend_rgb_zero() {
        let result = blend_rgb(
            ColorTriplet::new(10, 20, 30),
            ColorTriplet::new(30, 40, 50),
            0.0,
        );
        assert_eq!(result, ColorTriplet::new(10, 20, 30));
    }

    #[test]
    fn test_blend_rgb_one() {
        let result = blend_rgb(
            ColorTriplet::new(10, 20, 30),
            ColorTriplet::new(30, 40, 50),
            1.0,
        );
        assert_eq!(result, ColorTriplet::new(30, 40, 50));
    }

    // parse_rgb_hex tests
    #[test]
    fn test_parse_rgb_hex() {
        let result = parse_rgb_hex("aabbcc").unwrap();
        assert_eq!(result, ColorTriplet::new(0xaa, 0xbb, 0xcc));
    }

    #[test]
    fn test_parse_rgb_hex_lowercase() {
        let result = parse_rgb_hex("ffffff").unwrap();
        assert_eq!(result, ColorTriplet::new(255, 255, 255));
    }

    #[test]
    fn test_parse_rgb_hex_uppercase() {
        let result = parse_rgb_hex("AABBCC").unwrap();
        assert_eq!(result, ColorTriplet::new(0xaa, 0xbb, 0xcc));
    }

    #[test]
    fn test_parse_rgb_hex_invalid_length() {
        let result = parse_rgb_hex("aabb");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rgb_hex_invalid_chars() {
        let result = parse_rgb_hex("gghhii");
        assert!(result.is_err());
    }

    // System and query tests
    #[test]
    fn test_system() {
        assert_eq!(
            Color::parse("default").unwrap().system(),
            ColorSystem::Standard
        );
        assert_eq!(Color::parse("red").unwrap().system(), ColorSystem::Standard);
        assert_eq!(
            Color::parse("color(100)").unwrap().system(),
            ColorSystem::EightBit
        );
        assert_eq!(
            Color::parse("#ff0000").unwrap().system(),
            ColorSystem::TrueColor
        );
    }

    #[test]
    fn test_is_system_defined() {
        assert!(Color::parse("default").unwrap().is_system_defined());
        assert!(Color::parse("red").unwrap().is_system_defined());
        assert!(!Color::parse("color(100)").unwrap().is_system_defined());
        assert!(!Color::parse("#ff0000").unwrap().is_system_defined());
    }

    #[test]
    fn test_is_default() {
        assert!(Color::parse("default").unwrap().is_default());
        assert!(!Color::parse("red").unwrap().is_default());
        assert!(!Color::parse("#ff0000").unwrap().is_default());
    }

    // Display trait test
    #[test]
    fn test_display_trait() {
        let color = Color::parse("red").unwrap();
        let display = format!("{}", color);
        assert!(display.contains("red"));
        assert!(display.contains("ColorType::Standard"));
        assert!(display.contains("number=1"));
    }

    #[test]
    fn test_display_trait_no_number() {
        let color = Color::parse("#ff0000").unwrap();
        let display = format!("{}", color);
        assert!(display.contains("#ff0000"));
        assert!(display.contains("ColorType::TrueColor"));
        assert!(display.contains("number=None"));
    }

    // RGB to HLS tests
    #[test]
    fn test_rgb_to_hls_black() {
        let (_h, l, s) = rgb_to_hls((0.0, 0.0, 0.0));
        assert_eq!(l, 0.0);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn test_rgb_to_hls_white() {
        let (_h, l, s) = rgb_to_hls((1.0, 1.0, 1.0));
        assert_eq!(l, 1.0);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn test_rgb_to_hls_gray() {
        let (_h, l, s) = rgb_to_hls((0.5, 0.5, 0.5));
        assert_eq!(l, 0.5);
        assert_eq!(s, 0.0);
    }

    // Named color tests
    #[test]
    fn test_parse_case_insensitive() {
        let color1 = Color::parse("RED").unwrap();
        let color2 = Color::parse("red").unwrap();
        let color3 = Color::parse("Red").unwrap();
        assert_eq!(color1.number(), Some(1));
        assert_eq!(color2.number(), Some(1));
        assert_eq!(color3.number(), Some(1));
    }

    #[test]
    fn test_parse_grey_gray_alias() {
        let grey = Color::parse("grey0").unwrap();
        let gray = Color::parse("gray0").unwrap();
        assert_eq!(grey.number(), gray.number());
        assert_eq!(grey.number(), Some(16));
    }

    // Additional edge cases
    #[test]
    fn test_parse_color_15() {
        let color = Color::parse("color(15)").unwrap();
        assert_eq!(color.kind(), ColorType::Standard);
        assert_eq!(color.number(), Some(15));
    }

    #[test]
    fn test_parse_color_16() {
        let color = Color::parse("color(16)").unwrap();
        assert_eq!(color.kind(), ColorType::EightBit);
        assert_eq!(color.number(), Some(16));
    }

    #[test]
    fn test_downgrade_default() {
        let color = Color::default_color();
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.kind(), ColorType::Default);
    }

    #[test]
    fn downgrade_to_same_system_is_identity() {
        // A Standard color downgraded to Standard must NOT be re-matched through
        // the palette (which can change the index). It must return itself.
        let c = Color::Standard(5);
        assert_eq!(c.downgrade(ColorSystem::Standard), Color::Standard(5));
        // Discriminating case (audit #39): Standard(8) theme-RGB=(128,128,128)
        // is closer to palette index 7=(170,170,170) than to index 8=(85,85,85)
        // under the redmean metric — without the early-return it returns Standard(7).
        assert_eq!(
            Color::Standard(8).downgrade(ColorSystem::Standard),
            Color::Standard(8)
        );
        // EightBit -> EightBit identity.
        assert_eq!(
            Color::EightBit(123).downgrade(ColorSystem::EightBit),
            Color::EightBit(123)
        );
        // EightBit -> Standard still downgrades (different system, lower fidelity).
        // (no identity assertion here — just must not panic and must be Standard)
        assert!(matches!(
            Color::EightBit(123).downgrade(ColorSystem::Standard),
            Color::Standard(_)
        ));
    }
    #[test]
    fn eightbit_low_passthrough_to_windows() {
        // EightBit(0-15) ARE the Windows palette — downgrade to Windows
        // passes through directly to Windows(n) instead of round-tripping
        // through RGB nearest-match (audit #39 residual). Windows has 16
        // entries, so the full 0-15 range is valid.
        for n in 0u8..16 {
            assert_eq!(
                Color::EightBit(n).downgrade(ColorSystem::Windows),
                Color::Windows(n),
                "EightBit({n}) must pass through to Windows({n}) on downgrade",
            );
        }
    }

    /// Deep-review C1: `EightBit(0-15).downgrade(Standard)` must NOT
    /// passthrough to `Standard(n)`. Rich's `Color.downgrade` always resolves
    /// the EightBit color to its RGB triplet (via `EIGHT_BIT_PALETTE`) and
    /// then nearest-matches against the 16-entry `STANDARD_PALETTE` — it never
    /// short-circuits by index. Because gilt's `EIGHT_BIT_PALETTE[0-15]` RGBs
    /// differ from `STANDARD_PALETTE[0-15]` RGBs (e.g. EightBit(1)=(128,0,0)
    /// vs Standard(1)=(170,0,0)), the nearest-match is NOT identity for the
    /// bright colors 8-15. The expected indices below are computed by the same
    /// redmean nearest-match the code uses for all other EightBit colors.
    #[test]
    fn eightbit_bright_downgrade_to_standard_uses_nearest_match() {
        // Indices 0-7 are dark colors; their EightBit RGBs are close enough
        // to the corresponding Standard palette entry that nearest-match is
        // identity (verified: 0->0, 1->1, ..., 7->7).
        for n in 0u8..8 {
            assert_eq!(
                Color::EightBit(n).downgrade(ColorSystem::Standard),
                Color::Standard(n),
                "EightBit({n}) dark color nearest-matches to Standard({n})",
            );
        }
        // Bright colors 8-15: nearest-match into the 16-entry Standard palette.
        // These are the rich-correct values (redmean distance, computed against
        // gilt's EIGHT_BIT_PALETTE and STANDARD_PALETTE).
        let expected: &[(u8, u8)] = &[
            (8, 7),   // (128,128,128) -> (170,170,170)
            (9, 1),   // (255,0,0)     -> (170,0,0)
            (10, 2),  // (0,255,0)     -> (0,170,0)
            (11, 11), // (255,255,0)   -> (255,255,85)
            (12, 4),  // (0,0,255)     -> (0,0,170)
            (13, 13), // (255,0,255)   -> (255,85,255)
            (14, 14), // (0,255,255)   -> (85,255,255)
            (15, 15), // (255,255,255) -> (255,255,255)
        ];
        for &(src, dst) in expected {
            assert_eq!(
                Color::EightBit(src).downgrade(ColorSystem::Standard),
                Color::Standard(dst),
                "EightBit({src}) must nearest-match to Standard({dst}), not Standard({src})",
            );
        }
    }
    #[test]
    fn ansi_color_name_and_number_round_trip() {
        // Task 2 (Phase 7): `ansi_color_name` and `get_ansi_color_number`
        // must be public accessors for ANSI color name<->number lookups.
        use crate::color::{ansi_color_name, get_ansi_color_number};
        // The first 16 standard colors (per rich's ANSI_COLOR_NAMES table).
        for n in 0u8..16 {
            let name =
                ansi_color_name(n).unwrap_or_else(|| panic!("ansi_color_name({n}) must be Some"));
            assert_eq!(
                get_ansi_color_number(name),
                Some(n),
                "round-trip failed for n={n} (name={name})",
            );
        }
        // Spot-check the canonical name for index 1.
        assert_eq!(ansi_color_name(1), Some("red"));
        // Out-of-range (n >= 16) has no canonical name.
        assert_eq!(ansi_color_name(16), None);
        assert_eq!(ansi_color_name(255), None);
        // Unknown name returns None.
        assert_eq!(get_ansi_color_number("not_a_real_ansi_name"), None);
    }
}

// ============================================================================
// LRU Cache for Color Parsing
// ============================================================================

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// Global LRU cache for parsed colors with capacity for 1024 entries.
static COLOR_CACHE: Mutex<Option<LruCache<String, Color>>> = Mutex::new(None);

/// Lazily initialize and return the cache guard.
///
/// Recovers from a poisoned mutex: the cache is purely a parse accelerator
/// and contains no invariants that a previous-thread panic could corrupt.
fn get_color_cache() -> std::sync::MutexGuard<'static, Option<LruCache<String, Color>>> {
    let mut cache = COLOR_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if cache.is_none() {
        *cache = Some(LruCache::new(NonZeroUsize::new(1024).unwrap()));
    }
    cache
}

/// Clears the global color cache.
pub fn clear_color_cache() {
    let mut cache = COLOR_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *cache = None;
}

/// Returns the current number of entries in the color cache.
pub fn color_cache_size() -> usize {
    let cache = COLOR_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache.as_ref().map(|c| c.len()).unwrap_or(0)
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    use std::sync::Mutex;
    static CACHE_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn color_cache_populates_on_parse() {
        // Cache is process-global; serialise across tests that mutate it.
        let _g = CACHE_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        clear_color_cache();
        let before = color_cache_size();
        let _ = Color::parse("color(199)").unwrap();
        let _ = Color::parse("#abcdef").unwrap();
        let _ = Color::parse("color(98)").unwrap();
        assert!(color_cache_size() >= before + 3);
    }

    #[test]
    fn color_cache_returns_equivalent_value() {
        let _g = CACHE_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        clear_color_cache();
        let first = Color::parse("blue").unwrap();
        let second = Color::parse("blue").unwrap();
        assert_eq!(first.name(), second.name());
        assert_eq!(first.number(), second.number());
    }
}
