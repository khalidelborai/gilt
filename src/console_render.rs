//! Render-path methods for [`Console`]. Split out of console.rs in v1.3
//! Phase 5. The methods stay attached to `Console` via a separate
//! `impl Console` block; callers still write `console.print(...)` etc.
//!
//! Contains five sections from the original console.rs:
//!   - Core rendering (render, render_lines, render_str)
//!   - Print (print, print_styled, print_text, etc.)
//!   - Convenience methods (log, rule, line, print_json, inspect, print_error,
//!     print_exception)
//!   - Segment output (write_segments)
//!   - Buffering (enter_buffer, exit_buffer, check_buffer, flush_buffer,
//!     render_buffer)

use crate::cells::cell_len;
use crate::console::{Console, ConsoleOptions, Renderable};
use crate::error::traceback::Traceback;
#[cfg(feature = "json")]
use crate::json::{Json, JsonOptions};
use crate::markup;
use crate::measure::Measurement;
use crate::rule::Rule;
use crate::segment::Segment;
use crate::status::Status;
use crate::style::Style;
use crate::text::{JustifyMethod, OverflowMethod, Text};

impl Console {
    // -- Core rendering -----------------------------------------------------

    /// Render a Renderable into a flat list of Segments.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let console = Console::builder().width(80).build();
    /// let text = Text::new("Render me", Style::null());
    /// let segments = console.render(&text, None);
    /// let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
    /// assert!(combined.contains("Render me"));
    /// ```
    pub fn render(
        &self,
        renderable: &dyn Renderable,
        options: Option<&ConsoleOptions>,
    ) -> Vec<Segment> {
        let default_opts = self.options();
        let opts = options.unwrap_or(&default_opts);
        renderable.gilt_console(self, opts)
    }

    /// Render a Renderable into lines of Segments, with optional padding and newlines.
    pub fn render_lines(
        &self,
        renderable: &dyn Renderable,
        options: Option<&ConsoleOptions>,
        style: Option<&Style>,
        pad: bool,
        new_lines: bool,
    ) -> Vec<Vec<Segment>> {
        let default_opts = self.options();
        let opts = options.unwrap_or(&default_opts);
        let segments = renderable.gilt_console(self, opts);

        // Apply base style if present
        let segments = if let Some(base) = &self.base_style {
            Segment::apply_style(&segments, Some(base.clone()), None)
        } else {
            segments
        };

        Segment::split_and_crop_lines(&segments, opts.max_width, style, pad, new_lines)
    }

    /// Parse a string (optionally with markup) into a `Text` object.
    ///
    /// If markup is enabled on this console, rich markup tags (e.g. `[bold]`)
    /// are parsed and applied as spans.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let console = Console::builder().width(80).markup(false).build();
    /// let text = console.render_str("Hello, world!", None, None, None);
    /// assert_eq!(text.plain(), "Hello, world!");
    /// ```
    pub fn render_str(
        &self,
        text: &str,
        style: Option<&str>,
        justify: Option<JustifyMethod>,
        overflow: Option<OverflowMethod>,
    ) -> Text {
        let base_style = match style {
            Some(s) => Style::parse(s),
            None => Style::null(),
        };

        let mut gilt_text = if self.markup_enabled {
            markup::render(text, base_style.clone()).unwrap_or_else(|_| Text::new(text, base_style))
        } else {
            Text::new(text, base_style)
        };

        if let Some(j) = justify {
            gilt_text.justify = Some(j);
        }
        if let Some(o) = overflow {
            gilt_text.overflow = Some(o);
        }

        gilt_text
    }

    // -- Print --------------------------------------------------------------

    /// Print a Renderable to the console.
    ///
    /// Renders the object into segments and writes them to the output
    /// (terminal, capture buffer, or record buffer depending on mode).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder().width(80).no_color(true).build();
    /// console.begin_capture();
    /// let text = Text::new("Hello, world!", Style::null());
    /// console.print(&text);
    /// let output = console.end_capture();
    /// assert!(output.contains("Hello, world!"));
    /// ```
    pub fn print(&mut self, renderable: &dyn Renderable) {
        self.print_styled(renderable, None, None, None, false, true, false);
    }

    /// Print a Renderable with full styling options.
    #[allow(clippy::too_many_arguments)]
    pub fn print_styled(
        &mut self,
        renderable: &dyn Renderable,
        style: Option<&str>,
        justify: Option<JustifyMethod>,
        overflow: Option<OverflowMethod>,
        no_wrap: bool,
        crop: bool,
        soft_wrap: bool,
    ) {
        let mut opts = self.options();
        if let Some(j) = justify {
            opts.justify = Some(j);
        }
        if let Some(o) = overflow {
            opts.overflow = Some(o);
        }
        if no_wrap {
            opts.no_wrap = true;
        }

        let mut segments = renderable.gilt_console(self, &opts);

        // Apply additional style
        if let Some(style_str) = style {
            if let Ok(s) = Style::parse_strict(style_str) {
                segments = Segment::apply_style(&segments, Some(s), None);
            }
        }

        // Apply base style
        if let Some(base) = &self.base_style {
            segments = Segment::apply_style(&segments, Some(base.clone()), None);
        }

        // Handle no-color mode
        if self.no_color {
            segments = Segment::remove_color(&segments);
        }

        // Crop to width if requested
        if crop && !soft_wrap {
            let width = opts.max_width;
            let lines = Segment::split_and_crop_lines(&segments, width, None, false, true);
            segments = lines.into_iter().flatten().collect();
        }

        // Add newline if not ending with one
        if let Some(last) = segments.last() {
            if !last.text.ends_with('\n') {
                segments.push(Segment::line());
            }
        }

        self.write_segments(&segments);
    }

    /// Print a plain text string to the console.
    ///
    /// Parses the string through `render_str` (applying markup if enabled)
    /// before printing.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder().width(80).no_color(true).markup(false).build();
    /// console.begin_capture();
    /// console.print_text("Hello, terminal!");
    /// let output = console.end_capture();
    /// assert!(output.contains("Hello, terminal!"));
    /// ```
    pub fn print_text(&mut self, text: &str) {
        let gilt_text = self.render_str(text, None, None, None);
        self.print(&gilt_text);
    }

    /// Low-level raw print: write `text` verbatim to the buffer with the
    /// optional `style`, **without** markup parsing, emoji substitution,
    /// highlighting, or word wrap.
    ///
    /// Use this when you have content that already contains literal `[`
    /// brackets, `:tags:`, or other markup-like sequences and you don't want
    /// the console to interpret them. A trailing newline is appended.
    pub fn out(&mut self, text: &str, style: Option<&Style>) {
        let segment = match style {
            Some(s) => Segment::styled(text, s.clone()),
            None => Segment::text(text),
        };
        let mut buf = vec![segment];
        if !text.ends_with('\n') {
            buf.push(Segment::line());
        }
        self.write_segments(&buf);
    }

    // -- Convenience methods ------------------------------------------------

    /// Print a log line with a timestamp prefix.
    ///
    /// The current time is formatted as `[HH:MM:SS]` and styled with the
    /// `"log.time"` theme style, followed by a space and the rendered text.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder().width(80).no_color(true).markup(false).build();
    /// console.begin_capture();
    /// console.log("Processing started");
    /// let output = console.end_capture();
    /// assert!(output.contains("Processing started"));
    /// assert!(output.contains('['));  // timestamp bracket
    /// ```
    pub fn log(&mut self, text: &str) {
        let now = {
            // Get current local time using libc/localtime
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // Format manually to avoid pulling in chrono
            let secs_i64 = secs as i64;
            // Simple UTC-based formatting (matches Python's default local-time log,
            // but always UTC -- acceptable for a library without chrono).
            let secs_of_day = ((secs_i64 % 86400) + 86400) % 86400;
            let h = secs_of_day / 3600;
            let m = (secs_of_day % 3600) / 60;
            let s = secs_of_day % 60;
            format!("[{:02}:{:02}:{:02}]", h, m, s)
        };

        let time_style = self
            .get_style("log.time")
            .unwrap_or_else(|_| Style::parse("dim"));

        let time_text = Text::styled_with(&now, time_style);
        let body = self.render_str(text, None, None, None);

        // Combine: time + space + body
        let mut segments = time_text.gilt_console(self, &self.options());
        // Remove trailing newline from time segments
        segments.retain(|s| s.text != "\n");
        segments.push(Segment::text(" "));
        segments.extend(body.gilt_console(self, &self.options()));

        // Ensure trailing newline
        if let Some(last) = segments.last() {
            if !last.text.ends_with('\n') {
                segments.push(Segment::line());
            }
        }

        self.write_segments(&segments);
    }

    /// Print a horizontal rule, optionally with a title.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder().width(40).no_color(true).markup(false).build();
    /// console.begin_capture();
    /// console.rule(Some("Section"));
    /// let output = console.end_capture();
    /// assert!(output.contains("Section"));
    /// ```
    pub fn rule(&mut self, title: Option<&str>) {
        let rule = match title {
            Some(t) => Rule::with_title(t),
            None => Rule::new(),
        };
        self.print(&rule);
    }

    /// Print `count` blank lines.
    pub fn line(&mut self, count: usize) {
        for _ in 0..count {
            self.write_segments(&[Segment::line()]);
        }
    }

    /// Display a prompt and read a line of input from stdin.
    ///
    /// The prompt is rendered as markup text. Returns the input line
    /// (with trailing newline stripped).
    pub fn input(&mut self, prompt: &str) -> Result<String, std::io::Error> {
        // Render and print the prompt (without trailing newline)
        let text = self.render_str(prompt, None, None, None);
        let mut segments = text.gilt_console(self, &self.options());
        // Remove trailing newlines so the cursor stays on the prompt line
        segments.retain(|s| s.text != "\n");
        self.write_segments(&segments);

        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        // Strip the trailing newline
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        Ok(buf)
    }

    /// Display a prompt and read a line of password input from stdin.
    ///
    /// Like [`input`](Console::input), but terminal echo is disabled so the
    /// typed characters are not visible on screen. Uses `rpassword` for
    /// cross-platform hidden input.
    #[cfg(feature = "interactive")]
    pub fn input_password(&mut self, prompt: &str) -> Result<String, std::io::Error> {
        // Render and print the prompt (without trailing newline)
        let text = self.render_str(prompt, None, None, None);
        let mut segments = text.gilt_console(self, &self.options());
        segments.retain(|s| s.text != "\n");
        self.write_segments(&segments);

        rpassword::read_password()
    }

    /// Pretty-print a JSON string with syntax highlighting.
    ///
    /// If the input is not valid JSON, prints the raw string instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder().width(80).no_color(true).markup(false).build();
    /// console.begin_capture();
    /// console.print_json(r#"{"name": "Alice"}"#);
    /// let output = console.end_capture();
    /// assert!(output.contains("name"));
    /// assert!(output.contains("Alice"));
    /// ```
    #[cfg(feature = "json")]
    pub fn print_json(&mut self, json: &str) {
        match Json::new(json, JsonOptions::default()) {
            Ok(json_widget) => self.print(&json_widget),
            Err(_) => self.print_text(json),
        }
    }

    /// Inspect a value, printing its type, debug representation, and optional docs.
    ///
    /// Renders the value inside a styled panel using the [`Inspect`](crate::inspect::Inspect) widget.
    pub fn inspect<T: std::fmt::Debug + 'static>(&mut self, value: &T) {
        let widget = crate::inspect::Inspect::new(value);
        self.print(&widget);
    }

    /// Print an error with its causal chain, rendered inside a panel.
    pub fn print_error(&mut self, error: &dyn std::error::Error) {
        let tb = Traceback::from_error(error);
        self.print(&tb);
    }

    /// Print an exception (error) with its causal chain as a styled traceback.
    ///
    /// This is a convenience alias for [`print_error`](Console::print_error) that
    /// matches the  `Console.print_exception()` API name.
    pub fn print_exception(&mut self, error: &dyn std::error::Error) {
        self.print_error(error);
    }

    /// Measure the minimum and maximum width of a renderable.
    ///
    /// Returns a `Measurement` with the minimum (longest word) and
    /// maximum (longest line) cell widths. For types that implement
    /// their own measurement (like `Text`), this renders and measures
    /// the output segments.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let console = Console::builder().width(80).no_color(true).markup(false).build();
    /// let text = Text::new("Hello World", Style::null());
    /// let measurement = console.measure(&text);
    /// assert_eq!(measurement.minimum, 5);  // longest word: "Hello" or "World"
    /// assert_eq!(measurement.maximum, 11); // full line: "Hello World"
    /// ```
    pub fn measure(&self, renderable: &dyn Renderable) -> Measurement {
        let opts = self.options();
        let segments = renderable.gilt_console(self, &opts);
        // Collect all text, split by newlines to find line widths
        let full_text: String = segments
            .iter()
            .filter(|s| !s.is_control())
            .map(|s| s.text.as_str())
            .collect();
        if full_text.is_empty() {
            return Measurement::new(0, 0);
        }
        let max_width = full_text.lines().map(cell_len).max().unwrap_or(0);
        let min_width = full_text
            .split_whitespace()
            .map(cell_len)
            .max()
            .unwrap_or(0);
        Measurement::new(min_width, max_width)
    }

    /// Create a [`Status`] spinner with the given message.
    ///
    /// Returns a `Status` instance that can be started and stopped.
    /// Defaults to the `"dots"` spinner.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gilt::console::Console;
    ///
    /// let mut status = Console::new().status("Working...");
    /// status.start();
    /// // ... do work ...
    /// status.stop();
    /// ```
    pub fn status(self, message: &str) -> Status {
        Status::new(message).with_console(self)
    }

    /// Export recorded text and save it to a file.
    ///
    /// Requires `record` mode to be enabled when the Console was created.
    pub fn save_text(
        &mut self,
        path: &str,
        clear: bool,
        styles: bool,
    ) -> Result<(), std::io::Error> {
        let text = self.export_text(clear, styles);
        std::fs::write(path, text)
    }

    /// Export recorded output as HTML and save it to a file.
    ///
    /// Requires `record` mode to be enabled when the Console was created.
    pub fn save_html(&mut self, path: &str) -> Result<(), std::io::Error> {
        let html = self.export_html(None, false, true);
        std::fs::write(path, html)
    }

    /// Export recorded output as SVG and save it to a file.
    ///
    /// Requires `record` mode to be enabled when the Console was created.
    pub fn save_svg(&mut self, path: &str, title: Option<&str>) -> Result<(), std::io::Error> {
        let t = title.unwrap_or("gilt");
        let svg = self.export_svg(t, None, false, None, 0.61);
        std::fs::write(path, svg)
    }

    // -- Segment output -----------------------------------------------------

    pub(crate) fn write_segments(&mut self, segments: &[Segment]) {
        if self.quiet {
            return;
        }

        if self.record {
            self.record_buffer.extend(segments.iter().cloned());
        }

        if let Some(ref mut capture) = self.capture_buffer {
            capture.extend(segments.iter().cloned());
            return;
        }

        if self.buffer_index > 0 {
            self.buffer.extend(segments.iter().cloned());
            return;
        }

        // Default path: render to ANSI and write to stdout immediately.
        let output = self.render_buffer(segments);
        use std::io::Write;
        let _ = std::io::stdout().write_all(output.as_bytes());
        let _ = std::io::stdout().flush();
    }

    // -- Buffering ----------------------------------------------------------

    /// Enter a buffering context. Segments are accumulated until `exit_buffer`.
    pub fn enter_buffer(&mut self) {
        self.buffer_index += 1;
    }

    /// Exit the current buffering context. When the last buffer exits, flush.
    pub fn exit_buffer(&mut self) {
        if self.buffer_index > 0 {
            self.buffer_index -= 1;
        }
        if self.buffer_index == 0 {
            self.flush_buffer();
        }
    }

    /// Check if currently in a buffer context.
    pub fn check_buffer(&self) -> bool {
        self.buffer_index > 0
    }

    /// Flush the buffer, converting accumulated segments to an output string
    /// and writing it to stdout (or the active capture/record sink).
    ///
    /// Called by [`exit_buffer`](Self::exit_buffer) when the outermost buffer
    /// context closes. Without the stdout write, anything accumulated under
    /// `enter_buffer` would be silently discarded.
    fn flush_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        let segments = std::mem::take(&mut self.buffer);
        // If a capture is active, divert to the capture buffer; otherwise
        // render to ANSI and write to stdout (subject to `quiet`).
        if let Some(ref mut capture) = self.capture_buffer {
            capture.extend(segments);
            return;
        }
        if self.quiet {
            return;
        }
        let output = self.render_buffer(&segments);
        use std::io::Write;
        let _ = std::io::stdout().write_all(output.as_bytes());
        let _ = std::io::stdout().flush();
    }

    /// Convert a slice of segments into an ANSI-rendered string.
    ///
    /// Applies style rendering (colors, bold, links) based on the console's
    /// active color system. Control segments are passed through as-is.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::segment::Segment;
    ///
    /// let console = Console::builder().no_color(true).color_system("").build();
    /// let segments = vec![Segment::text("Hello")];
    /// let output = console.render_buffer(&segments);
    /// assert_eq!(output, "Hello");
    /// ```
    pub fn render_buffer(&self, buffer: &[Segment]) -> String {
        // Pre-size: text bytes + ~16 bytes per segment for SGR overhead
        // (`\x1b[1;38;5;NNNm...\x1b[0m` is ~12-20 bytes per styled segment).
        let estimated_bytes: usize =
            buffer.iter().map(|s| s.text.len()).sum::<usize>() + buffer.len() * 16;
        let mut output = String::with_capacity(estimated_bytes);
        let color_system = if self.no_color {
            None
        } else {
            self.color_system
        };

        // OSC 8 hyperlinks must be emitted as ONE wrapper around a run of
        // consecutive segments that all share the same URL — fragmenting the
        // run (open/close around each segment) makes many terminals fail to
        // treat the whole text as a single clickable link. We track the
        // currently-open link URL and only emit open/close at run boundaries.
        let mut current_link: Option<String> = None;

        let close_link = |out: &mut String, link: &mut Option<String>| {
            if link.take().is_some() {
                out.push_str("\x1b]8;;\x1b\\");
            }
        };

        for segment in buffer {
            if segment.is_control() {
                // Control codes (cursor moves, screen clears, OSC sequences
                // we don't manage) interrupt any active link wrapper.
                close_link(&mut output, &mut current_link);
                output.push_str(&segment.text);
                continue;
            }

            // Determine this segment's link, if any.
            let seg_link: Option<&str> = segment.style().and_then(|s| s.link());

            // Emit OSC 8 open/close only when the link changes.
            match (seg_link, current_link.as_deref()) {
                (Some(new), Some(cur)) if new == cur => {
                    // Same link continuing — leave wrapper open.
                }
                (Some(new), _) => {
                    close_link(&mut output, &mut current_link);
                    use std::fmt::Write;
                    let id = crate::style::next_link_id();
                    write!(output, "\x1b]8;id={};{}\x1b\\", id, new).unwrap();
                    current_link = Some(new.to_string());
                }
                (None, _) => {
                    close_link(&mut output, &mut current_link);
                }
            }

            if let Some(style) = segment.style() {
                // We've handled the link wrapper; render only colors/SGR.
                output.push_str(&style.render_no_link(&segment.text, color_system));
            } else {
                output.push_str(&segment.text);
            }
        }

        // Close any link still open at end of buffer.
        if current_link.is_some() {
            output.push_str("\x1b]8;;\x1b\\");
        }
        output
    }
}
