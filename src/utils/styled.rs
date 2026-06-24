//! Apply a style to a renderable.
//!
//! renderable together with an additional `Style` that is applied
//! on top of whatever styles the renderable already carries.

use std::fmt;

use crate::console::{Console, ConsoleOptions, Renderable, RenderableArc};
use crate::measure::Measurement;
use crate::segment::Segment;
use crate::style::Style;

/// A renderable with an additional style layered on top.
///
/// When rendered, the extra style is combined with every segment produced
/// by the inner renderable, exactly mirroring the `Styled` class.
#[derive(Clone)]
pub struct Styled {
    /// The inner renderable content (any renderable widget).
    pub renderable: RenderableArc,
    /// The style to apply on top of the renderable's own styles.
    pub style: Style,
}

// Manual Debug — RenderableArc (Arc<dyn Renderable + Send + Sync>) doesn't
// implement Debug, so we print a placeholder for the renderable field.
impl std::fmt::Debug for Styled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Styled")
            .field("renderable", &"<renderable>")
            .field("style", &self.style)
            .finish()
    }
}

impl Styled {
    /// Create a new `Styled` wrapping `renderable` with an additional `style`.
    pub fn new(renderable: impl Renderable + Send + Sync + 'static, style: Style) -> Self {
        Styled {
            renderable: std::sync::Arc::new(renderable),
            style,
        }
    }

    /// Return the measurement of the inner renderable (unchanged by the style overlay).
    pub fn measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.renderable.gilt_measure(console, options)
    }
}

impl Renderable for Styled {
    fn gilt_measure(&self, console: &Console, options: &ConsoleOptions) -> Measurement {
        self.measure(console, options)
    }

    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let rendered_segments = self.renderable.as_ref().gilt_console(console, options);
        Segment::apply_style(&rendered_segments, Some(self.style.clone()), None)
    }
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl fmt::Display for Styled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = f.width().unwrap_or(80);
        let mut console = Console::builder()
            .width(w)
            .force_terminal(true)
            .no_color(true)
            .build();
        console.begin_capture();
        console.print(self);
        let output = console.end_capture();
        write!(f, "{}", output.trim_end_matches('\n'))
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
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        let text = Text::new("Hello", Style::null());
        let style = Style::parse("bold");
        let styled = Styled::new(text.clone(), style.clone());
        // Cannot call .plain() on RenderableArc — verify style instead
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
        let style = Style::parse("bold italic red on blue");
        let styled = Styled::new(text, style.clone());
        assert_eq!(styled.style, style);
    }

    // -- Rendering applies style to all segments ----------------------------

    #[test]
    fn test_render_applies_style() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();

        let text = Text::new("Hello", Style::null());
        let style = Style::parse("bold");
        let styled = Styled::new(text, style);

        let segments = styled.gilt_console(&console, &opts);
        // Every non-control segment should have bold
        for seg in &segments {
            if !seg.is_control() && !seg.text.is_empty() && seg.text.trim() == seg.text {
                // The text content segment(s) should carry bold
                assert!(
                    seg.style().is_some_and(|s| s.bold() == Some(true)),
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
        let text = Text::styled("Hello World", "italic");
        let overlay = Style::parse("bold");
        let styled = Styled::new(text, overlay);

        let segments = styled.gilt_console(&console, &opts);
        // All non-empty text segments should have both bold and italic
        for seg in &segments {
            if !seg.is_control() && !seg.text.is_empty() && seg.text != "\n" {
                let s = seg.style().expect("segment should have a style");
                assert_eq!(
                    s.bold(),
                    Some(true),
                    "segment {:?} should be bold",
                    seg.text
                );
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
        text.stylize(Style::parse("italic"), 0, Some(2));
        let styled = Styled::new(text, Style::parse("bold"));

        let segments = styled.gilt_console(&console, &opts);
        // Find the segment(s) containing "AB"
        let ab_segments: Vec<&Segment> = segments
            .iter()
            .filter(|s| !s.is_control() && s.text.contains('A'))
            .collect();
        assert!(!ab_segments.is_empty());
        for seg in ab_segments {
            let s = seg.style().unwrap();
            assert_eq!(s.bold(), Some(true));
            assert_eq!(s.italic(), Some(true));
        }
    }

    #[test]
    fn test_style_overlay_color() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();

        let text = Text::new("color test", Style::parse("red"));
        let styled = Styled::new(text, Style::parse("bold"));

        let segments = styled.gilt_console(&console, &opts);
        for seg in &segments {
            if !seg.is_control() && !seg.text.is_empty() && seg.text != "\n" {
                let s = seg.style().unwrap();
                assert_eq!(s.bold(), Some(true));
                // Red should still be present since bold doesn't override color
                assert!(s.color().is_some());
            }
        }
    }

    // -- Measure returns renderable's measurement unchanged -----------------

    #[test]
    fn test_measure_unchanged() {
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        // Styled::measure now takes &Console, &ConsoleOptions
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();
        let text = Text::new("Hello, World!", Style::null());
        let expected = text.gilt_measure(&console, &opts);
        let styled = Styled::new(text, Style::parse("bold italic underline"));
        assert_eq!(styled.measure(&console, &opts), expected);
    }

    #[test]
    fn test_measure_multiline() {
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        // Styled::measure now takes &Console, &ConsoleOptions
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();
        let text = Text::new("short\na somewhat longer line", Style::null());
        let expected = text.gilt_measure(&console, &opts);
        let styled = Styled::new(text, Style::parse("red on blue"));
        assert_eq!(styled.measure(&console, &opts), expected);
    }

    #[test]
    fn test_measure_empty() {
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        // Styled::measure now takes &Console, &ConsoleOptions
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();
        let text = Text::new("", Style::null());
        let styled = Styled::new(text, Style::parse("bold"));
        assert_eq!(styled.measure(&console, &opts), Measurement::new(0, 0));
    }

    // -- Null style overlay is transparent ----------------------------------

    #[test]
    fn test_null_style_passthrough() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();

        let text = Text::new("pass through", Style::null());
        let styled = Styled::new(text.clone(), Style::null());

        let direct_segments = text.gilt_console(&console, &opts);
        let styled_segments = styled.gilt_console(&console, &opts);

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
        let styled = Styled::new(text, Style::parse("bold"));
        let segments = console.render(&styled, None);
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("via console"));
    }

    // -- Clone --------------------------------------------------------------

    #[test]
    fn test_clone() {
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        let styled = Styled::new(Text::new("clone me", Style::null()), Style::parse("italic"));
        let cloned = styled.clone();
        // Cannot call .plain() on RenderableArc — verify style is cloned correctly
        assert_eq!(cloned.style, styled.style);
    }

    // -- gilt_measure override -----------------------------------------------

    #[test]
    fn styled_gilt_measure_matches_standalone() {
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        // Styled::measure now takes &Console, &ConsoleOptions
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();
        let text = Text::new("Hello, World!", Style::null());
        let styled = Styled::new(text, Style::parse("bold"));
        let m_standalone = styled.measure(&console, &opts);
        let m_trait = styled.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Styled::gilt_measure must delegate to Styled::measure"
        );
    }

    #[test]
    fn styled_gilt_measure_multiline_matches_standalone() {
        // Updated: field is now RenderableArc (Text still works via Renderable impl)
        // Styled::measure now takes &Console, &ConsoleOptions
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();
        let text = Text::new("short\na somewhat longer line", Style::null());
        let styled = Styled::new(text, Style::parse("red on blue"));
        let m_standalone = styled.measure(&console, &opts);
        let m_trait = styled.gilt_measure(&console, &opts);
        assert_eq!(
            m_trait, m_standalone,
            "Styled::gilt_measure multiline must delegate to Styled::measure"
        );
    }

    // -- Task 4.5: RenderableArc constructor tests ---------------------------

    #[test]
    // Updated: constructors now accept impl Renderable + Send + Sync + 'static (Text still works via Renderable impl)
    fn styled_new_text_still_compiles() {
        let _ = Styled::new(Text::new("x", Style::null()), Style::null());
    }

    #[test]
    // Updated: constructors now accept impl Renderable + Send + Sync + 'static (Panel works too)
    fn styled_new_panel_compiles() {
        let p = crate::panel::Panel::new(Text::new("x", Style::null()));
        let _ = Styled::new(p, Style::null());
    }
}
