//! RGB color triplet representation.
//!
//! This module provides a simple RGB color tuple with utility methods
//! for converting between different color representations.

use std::fmt;

/// An RGB color triplet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorTriplet {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ColorTriplet {
    /// Creates a new ColorTriplet from red, green, and blue components.
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// Returns the color as a lowercase hexadecimal string (e.g., "#ffffff").
    pub fn hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }

    /// Returns the color as an RGB string (e.g., "rgb(255,255,255)").
    pub fn rgb(&self) -> String {
        format!("rgb({},{},{})", self.red, self.green, self.blue)
    }

    /// Returns the color components normalized to 0.0..1.0 range.
    pub fn normalized(&self) -> (f64, f64, f64) {
        (
            self.red as f64 / 255.0,
            self.green as f64 / 255.0,
            self.blue as f64 / 255.0,
        )
    }
}

impl fmt::Display for ColorTriplet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new() {
        let color = ColorTriplet::new(255, 128, 64);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 64);
    }

    #[test]
    fn test_hex_white() {
        assert_eq!(ColorTriplet::new(255, 255, 255).hex(), "#ffffff");
    }

    #[test]
    fn test_hex_green() {
        assert_eq!(ColorTriplet::new(0, 255, 0).hex(), "#00ff00");
    }

    #[test]
    fn test_hex_black() {
        assert_eq!(ColorTriplet::new(0, 0, 0).hex(), "#000000");
    }

    #[test]
    fn test_hex_arbitrary() {
        assert_eq!(ColorTriplet::new(128, 64, 32).hex(), "#804020");
    }

    #[test]
    fn test_rgb_white() {
        assert_eq!(ColorTriplet::new(255, 255, 255).rgb(), "rgb(255,255,255)");
    }

    #[test]
    fn test_rgb_green() {
        assert_eq!(ColorTriplet::new(0, 255, 0).rgb(), "rgb(0,255,0)");
    }

    #[test]
    fn test_rgb_black() {
        assert_eq!(ColorTriplet::new(0, 0, 0).rgb(), "rgb(0,0,0)");
    }

    #[test]
    fn test_rgb_arbitrary() {
        assert_eq!(ColorTriplet::new(128, 64, 32).rgb(), "rgb(128,64,32)");
    }

    #[test]
    fn test_normalized_white() {
        assert_eq!(
            ColorTriplet::new(255, 255, 255).normalized(),
            (1.0, 1.0, 1.0)
        );
    }

    #[test]
    fn test_normalized_green() {
        assert_eq!(ColorTriplet::new(0, 255, 0).normalized(), (0.0, 1.0, 0.0));
    }

    #[test]
    fn test_normalized_black() {
        assert_eq!(ColorTriplet::new(0, 0, 0).normalized(), (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_normalized_arbitrary() {
        let (r, g, b) = ColorTriplet::new(128, 64, 32).normalized();
        assert!((r - 0.5019607843137255).abs() < 1e-10);
        assert!((g - 0.25098039215686274).abs() < 1e-10);
        assert!((b - 0.12549019607843137).abs() < 1e-10);
    }

    #[test]
    fn test_display_trait() {
        let color = ColorTriplet::new(255, 128, 64);
        assert_eq!(format!("{}", color), "#ff8040");
    }

    #[test]
    fn test_display_matches_hex() {
        let color = ColorTriplet::new(0, 255, 0);
        assert_eq!(format!("{}", color), color.hex());
    }

    #[test]
    fn test_clone() {
        let color1 = ColorTriplet::new(255, 128, 64);
        let color2 = color1.clone();
        assert_eq!(color1, color2);
    }

    #[test]
    fn test_copy() {
        let color1 = ColorTriplet::new(255, 128, 64);
        let color2 = color1;
        assert_eq!(color1, color2);
        assert_eq!(color1.red, 255);
    }

    #[test]
    fn test_equality() {
        let color1 = ColorTriplet::new(255, 128, 64);
        let color2 = ColorTriplet::new(255, 128, 64);
        let color3 = ColorTriplet::new(255, 128, 65);
        assert_eq!(color1, color2);
        assert_ne!(color1, color3);
    }

    #[test]
    fn test_hash_in_collections() {
        let mut set = HashSet::new();
        let color1 = ColorTriplet::new(255, 128, 64);
        let color2 = ColorTriplet::new(255, 128, 64);
        let color3 = ColorTriplet::new(0, 255, 0);

        set.insert(color1);
        assert!(set.contains(&color2));
        assert!(!set.contains(&color3));

        set.insert(color3);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_debug_trait() {
        let color = ColorTriplet::new(255, 128, 64);
        let debug_str = format!("{:?}", color);
        assert!(debug_str.contains("ColorTriplet"));
        assert!(debug_str.contains("255"));
        assert!(debug_str.contains("128"));
        assert!(debug_str.contains("64"));
    }
}
