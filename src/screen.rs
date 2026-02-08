//! Screen module -- a renderable that fills the terminal screen and crops excess.
//!
//! Port of Python's `rich/screen.py`.

use crate::console::{Console, ConsoleOptions, Renderable};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::Text;

/// Iterate over a slice, yielding `(is_last, &item)` pairs.
fn loop_last<T>(items: &[T]) -> impl Iterator<Item = (bool, &T)> {
    let len = items.len();
    items
        .iter()
        .enumerate()
        .map(move |(i, item)| (i + 1 == len, item))
}

/// A renderable that fills the terminal screen and crops excess.
///
/// Screen renders its content into exactly `width x height` cells,
/// padding short lines and truncating long ones.  In application mode
/// the line separator is `\n\r` instead of `\n`.
#[derive(Debug, Clone)]
pub struct Screen {
    /// The content to render.
    pub renderable: Text,
    /// Optional background / fill style.
    pub style: Option<Style>,
    /// When `true`, use `\n\r` between lines instead of `\n`.
    pub application_mode: bool,
}

impl Screen {
    /// Create a new `Screen` with the given renderable.
    pub fn new(renderable: Text) -> Self {
        Screen {
            renderable,
            style: None,
            application_mode: false,
        }
    }

    /// Builder: set the background style.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Builder: enable or disable application mode.
    pub fn with_application_mode(mut self, mode: bool) -> Self {
        self.application_mode = mode;
        self
    }
}

impl Renderable for Screen {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let width = options.size.width;
        let height = options.size.height;

        // Build render options constrained to the screen dimensions.
        let render_options = options.update_dimensions(width, height);

        // Render the content into lines.
        let lines = console.render_lines(
            &self.renderable,
            Some(&render_options),
            self.style.as_ref(),
            true,  // pad
            false, // no trailing newlines from render_lines
        );

        // Crop / pad to exact width x height.
        let lines = Segment::set_shape(
            &lines,
            width,
            Some(height),
            self.style.as_ref(),
            false,
        );

        // Choose the inter-line separator.
        let new_line = if self.application_mode {
            Segment::text("\n\r")
        } else {
            Segment::line()
        };

        // Flatten the lines into a single segment stream.
        let mut result = Vec::new();
        for (is_last, line) in loop_last(&lines) {
            result.extend(line.iter().cloned());
            if !is_last {
                result.push(new_line.clone());
            }
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;
    use crate::segment::Segment;

    /// Helper: build a console with fixed dimensions and no colour.
    fn test_console(width: usize, height: usize) -> Console {
        Console::builder()
            .width(width)
            .height(height)
            .no_color(true)
            .markup(false)
            .force_terminal(true)
            .build()
    }

    // -- Construction -------------------------------------------------------

    #[test]
    fn test_default_construction() {
        let screen = Screen::new(Text::new("hello", Style::null()));
        assert!(screen.style.is_none());
        assert!(!screen.application_mode);
        assert_eq!(screen.renderable.plain(), "hello");
    }

    #[test]
    fn test_with_style() {
        let style = Style::parse("bold").unwrap();
        let screen = Screen::new(Text::new("x", Style::null())).with_style(style.clone());
        assert_eq!(screen.style, Some(style));
    }

    #[test]
    fn test_with_application_mode() {
        let screen = Screen::new(Text::new("x", Style::null())).with_application_mode(true);
        assert!(screen.application_mode);
    }

    // -- Rendering ----------------------------------------------------------

    #[test]
    fn test_render_exact_dimensions() {
        let width = 10;
        let height = 3;
        let console = test_console(width, height);
        let screen = Screen::new(Text::new("hi", Style::null()));
        let opts = console.options();
        let segments = screen.rich_console(&console, &opts);

        // Collect lines by splitting on newline segments.
        let lines = collect_lines(&segments, "\n");
        assert_eq!(lines.len(), height, "should produce exactly {height} lines");
        for (i, line) in lines.iter().enumerate() {
            let line_width: usize = line.iter().map(|s| s.cell_length()).sum();
            assert_eq!(
                line_width, width,
                "line {i} should be {width} cells wide, got {line_width}"
            );
        }
    }

    #[test]
    fn test_newlines_between_lines_not_after_last() {
        let width = 5;
        let height = 3;
        let console = test_console(width, height);
        let screen = Screen::new(Text::new("A", Style::null()));
        let opts = console.options();
        let segments = screen.rich_console(&console, &opts);

        // Count newline segments.
        let nl_count = segments.iter().filter(|s| s.text == "\n").count();
        assert_eq!(
            nl_count,
            height - 1,
            "should have height-1 newlines between lines"
        );

        // Last segment must not be a newline.
        let last = segments.last().unwrap();
        assert_ne!(last.text, "\n", "last segment should not be a newline");
    }

    #[test]
    fn test_application_mode_uses_cr() {
        let width = 5;
        let height = 2;
        let console = test_console(width, height);
        let screen = Screen::new(Text::new("X", Style::null())).with_application_mode(true);
        let opts = console.options();
        let segments = screen.rich_console(&console, &opts);

        // There should be exactly one separator and it should be "\n\r".
        let seps: Vec<&Segment> = segments.iter().filter(|s| s.text.contains('\n')).collect();
        assert_eq!(seps.len(), 1);
        assert_eq!(seps[0].text, "\n\r");
    }

    #[test]
    fn test_normal_mode_uses_lf() {
        let width = 5;
        let height = 2;
        let console = test_console(width, height);
        let screen = Screen::new(Text::new("X", Style::null())).with_application_mode(false);
        let opts = console.options();
        let segments = screen.rich_console(&console, &opts);

        let seps: Vec<&Segment> = segments.iter().filter(|s| s.text.contains('\n')).collect();
        assert_eq!(seps.len(), 1);
        assert_eq!(seps[0].text, "\n");
    }

    #[test]
    fn test_multiline_content_cropped_to_height() {
        let width = 10;
        let height = 2;
        let console = test_console(width, height);
        // Content has 5 lines but screen height is 2.
        let screen = Screen::new(Text::new("A\nB\nC\nD\nE", Style::null()));
        let opts = console.options();
        let segments = screen.rich_console(&console, &opts);

        let lines = collect_lines(&segments, "\n");
        assert_eq!(lines.len(), height);
    }

    #[test]
    fn test_content_shorter_than_height_is_padded() {
        let width = 6;
        let height = 5;
        let console = test_console(width, height);
        let screen = Screen::new(Text::new("Hi", Style::null()));
        let opts = console.options();
        let segments = screen.rich_console(&console, &opts);

        let lines = collect_lines(&segments, "\n");
        assert_eq!(lines.len(), height);
        // All lines should be exactly `width` cells wide.
        for (i, line) in lines.iter().enumerate() {
            let w: usize = line.iter().map(|s| s.cell_length()).sum();
            assert_eq!(w, width, "line {i} width mismatch");
        }
    }

    #[test]
    fn test_empty_content() {
        let width = 4;
        let height = 3;
        let console = test_console(width, height);
        let screen = Screen::new(Text::new("", Style::null()));
        let opts = console.options();
        let segments = screen.rich_console(&console, &opts);

        let lines = collect_lines(&segments, "\n");
        assert_eq!(lines.len(), height);
        for line in &lines {
            let w: usize = line.iter().map(|s| s.cell_length()).sum();
            assert_eq!(w, width);
        }
    }

    #[test]
    fn test_loop_last_helper() {
        let items = vec![1, 2, 3];
        let result: Vec<(bool, &i32)> = loop_last(&items).collect();
        assert_eq!(result, vec![(false, &1), (false, &2), (true, &3)]);
    }

    #[test]
    fn test_loop_last_single() {
        let items = vec![42];
        let result: Vec<(bool, &i32)> = loop_last(&items).collect();
        assert_eq!(result, vec![(true, &42)]);
    }

    #[test]
    fn test_loop_last_empty() {
        let items: Vec<i32> = vec![];
        let result: Vec<(bool, &i32)> = loop_last(&items).collect();
        assert!(result.is_empty());
    }

    // -- Helpers ------------------------------------------------------------

    /// Split a flat list of segments into lines, using the given separator text.
    fn collect_lines<'a>(segments: &'a [Segment], sep: &str) -> Vec<Vec<&'a Segment>> {
        let mut lines: Vec<Vec<&Segment>> = Vec::new();
        let mut current: Vec<&Segment> = Vec::new();

        for seg in segments {
            if seg.text.contains(sep) {
                lines.push(std::mem::take(&mut current));
            } else {
                current.push(seg);
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
        lines
    }
}
