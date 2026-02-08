//! Terminal theme definitions for color resolution and export rendering.
//!
//! Provides [`TerminalTheme`] and several built-in themes (default, SVG export,
//! Monokai, Dimmed Monokai, Night Owlish) used when resolving named/system
//! colors to RGB values.

use crate::color_triplet::ColorTriplet;
use crate::palette::Palette;
use once_cell::sync::Lazy;

/// A terminal theme definition consisting of foreground, background, and ANSI colors.
pub struct TerminalTheme {
    /// Background color of the terminal.
    pub background_color: ColorTriplet,
    /// Foreground (text) color of the terminal.
    pub foreground_color: ColorTriplet,
    /// ANSI color palette (typically 16 colors: 8 normal + 8 bright).
    pub ansi_colors: Palette,
}

impl TerminalTheme {
    /// Creates a new terminal theme.
    ///
    /// # Arguments
    /// * `background` - Background color as (r, g, b) tuple
    /// * `foreground` - Foreground color as (r, g, b) tuple
    /// * `normal` - Normal intensity ANSI colors (typically 8 colors)
    /// * `bright` - Optional bright intensity ANSI colors. If None, normal colors are duplicated.
    pub fn new(
        background: (u8, u8, u8),
        foreground: (u8, u8, u8),
        normal: Vec<(u8, u8, u8)>,
        bright: Option<Vec<(u8, u8, u8)>>,
    ) -> Self {
        let mut colors = normal;
        match bright {
            Some(b) => colors.extend(b),
            None => {
                let dup = colors.clone();
                colors.extend(dup);
            }
        }
        Self {
            background_color: ColorTriplet::new(background.0, background.1, background.2),
            foreground_color: ColorTriplet::new(foreground.0, foreground.1, foreground.2),
            ansi_colors: Palette::new(colors),
        }
    }
}

/// Default terminal theme with standard colors.
pub static DEFAULT_TERMINAL_THEME: Lazy<TerminalTheme> = Lazy::new(|| {
    TerminalTheme::new(
        (255, 255, 255),
        (0, 0, 0),
        vec![
            (0, 0, 0),
            (128, 0, 0),
            (0, 128, 0),
            (128, 128, 0),
            (0, 0, 128),
            (128, 0, 128),
            (0, 128, 128),
            (192, 192, 192),
        ],
        Some(vec![
            (128, 128, 128),
            (255, 0, 0),
            (0, 255, 0),
            (255, 255, 0),
            (0, 0, 255),
            (255, 0, 255),
            (0, 255, 255),
            (255, 255, 255),
        ]),
    )
});

/// SVG export theme with neutral colors suitable for rendering.
pub static SVG_EXPORT_THEME: Lazy<TerminalTheme> = Lazy::new(|| {
    TerminalTheme::new(
        (41, 41, 41),
        (197, 200, 198),
        vec![
            (75, 78, 85),
            (204, 85, 90),
            (152, 168, 75),
            (208, 179, 68),
            (96, 138, 177),
            (152, 114, 159),
            (104, 160, 179),
            (197, 200, 198),
            (154, 155, 153),
        ],
        Some(vec![
            (255, 38, 39),
            (0, 130, 61),
            (208, 132, 66),
            (25, 132, 233),
            (255, 44, 122),
            (57, 130, 128),
            (253, 253, 197),
        ]),
    )
});

/// Monokai theme with dark background and vibrant colors.
pub static MONOKAI: Lazy<TerminalTheme> = Lazy::new(|| {
    TerminalTheme::new(
        (12, 12, 12),
        (217, 217, 217),
        vec![
            (26, 26, 26),
            (244, 0, 95),
            (152, 224, 36),
            (253, 151, 31),
            (157, 101, 255),
            (244, 0, 95),
            (88, 209, 235),
            (196, 197, 181),
            (98, 94, 76),
        ],
        Some(vec![
            (244, 0, 95),
            (152, 224, 36),
            (224, 213, 97),
            (157, 101, 255),
            (244, 0, 95),
            (88, 209, 235),
            (246, 246, 239),
        ]),
    )
});

/// Dimmed Monokai theme with muted colors.
pub static DIMMED_MONOKAI: Lazy<TerminalTheme> = Lazy::new(|| {
    TerminalTheme::new(
        (25, 25, 25),
        (185, 188, 186),
        vec![
            (58, 61, 67),
            (190, 63, 72),
            (135, 154, 59),
            (197, 166, 53),
            (79, 118, 161),
            (133, 92, 141),
            (87, 143, 164),
            (185, 188, 186),
            (136, 137, 135),
        ],
        Some(vec![
            (251, 0, 31),
            (15, 114, 47),
            (196, 112, 51),
            (24, 109, 227),
            (251, 0, 103),
            (46, 112, 109),
            (253, 255, 185),
        ]),
    )
});

/// Night Owlish theme with light background.
pub static NIGHT_OWLISH: Lazy<TerminalTheme> = Lazy::new(|| {
    TerminalTheme::new(
        (255, 255, 255),
        (64, 63, 83),
        vec![
            (1, 22, 39),
            (211, 66, 62),
            (42, 162, 152),
            (218, 170, 1),
            (72, 118, 214),
            (64, 63, 83),
            (8, 145, 106),
            (122, 129, 129),
            (122, 129, 129),
        ],
        Some(vec![
            (247, 110, 110),
            (73, 208, 197),
            (218, 194, 107),
            (92, 167, 228),
            (105, 112, 152),
            (0, 201, 144),
            (152, 159, 177),
        ]),
    )
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_terminal_theme_foreground_background() {
        assert_eq!(DEFAULT_TERMINAL_THEME.foreground_color.red, 0);
        assert_eq!(DEFAULT_TERMINAL_THEME.foreground_color.green, 0);
        assert_eq!(DEFAULT_TERMINAL_THEME.foreground_color.blue, 0);

        assert_eq!(DEFAULT_TERMINAL_THEME.background_color.red, 255);
        assert_eq!(DEFAULT_TERMINAL_THEME.background_color.green, 255);
        assert_eq!(DEFAULT_TERMINAL_THEME.background_color.blue, 255);
    }

    #[test]
    fn test_default_terminal_theme_ansi_black() {
        let black = DEFAULT_TERMINAL_THEME.ansi_colors.get(0);
        assert_eq!(black.red, 0);
        assert_eq!(black.green, 0);
        assert_eq!(black.blue, 0);
    }

    #[test]
    fn test_default_terminal_theme_ansi_dark_red() {
        let dark_red = DEFAULT_TERMINAL_THEME.ansi_colors.get(1);
        assert_eq!(dark_red.red, 128);
        assert_eq!(dark_red.green, 0);
        assert_eq!(dark_red.blue, 0);
    }

    #[test]
    fn test_svg_export_theme_background() {
        assert_eq!(SVG_EXPORT_THEME.background_color.red, 41);
        assert_eq!(SVG_EXPORT_THEME.background_color.green, 41);
        assert_eq!(SVG_EXPORT_THEME.background_color.blue, 41);
    }

    #[test]
    fn test_monokai_foreground() {
        assert_eq!(MONOKAI.foreground_color.red, 217);
        assert_eq!(MONOKAI.foreground_color.green, 217);
        assert_eq!(MONOKAI.foreground_color.blue, 217);
    }

    #[test]
    fn test_theme_with_no_bright_colors() {
        let theme = TerminalTheme::new(
            (255, 255, 255),
            (0, 0, 0),
            vec![(0, 0, 0), (128, 0, 0), (0, 128, 0), (128, 128, 0)],
            None,
        );

        // Should have 8 colors total (4 normal + 4 duplicated)
        // Verify first normal color
        let color0 = theme.ansi_colors.get(0);
        assert_eq!(color0.red, 0);
        assert_eq!(color0.green, 0);
        assert_eq!(color0.blue, 0);

        // Verify first duplicated color (at index 4)
        let color4 = theme.ansi_colors.get(4);
        assert_eq!(color4.red, 0);
        assert_eq!(color4.green, 0);
        assert_eq!(color4.blue, 0);

        // Verify second normal color
        let color1 = theme.ansi_colors.get(1);
        assert_eq!(color1.red, 128);
        assert_eq!(color1.green, 0);
        assert_eq!(color1.blue, 0);

        // Verify second duplicated color (at index 5)
        let color5 = theme.ansi_colors.get(5);
        assert_eq!(color5.red, 128);
        assert_eq!(color5.green, 0);
        assert_eq!(color5.blue, 0);
    }

    #[test]
    fn test_dimmed_monokai_theme() {
        assert_eq!(DIMMED_MONOKAI.background_color.red, 25);
        assert_eq!(DIMMED_MONOKAI.background_color.green, 25);
        assert_eq!(DIMMED_MONOKAI.background_color.blue, 25);

        assert_eq!(DIMMED_MONOKAI.foreground_color.red, 185);
        assert_eq!(DIMMED_MONOKAI.foreground_color.green, 188);
        assert_eq!(DIMMED_MONOKAI.foreground_color.blue, 186);
    }

    #[test]
    fn test_night_owlish_theme() {
        assert_eq!(NIGHT_OWLISH.background_color.red, 255);
        assert_eq!(NIGHT_OWLISH.background_color.green, 255);
        assert_eq!(NIGHT_OWLISH.background_color.blue, 255);

        assert_eq!(NIGHT_OWLISH.foreground_color.red, 64);
        assert_eq!(NIGHT_OWLISH.foreground_color.green, 63);
        assert_eq!(NIGHT_OWLISH.foreground_color.blue, 83);
    }
}
