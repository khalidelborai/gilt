//! Live render module -- a renderable that can be updated and tracks its dimensions.
//!
//! Port of Python's `rich/live_render.py`. Used by `Live` to display content
//! that can be refreshed in-place by emitting cursor movement control codes.

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::{ControlCode, ControlType, Segment};
use crate::style::Style;
use crate::text::{JustifyMethod, OverflowMethod, Text};

/// How to handle content that exceeds the available vertical space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalOverflowMethod {
    /// Crop excess lines (discard lines beyond the height).
    Crop,
    /// Show an ellipsis ("...") line in place of the last visible line.
    Ellipsis,
    /// Show all lines regardless of the height constraint.
    Visible,
}

/// A renderable wrapper that tracks the dimensions of its last render,
/// enabling cursor-based in-place updates for live terminal displays.
pub struct LiveRender {
    /// The content to render.
    pub renderable: Text,
    /// An optional style overlay applied when rendering.
    pub style: Style,
    /// How to handle vertical overflow.
    pub vertical_overflow: VerticalOverflowMethod,
    /// The (width, height) of the last render, or `None` if never rendered.
    shape: Option<(usize, usize)>,
}

impl LiveRender {
    /// Create a new `LiveRender` with the given renderable content.
    ///
    /// Defaults to a null style and `VerticalOverflowMethod::Ellipsis`.
    pub fn new(renderable: Text) -> Self {
        LiveRender {
            renderable,
            style: Style::null(),
            vertical_overflow: VerticalOverflowMethod::Ellipsis,
            shape: None,
        }
    }

    /// Set the style overlay (builder pattern).
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the vertical overflow method (builder pattern).
    #[must_use]
    pub fn with_vertical_overflow(mut self, overflow: VerticalOverflowMethod) -> Self {
        self.vertical_overflow = overflow;
        self
    }

    /// Return the height (in lines) of the last render.
    ///
    /// Returns `0` if nothing has been rendered yet.
    pub fn last_render_height(&self) -> usize {
        match self.shape {
            Some((_, height)) => height,
            None => 0,
        }
    }

    /// Replace the renderable content.
    pub fn set_renderable(&mut self, renderable: Text) {
        self.renderable = renderable;
    }

    /// Return control segments that move the cursor back to the start of the
    /// last render output so that it can be overwritten.
    ///
    /// Produces: CR, ERASE_IN_LINE(2), then (height-1) repetitions of
    /// CURSOR_UP(1) + ERASE_IN_LINE(2).
    pub fn position_cursor(&self) -> Vec<Segment> {
        let Some((_, height)) = self.shape else {
            return Vec::new();
        };
        if height == 0 {
            return Vec::new();
        }

        let mut codes: Vec<ControlCode> = Vec::new();
        codes.push(ControlCode::Simple(ControlType::CarriageReturn));
        codes.push(ControlCode::WithParam(ControlType::EraseInLine, 2));
        for _ in 0..height.saturating_sub(1) {
            codes.push(ControlCode::WithParam(ControlType::CursorUp, 1));
            codes.push(ControlCode::WithParam(ControlType::EraseInLine, 2));
        }

        vec![Segment::new("", None, Some(codes))]
    }

    /// Return control segments that erase the last render output and move the
    /// cursor back to its position before the render.
    ///
    /// Produces: CR, then `height` repetitions of CURSOR_UP(1) + ERASE_IN_LINE(2).
    pub fn restore_cursor(&self) -> Vec<Segment> {
        let Some((_, height)) = self.shape else {
            return Vec::new();
        };
        if height == 0 {
            return Vec::new();
        }

        let mut codes: Vec<ControlCode> = Vec::new();
        codes.push(ControlCode::Simple(ControlType::CarriageReturn));
        for _ in 0..height {
            codes.push(ControlCode::WithParam(ControlType::CursorUp, 1));
            codes.push(ControlCode::WithParam(ControlType::EraseInLine, 2));
        }

        vec![Segment::new("", None, Some(codes))]
    }
}

impl Renderable for LiveRender {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        // Render the inner content into lines.
        let style_ref = if self.style.is_null() {
            None
        } else {
            Some(&self.style)
        };
        let mut lines = console.render_lines(&self.renderable, Some(options), style_ref, false, false);

        // Check the shape and apply vertical overflow if needed.
        let (_, height) = Segment::get_shape(&lines);
        let max_height = options.height.unwrap_or(options.size.height);

        if height > max_height {
            match self.vertical_overflow {
                VerticalOverflowMethod::Crop => {
                    lines.truncate(max_height);
                }
                VerticalOverflowMethod::Ellipsis => {
                    let ellipsis_lines = if max_height > 0 { max_height - 1 } else { 0 };
                    lines.truncate(ellipsis_lines);
                    // Build an ellipsis text line.
                    let mut overflow_text = Text::new("...", Style::null());
                    overflow_text.overflow = Some(OverflowMethod::Crop);
                    overflow_text.justify = Some(JustifyMethod::Center);
                    overflow_text.end = String::new();
                    let ellipsis_segments = console.render(&overflow_text, Some(options));
                    lines.push(ellipsis_segments);
                }
                VerticalOverflowMethod::Visible => {
                    // Keep all lines; do not truncate.
                }
            }
        }

        // Compute and store the final shape.
        let final_shape = Segment::get_shape(&lines);

        // SAFETY: We need interior mutability to store shape while implementing
        // the trait method which takes &self. We use a raw pointer cast here
        // because LiveRender is not used concurrently and shape is purely a
        // cache. An alternative would be Cell/RefCell, but we keep the struct
        // simple.
        #[allow(invalid_reference_casting)]
        {
            let self_mut = unsafe { &mut *(self as *const LiveRender as *mut LiveRender) };
            self_mut.shape = Some(final_shape);
        }

        // Flatten lines into a single segment list, inserting newlines between
        // lines (but not after the last line).
        let mut segments = Vec::new();
        let line_count = lines.len();
        for (i, line) in lines.into_iter().enumerate() {
            segments.extend(line);
            if i + 1 < line_count {
                segments.push(Segment::line());
            }
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Construction -------------------------------------------------------

    #[test]
    fn test_default_construction() {
        let lr = LiveRender::new(Text::new("hello", Style::null()));
        assert_eq!(lr.renderable.plain(), "hello");
        assert!(lr.style.is_null());
        assert_eq!(lr.vertical_overflow, VerticalOverflowMethod::Ellipsis);
        assert!(lr.shape.is_none());
    }

    // -- Builder methods ----------------------------------------------------

    #[test]
    fn test_with_style() {
        let style = Style::parse("bold").unwrap();
        let lr = LiveRender::new(Text::new("x", Style::null())).with_style(style.clone());
        assert_eq!(lr.style, style);
    }

    #[test]
    fn test_with_vertical_overflow() {
        let lr = LiveRender::new(Text::new("x", Style::null()))
            .with_vertical_overflow(VerticalOverflowMethod::Crop);
        assert_eq!(lr.vertical_overflow, VerticalOverflowMethod::Crop);
    }

    // -- last_render_height -------------------------------------------------

    #[test]
    fn test_last_render_height_before_render() {
        let lr = LiveRender::new(Text::new("hello", Style::null()));
        assert_eq!(lr.last_render_height(), 0);
    }

    #[test]
    fn test_last_render_height_after_render() {
        let console = Console::builder().width(80).build();
        let lr = LiveRender::new(Text::new("line1\nline2\nline3", Style::null()));
        let opts = console.options();
        let _ = lr.rich_console(&console, &opts);
        assert_eq!(lr.last_render_height(), 3);
    }

    // -- set_renderable -----------------------------------------------------

    #[test]
    fn test_set_renderable() {
        let mut lr = LiveRender::new(Text::new("old", Style::null()));
        lr.set_renderable(Text::new("new", Style::null()));
        assert_eq!(lr.renderable.plain(), "new");
    }

    // -- Renderable trait ---------------------------------------------------

    #[test]
    fn test_renderable_basic() {
        let console = Console::builder().width(80).markup(false).build();
        let lr = LiveRender::new(Text::new("Hello, World!", Style::null()));
        let opts = console.options();
        let segments = lr.rich_console(&console, &opts);
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("Hello, World!"));
    }

    #[test]
    fn test_renderable_multiline() {
        let console = Console::builder().width(80).markup(false).build();
        let lr = LiveRender::new(Text::new("Line1\nLine2", Style::null()));
        let opts = console.options();
        let segments = lr.rich_console(&console, &opts);

        // There should be a newline segment between lines.
        let has_newline = segments.iter().any(|s| s.text == "\n");
        assert!(has_newline);

        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("Line1"));
        assert!(combined.contains("Line2"));
    }

    // -- Vertical overflow: crop --------------------------------------------

    #[test]
    fn test_vertical_overflow_crop() {
        let console = Console::builder().width(80).height(3).build();
        // Create content with 5 lines.
        let lr = LiveRender::new(Text::new("L1\nL2\nL3\nL4\nL5", Style::null()))
            .with_vertical_overflow(VerticalOverflowMethod::Crop);
        let opts = console.options();
        let segments = lr.rich_console(&console, &opts);

        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("L1"));
        assert!(combined.contains("L2"));
        assert!(combined.contains("L3"));
        assert!(!combined.contains("L4"));
        assert!(!combined.contains("L5"));

        assert_eq!(lr.last_render_height(), 3);
    }

    // -- Vertical overflow: ellipsis ----------------------------------------

    #[test]
    fn test_vertical_overflow_ellipsis() {
        let console = Console::builder().width(80).height(3).build();
        let lr = LiveRender::new(Text::new("L1\nL2\nL3\nL4\nL5", Style::null()))
            .with_vertical_overflow(VerticalOverflowMethod::Ellipsis);
        let opts = console.options();
        let segments = lr.rich_console(&console, &opts);

        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("L1"));
        assert!(combined.contains("L2"));
        assert!(combined.contains("..."));
        assert!(!combined.contains("L3\n"));

        assert_eq!(lr.last_render_height(), 3);
    }

    // -- Vertical overflow: visible -----------------------------------------

    #[test]
    fn test_vertical_overflow_visible() {
        let console = Console::builder().width(80).height(3).build();
        let lr = LiveRender::new(Text::new("L1\nL2\nL3\nL4\nL5", Style::null()))
            .with_vertical_overflow(VerticalOverflowMethod::Visible);
        let opts = console.options();
        let segments = lr.rich_console(&console, &opts);

        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("L1"));
        assert!(combined.contains("L5"));

        assert_eq!(lr.last_render_height(), 5);
    }

    // -- position_cursor ----------------------------------------------------

    #[test]
    fn test_position_cursor_no_render() {
        let lr = LiveRender::new(Text::new("hello", Style::null()));
        let segments = lr.position_cursor();
        assert!(segments.is_empty());
    }

    #[test]
    fn test_position_cursor_after_render() {
        let console = Console::builder().width(80).build();
        let lr = LiveRender::new(Text::new("L1\nL2\nL3", Style::null()));
        let opts = console.options();
        let _ = lr.rich_console(&console, &opts);

        let segments = lr.position_cursor();
        assert_eq!(segments.len(), 1);
        let ctrl = segments[0].control.as_ref().unwrap();

        // First code: CarriageReturn
        assert_eq!(ctrl[0], ControlCode::Simple(ControlType::CarriageReturn));
        // Second code: EraseInLine(2)
        assert_eq!(
            ctrl[1],
            ControlCode::WithParam(ControlType::EraseInLine, 2)
        );

        // Then (height-1) = 2 pairs of CursorUp(1), EraseInLine(2)
        // Total codes: 1 + 1 + 2*2 = 6
        assert_eq!(ctrl.len(), 6);
        assert_eq!(
            ctrl[2],
            ControlCode::WithParam(ControlType::CursorUp, 1)
        );
        assert_eq!(
            ctrl[3],
            ControlCode::WithParam(ControlType::EraseInLine, 2)
        );
        assert_eq!(
            ctrl[4],
            ControlCode::WithParam(ControlType::CursorUp, 1)
        );
        assert_eq!(
            ctrl[5],
            ControlCode::WithParam(ControlType::EraseInLine, 2)
        );
    }

    // -- restore_cursor -----------------------------------------------------

    #[test]
    fn test_restore_cursor_no_render() {
        let lr = LiveRender::new(Text::new("hello", Style::null()));
        let segments = lr.restore_cursor();
        assert!(segments.is_empty());
    }

    #[test]
    fn test_restore_cursor_after_render() {
        let console = Console::builder().width(80).build();
        let lr = LiveRender::new(Text::new("L1\nL2\nL3", Style::null()));
        let opts = console.options();
        let _ = lr.rich_console(&console, &opts);

        let segments = lr.restore_cursor();
        assert_eq!(segments.len(), 1);
        let ctrl = segments[0].control.as_ref().unwrap();

        // First code: CarriageReturn
        assert_eq!(ctrl[0], ControlCode::Simple(ControlType::CarriageReturn));

        // Then height = 3 pairs of CursorUp(1), EraseInLine(2)
        // Total codes: 1 + 3*2 = 7
        assert_eq!(ctrl.len(), 7);
        for i in 0..3 {
            assert_eq!(
                ctrl[1 + i * 2],
                ControlCode::WithParam(ControlType::CursorUp, 1)
            );
            assert_eq!(
                ctrl[2 + i * 2],
                ControlCode::WithParam(ControlType::EraseInLine, 2)
            );
        }
    }

    // -- Shape tracking after render ----------------------------------------

    #[test]
    fn test_shape_tracking() {
        let console = Console::builder().width(40).build();
        let lr = LiveRender::new(Text::new("Hello", Style::null()));
        let opts = console.options();

        assert!(lr.shape.is_none());
        let _ = lr.rich_console(&console, &opts);
        assert!(lr.shape.is_some());
        let (w, h) = lr.shape.unwrap();
        assert!(w > 0);
        assert_eq!(h, 1);
    }

    // -- Single-line position_cursor ----------------------------------------

    #[test]
    fn test_position_cursor_single_line() {
        let console = Console::builder().width(80).build();
        let lr = LiveRender::new(Text::new("Hello", Style::null()));
        let opts = console.options();
        let _ = lr.rich_console(&console, &opts);

        let segments = lr.position_cursor();
        assert_eq!(segments.len(), 1);
        let ctrl = segments[0].control.as_ref().unwrap();
        // height=1, so: CR, EraseInLine(2), no CursorUp pairs
        assert_eq!(ctrl.len(), 2);
        assert_eq!(ctrl[0], ControlCode::Simple(ControlType::CarriageReturn));
        assert_eq!(
            ctrl[1],
            ControlCode::WithParam(ControlType::EraseInLine, 2)
        );
    }

    // -- Single-line restore_cursor -----------------------------------------

    #[test]
    fn test_restore_cursor_single_line() {
        let console = Console::builder().width(80).build();
        let lr = LiveRender::new(Text::new("Hello", Style::null()));
        let opts = console.options();
        let _ = lr.rich_console(&console, &opts);

        let segments = lr.restore_cursor();
        assert_eq!(segments.len(), 1);
        let ctrl = segments[0].control.as_ref().unwrap();
        // height=1: CR, CursorUp(1), EraseInLine(2)
        assert_eq!(ctrl.len(), 3);
        assert_eq!(ctrl[0], ControlCode::Simple(ControlType::CarriageReturn));
        assert_eq!(
            ctrl[1],
            ControlCode::WithParam(ControlType::CursorUp, 1)
        );
        assert_eq!(
            ctrl[2],
            ControlCode::WithParam(ControlType::EraseInLine, 2)
        );
    }

    // -- Vertical overflow enum variants ------------------------------------

    #[test]
    fn test_vertical_overflow_method_variants() {
        assert_ne!(VerticalOverflowMethod::Crop, VerticalOverflowMethod::Ellipsis);
        assert_ne!(VerticalOverflowMethod::Ellipsis, VerticalOverflowMethod::Visible);
        assert_ne!(VerticalOverflowMethod::Crop, VerticalOverflowMethod::Visible);
    }

    // -- Render with style --------------------------------------------------

    #[test]
    fn test_render_with_style() {
        let console = Console::builder().width(80).markup(false).build();
        let style = Style::parse("bold").unwrap();
        let lr = LiveRender::new(Text::new("styled", Style::null())).with_style(style);
        let opts = console.options();
        let segments = lr.rich_console(&console, &opts);
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("styled"));
    }

    // -- Content fits within height (no overflow) ---------------------------

    #[test]
    fn test_no_overflow_when_fits() {
        let console = Console::builder().width(80).height(10).build();
        let lr = LiveRender::new(Text::new("L1\nL2\nL3", Style::null()))
            .with_vertical_overflow(VerticalOverflowMethod::Ellipsis);
        let opts = console.options();
        let segments = lr.rich_console(&console, &opts);

        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("L1"));
        assert!(combined.contains("L2"));
        assert!(combined.contains("L3"));
        assert!(!combined.contains("..."));
    }
}
