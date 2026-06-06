//! Scoped recording API for [`Console`].
//!
//! [`Console::scoped_record`] runs a closure with recording enabled, captures
//! only the output produced by that closure, restores the prior record state,
//! and returns a [`Recording`] value that can be exported to text, HTML, or
//! SVG without affecting the console's ongoing record buffer.

use crate::console::Console;
use crate::segment::Segment;
use crate::terminal_theme::{TerminalTheme, SVG_EXPORT_THEME};

// ---------------------------------------------------------------------------
// Recording
// ---------------------------------------------------------------------------

/// The captured output of a [`Console::scoped_record`] closure.
///
/// Holds the raw [`Segment`] list produced during the recording session and
/// carries the rendering settings needed to convert them back to text / HTML /
/// SVG.  All exports are derived from the same segment snapshot, so calling
/// any of the `to_*` methods multiple times is safe and idempotent.
///
/// # Examples
///
/// ```
/// use gilt::console::Console;
///
/// let mut console = Console::builder()
///     .width(80)
///     .force_terminal(true)
///     .record(true)
///     .build();
///
/// let rec = console.scoped_record(|c| {
///     c.print_text("[bold red]hi[/]");
/// });
/// assert!(rec.to_text().contains("hi"));
/// assert!(rec.to_html().contains("hi"));
/// ```
pub struct Recording {
    /// The captured raw segments.
    pub(crate) segments: Vec<Segment>,
    /// Clone of the console's color system at capture time (None = no color).
    pub(crate) color_system: Option<crate::color::ColorSystem>,
    /// Whether color output was disabled at capture time.
    pub(crate) no_color: bool,
    /// The console width at capture time (used by SVG export).
    pub(crate) width: usize,
}

impl Recording {
    /// Export as plain text (no ANSI codes, no colour).
    ///
    /// Control segments are stripped; only the text content of each segment
    /// is concatenated.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for seg in &self.segments {
            if !seg.is_control() {
                out.push_str(&seg.text);
            }
        }
        out
    }

    /// Export as a complete HTML document.
    ///
    /// Uses the default [`TerminalTheme`] and class-based styles, matching
    /// what [`Console::export_html`] produces.
    pub fn to_html(&self) -> String {
        self.to_html_with_theme(None, false)
    }

    /// Export as HTML, optionally choosing a terminal theme and inline-styles
    /// rendering.
    pub fn to_html_with_theme(
        &self,
        terminal_theme: Option<&TerminalTheme>,
        inline_styles: bool,
    ) -> String {
        // Build a temporary throw-away console pointing at our captured
        // segments so we can reuse the existing export_html logic.
        let mut tmp = build_tmp_console(self);
        tmp.export_html(terminal_theme, false, inline_styles)
    }

    /// Export as an SVG document with a window title.
    pub fn to_svg(&self, title: &str) -> String {
        self.to_svg_with_theme(title, None)
    }

    /// Export as an SVG document with an explicit terminal theme.
    pub fn to_svg_with_theme(&self, title: &str, terminal_theme: Option<&TerminalTheme>) -> String {
        let theme = terminal_theme.unwrap_or(&SVG_EXPORT_THEME);
        let mut tmp = build_tmp_console(self);
        tmp.export_svg(title, Some(theme), false, None, 0.61)
    }
}

// ---------------------------------------------------------------------------
// Console::scoped_record
// ---------------------------------------------------------------------------

impl Console {
    /// Run `f` with recording enabled, capture only its output, then restore
    /// the prior record state and return the captured output as a [`Recording`].
    ///
    /// This is the scoped recording API, modelled after Spectre.Console's
    /// `Record()` pattern.  Unlike the builder-level `record(true)` option,
    /// `scoped_record` is:
    ///
    /// - **Additive**: the snapshot covers only the output produced inside `f`.
    ///   The console's own record buffer accumulation is unaffected.
    /// - **State-restoring**: after the call the `record` flag and the record
    ///   buffer are exactly as they were before.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .force_terminal(true)
    ///     .record(true)
    ///     .build();
    ///
    /// let rec = console.scoped_record(|c| {
    ///     c.print_text("[bold red]hi[/]");
    /// });
    ///
    /// assert!(rec.to_html().contains("hi"));
    /// assert!(rec.to_text().contains("hi"));
    /// ```
    pub fn scoped_record<F>(&mut self, f: F) -> Recording
    where
        F: FnOnce(&mut Console),
    {
        // Save prior state.
        let prior_record = self.record;
        let prior_record_buffer = std::mem::take(&mut self.record_buffer);

        // Enable recording and run the closure.
        self.record = true;
        f(self);

        // Collect what the closure produced.
        let captured_segments = std::mem::take(&mut self.record_buffer);

        // Restore prior state.
        self.record = prior_record;
        self.record_buffer = prior_record_buffer;

        Recording {
            segments: captured_segments,
            color_system: self.color_system,
            no_color: self.no_color,
            width: self.width(),
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helper: build a throw-away Console for export
// ---------------------------------------------------------------------------

/// Construct a minimal `Console` that holds `rec.segments` in its record
/// buffer and is otherwise configured to match the capture settings.
fn build_tmp_console(rec: &Recording) -> Console {
    use crate::console::ConsoleBuilder;

    let mut tmp = ConsoleBuilder::default()
        .width(rec.width)
        .record(true)
        .no_color(rec.no_color)
        .build();

    // If the capture had a specific color system, override.
    if let Some(cs) = rec.color_system {
        tmp.color_system = Some(cs);
    }

    tmp.record_buffer = rec.segments.clone();
    tmp
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn recording_console() -> Console {
        Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build()
    }

    #[test]
    fn scoped_record_basic_text() {
        let mut c = recording_console();
        let rec = c.scoped_record(|c| {
            c.print_text("hello recording");
        });
        assert!(
            rec.to_text().contains("hello recording"),
            "to_text should contain printed text"
        );
    }

    #[test]
    fn scoped_record_html_contains_text() {
        let mut c = recording_console();
        let rec = c.scoped_record(|c| {
            c.print_text("html test");
        });
        let html = rec.to_html();
        assert!(html.contains("html test"), "to_html should contain text");
        assert!(
            html.contains("<!DOCTYPE html>"),
            "to_html should be a full HTML document"
        );
    }

    #[test]
    fn scoped_record_svg_contains_text() {
        let mut c = Console::builder()
            .width(40)
            .no_color(true)
            .markup(false)
            .build();
        let rec = c.scoped_record(|c| {
            c.print_text("svg content");
        });
        let svg = rec.to_svg("Title");
        assert!(svg.contains("<svg"), "to_svg should produce SVG");
        assert!(svg.contains("svg content"), "SVG should contain text");
    }

    #[test]
    fn scoped_record_restores_record_false() {
        let mut c = recording_console();
        assert!(!c.record, "precondition: record should be false");
        let _ = c.scoped_record(|_| {});
        assert!(!c.record, "record should be restored to false");
    }

    #[test]
    fn scoped_record_restores_record_true() {
        let mut c = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .record(true)
            .build();
        assert!(c.record, "precondition: record should be true");
        let _ = c.scoped_record(|_| {});
        assert!(c.record, "record should be restored to true");
    }

    #[test]
    fn scoped_record_does_not_mix_with_outer_buffer() {
        let mut c = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .record(true)
            .build();

        c.print_text("before");

        let _ = c.scoped_record(|c| {
            c.print_text("inside");
        });

        c.print_text("after");

        // The outer record buffer should have "before" and "after" but not "inside".
        let exported = c.export_text(false, false);
        assert!(exported.contains("before"), "outer buffer: before");
        assert!(exported.contains("after"), "outer buffer: after");
        assert!(
            !exported.contains("inside"),
            "outer buffer should NOT have inside; got {:?}",
            exported
        );
    }
}
