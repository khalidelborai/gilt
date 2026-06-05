//! Terminal theme definitions for color resolution and export rendering.
//!
//! Provides [`TerminalTheme`] and several built-in themes (default, SVG export,
//! Monokai, Dimmed Monokai, Night Owlish) used when resolving named/system
//! colors to RGB values.

use crate::color::color_triplet::ColorTriplet;
use crate::palette::Palette;
use std::sync::LazyLock;

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
pub static DEFAULT_TERMINAL_THEME: LazyLock<TerminalTheme> = LazyLock::new(|| {
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
///
/// Layout: 8 normal (ANSI 0-7) + 8 bright (ANSI 8-15).
/// Finding #17: the previous definition had 9 normal entries, shifting all
/// bright-color indices by one. The stray 9th entry (154,155,153) has been
/// relocated as the first bright color (bright-black / ANSI 8).
pub static SVG_EXPORT_THEME: LazyLock<TerminalTheme> = LazyLock::new(|| {
    TerminalTheme::new(
        (41, 41, 41),
        (197, 200, 198),
        // Normal colors: ANSI 0-7
        vec![
            (75, 78, 85),    // 0 black
            (204, 85, 90),   // 1 red
            (152, 168, 75),  // 2 green
            (208, 179, 68),  // 3 yellow
            (96, 138, 177),  // 4 blue
            (152, 114, 159), // 5 magenta
            (104, 160, 179), // 6 cyan
            (197, 200, 198), // 7 white
        ],
        // Bright colors: ANSI 8-15
        Some(vec![
            (154, 155, 153), // 8  bright black (was the stray 9th normal entry)
            (255, 38, 39),   // 9  bright red
            (0, 130, 61),    // 10 bright green
            (208, 132, 66),  // 11 bright yellow
            (25, 132, 233),  // 12 bright blue
            (255, 44, 122),  // 13 bright magenta
            (57, 130, 128),  // 14 bright cyan
            (253, 253, 197), // 15 bright white
        ]),
    )
});

/// Monokai theme with dark background and vibrant colors.
pub static MONOKAI: LazyLock<TerminalTheme> = LazyLock::new(|| {
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
pub static DIMMED_MONOKAI: LazyLock<TerminalTheme> = LazyLock::new(|| {
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
pub static NIGHT_OWLISH: LazyLock<TerminalTheme> = LazyLock::new(|| {
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

// ---------------------------------------------------------------------------
// Named built-in palettes (Dracula, Nord, Gruvbox dark, Monokai, Solarized dark)
// ---------------------------------------------------------------------------

/// Dracula color theme.
///
/// Colors taken from the canonical Dracula specification at <https://draculatheme.com/contribute>.
pub static DRACULA: LazyLock<TerminalTheme> = LazyLock::new(|| {
    TerminalTheme::new(
        (40, 42, 54),    // background #282a36
        (248, 248, 242), // foreground #f8f8f2
        // Normal (ANSI 0-7)
        vec![
            (40, 42, 54),    // 0  black     #282a36
            (255, 85, 85),   // 1  red       #ff5555
            (80, 250, 123),  // 2  green     #50fa7b
            (241, 250, 140), // 3  yellow    #f1fa8c
            (189, 147, 249), // 4  blue      #bd93f9
            (255, 121, 198), // 5  magenta   #ff79c6
            (139, 233, 253), // 6  cyan      #8be9fd
            (191, 191, 191), // 7  white     #bfbfbf
        ],
        Some(vec![
            (98, 114, 164),  // 8  bright black   #626272
            (255, 110, 110), // 9  bright red      #ff6e6e
            (105, 255, 156), // 10 bright green    #69ff94
            (255, 255, 165), // 11 bright yellow   #ffffa5
            (214, 189, 255), // 12 bright blue     #d6acff
            (255, 146, 213), // 13 bright magenta  #ff92d0
            (164, 255, 255), // 14 bright cyan     #a4ffff
            (255, 255, 255), // 15 bright white    #ffffff
        ]),
    )
});

/// Nord color theme.
///
/// Colors from the Nord palette specification at <https://www.nordtheme.com/docs/colors-and-palettes>.
pub static NORD: LazyLock<TerminalTheme> = LazyLock::new(|| {
    TerminalTheme::new(
        (46, 52, 64),    // background #2e3440 (Nord0)
        (216, 222, 233), // foreground #d8dee9 (Nord4)
        // Normal (ANSI 0-7)
        vec![
            (59, 66, 82),    // 0  black     #3b4252 Nord1
            (191, 97, 106),  // 1  red       #bf616a Nord11
            (163, 190, 140), // 2  green     #a3be8c Nord14
            (235, 203, 139), // 3  yellow    #ebcb8b Nord13
            (129, 161, 193), // 4  blue      #81a1c1 Nord9
            (180, 142, 173), // 5  magenta   #b48ead Nord15
            (136, 192, 208), // 6  cyan      #88c0d0 Nord8
            (229, 233, 240), // 7  white     #e5e9f0 Nord5
        ],
        Some(vec![
            (76, 86, 106),   // 8  bright black   #4c5668 (Nord2 darkened)
            (191, 97, 106),  // 9  bright red      #bf616a
            (163, 190, 140), // 10 bright green    #a3be8c
            (235, 203, 139), // 11 bright yellow   #ebcb8b
            (129, 161, 193), // 12 bright blue     #81a1c1
            (180, 142, 173), // 13 bright magenta  #b48ead
            (143, 188, 187), // 14 bright cyan     #8fbcbb Nord7
            (236, 239, 244), // 15 bright white    #eceff4 Nord6
        ]),
    )
});

/// Gruvbox Dark color theme.
///
/// Colors from the canonical Gruvbox palette at <https://github.com/morhetz/gruvbox>.
pub static GRUVBOX: LazyLock<TerminalTheme> = LazyLock::new(|| {
    TerminalTheme::new(
        (40, 40, 40),    // background #282828
        (235, 219, 178), // foreground #ebdbb2 (fg1)
        // Normal (ANSI 0-7)
        vec![
            (40, 40, 40),    // 0  black     #282828
            (204, 36, 29),   // 1  red       #cc241d
            (152, 151, 26),  // 2  green     #98971a
            (215, 153, 33),  // 3  yellow    #d79921
            (69, 133, 136),  // 4  blue      #458588
            (177, 98, 134),  // 5  magenta   #b16286
            (104, 157, 106), // 6  cyan      #689d6a
            (168, 153, 132), // 7  white     #a89984 (fg4)
        ],
        Some(vec![
            (146, 131, 116), // 8  bright black   #928374
            (251, 73, 52),   // 9  bright red      #fb4934
            (184, 187, 38),  // 10 bright green    #b8bb26
            (250, 189, 47),  // 11 bright yellow   #fabd2f
            (131, 165, 152), // 12 bright blue     #83a598
            (211, 134, 155), // 13 bright magenta  #d3869b
            (142, 192, 124), // 14 bright cyan     #8ec07c
            (235, 219, 178), // 15 bright white    #ebdbb2
        ]),
    )
});

/// Monokai Pro terminal theme (canonical 16-color palette).
///
/// Colors from the Monokai Pro specification at <https://monokai.pro>.
pub static MONOKAI_PRO: LazyLock<TerminalTheme> = LazyLock::new(|| {
    TerminalTheme::new(
        (45, 42, 46),    // background #2d2a2e
        (252, 252, 250), // foreground #fcfcfa
        // Normal (ANSI 0-7)
        vec![
            (45, 42, 46),    // 0  black     #2d2a2e
            (255, 97, 136),  // 1  red       #ff6188
            (169, 220, 118), // 2  green     #a9dc76
            (255, 216, 102), // 3  yellow    #ffd866
            (120, 220, 232), // 4  blue      #78dce8
            (171, 157, 242), // 5  magenta   #ab9df2
            (120, 220, 232), // 6  cyan      #78dce8
            (252, 252, 250), // 7  white     #fcfcfa
        ],
        Some(vec![
            (114, 112, 114), // 8  bright black   #727072
            (255, 97, 136),  // 9  bright red      #ff6188
            (169, 220, 118), // 10 bright green    #a9dc76
            (255, 216, 102), // 11 bright yellow   #ffd866
            (120, 220, 232), // 12 bright blue     #78dce8
            (171, 157, 242), // 13 bright magenta  #ab9df2
            (120, 220, 232), // 14 bright cyan     #78dce8
            (252, 252, 250), // 15 bright white    #fcfcfa
        ]),
    )
});

/// Solarized Dark color theme.
///
/// Colors from the canonical Solarized specification at <https://ethanschoonover.com/solarized/>.
pub static SOLARIZED_DARK: LazyLock<TerminalTheme> = LazyLock::new(|| {
    TerminalTheme::new(
        (0, 43, 54),     // background #002b36 (base03)
        (131, 148, 150), // foreground #839496 (base0)
        // Normal (ANSI 0-7)
        vec![
            (7, 54, 66),     // 0  black     #073642 base02
            (220, 50, 47),   // 1  red       #dc322f
            (133, 153, 0),   // 2  green     #859900
            (181, 137, 0),   // 3  yellow    #b58900
            (38, 139, 210),  // 4  blue      #268bd2
            (211, 54, 130),  // 5  magenta   #d33682
            (42, 161, 152),  // 6  cyan      #2aa198
            (238, 232, 213), // 7  white     #eee8d5 base2
        ],
        Some(vec![
            (0, 43, 54),     // 8  bright black   #002b36 base03
            (203, 75, 22),   // 9  bright red      #cb4b16
            (88, 110, 117),  // 10 bright green    #586e75 base01
            (101, 123, 131), // 11 bright yellow   #657b83 base00
            (131, 148, 150), // 12 bright blue     #839496 base0
            (108, 113, 196), // 13 bright magenta  #6c71c4 violet
            (147, 161, 161), // 14 bright cyan     #93a1a1 base1
            (253, 246, 227), // 15 bright white    #fdf6e3 base3
        ]),
    )
});

// ---------------------------------------------------------------------------
// ThemeRegistry
// ---------------------------------------------------------------------------

/// Registry of named built-in [`TerminalTheme`] palettes.
///
/// Provides access to 5 popular terminal color schemes by name.
/// Name lookup is case-insensitive.
///
/// # Available themes
///
/// | Name | Aliases |
/// |------|---------|
/// | `"dracula"` | — |
/// | `"nord"` | — |
/// | `"gruvbox"` | — |
/// | `"monokai"` | — |
/// | `"solarized-dark"` | `"solarized"` |
///
/// # Examples
///
/// ```rust
/// use gilt::terminal_theme::ThemeRegistry;
///
/// let theme = ThemeRegistry::terminal_theme("dracula").unwrap();
/// assert_eq!(theme.background_color.red, 40);
///
/// let names = ThemeRegistry::names();
/// assert_eq!(names.len(), 5);
/// ```
pub struct ThemeRegistry;

impl ThemeRegistry {
    /// Returns the names of all built-in themes.
    ///
    /// The returned slice always has 5 entries, one per built-in palette.
    pub fn names() -> &'static [&'static str] {
        &["dracula", "nord", "gruvbox", "monokai", "solarized-dark"]
    }

    /// Look up a built-in [`TerminalTheme`] by name.
    ///
    /// Matching is case-insensitive.  `"solarized"` is accepted as an alias
    /// for `"solarized-dark"`.
    ///
    /// Returns `None` if `name` does not match any known palette.
    pub fn terminal_theme(name: &str) -> Option<&'static TerminalTheme> {
        match name.to_ascii_lowercase().as_str() {
            "dracula" => Some(&DRACULA),
            "nord" => Some(&NORD),
            "gruvbox" => Some(&GRUVBOX),
            "monokai" => Some(&MONOKAI_PRO),
            "solarized-dark" | "solarized" => Some(&SOLARIZED_DARK),
            _ => None,
        }
    }
}

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

    /// Finding #17: SVG_EXPORT_THEME must have exactly 16 ANSI palette entries
    /// (8 normal + 8 bright) so bright-color indices are correct.
    #[test]
    fn test_svg_export_theme_palette_is_16_entries() {
        // Normal colors 0-7: index 7 should be (197,200,198) — the white entry.
        let normal_white = SVG_EXPORT_THEME.ansi_colors.get(7);
        assert_eq!(
            (normal_white.red, normal_white.green, normal_white.blue),
            (197, 200, 198),
            "ANSI 7 (normal white) should be (197,200,198)"
        );

        // Bright colors 8-15: index 8 was the stray 9th entry (154,155,153).
        let bright_black = SVG_EXPORT_THEME.ansi_colors.get(8);
        assert_eq!(
            (bright_black.red, bright_black.green, bright_black.blue),
            (154, 155, 153),
            "ANSI 8 (bright black) should be (154,155,153) — relocated from old index 8"
        );

        // ANSI 9 (bright red) should be (255,38,39).
        let bright_red = SVG_EXPORT_THEME.ansi_colors.get(9);
        assert_eq!(
            (bright_red.red, bright_red.green, bright_red.blue),
            (255, 38, 39),
            "ANSI 9 (bright red) should be (255,38,39)"
        );

        // ANSI 15 (bright white) should be (253,253,197).
        let bright_white = SVG_EXPORT_THEME.ansi_colors.get(15);
        assert_eq!(
            (bright_white.red, bright_white.green, bright_white.blue),
            (253, 253, 197),
            "ANSI 15 (bright white) should be (253,253,197)"
        );
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

    // -- ThemeRegistry tests --------------------------------------------------

    #[test]
    fn test_registry_names_count() {
        assert_eq!(ThemeRegistry::names().len(), 5);
    }

    #[test]
    fn test_registry_names_contains_all() {
        let names = ThemeRegistry::names();
        assert!(names.contains(&"dracula"));
        assert!(names.contains(&"nord"));
        assert!(names.contains(&"gruvbox"));
        assert!(names.contains(&"monokai"));
        assert!(names.contains(&"solarized-dark"));
    }

    #[test]
    fn test_registry_lookup_dracula() {
        let theme = ThemeRegistry::terminal_theme("dracula").expect("dracula must resolve");
        // Background is #282a36 = (40, 42, 54)
        assert_eq!(theme.background_color.red, 40);
        assert_eq!(theme.background_color.green, 42);
        assert_eq!(theme.background_color.blue, 54);
    }

    #[test]
    fn test_registry_lookup_case_insensitive() {
        assert!(ThemeRegistry::terminal_theme("Dracula").is_some());
        assert!(ThemeRegistry::terminal_theme("NORD").is_some());
        assert!(ThemeRegistry::terminal_theme("Gruvbox").is_some());
        assert!(ThemeRegistry::terminal_theme("MONOKAI").is_some());
        assert!(ThemeRegistry::terminal_theme("Solarized-Dark").is_some());
    }

    #[test]
    fn test_registry_solarized_alias() {
        let a = ThemeRegistry::terminal_theme("solarized-dark").unwrap();
        let b = ThemeRegistry::terminal_theme("solarized").unwrap();
        // Both aliases point to the same static
        assert_eq!(a.background_color.red, b.background_color.red);
    }

    #[test]
    fn test_registry_unknown_returns_none() {
        assert!(ThemeRegistry::terminal_theme("unknown-theme-xyz").is_none());
    }

    #[test]
    fn test_registry_nord_background() {
        let theme = ThemeRegistry::terminal_theme("nord").unwrap();
        // Nord background is #2e3440 = (46, 52, 64)
        assert_eq!(theme.background_color.red, 46);
        assert_eq!(theme.background_color.green, 52);
        assert_eq!(theme.background_color.blue, 64);
    }

    #[test]
    fn test_registry_gruvbox_background() {
        let theme = ThemeRegistry::terminal_theme("gruvbox").unwrap();
        // Gruvbox dark background is #282828 = (40, 40, 40)
        assert_eq!(theme.background_color.red, 40);
        assert_eq!(theme.background_color.green, 40);
        assert_eq!(theme.background_color.blue, 40);
    }

    #[test]
    fn test_registry_monokai_background() {
        let theme = ThemeRegistry::terminal_theme("monokai").unwrap();
        // Monokai Pro background is #2d2a2e = (45, 42, 46)
        assert_eq!(theme.background_color.red, 45);
        assert_eq!(theme.background_color.green, 42);
        assert_eq!(theme.background_color.blue, 46);
    }

    #[test]
    fn test_registry_solarized_dark_background() {
        let theme = ThemeRegistry::terminal_theme("solarized-dark").unwrap();
        // Solarized dark background is #002b36 = (0, 43, 54)
        assert_eq!(theme.background_color.red, 0);
        assert_eq!(theme.background_color.green, 43);
        assert_eq!(theme.background_color.blue, 54);
    }

    #[test]
    fn test_dracula_has_16_ansi_colors() {
        // Each palette should have 8 normal + 8 bright = 16 entries
        let theme = &*DRACULA;
        // ANSI index 15 (bright white) must be accessible
        let bright_white = theme.ansi_colors.get(15);
        assert_eq!(bright_white.red, 255);
        assert_eq!(bright_white.green, 255);
        assert_eq!(bright_white.blue, 255);
    }
}
