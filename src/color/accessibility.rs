//! Accessibility utilities for terminal output.
//!
//! Provides WCAG 2.1 contrast ratio calculations and motion preference detection.
//!
//! # Examples
//!
//! ```
//! use gilt::accessibility::{contrast_ratio, meets_aa, meets_aaa};
//! use gilt::color_triplet::ColorTriplet;
//!
//! let black = ColorTriplet::new(0, 0, 0);
//! let white = ColorTriplet::new(255, 255, 255);
//!
//! // Maximum contrast
//! assert!((contrast_ratio(&black, &white) - 21.0).abs() < 0.1);
//! assert!(meets_aa(&black, &white));
//! assert!(meets_aaa(&black, &white));
//! ```

use crate::color::color_triplet::ColorTriplet;

/// Compute relative luminance of a color per WCAG 2.1.
///
/// See: <https://www.w3.org/TR/WCAG21/#dfn-relative-luminance>
fn relative_luminance(color: &ColorTriplet) -> f64 {
    let (r, g, b) = color.normalized();
    let linearize = |c: f64| -> f64 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

/// Calculate the WCAG 2.1 contrast ratio between two colors.
///
/// Returns a value from 1.0 (no contrast) to 21.0 (maximum contrast).
///
/// # Examples
///
/// ```
/// use gilt::accessibility::contrast_ratio;
/// use gilt::color_triplet::ColorTriplet;
///
/// let black = ColorTriplet::new(0, 0, 0);
/// let white = ColorTriplet::new(255, 255, 255);
/// assert!((contrast_ratio(&black, &white) - 21.0).abs() < 0.1);
/// ```
pub fn contrast_ratio(fg: &ColorTriplet, bg: &ColorTriplet) -> f64 {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Check if two colors meet WCAG AA contrast requirements (>= 4.5:1).
///
/// AA is the minimum acceptable contrast for normal-sized text.
pub fn meets_aa(fg: &ColorTriplet, bg: &ColorTriplet) -> bool {
    contrast_ratio(fg, bg) >= 4.5
}

/// Check if two colors meet WCAG AAA contrast requirements (>= 7:1).
///
/// AAA is the enhanced contrast level for normal-sized text.
pub fn meets_aaa(fg: &ColorTriplet, bg: &ColorTriplet) -> bool {
    contrast_ratio(fg, bg) >= 7.0
}

/// Check if large text meets WCAG AA requirements (>= 3:1).
///
/// Large text is defined as 14pt bold or 18pt normal.
pub fn meets_aa_large(fg: &ColorTriplet, bg: &ColorTriplet) -> bool {
    contrast_ratio(fg, bg) >= 3.0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn black() -> ColorTriplet {
        ColorTriplet::new(0, 0, 0)
    }

    fn white() -> ColorTriplet {
        ColorTriplet::new(255, 255, 255)
    }

    // --- contrast_ratio ---

    #[test]
    fn test_black_on_white_max_contrast() {
        let ratio = contrast_ratio(&black(), &white());
        assert!((ratio - 21.0).abs() < 0.1, "expected ~21.0, got {ratio}");
    }

    #[test]
    fn test_white_on_white_no_contrast() {
        let ratio = contrast_ratio(&white(), &white());
        assert!((ratio - 1.0).abs() < 0.01, "expected ~1.0, got {ratio}");
    }

    #[test]
    fn test_black_on_black_no_contrast() {
        let ratio = contrast_ratio(&black(), &black());
        assert!((ratio - 1.0).abs() < 0.01, "expected ~1.0, got {ratio}");
    }

    #[test]
    fn test_symmetry() {
        let red = ColorTriplet::new(255, 0, 0);
        let blue = ColorTriplet::new(0, 0, 255);
        let ratio_ab = contrast_ratio(&red, &blue);
        let ratio_ba = contrast_ratio(&blue, &red);
        assert!(
            (ratio_ab - ratio_ba).abs() < 1e-10,
            "contrast_ratio should be symmetric: {ratio_ab} vs {ratio_ba}"
        );
    }

    #[test]
    fn test_symmetry_with_white() {
        let color = ColorTriplet::new(128, 128, 128);
        let ratio_fw = contrast_ratio(&color, &white());
        let ratio_wf = contrast_ratio(&white(), &color);
        assert!(
            (ratio_fw - ratio_wf).abs() < 1e-10,
            "contrast_ratio should be symmetric"
        );
    }

    #[test]
    fn test_known_pair_red_on_white() {
        // Pure red (#ff0000) on white: WCAG expected ~4.0
        let red = ColorTriplet::new(255, 0, 0);
        let ratio = contrast_ratio(&red, &white());
        assert!(
            ratio > 3.9 && ratio < 4.1,
            "red on white expected ~4.0, got {ratio}"
        );
    }

    #[test]
    fn test_known_pair_green_on_black() {
        // Pure green (#00ff00) on black: luminance of green is high
        let green = ColorTriplet::new(0, 255, 0);
        let ratio = contrast_ratio(&green, &black());
        // Green luminance ~ 0.7152, ratio = (0.7152 + 0.05) / (0 + 0.05) ~ 15.3
        assert!(
            ratio > 15.0 && ratio < 16.0,
            "green on black expected ~15.3, got {ratio}"
        );
    }

    #[test]
    fn test_mid_gray_on_white() {
        let gray = ColorTriplet::new(128, 128, 128);
        let ratio = contrast_ratio(&gray, &white());
        // Mid gray (~0.216 luminance) on white (~1.0 luminance)
        // ratio = (1.0 + 0.05) / (0.216 + 0.05) ~ 3.95
        assert!(
            ratio > 3.5 && ratio < 4.5,
            "mid gray on white expected ~3.95, got {ratio}"
        );
    }

    #[test]
    fn test_ratio_always_at_least_one() {
        // Any two identical colors should give exactly 1.0
        for val in [0u8, 64, 128, 192, 255] {
            let c = ColorTriplet::new(val, val, val);
            let ratio = contrast_ratio(&c, &c);
            assert!(
                (ratio - 1.0).abs() < 1e-10,
                "identical colors should have ratio 1.0, got {ratio}"
            );
        }
    }

    // --- meets_aa ---

    #[test]
    fn test_meets_aa_black_on_white() {
        assert!(meets_aa(&black(), &white()));
    }

    #[test]
    fn test_meets_aa_fails_for_low_contrast() {
        // Light gray on white should fail AA
        let light_gray = ColorTriplet::new(200, 200, 200);
        assert!(!meets_aa(&light_gray, &white()));
    }

    #[test]
    fn test_meets_aa_threshold_boundary() {
        // Red on white is about 4.0 — below the 4.5 threshold
        let red = ColorTriplet::new(255, 0, 0);
        assert!(!meets_aa(&red, &white()));
    }

    // --- meets_aaa ---

    #[test]
    fn test_meets_aaa_black_on_white() {
        assert!(meets_aaa(&black(), &white()));
    }

    #[test]
    fn test_meets_aaa_fails_for_moderate_contrast() {
        // Dark gray on white might pass AA but not AAA
        let dark_gray = ColorTriplet::new(100, 100, 100);
        let ratio = contrast_ratio(&dark_gray, &white());
        // Should be around 5.3 — passes AA but not AAA
        assert!(ratio >= 4.5, "should pass AA");
        assert!(!meets_aaa(&dark_gray, &white()), "should fail AAA");
    }

    // --- meets_aa_large ---

    #[test]
    fn test_meets_aa_large_black_on_white() {
        assert!(meets_aa_large(&black(), &white()));
    }

    #[test]
    fn test_meets_aa_large_red_on_white() {
        // Red on white is ~4.0, above the 3.0 large-text threshold
        let red = ColorTriplet::new(255, 0, 0);
        assert!(meets_aa_large(&red, &white()));
    }

    #[test]
    fn test_meets_aa_large_fails_for_very_low_contrast() {
        // Very light gray on white
        let very_light = ColorTriplet::new(220, 220, 220);
        assert!(!meets_aa_large(&very_light, &white()));
    }

    // --- relative_luminance (internal, tested via contrast_ratio) ---

    #[test]
    fn test_luminance_black_is_zero() {
        let lum = relative_luminance(&black());
        assert!(lum.abs() < 1e-10, "black luminance should be 0.0");
    }

    #[test]
    fn test_luminance_white_is_one() {
        let lum = relative_luminance(&white());
        assert!((lum - 1.0).abs() < 1e-10, "white luminance should be 1.0");
    }

    #[test]
    fn test_luminance_pure_green_highest_among_primaries() {
        let r_lum = relative_luminance(&ColorTriplet::new(255, 0, 0));
        let g_lum = relative_luminance(&ColorTriplet::new(0, 255, 0));
        let b_lum = relative_luminance(&ColorTriplet::new(0, 0, 255));
        assert!(
            g_lum > r_lum && g_lum > b_lum,
            "green should have highest luminance: R={r_lum}, G={g_lum}, B={b_lum}"
        );
    }
}
