//! Apply a style to a renderable.
//!
//! Rust port of Python's `rich/styled.py`. The `Styled` struct wraps a
//! renderable (`Text`) together with an additional `Style` that is applied
//! on top of whatever styles the renderable already carries.

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

/// A renderable with an additional style layered on top.
///
/// When rendered, the extra style is combined with every segment produced
/// by the inner renderable, exactly mirroring Python rich's `Styled` class.
#[derive(Debug, Clone)]
pub struct Styled {
    /// The inner renderable content.
    pub renderable: Text,
    /// The style to apply on top of the renderable's own styles.
    pub style: Style,
}

impl Styled {
    /// Create a new `Styled` wrapping `renderable` with an additional `style`.
    pub fn new(renderable: Text, style: Style) -> Self {
        Styled { renderable, style }
    }

    /// Return the measurement of the inner renderable (unchanged by the style overlay).
    pub fn measure(&self) -> Measurement {
        self.renderable.measure()
    }
}

impl Renderable for Styled {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let rendered_segments = self.renderable.rich_console(console, options);
        Segment::apply_style(&rendered_segments, Some(self.style.clone()), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;
    use crate::measure::Measurement;
    use crate::segment::Segment;
    use crate::style::Style;
    use crate::text::Text;

    // -- Construction -------------------------------------------------------

    #[test]
    fn test_new_basic() {
        let text = Text::new("Hello", Style::null());
        let style = Style::parse("bold").unwrap();
        let styled = Styled::new(text.clone(), style.clone());
        assert_eq!(styled.renderable.plain(), "Hello");
        assert_eq!(styled.style, style);
    }

    #[test]
    fn test_new_null_style() {
        let text = Text::new("content", Style::null());
        let styled = Styled::new(text, Style::null());
        assert!(styled.style.is_null());
    }

    #[test]
    fn test_new_complex_style() {
        let text = Text::new("fancy", Style::null());
        let style = Style::parse("bold italic red on blue").unwrap();
        let styled = Styled::new(text, style.clone());
        assert_eq!(styled.style, style);
    }

    // -- Rendering applies style to all segments ----------------------------

    #[test]
    fn test_render_applies_style() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();

        let text = Text::new("Hello", Style::null());
        let style = Style::parse("bold").unwrap();
        let styled = Styled::new(text, style);

        let segments = styled.rich_console(&console, &opts);
        // Every non-control segment should have bold
        for seg in &segments {
            if !seg.is_control() && !seg.text.is_empty() && seg.text.trim() == seg.text {
                // The text content segment(s) should carry bold
                assert!(
                    seg.style.as_ref().map_or(false, |s| s.bold() == Some(true)),
                    "segment {:?} should be bold",
                    seg.text,
                );
            }
        }
    }

    #[test]
    fn test_render_applies_style_to_all_segments() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();

        // Create text with an existing span style
        let text = Text::styled("Hello World", Style::parse("italic").unwrap());
        let overlay = Style::parse("bold").unwrap();
        let styled = Styled::new(text, overlay);

        let segments = styled.rich_console(&console, &opts);
        // All non-empty text segments should have both bold and italic
        for seg in &segments {
            if !seg.is_control() && !seg.text.is_empty() && seg.text != "\n" {
                let s = seg.style.as_ref().expect("segment should have a style");
                assert_eq!(s.bold(), Some(true), "segment {:?} should be bold", seg.text);
                assert_eq!(
                    s.italic(),
                    Some(true),
                    "segment {:?} should be italic",
                    seg.text,
                );
            }
        }
    }

    // -- Style combines with existing segment styles ------------------------

    #[test]
    fn test_style_combines_with_existing() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();

        let mut text = Text::new("AB", Style::null());
        text.stylize(Style::parse("italic").unwrap(), 0, Some(2));
        let styled = Styled::new(text, Style::parse("bold").unwrap());

        let segments = styled.rich_console(&console, &opts);
        // Find the segment(s) containing "AB"
        let ab_segments: Vec<&Segment> = segments
            .iter()
            .filter(|s| !s.is_control() && s.text.contains('A'))
            .collect();
        assert!(!ab_segments.is_empty());
        for seg in ab_segments {
            let s = seg.style.as_ref().unwrap();
            assert_eq!(s.bold(), Some(true));
            assert_eq!(s.italic(), Some(true));
        }
    }

    #[test]
    fn test_style_overlay_color() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();

        let text = Text::new("color test", Style::parse("red").unwrap());
        let styled = Styled::new(text, Style::parse("bold").unwrap());

        let segments = styled.rich_console(&console, &opts);
        for seg in &segments {
            if !seg.is_control() && !seg.text.is_empty() && seg.text != "\n" {
                let s = seg.style.as_ref().unwrap();
                assert_eq!(s.bold(), Some(true));
                // Red should still be present since bold doesn't override color
                assert!(s.color().is_some());
            }
        }
    }

    // -- Measure returns renderable's measurement unchanged -----------------

    #[test]
    fn test_measure_unchanged() {
        let text = Text::new("Hello, World!", Style::null());
        let expected = text.measure();
        let styled = Styled::new(text, Style::parse("bold italic underline").unwrap());
        assert_eq!(styled.measure(), expected);
    }

    #[test]
    fn test_measure_multiline() {
        let text = Text::new("short\na somewhat longer line", Style::null());
        let expected = text.measure();
        let styled = Styled::new(text, Style::parse("red on blue").unwrap());
        assert_eq!(styled.measure(), expected);
    }

    #[test]
    fn test_measure_empty() {
        let text = Text::new("", Style::null());
        let styled = Styled::new(text, Style::parse("bold").unwrap());
        assert_eq!(styled.measure(), Measurement::new(0, 0));
    }

    // -- Null style overlay is transparent ----------------------------------

    #[test]
    fn test_null_style_passthrough() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();

        let text = Text::new("pass through", Style::null());
        let styled = Styled::new(text.clone(), Style::null());

        let direct_segments = text.rich_console(&console, &opts);
        let styled_segments = styled.rich_console(&console, &opts);

        // With a null overlay, apply_style should produce equivalent segments
        assert_eq!(direct_segments.len(), styled_segments.len());
        for (d, s) in direct_segments.iter().zip(styled_segments.iter()) {
            assert_eq!(d.text, s.text);
        }
    }

    // -- Integration: render through Console --------------------------------

    #[test]
    fn test_console_render() {
        let console = Console::builder().width(80).markup(false).build();
        let text = Text::new("via console", Style::null());
        let styled = Styled::new(text, Style::parse("bold").unwrap());
        let segments = console.render(&styled, None);
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("via console"));
    }

    // -- Clone --------------------------------------------------------------

    #[test]
    fn test_clone() {
        let styled = Styled::new(
            Text::new("clone me", Style::null()),
            Style::parse("italic").unwrap(),
        );
        let cloned = styled.clone();
        assert_eq!(cloned.renderable.plain(), "clone me");
        assert_eq!(cloned.style, styled.style);
    }
}
