//! Color palette management and color matching.
//!
//! This module provides color palettes used for terminal color mapping,
//! including ANSI standard, 8-bit, and Windows console palettes.

use crate::color::color_triplet::ColorTriplet;
use lru::LruCache;
use std::cell::RefCell;
use std::num::NonZeroUsize;

// Per-thread LRU cache: (palette-data-pointer, ColorTriplet) -> index.
//
// `match_color` is on the hot path for downgrade / palette rendering;
// caching avoids a full linear scan of the palette for repeat lookups
// (which are the common case: same RGB queried for many cells).
//
// Capacity 1024: well above the working set of a single screen of unique
// RGB triplets, with bounded memory.
//
// Keyed by `(palette identity, triplet)` so STANDARD_PALETTE,
// WINDOWS_PALETTE, and EIGHT_BIT_PALETTE do not collide. Identity is the
// data pointer of the palette's underlying `Vec`; this is stable for the
// static `LazyLock` palettes (the pointer never changes after init). A
// cloned `Palette` owns a fresh `Vec` with a different pointer, so it gets
// its own cache slot — this is acceptable because clones are rare (the hot
// path always goes through the static palettes).
thread_local! {
    static MATCH_COLOR_CACHE: RefCell<LruCache<(*const u8, ColorTriplet), usize>> =
        RefCell::new(LruCache::new(NonZeroUsize::new(1024).unwrap()));
}

#[cfg(test)]
// Test-only counter: number of cache misses in the current thread.
// Reset at thread start (thread_local). Read via `match_color_misses`.
std::thread_local! {
    static MATCH_COLOR_MISSES: RefCell<u64> = const { RefCell::new(0) };
}

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

    /// Returns the number of colors in the palette.
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Returns `true` if the palette contains no colors.
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
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
    /// perceptually accurate color matching. Results are memoised in a
    /// thread-local LRU keyed by `(palette identity, triplet)` so repeat
    /// lookups skip the linear scan (Phase 7 perf fix).
    pub fn match_color(&self, color: &ColorTriplet) -> usize {
        // Identity by data pointer: stable for the static palettes and for
        // any instance that owns its `colors` vec. Cast through `*const u8`
        // (a `*const (u8, u8, u8)` would still be a valid `Hash`/`Eq` key,
        // but `*const u8` is the canonical erased-pointer type).
        let key = (self.colors.as_ptr() as *const u8, *color);
        MATCH_COLOR_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(&idx) = cache.get(&key) {
                return idx;
            }
            // Cache miss: run the linear scan and record the result.
            #[cfg(test)]
            MATCH_COLOR_MISSES.with(|m| *m.borrow_mut() += 1);
            let idx = self.match_color_uncached(color);
            cache.put(key, idx);
            idx
        })
    }

    /// Pure (uncached) implementation of [`Self::match_color`].
    fn match_color_uncached(&self, color: &ColorTriplet) -> usize {
        let red1 = color.red as i32;
        let green1 = color.green as i32;
        let blue1 = color.blue as i32;

        let mut min_index = 0;
        let mut min_distance = i32::MAX;

        for (index, &(r, g, b)) in self.colors.iter().enumerate() {
            let red2 = r as i32;
            let green2 = g as i32;
            let blue2 = b as i32;

            let red_mean = (red1 + red2) / 2;
            let red_diff = red1 - red2;
            let green_diff = green1 - green2;
            let blue_diff = blue1 - blue2;

            // Redmean weighted Euclidean distance (×256, integer-only).
            // Equivalent to the f64 formula divided by 256; ordering is
            // identical so the minimum is unchanged.
            let distance_sq = (512 + red_mean) * red_diff * red_diff
                + 1024 * green_diff * green_diff
                + (767 - red_mean) * blue_diff * blue_diff;

            if distance_sq < min_distance {
                min_distance = distance_sq;
                min_index = index;
            }
        }

        min_index
    }
}

/// Standard 16-color ANSI palette.
pub static STANDARD_PALETTE: std::sync::LazyLock<Palette> = std::sync::LazyLock::new(|| {
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
pub static WINDOWS_PALETTE: std::sync::LazyLock<Palette> = std::sync::LazyLock::new(|| {
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
pub static EIGHT_BIT_PALETTE: std::sync::LazyLock<Palette> =
    std::sync::LazyLock::new(|| Palette::new(generate_eight_bit_palette()));

#[cfg(test)]
/// Test-only accessor: number of entries in the match_color thread-local
/// cache (across all palettes, in the current thread).
fn match_color_cache_size() -> usize {
    MATCH_COLOR_CACHE.with(|c| c.borrow().len())
}

#[cfg(test)]
/// Test-only accessor: cumulative miss count for the match_color cache in
/// the current thread. Monotonic across the thread's lifetime.
fn match_color_misses() -> u64 {
    MATCH_COLOR_MISSES.with(|m| *m.borrow())
}

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
    #[test]
    fn match_color_uses_thread_local_cache() {
        // Task 3 (Phase 7): Palette::match_color must cache (palette, triplet)
        // -> index lookups in a thread-local LRU. Verify by:
        //   1. miss counter is 0 on a fresh thread-local state (best-effort:
        //      see RESET below — counter is monotonic across tests, so we
        //      snapshot the baseline and assert the *delta*).
        //   2. first call for a triplet increments miss counter by 1.
        //   3. repeat call for the same triplet does NOT increment miss
        //      counter (cache hit).
        //   4. cache_size() grows with unique triplets and is bounded.
        let palette = STANDARD_PALETTE.clone();
        let triplet_a = ColorTriplet::new(120, 60, 30);
        let triplet_b = ColorTriplet::new(50, 200, 80);
        let triplet_c = ColorTriplet::new(10, 10, 200);

        let baseline_misses = match_color_misses();
        let baseline_size = match_color_cache_size();

        // First call for each unique triplet — each should miss.
        let idx_a1 = palette.match_color(&triplet_a);
        let idx_b1 = palette.match_color(&triplet_b);
        let idx_c1 = palette.match_color(&triplet_c);
        let misses_after_first = match_color_misses();
        assert_eq!(
            misses_after_first - baseline_misses,
            3,
            "first calls for 3 unique triplets must each miss the cache",
        );
        assert_eq!(
            match_color_cache_size() - baseline_size,
            3,
            "cache must hold 3 entries after 3 unique lookups",
        );

        // Repeat calls — all must hit the cache (deterministic result, no
        // new misses).
        let idx_a2 = palette.match_color(&triplet_a);
        let idx_b2 = palette.match_color(&triplet_b);
        let idx_c2 = palette.match_color(&triplet_c);
        assert_eq!(idx_a2, idx_a1);
        assert_eq!(idx_b2, idx_b1);
        assert_eq!(idx_c2, idx_c1);
        let misses_after_repeat = match_color_misses();
        assert_eq!(
            misses_after_repeat, misses_after_first,
            "repeat calls must be cache hits (no new misses)",
        );
    }
}
