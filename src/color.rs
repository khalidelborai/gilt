//! Terminal color representation and manipulation.
//!
//! This module provides the Color type that represents terminal colors,
//! supporting different color systems (standard 16, 8-bit 256, truecolor).

use crate::color_triplet::ColorTriplet;
use crate::errors::ColorParseError;
use crate::palette::{EIGHT_BIT_PALETTE, STANDARD_PALETTE, WINDOWS_PALETTE};
use crate::terminal_theme::{TerminalTheme, DEFAULT_TERMINAL_THEME};
use std::fmt;

/// Color system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ColorSystem {
    Standard = 1,
    EightBit = 2,
    TrueColor = 3,
    Windows = 4,
}

/// Color type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ColorType {
    Default = 0,
    Standard = 1,
    EightBit = 2,
    TrueColor = 3,
    Windows = 4,
}

/// A terminal color representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Color {
    pub name: String,
    pub color_type: ColorType,
    pub number: Option<u8>,
    pub triplet: Option<ColorTriplet>,
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
            let color_type = if number < 16 {
                ColorType::Standard
            } else {
                ColorType::EightBit
            };
            return Ok(Color {
                name: color_trimmed.to_string(),
                color_type,
                number: Some(number),
                triplet: None,
            });
        }

        Err(ColorParseError::UnknownColorName(color.to_string()))
    }

    /// Creates a Color from an 8-bit ANSI color number.
    pub fn from_ansi(number: u8) -> Color {
        let color_type = if number < 16 {
            ColorType::Standard
        } else {
            ColorType::EightBit
        };
        Color {
            name: format!("color({})", number),
            color_type,
            number: Some(number),
            triplet: None,
        }
    }

    /// Creates a Color from an RGB triplet.
    pub fn from_triplet(triplet: ColorTriplet) -> Color {
        Color {
            name: triplet.hex(),
            color_type: ColorType::TrueColor,
            number: None,
            triplet: Some(triplet),
        }
    }

    /// Creates a Color from RGB components.
    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Color {
        Color::from_triplet(ColorTriplet::new(red, green, blue))
    }

    /// Returns the default terminal color.
    pub fn default_color() -> Color {
        Color {
            name: "default".to_string(),
            color_type: ColorType::Default,
            number: None,
            triplet: None,
        }
    }

    /// Returns the native color system for this color.
    pub fn system(&self) -> ColorSystem {
        match self.color_type {
            ColorType::Default => ColorSystem::Standard,
            ColorType::Standard => ColorSystem::Standard,
            ColorType::EightBit => ColorSystem::EightBit,
            ColorType::TrueColor => ColorSystem::TrueColor,
            ColorType::Windows => ColorSystem::Windows,
        }
    }

    /// Returns true if the color is system-defined (not 8-bit/truecolor).
    pub fn is_system_defined(&self) -> bool {
        matches!(
            self.color_type,
            ColorType::Default | ColorType::Standard | ColorType::Windows
        )
    }

    /// Returns true if this is the default color.
    pub fn is_default(&self) -> bool {
        self.color_type == ColorType::Default
    }

    /// Resolves the color to an RGB triplet.
    ///
    /// # Arguments
    /// * `theme` - Optional theme to use for resolving system colors. If None, uses DEFAULT_TERMINAL_THEME.
    /// * `foreground` - Whether this is a foreground color (affects default color resolution).
    pub fn get_truecolor(&self, theme: Option<&TerminalTheme>, foreground: bool) -> ColorTriplet {
        let theme = theme.unwrap_or(&DEFAULT_TERMINAL_THEME);

        match self.color_type {
            ColorType::Default => {
                if foreground {
                    theme.foreground_color
                } else {
                    theme.background_color
                }
            }
            ColorType::Standard | ColorType::Windows => {
                if let Some(number) = self.number {
                    theme.ansi_colors.get(number as usize)
                } else {
                    theme.foreground_color
                }
            }
            ColorType::EightBit => {
                if let Some(number) = self.number {
                    EIGHT_BIT_PALETTE.get(number as usize)
                } else {
                    theme.foreground_color
                }
            }
            ColorType::TrueColor => self.triplet.unwrap_or(theme.foreground_color),
        }
    }

    /// Gets the ANSI escape codes for this color.
    ///
    /// # Arguments
    /// * `foreground` - If true, returns foreground codes; otherwise background codes.
    pub fn get_ansi_codes(&self, foreground: bool) -> Vec<String> {
        match self.color_type {
            ColorType::Default => {
                vec![if foreground {
                    "39".to_string()
                } else {
                    "49".to_string()
                }]
            }
            ColorType::Standard | ColorType::Windows => {
                if let Some(number) = self.number {
                    let base = if foreground { 30 } else { 40 };
                    if number < 8 {
                        vec![format!("{}", base + number)]
                    } else {
                        vec![format!("{}", base + 60 + (number - 8))]
                    }
                } else {
                    vec![if foreground {
                        "39".to_string()
                    } else {
                        "49".to_string()
                    }]
                }
            }
            ColorType::EightBit => {
                if let Some(number) = self.number {
                    let prefix = if foreground { "38" } else { "48" };
                    vec![prefix.to_string(), "5".to_string(), format!("{}", number)]
                } else {
                    vec![if foreground {
                        "39".to_string()
                    } else {
                        "49".to_string()
                    }]
                }
            }
            ColorType::TrueColor => {
                if let Some(triplet) = self.triplet {
                    let prefix = if foreground { "38" } else { "48" };
                    vec![
                        prefix.to_string(),
                        "2".to_string(),
                        format!("{}", triplet.red),
                        format!("{}", triplet.green),
                        format!("{}", triplet.blue),
                    ]
                } else {
                    vec![if foreground {
                        "39".to_string()
                    } else {
                        "49".to_string()
                    }]
                }
            }
        }
    }

    /// Downgrades the color to a lower color system.
    pub fn downgrade(&self, system: ColorSystem) -> Color {
        if self.color_type == ColorType::Default {
            return self.clone();
        }

        match system {
            ColorSystem::TrueColor => self.clone(),
            ColorSystem::EightBit => {
                if self.color_type == ColorType::TrueColor {
                    if let Some(triplet) = self.triplet {
                        // Downgrade truecolor to 8-bit
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
                    } else {
                        self.clone()
                    }
                } else {
                    self.clone()
                }
            }
            ColorSystem::Standard => {
                let triplet = self.get_truecolor(None, true);
                let index = STANDARD_PALETTE.match_color(&triplet);
                Color {
                    name: format!("color({})", index),
                    color_type: ColorType::Standard,
                    number: Some(index as u8),
                    triplet: None,
                }
            }
            ColorSystem::Windows => {
                let triplet = self.get_truecolor(None, true);
                let index = WINDOWS_PALETTE.match_color(&triplet);
                Color {
                    name: format!("color({})", index),
                    color_type: ColorType::Windows,
                    number: Some(index as u8),
                    triplet: None,
                }
            }
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Color('{}', ColorType::{:?}, number={})",
            self.name,
            self.color_type,
            match self.number {
                Some(n) => n.to_string(),
                None => "None".to_string(),
            }
        )
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
    let r = (color1.red as f64 * (1.0 - cross_fade) + color2.red as f64 * cross_fade).round() as u8;
    let g =
        (color1.green as f64 * (1.0 - cross_fade) + color2.green as f64 * cross_fade).round() as u8;
    let b =
        (color1.blue as f64 * (1.0 - cross_fade) + color2.blue as f64 * cross_fade).round() as u8;
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

/// Gets the ANSI color number for a named color.
fn get_ansi_color_number(name: &str) -> Option<u8> {
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
        assert_eq!(color.name, "default");
        assert_eq!(color.color_type, ColorType::Default);
        assert_eq!(color.number, None);
        assert_eq!(color.triplet, None);
    }

    #[test]
    fn test_parse_red() {
        let color = Color::parse("red").unwrap();
        assert_eq!(color.name, "red");
        assert_eq!(color.color_type, ColorType::Standard);
        assert_eq!(color.number, Some(1));
        assert_eq!(color.triplet, None);
    }

    #[test]
    fn test_parse_bright_red() {
        let color = Color::parse("bright_red").unwrap();
        assert_eq!(color.name, "bright_red");
        assert_eq!(color.color_type, ColorType::Standard);
        assert_eq!(color.number, Some(9));
        assert_eq!(color.triplet, None);
    }

    #[test]
    fn test_parse_yellow4() {
        let color = Color::parse("yellow4").unwrap();
        assert_eq!(color.name, "yellow4");
        assert_eq!(color.color_type, ColorType::EightBit);
        assert_eq!(color.number, Some(106));
        assert_eq!(color.triplet, None);
    }

    #[test]
    fn test_parse_color_100() {
        let color = Color::parse("color(100)").unwrap();
        assert_eq!(color.name, "color(100)");
        assert_eq!(color.color_type, ColorType::EightBit);
        assert_eq!(color.number, Some(100));
        assert_eq!(color.triplet, None);
    }

    #[test]
    fn test_parse_hex() {
        let color = Color::parse("#112233").unwrap();
        assert_eq!(color.name, "#112233");
        assert_eq!(color.color_type, ColorType::TrueColor);
        assert_eq!(color.number, None);
        assert_eq!(color.triplet, Some(ColorTriplet::new(0x11, 0x22, 0x33)));
    }

    #[test]
    fn test_parse_rgb() {
        let color = Color::parse("rgb(90,100,110)").unwrap();
        assert_eq!(color.name, "#5a646e");
        assert_eq!(color.color_type, ColorType::TrueColor);
        assert_eq!(color.number, None);
        assert_eq!(color.triplet, Some(ColorTriplet::new(90, 100, 110)));
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
        assert_eq!(color.name, "#102030");
        assert_eq!(color.color_type, ColorType::TrueColor);
    }

    // from_ansi tests
    #[test]
    fn test_from_ansi_standard() {
        let color = Color::from_ansi(1);
        assert_eq!(color.color_type, ColorType::Standard);
        assert_eq!(color.number, Some(1));
    }

    #[test]
    fn test_from_ansi_eightbit() {
        let color = Color::from_ansi(100);
        assert_eq!(color.color_type, ColorType::EightBit);
        assert_eq!(color.number, Some(100));
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
        assert_eq!(downgraded.number, Some(16));
    }

    #[test]
    fn test_downgrade_white_to_eightbit() {
        let color = Color::parse("#ffffff").unwrap();
        let downgraded = color.downgrade(ColorSystem::EightBit);
        assert_eq!(downgraded.number, Some(231));
    }

    #[test]
    fn test_downgrade_red_to_eightbit() {
        let color = Color::parse("#ff0000").unwrap();
        let downgraded = color.downgrade(ColorSystem::EightBit);
        assert_eq!(downgraded.number, Some(196));
    }

    #[test]
    fn test_downgrade_red_to_standard() {
        let color = Color::parse("#ff0000").unwrap();
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.number, Some(1));
    }

    #[test]
    fn test_downgrade_green_to_standard() {
        let color = Color::parse("#00ff00").unwrap();
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.number, Some(2));
    }

    #[test]
    fn test_downgrade_color_20_to_standard() {
        let color = Color::parse("color(20)").unwrap();
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.number, Some(4));
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
        assert_eq!(color1.number, Some(1));
        assert_eq!(color2.number, Some(1));
        assert_eq!(color3.number, Some(1));
    }

    #[test]
    fn test_parse_grey_gray_alias() {
        let grey = Color::parse("grey0").unwrap();
        let gray = Color::parse("gray0").unwrap();
        assert_eq!(grey.number, gray.number);
        assert_eq!(grey.number, Some(16));
    }

    // Additional edge cases
    #[test]
    fn test_parse_color_15() {
        let color = Color::parse("color(15)").unwrap();
        assert_eq!(color.color_type, ColorType::Standard);
        assert_eq!(color.number, Some(15));
    }

    #[test]
    fn test_parse_color_16() {
        let color = Color::parse("color(16)").unwrap();
        assert_eq!(color.color_type, ColorType::EightBit);
        assert_eq!(color.number, Some(16));
    }

    #[test]
    fn test_downgrade_default() {
        let color = Color::default_color();
        let downgraded = color.downgrade(ColorSystem::Standard);
        assert_eq!(downgraded.color_type, ColorType::Default);
    }
}
