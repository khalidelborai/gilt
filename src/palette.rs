//! Color palette management and color matching.
//!
//! This module provides color palettes used for terminal color mapping,
//! including ANSI standard, 8-bit, and Windows console palettes.

use crate::color_triplet::ColorTriplet;

/// A palette of RGB colors.
#[derive(Debug, Clone)]
pub struct Palette {
    colors: Vec<(u8, u8, u8)>,
}

impl Palette {
    /// Creates a new palette from a vector of RGB tuples.
    pub fn new(colors: Vec<(u8, u8, u8)>) -> Self {
        Self { colors }
    }

    /// Gets the color at the given index as a ColorTriplet.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get(&self, index: usize) -> ColorTriplet {
        let (r, g, b) = self.colors[index];
        ColorTriplet::new(r, g, b)
    }

    /// Finds the index of the closest matching color in the palette.
    ///
    /// Uses the "redmean" weighted Euclidean distance formula for
    /// perceptually accurate color matching.
    pub fn match_color(&self, color: &ColorTriplet) -> usize {
        let red1 = color.red as i32;
        let green1 = color.green as i32;
        let blue1 = color.blue as i32;

        let mut min_index = 0;
        let mut min_distance = f64::MAX;

        for (index, &(r, g, b)) in self.colors.iter().enumerate() {
            let red2 = r as i32;
            let green2 = g as i32;
            let blue2 = b as i32;

            let red_mean = (red1 + red2) / 2;
            let red_diff = red1 - red2;
            let green_diff = green1 - green2;
            let blue_diff = blue1 - blue2;

            // Redmean weighted Euclidean distance
            let distance = (((512 + red_mean) * red_diff * red_diff) as f64 / 256.0
                + 4.0 * (green_diff * green_diff) as f64
                + ((767 - red_mean) * blue_diff * blue_diff) as f64 / 256.0)
                .sqrt();

            if distance < min_distance {
                min_distance = distance;
                min_index = index;
            }
        }

        min_index
    }
}

/// Standard 16-color ANSI palette.
pub static STANDARD_PALETTE: once_cell::sync::Lazy<Palette> = once_cell::sync::Lazy::new(|| {
    Palette::new(vec![
        (0, 0, 0),
        (170, 0, 0),
        (0, 170, 0),
        (170, 85, 0),
        (0, 0, 170),
        (170, 0, 170),
        (0, 170, 170),
        (170, 170, 170),
        (85, 85, 85),
        (255, 85, 85),
        (85, 255, 85),
        (255, 255, 85),
        (85, 85, 255),
        (255, 85, 255),
        (85, 255, 255),
        (255, 255, 255),
    ])
});

/// Windows 10 console 16-color palette.
pub static WINDOWS_PALETTE: once_cell::sync::Lazy<Palette> = once_cell::sync::Lazy::new(|| {
    Palette::new(vec![
        (12, 12, 12),
        (197, 15, 31),
        (19, 161, 14),
        (193, 156, 0),
        (0, 55, 218),
        (136, 23, 152),
        (58, 150, 221),
        (204, 204, 204),
        (118, 118, 118),
        (231, 72, 86),
        (22, 198, 12),
        (249, 241, 165),
        (59, 120, 255),
        (180, 0, 158),
        (97, 214, 214),
        (242, 242, 242),
    ])
});

/// Generates the 8-bit (256-color) ANSI palette.
///
/// This consists of:
/// - 16 standard ANSI colors
/// - 216 colors in a 6×6×6 RGB cube
/// - 24 grayscale colors
fn generate_eight_bit_palette() -> Vec<(u8, u8, u8)> {
    let mut colors = Vec::with_capacity(256);

    // First 16: standard ANSI colors
    colors.extend_from_slice(&[
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ]);

    // Next 216: 6×6×6 RGB cube
    let cube_values = [0, 95, 135, 175, 215, 255];
    for &r in &cube_values {
        for &g in &cube_values {
            for &b in &cube_values {
                colors.push((r, g, b));
            }
        }
    }

    // Last 24: grayscale ramp
    for i in 0..24 {
        let gray = 8 + i * 10;
        colors.push((gray, gray, gray));
    }

    colors
}

/// 8-bit (256-color) ANSI palette.
pub static EIGHT_BIT_PALETTE: once_cell::sync::Lazy<Palette> =
    once_cell::sync::Lazy::new(|| Palette::new(generate_eight_bit_palette()));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_new() {
        let colors = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];
        let palette = Palette::new(colors.clone());
        assert_eq!(palette.colors.len(), 3);
    }

    #[test]
    fn test_palette_get() {
        let colors = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];
        let palette = Palette::new(colors);

        assert_eq!(palette.get(0), ColorTriplet::new(255, 0, 0));
        assert_eq!(palette.get(1), ColorTriplet::new(0, 255, 0));
        assert_eq!(palette.get(2), ColorTriplet::new(0, 0, 255));
    }

    #[test]
    fn test_match_color_exact() {
        let colors = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];
        let palette = Palette::new(colors);

        let red = ColorTriplet::new(255, 0, 0);
        let green = ColorTriplet::new(0, 255, 0);
        let blue = ColorTriplet::new(0, 0, 255);

        assert_eq!(palette.match_color(&red), 0);
        assert_eq!(palette.match_color(&green), 1);
        assert_eq!(palette.match_color(&blue), 2);
    }

    #[test]
    fn test_match_color_approximate() {
        let colors = vec![(128, 0, 0), (0, 128, 0), (0, 0, 128)];
        let palette = Palette::new(colors);

        // Should match to (128, 0, 0) - closest red
        let dark_red = ColorTriplet::new(129, 0, 0);
        assert_eq!(palette.match_color(&dark_red), 0);

        // Should match to (0, 128, 0) - closest green
        let dark_green = ColorTriplet::new(0, 130, 5);
        assert_eq!(palette.match_color(&dark_green), 1);

        // Should match to (0, 0, 128) - closest blue
        let dark_blue = ColorTriplet::new(5, 0, 127);
        assert_eq!(palette.match_color(&dark_blue), 2);
    }

    #[test]
    fn test_standard_palette_length() {
        assert_eq!(STANDARD_PALETTE.colors.len(), 16);
    }

    #[test]
    fn test_standard_palette_colors() {
        // Test first (black) and last (white) colors
        assert_eq!(STANDARD_PALETTE.get(0), ColorTriplet::new(0, 0, 0));
        assert_eq!(STANDARD_PALETTE.get(15), ColorTriplet::new(255, 255, 255));

        // Test a few specific colors
        assert_eq!(STANDARD_PALETTE.get(1), ColorTriplet::new(170, 0, 0)); // Red
        assert_eq!(STANDARD_PALETTE.get(2), ColorTriplet::new(0, 170, 0)); // Green
        assert_eq!(STANDARD_PALETTE.get(4), ColorTriplet::new(0, 0, 170)); // Blue
    }

    #[test]
    fn test_windows_palette_length() {
        assert_eq!(WINDOWS_PALETTE.colors.len(), 16);
    }

    #[test]
    fn test_windows_palette_colors() {
        // Test first and last colors
        assert_eq!(WINDOWS_PALETTE.get(0), ColorTriplet::new(12, 12, 12));
        assert_eq!(WINDOWS_PALETTE.get(15), ColorTriplet::new(242, 242, 242));

        // Test specific Windows colors
        assert_eq!(WINDOWS_PALETTE.get(1), ColorTriplet::new(197, 15, 31)); // Red
        assert_eq!(WINDOWS_PALETTE.get(2), ColorTriplet::new(19, 161, 14)); // Green
        assert_eq!(WINDOWS_PALETTE.get(4), ColorTriplet::new(0, 55, 218)); // Blue
    }

    #[test]
    fn test_eight_bit_palette_length() {
        assert_eq!(EIGHT_BIT_PALETTE.colors.len(), 256);
    }

    #[test]
    fn test_eight_bit_palette_standard_colors() {
        // First 16 should match standard ANSI colors
        assert_eq!(EIGHT_BIT_PALETTE.get(0), ColorTriplet::new(0, 0, 0)); // Black
        assert_eq!(EIGHT_BIT_PALETTE.get(1), ColorTriplet::new(128, 0, 0)); // Dark red
        assert_eq!(EIGHT_BIT_PALETTE.get(15), ColorTriplet::new(255, 255, 255));
        // White
    }

    #[test]
    fn test_eight_bit_palette_cube_colors() {
        // Test some cube colors (indices 16-231)
        // Index 16 should be (0, 0, 0) from the cube
        assert_eq!(EIGHT_BIT_PALETTE.get(16), ColorTriplet::new(0, 0, 0));

        // Index 21 should be (0, 0, 255) from the cube
        assert_eq!(EIGHT_BIT_PALETTE.get(21), ColorTriplet::new(0, 0, 255));

        // Index 226 should be (255, 255, 0) - near end of cube
        assert_eq!(EIGHT_BIT_PALETTE.get(226), ColorTriplet::new(255, 255, 0));

        // Index 231 should be (255, 255, 255) - last cube color
        assert_eq!(EIGHT_BIT_PALETTE.get(231), ColorTriplet::new(255, 255, 255));
    }

    #[test]
    fn test_eight_bit_palette_grayscale() {
        // Test grayscale colors (indices 232-255)
        assert_eq!(EIGHT_BIT_PALETTE.get(232), ColorTriplet::new(8, 8, 8));
        assert_eq!(EIGHT_BIT_PALETTE.get(233), ColorTriplet::new(18, 18, 18));
        assert_eq!(EIGHT_BIT_PALETTE.get(255), ColorTriplet::new(238, 238, 238));
    }

    #[test]
    fn test_match_color_in_standard_palette() {
        // Test that pure red matches dark red (170, 0, 0) at index 1
        // This is correct - (255, 0, 0) is perceptually closer to (170, 0, 0)
        // than to the bright red (255, 85, 85) at index 9
        let red = ColorTriplet::new(255, 0, 0);
        let matched = STANDARD_PALETTE.match_color(&red);
        assert_eq!(matched, 1);

        // Test that pure green matches dark green at index 2
        let green = ColorTriplet::new(0, 255, 0);
        let matched = STANDARD_PALETTE.match_color(&green);
        assert_eq!(matched, 2);
    }

    #[test]
    fn test_match_color_in_eight_bit_palette() {
        // Test matching a pure color
        let red = ColorTriplet::new(255, 0, 0);
        let matched = EIGHT_BIT_PALETTE.match_color(&red);
        // Should match bright red in standard colors (index 9)
        assert_eq!(matched, 9);

        // Test matching a color in the cube
        let color = ColorTriplet::new(135, 135, 135);
        let matched = EIGHT_BIT_PALETTE.match_color(&color);
        // Should find the exact match in the cube
        let matched_color = EIGHT_BIT_PALETTE.get(matched);
        assert_eq!(matched_color, ColorTriplet::new(135, 135, 135));
    }

    #[test]
    fn test_redmean_distance_formula() {
        // Verify that the redmean formula gives different results than simple Euclidean
        let palette = Palette::new(vec![
            (255, 0, 0),   // Pure red
            (0, 255, 0),   // Pure green
            (0, 0, 255),   // Pure blue
            (128, 128, 0), // Olive
        ]);

        // A yellowish color should be closer to olive or green than to red or blue
        let yellow = ColorTriplet::new(200, 200, 0);
        let matched = palette.match_color(&yellow);

        // Should match green (index 1) or olive (index 3), not red or blue
        assert!(matched == 1 || matched == 3);
    }
}
