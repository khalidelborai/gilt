//! Progress bar renderable -- a styled progress bar with pulse animation.
//!
//! Rust port of Python's `rich/progress_bar.py`.
//!
//! Renders a horizontal progress bar using Unicode box-drawing characters,
//! with support for completed/remaining portions, pulse animation, and
//! half-character precision.

use std::f64::consts::PI;
use std::fmt;
use std::time::SystemTime;

use crate::color::{blend_rgb, Color, ColorSystem};
use crate::color::color_triplet::ColorTriplet;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

/// Number of characters before the pulse animation repeats.
const PULSE_SIZE: usize = 20;

// ---------------------------------------------------------------------------
// ProgressBar struct
// ---------------------------------------------------------------------------

/// Renders a styled progress bar with completed/remaining portions and
/// optional pulse animation.
///
/// Used by rich.progress to visualise task completion.
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// Total number of steps (None = indeterminate/pulse mode).
    pub total: Option<f64>,
    /// Number of steps completed.
    pub completed: f64,
    /// Fixed width in cells, or None to use max_width from console options.
    pub width: Option<usize>,
    /// Enable pulse animation.
    pub pulse: bool,
    /// Style name for bar background.
    pub style: String,
    /// Style name for completed portion.
    pub complete_style: String,
    /// Style name for finished bar.
    pub finished_style: String,
    /// Style name for pulse animation.
    pub pulse_style: String,
    /// Fixed time for animation (None = use system time).
    pub animation_time: Option<f64>,
}

impl ProgressBar {
    /// Create a new `ProgressBar` with sensible defaults.
    ///
    /// Defaults: total=100.0, completed=0, pulse=false, styles use bar.*
    pub fn new() -> Self {
        Self {
            total: Some(100.0),
            completed: 0.0,
            width: None,
            pulse: false,
            style: "bar.back".to_string(),
            complete_style: "bar.complete".to_string(),
            finished_style: "bar.finished".to_string(),
            pulse_style: "bar.pulse".to_string(),
            animation_time: None,
        }
    }

    /// Set the total steps (builder pattern).
    #[must_use]
    pub fn with_total(mut self, total: Option<f64>) -> Self {
        self.total = total;
        self
    }

    /// Set the completed steps (builder pattern).
    #[must_use]
    pub fn with_completed(mut self, completed: f64) -> Self {
        self.completed = completed;
        self
    }

    /// Set a fixed width (builder pattern).
    #[must_use]
    pub fn with_width(mut self, width: Option<usize>) -> Self {
        self.width = width;
        self
    }

    /// Set the pulse flag (builder pattern).
    #[must_use]
    pub fn with_pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    /// Set the background style name (builder pattern).
    #[must_use]
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = style.to_string();
        self
    }

    /// Set the completed portion style name (builder pattern).
    #[must_use]
    pub fn with_complete_style(mut self, style: &str) -> Self {
        self.complete_style = style.to_string();
        self
    }

    /// Set the finished bar style name (builder pattern).
    #[must_use]
    pub fn with_finished_style(mut self, style: &str) -> Self {
        self.finished_style = style.to_string();
        self
    }

    /// Set the pulse animation style name (builder pattern).
    #[must_use]
    pub fn with_pulse_style(mut self, style: &str) -> Self {
        self.pulse_style = style.to_string();
        self
    }

    /// Set a fixed animation time (builder pattern).
    #[must_use]
    pub fn with_animation_time(mut self, time: Option<f64>) -> Self {
        self.animation_time = time;
        self
    }

    /// Calculate percentage complete, clamped to 0..100.
    ///
    /// Returns `None` if total is `None` (indeterminate mode).
    pub fn percentage_completed(&self) -> Option<f64> {
        self.total.map(|total| {
            let pct = (self.completed / total) * 100.0;
            pct.clamp(0.0, 100.0)
        })
    }

    /// Update progress with new completed value and optional new total.
    pub fn update(&mut self, completed: f64, total: Option<f64>) {
        self.completed = completed;
        if let Some(t) = total {
            self.total = Some(t);
        }
    }

    /// Return the measurement for this progress bar.
    pub fn measure(&self, _console: &Console, options: &ConsoleOptions) -> Measurement {
        if let Some(w) = self.width {
            Measurement::new(w, w)
        } else {
            Measurement::new(4, options.max_width)
        }
    }

    /// Generate pulse animation segments.
    ///
    /// Creates PULSE_SIZE segments with cosine-blended colors between the
    /// pulse foreground and bar background styles, then tiles them across
    /// the given width with a time-based offset for scrolling.
    fn get_pulse_segments(
        &self,
        fore_style: &Style,
        back_style: &Style,
        color_system: Option<ColorSystem>,
    ) -> Vec<Segment> {
        let bar = "\u{2501}"; // ━

        // If color system is insufficient for gradients, fall back to simple pulse
        let has_color = matches!(
            color_system,
            Some(ColorSystem::Standard | ColorSystem::EightBit | ColorSystem::TrueColor)
        );
        if !has_color {
            let mut segments = Vec::with_capacity(PULSE_SIZE);
            let half = PULSE_SIZE / 2;
            for _ in 0..half {
                segments.push(Segment::styled(bar, fore_style.clone()));
            }
            let back_char = if color_system.is_none() { " " } else { bar };
            for _ in 0..(PULSE_SIZE - half) {
                segments.push(Segment::styled(back_char, back_style.clone()));
            }
            return segments;
        }

        let fore_color = fore_style
            .color()
            .map(|c| c.get_truecolor(None, true))
            .unwrap_or(ColorTriplet::new(255, 0, 255));

        let back_color = back_style
            .color()
            .map(|c| c.get_truecolor(None, true))
            .unwrap_or(ColorTriplet::new(0, 0, 0));

        let mut segments = Vec::with_capacity(PULSE_SIZE);
        for index in 0..PULSE_SIZE {
            let position = index as f64 / PULSE_SIZE as f64;
            let fade = 0.5 + (position * PI * 2.0).cos() / 2.0;
            let color = blend_rgb(fore_color, back_color, fade);
            let style = Style::from_color(Some(Color::from_triplet(color)), None);
            segments.push(Segment::styled(bar, style));
        }

        segments
    }

    /// Render pulse animation across the given width.
    fn render_pulse(&self, console: &Console, width: usize) -> Vec<Segment> {
        let fore_style = console
            .get_style(&self.pulse_style)
            .unwrap_or_else(|_| Style::parse("white").unwrap_or_else(|_| Style::null()));
        let back_style = console
            .get_style(&self.style)
            .unwrap_or_else(|_| Style::parse("black").unwrap_or_else(|_| Style::null()));

        let color_system = console.color_system();
        let pulse_segments = self.get_pulse_segments(&fore_style, &back_style, color_system);
        let segment_count = pulse_segments.len();
        if segment_count == 0 {
            return Vec::new();
        }

        let current_time = self.animation_time.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0)
        });

        // Tile segments across width with scrolling offset
        let repeats = width / segment_count + 2;
        let mut tiled: Vec<Segment> = Vec::with_capacity(repeats * segment_count);
        for _ in 0..repeats {
            tiled.extend(pulse_segments.iter().cloned());
        }

        let offset = ((-current_time * 15.0) as isize).rem_euclid(segment_count as isize) as usize;
        tiled[offset..offset + width].to_vec()
    }
}

// ---------------------------------------------------------------------------
// Default
// ---------------------------------------------------------------------------

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for ProgressBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut console = Console::builder()
            .width(f.width().unwrap_or(80))
            .force_terminal(true)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(self);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for ProgressBar {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let width = match self.width {
            Some(w) => w.min(options.max_width),
            None => options.max_width,
        };

        let ascii = options.legacy_windows || options.ascii_only();
        let should_pulse = self.pulse || self.total.is_none();

        if should_pulse {
            return self.render_pulse(console, width);
        }

        // Normal mode: render completed/remaining portions
        let total = self.total.unwrap_or(100.0);
        let completed = self.completed.clamp(0.0, total);

        let bar = if ascii { "-" } else { "\u{2501}" }; // ━
        let half_bar_right = if ascii { " " } else { "\u{257A}" }; // ╸
        let half_bar_left = if ascii { " " } else { "\u{2578}" }; // ╺

        let complete_halves = if total > 0.0 {
            (width as f64 * 2.0 * completed / total) as usize
        } else {
            width * 2
        };
        let bar_count = complete_halves / 2;
        let half_bar_count = complete_halves % 2;

        let back_style = console
            .get_style(&self.style)
            .unwrap_or_else(|_| Style::null());
        let is_finished = completed >= total;
        let complete_style = if is_finished {
            console
                .get_style(&self.finished_style)
                .unwrap_or_else(|_| Style::null())
        } else {
            console
                .get_style(&self.complete_style)
                .unwrap_or_else(|_| Style::null())
        };

        let mut segments = Vec::new();

        if bar_count > 0 {
            segments.push(Segment::styled(
                &bar.repeat(bar_count),
                complete_style.clone(),
            ));
        }
        if half_bar_count > 0 {
            segments.push(Segment::styled(half_bar_right, complete_style.clone()));
        }

        // Remaining portion (only when color system is active)
        if console.color_system().is_some() {
            let remaining_bars = width.saturating_sub(bar_count + half_bar_count);
            if remaining_bars > 0 {
                if half_bar_count == 0 && bar_count > 0 {
                    segments.push(Segment::styled(half_bar_left, back_style.clone()));
                    let after = remaining_bars.saturating_sub(1);
                    if after > 0 {
                        segments.push(Segment::styled(&bar.repeat(after), back_style));
                    }
                } else {
                    segments.push(Segment::styled(&bar.repeat(remaining_bars), back_style));
                }
            }
        }

        segments
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::{ConsoleDimensions, ConsoleOptions};

    /// Build a `ConsoleOptions` with a given `max_width`.
    fn make_options(max_width: usize) -> ConsoleOptions {
        ConsoleOptions {
            size: ConsoleDimensions {
                width: max_width,
                height: 25,
            },
            legacy_windows: false,
            min_width: 1,
            max_width,
            is_terminal: false,
            encoding: "utf-8".to_string(),
            max_height: 25,
            justify: None,
            overflow: None,
            no_wrap: false,
            highlight: None,
            markup: None,
            height: None,
        }
    }

    /// Render a ProgressBar through its Renderable impl and return segments.
    fn render_segments(bar: &ProgressBar, max_width: usize) -> Vec<Segment> {
        let console = Console::builder()
            .width(max_width)
            .color_system("truecolor")
            .build();
        let opts = make_options(max_width);
        bar.gilt_console(&console, &opts)
    }

    /// Render a ProgressBar and return the concatenated text (ignoring styles).
    fn render_text(bar: &ProgressBar, max_width: usize) -> String {
        let segments = render_segments(bar, max_width);
        segments.iter().map(|s| s.text.as_str()).collect()
    }

    // -- Construction -------------------------------------------------------

    #[test]
    fn test_default_construction() {
        let bar = ProgressBar::new();
        assert_eq!(bar.total, Some(100.0));
        assert_eq!(bar.completed, 0.0);
        assert_eq!(bar.width, None);
        assert!(!bar.pulse);
        assert_eq!(bar.style, "bar.back");
        assert_eq!(bar.complete_style, "bar.complete");
        assert_eq!(bar.finished_style, "bar.finished");
        assert_eq!(bar.pulse_style, "bar.pulse");
        assert_eq!(bar.animation_time, None);
    }

    #[test]
    fn test_default_trait() {
        let bar = ProgressBar::default();
        assert_eq!(bar.total, Some(100.0));
        assert_eq!(bar.completed, 0.0);
    }

    // -- Builder methods ----------------------------------------------------

    #[test]
    fn test_with_total() {
        let bar = ProgressBar::new().with_total(Some(200.0));
        assert_eq!(bar.total, Some(200.0));
    }

    #[test]
    fn test_with_total_none() {
        let bar = ProgressBar::new().with_total(None);
        assert_eq!(bar.total, None);
    }

    #[test]
    fn test_with_completed() {
        let bar = ProgressBar::new().with_completed(42.0);
        assert_eq!(bar.completed, 42.0);
    }

    #[test]
    fn test_with_width() {
        let bar = ProgressBar::new().with_width(Some(50));
        assert_eq!(bar.width, Some(50));
    }

    #[test]
    fn test_with_width_none() {
        let bar = ProgressBar::new().with_width(None);
        assert_eq!(bar.width, None);
    }

    #[test]
    fn test_with_pulse() {
        let bar = ProgressBar::new().with_pulse(true);
        assert!(bar.pulse);
    }

    #[test]
    fn test_with_style() {
        let bar = ProgressBar::new().with_style("custom.back");
        assert_eq!(bar.style, "custom.back");
    }

    #[test]
    fn test_with_complete_style() {
        let bar = ProgressBar::new().with_complete_style("custom.complete");
        assert_eq!(bar.complete_style, "custom.complete");
    }

    #[test]
    fn test_with_finished_style() {
        let bar = ProgressBar::new().with_finished_style("custom.finished");
        assert_eq!(bar.finished_style, "custom.finished");
    }

    #[test]
    fn test_with_pulse_style() {
        let bar = ProgressBar::new().with_pulse_style("custom.pulse");
        assert_eq!(bar.pulse_style, "custom.pulse");
    }

    #[test]
    fn test_with_animation_time() {
        let bar = ProgressBar::new().with_animation_time(Some(1.5));
        assert_eq!(bar.animation_time, Some(1.5));
    }

    #[test]
    fn test_builder_chaining() {
        let bar = ProgressBar::new()
            .with_total(Some(200.0))
            .with_completed(50.0)
            .with_width(Some(40))
            .with_pulse(false)
            .with_style("red")
            .with_animation_time(Some(2.0));
        assert_eq!(bar.total, Some(200.0));
        assert_eq!(bar.completed, 50.0);
        assert_eq!(bar.width, Some(40));
        assert!(!bar.pulse);
        assert_eq!(bar.style, "red");
        assert_eq!(bar.animation_time, Some(2.0));
    }

    // -- percentage_completed -----------------------------------------------

    #[test]
    fn test_percentage_completed_normal() {
        let bar = ProgressBar::new().with_completed(50.0);
        assert_eq!(bar.percentage_completed(), Some(50.0));
    }

    #[test]
    fn test_percentage_completed_zero() {
        let bar = ProgressBar::new().with_completed(0.0);
        assert_eq!(bar.percentage_completed(), Some(0.0));
    }

    #[test]
    fn test_percentage_completed_full() {
        let bar = ProgressBar::new().with_completed(100.0);
        assert_eq!(bar.percentage_completed(), Some(100.0));
    }

    #[test]
    fn test_percentage_completed_over_100() {
        let bar = ProgressBar::new().with_completed(150.0);
        // Should clamp to 100
        assert_eq!(bar.percentage_completed(), Some(100.0));
    }

    #[test]
    fn test_percentage_completed_negative() {
        let bar = ProgressBar::new().with_completed(-10.0);
        // Should clamp to 0
        assert_eq!(bar.percentage_completed(), Some(0.0));
    }

    #[test]
    fn test_percentage_completed_none_total() {
        let bar = ProgressBar::new().with_total(None);
        assert_eq!(bar.percentage_completed(), None);
    }

    #[test]
    fn test_percentage_completed_custom_total() {
        let bar = ProgressBar::new()
            .with_total(Some(200.0))
            .with_completed(100.0);
        assert_eq!(bar.percentage_completed(), Some(50.0));
    }

    // -- Display trait ------------------------------------------------------

    #[test]
    fn test_display_with_total() {
        let bar = ProgressBar::new().with_completed(50.0);
        let s = format!("{bar}");
        // Console-based rendering produces bar characters, not debug format
        assert!(!s.is_empty());
    }

    #[test]
    fn test_display_without_total() {
        let bar = ProgressBar::new().with_total(None).with_completed(30.0);
        let s = format!("{bar}");
        // Indeterminate bar renders pulse animation characters
        assert!(!s.is_empty());
    }

    #[test]
    fn test_display_zero() {
        let bar = ProgressBar::new();
        let s = format!("{bar}");
        // Zero progress still renders the bar track
        // (may be empty if bar renders nothing for 0%)
        let _ = s;
    }

    // -- update() method ----------------------------------------------------

    #[test]
    fn test_update_completed() {
        let mut bar = ProgressBar::new();
        bar.update(75.0, None);
        assert_eq!(bar.completed, 75.0);
        assert_eq!(bar.total, Some(100.0)); // unchanged
    }

    #[test]
    fn test_update_completed_and_total() {
        let mut bar = ProgressBar::new();
        bar.update(50.0, Some(200.0));
        assert_eq!(bar.completed, 50.0);
        assert_eq!(bar.total, Some(200.0));
    }

    #[test]
    fn test_update_preserves_total_when_none() {
        let mut bar = ProgressBar::new().with_total(Some(50.0));
        bar.update(25.0, None);
        assert_eq!(bar.total, Some(50.0));
    }

    // -- Normal rendering: bar characters -----------------------------------

    #[test]
    fn test_render_empty_bar() {
        let bar = ProgressBar::new().with_completed(0.0).with_width(Some(10));
        let text = render_text(&bar, 10);
        // At 0%, the entire bar should be remaining (back style)
        // Should use ━ characters for the remaining portion
        assert_eq!(text.chars().count(), 10);
        // No completed portion, all remaining
        assert!(!text.is_empty());
    }

    #[test]
    fn test_render_half_bar() {
        let bar = ProgressBar::new().with_completed(50.0).with_width(Some(10));
        let text = render_text(&bar, 10);
        // Half complete: 5 complete chars + 5 remaining chars = 10 total
        assert_eq!(text.chars().count(), 10);
        assert!(text.contains('\u{2501}')); // ━
    }

    #[test]
    fn test_render_full_bar() {
        let bar = ProgressBar::new()
            .with_completed(100.0)
            .with_width(Some(10));
        let text = render_text(&bar, 10);
        // Full bar: all ━ characters
        assert_eq!(text.chars().count(), 10);
        let expected = "\u{2501}".repeat(10);
        assert_eq!(text, expected);
    }

    #[test]
    fn test_render_over_100_percent() {
        let bar = ProgressBar::new()
            .with_completed(150.0)
            .with_width(Some(10));
        let text = render_text(&bar, 10);
        // Over 100%: should be clamped to full bar
        assert_eq!(text.chars().count(), 10);
        let expected = "\u{2501}".repeat(10);
        assert_eq!(text, expected);
    }

    #[test]
    fn test_bar_uses_correct_characters() {
        let bar = ProgressBar::new().with_completed(50.0).with_width(Some(20));
        let text = render_text(&bar, 20);
        // Should use ━ (U+2501), and possibly ╸ (U+257A) and ╺ (U+2578)
        for ch in text.chars() {
            assert!(
                ch == '\u{2501}' || ch == '\u{257A}' || ch == '\u{2578}',
                "unexpected character: {:?} (U+{:04X})",
                ch,
                ch as u32
            );
        }
    }

    #[test]
    fn test_half_bar_right_character() {
        // With an odd number of complete halves, we should get a ╸ character
        // 25% of 10 = 2.5 bars = 5 halves => 2 full + 1 half
        let bar = ProgressBar::new().with_completed(25.0).with_width(Some(10));
        let text = render_text(&bar, 10);
        assert!(text.contains('\u{257A}'), "expected ╸ in output: {text}");
    }

    #[test]
    fn test_half_bar_left_character() {
        // When there are complete bars but no half bar, we should see ╺
        // 40% of 10 = 4 bars = 8 halves => 4 full + 0 half
        // With bar_count=4, half_bar_count=0, and bar_count>0,
        // the first remaining char should be ╺
        let bar = ProgressBar::new().with_completed(40.0).with_width(Some(10));
        let text = render_text(&bar, 10);
        assert!(text.contains('\u{2578}'), "expected ╺ in output: {text}");
    }

    // -- Finished style applied when completed >= total ---------------------

    #[test]
    fn test_finished_style_applied() {
        let bar = ProgressBar::new()
            .with_completed(100.0)
            .with_width(Some(10))
            .with_complete_style("red")
            .with_finished_style("green");
        let console = Console::builder()
            .width(10)
            .color_system("truecolor")
            .build();
        let opts = make_options(10);
        let segments = bar.gilt_console(&console, &opts);
        // Should use the "green" (finished) style, not "red" (complete)
        assert!(!segments.is_empty());
        let first = &segments[0];
        let finished = console.get_style("green").unwrap();
        assert_eq!(first.style, Some(finished));
    }

    #[test]
    fn test_complete_style_when_not_finished() {
        let bar = ProgressBar::new()
            .with_completed(50.0)
            .with_width(Some(10))
            .with_complete_style("red")
            .with_finished_style("green");
        let console = Console::builder()
            .width(10)
            .color_system("truecolor")
            .build();
        let opts = make_options(10);
        let segments = bar.gilt_console(&console, &opts);
        assert!(!segments.is_empty());
        let first = &segments[0];
        let complete = console.get_style("red").unwrap();
        assert_eq!(first.style, Some(complete));
    }

    // -- Pulse rendering ----------------------------------------------------

    #[test]
    fn test_pulse_rendering_width() {
        let bar = ProgressBar::new()
            .with_total(None)
            .with_width(Some(30))
            .with_animation_time(Some(0.0));
        let segments = render_segments(&bar, 30);
        // Pulse should produce exactly 30 segments (one per character)
        assert_eq!(segments.len(), 30);
    }

    #[test]
    fn test_pulse_rendering_enabled_by_flag() {
        let bar = ProgressBar::new()
            .with_pulse(true)
            .with_width(Some(20))
            .with_animation_time(Some(0.0));
        let segments = render_segments(&bar, 20);
        assert_eq!(segments.len(), 20);
    }

    #[test]
    fn test_pulse_rendering_each_segment_is_bar_char() {
        let bar = ProgressBar::new()
            .with_total(None)
            .with_width(Some(20))
            .with_animation_time(Some(0.0));
        let segments = render_segments(&bar, 20);
        for seg in &segments {
            assert_eq!(seg.text, "\u{2501}");
        }
    }

    #[test]
    fn test_pulse_color_blending_produces_gradient() {
        let bar = ProgressBar::new()
            .with_total(None)
            .with_width(Some(PULSE_SIZE))
            .with_animation_time(Some(0.0))
            .with_pulse_style("white")
            .with_style("black");
        let console = Console::builder()
            .width(PULSE_SIZE)
            .color_system("truecolor")
            .build();
        let opts = make_options(PULSE_SIZE);
        let segments = bar.gilt_console(&console, &opts);

        // Collect unique styles to verify gradient (not all the same)
        let styles: Vec<_> = segments.iter().map(|s| s.style.clone()).collect();
        let unique_count = styles
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        // Should have multiple different colors in the gradient
        assert!(
            unique_count > 1,
            "pulse gradient should have more than 1 unique color, got {unique_count}"
        );
    }

    #[test]
    fn test_pulse_scrolling_offset() {
        let bar1 = ProgressBar::new()
            .with_total(None)
            .with_width(Some(20))
            .with_animation_time(Some(0.0));
        let bar2 = ProgressBar::new()
            .with_total(None)
            .with_width(Some(20))
            .with_animation_time(Some(1.0));

        let seg1 = render_segments(&bar1, 20);
        let seg2 = render_segments(&bar2, 20);

        // Different animation times should produce different segment ordering
        let styles1: Vec<_> = seg1.iter().map(|s| s.style.clone()).collect();
        let styles2: Vec<_> = seg2.iter().map(|s| s.style.clone()).collect();
        assert_ne!(
            styles1, styles2,
            "different animation times should produce different patterns"
        );
    }

    // -- Fixed width vs max_width -------------------------------------------

    #[test]
    fn test_fixed_width() {
        let bar = ProgressBar::new().with_completed(50.0).with_width(Some(15));
        let text = render_text(&bar, 80);
        assert_eq!(text.chars().count(), 15);
    }

    #[test]
    fn test_max_width_used_when_no_fixed_width() {
        let bar = ProgressBar::new().with_completed(50.0);
        let text = render_text(&bar, 40);
        assert_eq!(text.chars().count(), 40);
    }

    #[test]
    fn test_fixed_width_capped_by_max_width() {
        let bar = ProgressBar::new()
            .with_completed(50.0)
            .with_width(Some(100));
        let text = render_text(&bar, 30);
        // width=100 but max_width=30, so capped to 30
        assert_eq!(text.chars().count(), 30);
    }

    // -- Measure ------------------------------------------------------------

    #[test]
    fn test_measure_with_fixed_width() {
        let bar = ProgressBar::new().with_width(Some(25));
        let console = Console::new();
        let opts = make_options(80);
        let m = bar.measure(&console, &opts);
        assert_eq!(m, Measurement::new(25, 25));
    }

    #[test]
    fn test_measure_without_fixed_width() {
        let bar = ProgressBar::new();
        let console = Console::new();
        let opts = make_options(80);
        let m = bar.measure(&console, &opts);
        assert_eq!(m, Measurement::new(4, 80));
    }

    #[test]
    fn test_measure_max_width_varies() {
        let bar = ProgressBar::new();
        let console = Console::new();
        let opts = make_options(120);
        let m = bar.measure(&console, &opts);
        assert_eq!(m, Measurement::new(4, 120));
    }

    // -- Renderable trait integration ---------------------------------------

    #[test]
    fn test_renderable_trait() {
        let bar = ProgressBar::new().with_completed(50.0).with_width(Some(10));
        let console = Console::builder()
            .width(80)
            .color_system("truecolor")
            .build();
        let opts = make_options(80);
        let renderable: &dyn Renderable = &bar;
        let segments = renderable.gilt_console(&console, &opts);
        assert!(!segments.is_empty());
    }

    // -- Width consistency across fill levels -------------------------------

    #[test]
    fn test_width_consistency() {
        for pct in (0..=100).step_by(5) {
            let bar = ProgressBar::new()
                .with_completed(pct as f64)
                .with_width(Some(20));
            let text = render_text(&bar, 20);
            assert_eq!(text.chars().count(), 20, "width mismatch at {}%", pct);
        }
    }

    #[test]
    fn test_width_consistency_odd_widths() {
        for width in [1, 3, 7, 11, 13, 17, 19, 23] {
            let bar = ProgressBar::new()
                .with_completed(50.0)
                .with_width(Some(width));
            let text = render_text(&bar, width);
            assert_eq!(
                text.chars().count(),
                width,
                "width mismatch for width={}",
                width
            );
        }
    }

    // -- Clone and Debug ---------------------------------------------------

    #[test]
    fn test_clone() {
        let bar = ProgressBar::new().with_completed(42.0).with_width(Some(30));
        let cloned = bar.clone();
        assert_eq!(cloned.total, bar.total);
        assert_eq!(cloned.completed, bar.completed);
        assert_eq!(cloned.width, bar.width);
        assert_eq!(cloned.pulse, bar.pulse);
        assert_eq!(cloned.style, bar.style);
    }

    #[test]
    fn test_debug() {
        let bar = ProgressBar::new().with_completed(50.0);
        let debug = format!("{bar:?}");
        assert!(debug.contains("ProgressBar"));
        assert!(debug.contains("50"));
    }

    // -- Edge cases ---------------------------------------------------------

    #[test]
    fn test_zero_total() {
        let bar = ProgressBar::new()
            .with_total(Some(0.0))
            .with_completed(0.0)
            .with_width(Some(10));
        let text = render_text(&bar, 10);
        // Zero total treated as full bar
        assert_eq!(text.chars().count(), 10);
    }

    #[test]
    fn test_very_small_completed() {
        let bar = ProgressBar::new()
            .with_completed(0.001)
            .with_width(Some(10));
        let text = render_text(&bar, 10);
        assert_eq!(text.chars().count(), 10);
    }

    #[test]
    fn test_width_one() {
        let bar = ProgressBar::new().with_completed(50.0).with_width(Some(1));
        let text = render_text(&bar, 1);
        assert_eq!(text.chars().count(), 1);
    }

    #[test]
    fn test_pulse_with_width_one() {
        let bar = ProgressBar::new()
            .with_total(None)
            .with_width(Some(1))
            .with_animation_time(Some(0.0));
        let segments = render_segments(&bar, 1);
        assert_eq!(segments.len(), 1);
    }

    #[test]
    fn test_display_trait() {
        let bar = ProgressBar::new().with_completed(50.0);
        let s = format!("{}", bar);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_display_with_width() {
        let bar = ProgressBar::new().with_completed(75.0);
        let s = format!("{:40}", bar);
        assert!(!s.is_empty());
    }
}
