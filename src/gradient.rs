//! True color gradient rendering across text.
//!
//! This module provides the [`Gradient`] widget that creates smoothly
//! interpolated color gradients across text, supporting multi-stop
//! gradients, rainbow presets, and per-character color blending.
//!
//! # Example
//!
//! ```rust
//! use gilt::gradient::Gradient;
//! use gilt::color::Color;
//!
//! // Two-color gradient from red to blue
//! let g = Gradient::two_color("Hello, world!", Color::from_rgb(255, 0, 0), Color::from_rgb(0, 0, 255));
//!
//! // Rainbow gradient
//! let g = Gradient::rainbow("All the colors!");
//! ```

use crate::color::Color;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::JustifyMethod;

// ---------------------------------------------------------------------------
// Color interpolation
// ---------------------------------------------------------------------------

/// Linearly interpolates between two colors at parameter `t` (0.0 = c1, 1.0 = c2).
///
/// Both colors are resolved to RGB triplets (truecolor) and each channel is
/// interpolated independently.  The result is always a `Color::TrueColor`.
fn interpolate_color(c1: &Color, c2: &Color, t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let t1 = c1.get_truecolor(None, true);
    let t2 = c2.get_truecolor(None, true);

    let r = (t1.red as f64 + (t2.red as f64 - t1.red as f64) * t).round() as u8;
    let g = (t1.green as f64 + (t2.green as f64 - t1.green as f64) * t).round() as u8;
    let b = (t1.blue as f64 + (t2.blue as f64 - t1.blue as f64) * t).round() as u8;

    Color::from_rgb(r, g, b)
}

// ---------------------------------------------------------------------------
// Gradient
// ---------------------------------------------------------------------------

/// A text widget that renders with a smooth color gradient across characters.
///
/// The gradient distributes the given color stops evenly across the text
/// length and interpolates between adjacent stops for each character.
#[derive(Debug, Clone)]
pub struct Gradient {
    /// The plain text to render.
    pub text: String,
    /// Gradient color stops (at least 2 for a visible gradient).
    pub colors: Vec<Color>,
    /// Base style applied to every character (bold, italic, etc.).
    /// The foreground color in this style is *overridden* by the gradient.
    pub style: Style,
    /// Optional text justification.
    pub justify: Option<JustifyMethod>,
}

impl Gradient {
    // -- constructors -------------------------------------------------------

    /// Creates a new `Gradient` with the given text and color stops.
    ///
    /// At least two colors should be provided; with fewer than two the text
    /// will render in a single color (the first stop, or default if empty).
    pub fn new(text: &str, colors: Vec<Color>) -> Self {
        Self {
            text: text.to_string(),
            colors,
            style: Style::null(),
            justify: None,
        }
    }

    /// Creates a simple two-color gradient.
    pub fn two_color(text: &str, start: Color, end: Color) -> Self {
        Self::new(text, vec![start, end])
    }

    /// Creates a rainbow gradient (red -> orange -> yellow -> green -> cyan -> blue -> violet).
    pub fn rainbow(text: &str) -> Self {
        Self::new(
            text,
            vec![
                Color::from_rgb(255, 0, 0),     // red
                Color::from_rgb(255, 165, 0),   // orange
                Color::from_rgb(255, 255, 0),   // yellow
                Color::from_rgb(0, 255, 0),     // green
                Color::from_rgb(0, 255, 255),   // cyan
                Color::from_rgb(0, 0, 255),     // blue
                Color::from_rgb(148, 0, 211),   // violet
            ],
        )
    }

    // -- builder methods ----------------------------------------------------

    /// Sets the base style for the gradient text.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the text justification.
    #[must_use]
    pub fn with_justify(mut self, justify: JustifyMethod) -> Self {
        self.justify = Some(justify);
        self
    }

    // -- internal helpers ---------------------------------------------------

    /// Computes the interpolated color for position `index` out of `total`
    /// characters, distributing `self.colors` evenly.
    fn color_at(&self, index: usize, total: usize) -> Color {
        if self.colors.is_empty() {
            return Color::default_color();
        }
        if self.colors.len() == 1 || total <= 1 {
            return self.colors[0].clone();
        }

        let t = index as f64 / (total - 1) as f64; // 0.0 .. 1.0
        let segments = self.colors.len() - 1;
        let scaled = t * segments as f64;
        let seg = (scaled.floor() as usize).min(segments - 1);
        let local_t = scaled - seg as f64;

        interpolate_color(&self.colors[seg], &self.colors[seg + 1], local_t)
    }

    /// Renders a single line of text into gradient-colored segments.
    fn render_line(&self, line: &str, style: &Style) -> Vec<Segment> {
        let chars: Vec<char> = line.chars().collect();
        let total = chars.len();
        if total == 0 {
            return Vec::new();
        }

        let mut segments = Vec::with_capacity(total);
        for (i, ch) in chars.iter().enumerate() {
            let fg = self.color_at(i, total);
            let char_style = Style::from_color(Some(fg), None) + style.clone();
            segments.push(Segment::styled(&ch.to_string(), char_style));
        }
        segments
    }
}

// ---------------------------------------------------------------------------
// Renderable
// ---------------------------------------------------------------------------

impl Renderable for Gradient {
    fn rich_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let justify = self.justify.or(options.justify);

        let lines: Vec<&str> = self.text.split('\n').collect();
        let mut all_segments = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let mut line_segs = self.render_line(line, &self.style);

            // Apply justification if requested
            if let Some(just) = justify {
                let line_len = line.chars().count();
                if line_len < options.max_width {
                    let padding = options.max_width - line_len;
                    match just {
                        JustifyMethod::Center => {
                            let left = padding / 2;
                            let right = padding - left;
                            let mut padded = vec![Segment::styled(
                                &" ".repeat(left),
                                self.style.clone(),
                            )];
                            padded.append(&mut line_segs);
                            padded.push(Segment::styled(
                                &" ".repeat(right),
                                self.style.clone(),
                            ));
                            line_segs = padded;
                        }
                        JustifyMethod::Right => {
                            let mut padded = vec![Segment::styled(
                                &" ".repeat(padding),
                                self.style.clone(),
                            )];
                            padded.append(&mut line_segs);
                            line_segs = padded;
                        }
                        JustifyMethod::Left | JustifyMethod::Full | JustifyMethod::Default => {
                            // Left-align: pad on the right
                            line_segs.push(Segment::styled(
                                &" ".repeat(padding),
                                self.style.clone(),
                            ));
                        }
                    }
                }
            }

            all_segments.append(&mut line_segs);

            // Add newline between lines (and after the last line)
            if line_idx < lines.len() - 1 {
                all_segments.push(Segment::line());
            }
        }

        // Trailing newline
        all_segments.push(Segment::new("\n", None, None));

        all_segments
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Gradient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color_triplet::ColorTriplet;

    #[test]
    fn test_two_color_gradient_segment_count() {
        let g = Gradient::two_color(
            "Hello",
            Color::from_rgb(255, 0, 0),
            Color::from_rgb(0, 0, 255),
        );
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .build();
        let options = console.options();
        let segments = g.rich_console(&console, &options);
        // 5 char segments + 1 trailing newline
        assert_eq!(segments.len(), 6);
        // First 5 segments should each be one character
        for seg in &segments[..5] {
            assert_eq!(seg.text.chars().count(), 1);
        }
    }

    #[test]
    fn test_rainbow_gradient_works() {
        let g = Gradient::rainbow("Rainbow!");
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .build();
        let options = console.options();
        let segments = g.rich_console(&console, &options);
        // 8 characters + 1 trailing newline
        assert_eq!(segments.len(), 9);
    }

    #[test]
    fn test_interpolation_at_zero() {
        let c1 = Color::from_rgb(255, 0, 0);
        let c2 = Color::from_rgb(0, 0, 255);
        let result = interpolate_color(&c1, &c2, 0.0);
        let triplet = result.get_truecolor(None, true);
        assert_eq!(triplet, ColorTriplet::new(255, 0, 0));
    }

    #[test]
    fn test_interpolation_at_one() {
        let c1 = Color::from_rgb(255, 0, 0);
        let c2 = Color::from_rgb(0, 0, 255);
        let result = interpolate_color(&c1, &c2, 1.0);
        let triplet = result.get_truecolor(None, true);
        assert_eq!(triplet, ColorTriplet::new(0, 0, 255));
    }

    #[test]
    fn test_interpolation_at_midpoint() {
        let c1 = Color::from_rgb(0, 0, 0);
        let c2 = Color::from_rgb(254, 100, 200);
        let result = interpolate_color(&c1, &c2, 0.5);
        let triplet = result.get_truecolor(None, true);
        assert_eq!(triplet, ColorTriplet::new(127, 50, 100));
    }

    #[test]
    fn test_multi_stop_gradient_distributes_evenly() {
        // Red -> Green -> Blue, 5 characters
        let g = Gradient::new(
            "ABCDE",
            vec![
                Color::from_rgb(255, 0, 0),
                Color::from_rgb(0, 255, 0),
                Color::from_rgb(0, 0, 255),
            ],
        );
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .build();
        let options = console.options();
        let segments = g.rich_console(&console, &options);

        // First character should be red
        let first_style = segments[0].style.as_ref().unwrap();
        let first_fg = first_style.color().unwrap().get_truecolor(None, true);
        assert_eq!(first_fg, ColorTriplet::new(255, 0, 0));

        // Middle character (index 2, t=0.5) should be green
        let mid_style = segments[2].style.as_ref().unwrap();
        let mid_fg = mid_style.color().unwrap().get_truecolor(None, true);
        assert_eq!(mid_fg, ColorTriplet::new(0, 255, 0));

        // Last character should be blue
        let last_style = segments[4].style.as_ref().unwrap();
        let last_fg = last_style.color().unwrap().get_truecolor(None, true);
        assert_eq!(last_fg, ColorTriplet::new(0, 0, 255));
    }

    #[test]
    fn test_empty_text_produces_empty_segments() {
        let g = Gradient::two_color(
            "",
            Color::from_rgb(255, 0, 0),
            Color::from_rgb(0, 0, 255),
        );
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .build();
        let options = console.options();
        let segments = g.rich_console(&console, &options);
        // Only the trailing newline
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "\n");
    }

    #[test]
    fn test_single_character_text() {
        let g = Gradient::two_color(
            "X",
            Color::from_rgb(100, 200, 50),
            Color::from_rgb(0, 0, 255),
        );
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .build();
        let options = console.options();
        let segments = g.rich_console(&console, &options);
        // 1 char segment + 1 trailing newline
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "X");
        // Single char: uses the first color stop
        let fg = segments[0]
            .style
            .as_ref()
            .unwrap()
            .color()
            .unwrap()
            .get_truecolor(None, true);
        assert_eq!(fg, ColorTriplet::new(100, 200, 50));
    }

    #[test]
    fn test_gradient_with_newlines() {
        let g = Gradient::two_color(
            "AB\nCD",
            Color::from_rgb(255, 0, 0),
            Color::from_rgb(0, 0, 255),
        );
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .build();
        let options = console.options();
        let segments = g.rich_console(&console, &options);

        // Line 1: A, B | newline | Line 2: C, D | trailing newline
        // = 2 + 1 + 2 + 1 = 6
        assert_eq!(segments.len(), 6);
        assert_eq!(segments[0].text, "A");
        assert_eq!(segments[1].text, "B");
        assert_eq!(segments[2].text, "\n");
        assert_eq!(segments[3].text, "C");
        assert_eq!(segments[4].text, "D");
        assert_eq!(segments[5].text, "\n");

        // Each line has its own gradient: A is red(ish), B is blue(ish)
        let a_fg = segments[0]
            .style
            .as_ref()
            .unwrap()
            .color()
            .unwrap()
            .get_truecolor(None, true);
        let b_fg = segments[1]
            .style
            .as_ref()
            .unwrap()
            .color()
            .unwrap()
            .get_truecolor(None, true);
        assert_eq!(a_fg, ColorTriplet::new(255, 0, 0));
        assert_eq!(b_fg, ColorTriplet::new(0, 0, 255));
    }

    #[test]
    fn test_display_trait_works() {
        let g = Gradient::two_color(
            "Hi",
            Color::from_rgb(255, 0, 0),
            Color::from_rgb(0, 0, 255),
        );
        // Display with no_color=true via the fmt implementation
        let output = format!("{}", g);
        assert_eq!(output, "Hi");
    }

    #[test]
    fn test_builder_methods() {
        let g = Gradient::rainbow("test")
            .with_style(
                Style::new(
                    None,
                    None,
                    Some(true),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap(),
            )
            .with_justify(JustifyMethod::Center);

        assert_eq!(g.text, "test");
        assert_eq!(g.colors.len(), 7);
        assert_eq!(g.justify, Some(JustifyMethod::Center));
        assert_eq!(g.style.bold(), Some(true));
    }

    #[test]
    fn test_no_colors_uses_default() {
        let g = Gradient::new("Hi", vec![]);
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .build();
        let options = console.options();
        let segments = g.rich_console(&console, &options);
        // 2 chars + trailing newline
        assert_eq!(segments.len(), 3);
    }

    #[test]
    fn test_single_color_stop() {
        let g = Gradient::new("ABC", vec![Color::from_rgb(0, 128, 255)]);
        let console = Console::builder()
            .width(80)
            .force_terminal(true)
            .build();
        let options = console.options();
        let segments = g.rich_console(&console, &options);
        // All chars get the same color
        for seg in &segments[..3] {
            let fg = seg
                .style
                .as_ref()
                .unwrap()
                .color()
                .unwrap()
                .get_truecolor(None, true);
            assert_eq!(fg, ColorTriplet::new(0, 128, 255));
        }
    }

    #[test]
    fn test_interpolation_clamped() {
        let c1 = Color::from_rgb(255, 0, 0);
        let c2 = Color::from_rgb(0, 0, 255);
        // Out-of-range t values should be clamped
        let below = interpolate_color(&c1, &c2, -0.5);
        let above = interpolate_color(&c1, &c2, 1.5);
        assert_eq!(below.get_truecolor(None, true), ColorTriplet::new(255, 0, 0));
        assert_eq!(above.get_truecolor(None, true), ColorTriplet::new(0, 0, 255));
    }
}
