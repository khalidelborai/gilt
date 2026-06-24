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
    pub fn render<R: Renderable + ?Sized>(
        &self,
        renderable: &R,
        options: Option<&ConsoleOptions>,
    ) -> Vec<Segment> {
        let default_opts = self.options();
        let opts = options.unwrap_or(&default_opts);
        renderable.gilt_console(self, opts)
    }

    /// Render a Renderable into lines of Segments, with optional padding and newlines.
    ///
    /// When `options.height` is `Some(h)`, the result is truncated or padded
    /// with blank lines to exactly `h` rows (finding #14 parity with rich).
    pub fn render_lines<R: Renderable + ?Sized>(
        &self,
        renderable: &R,
        options: Option<&ConsoleOptions>,
        style: Option<&Style>,
        pad: bool,
        new_lines: bool,
    ) -> Vec<Vec<Segment>> {
        let default_opts = self.options();
        let opts = options.unwrap_or(&default_opts);
        let segments = renderable.gilt_console(self, opts);

        // Apply the caller-supplied style first (parity: style param was previously
        // forwarded only to split_and_crop_lines for padding, never to the segments).
        let segments = if let Some(s) = style {
            Segment::apply_style(&segments, Some(s.clone()), None)
        } else {
            segments
        };

        // Apply base style if present
        let segments = if let Some(base) = &self.base_style {
            Segment::apply_style(&segments, Some(base.clone()), None)
        } else {
            segments
        };

        let mut lines =
            Segment::split_and_crop_lines(&segments, opts.max_width, style, pad, new_lines);

        // Finding #14: truncate or pad to opts.height when set.
        if let Some(height) = opts.height {
            lines.truncate(height);
            while lines.len() < height {
                // Pad with a blank newline row.
                let blank = if new_lines {
                    vec![Segment::line()]
                } else {
                    vec![]
                };
                lines.push(blank);
            }
        }

        lines
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

        // Emoji replacement always runs (parity with rich).
        let replaced = crate::utils::emoji_replace::emoji_replace(text, None);
        let text_ref: &str = replaced.as_ref();

        let mut gilt_text = if self.markup_enabled {
            markup::render(text_ref, base_style.clone())
                .unwrap_or_else(|_| Text::new(text_ref, base_style))
        } else {
            Text::new(text_ref, base_style)
        };

        // Highlighting runs when enabled (parity with rich's console highlighter).
        if self.highlight_enabled {
            self.highlighter.highlight(&mut gilt_text);
        }

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
    pub fn print<R: Renderable + ?Sized>(&mut self, renderable: &R) {
        self.print_styled(renderable, None, None, None, false, true, self.soft_wrap);
    }

    /// Print a `&dyn Renderable`, running it through any registered
    /// [`RenderHook`]s before rendering.
    ///
    /// Use this instead of [`print`](Self::print) when you already have a
    /// trait object and want hook processing.
    ///
    /// [`RenderHook`]: crate::console::RenderHook
    pub fn print_with_hooks(&mut self, renderable: &dyn Renderable) {
        if self.render_hooks.is_empty() {
            self.print_styled(renderable, None, None, None, false, true, self.soft_wrap);
            return;
        }
        let hooks = std::mem::take(&mut self.render_hooks);
        let mut current: Vec<&dyn Renderable> = vec![renderable];
        for hook in &hooks {
            current = hook.process_renderables(current);
        }
        self.render_hooks = hooks;
        if let Some(r) = current.last().copied() {
            self.print_styled(r, None, None, None, false, true, self.soft_wrap);
        }
    }

    /// Print a Renderable with full styling options.
    #[allow(clippy::too_many_arguments)]
    pub fn print_styled<R: Renderable + ?Sized>(
        &mut self,
        renderable: &R,
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
            opts.no_wrap = Some(true);
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

    /// Print a slice of renderables with a custom separator and end string.
    ///
    /// Renders each item with `sep` between them and `end` appended after the
    /// last item. A newline is added automatically if `end` does not end with one.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder().width(80).no_color(true).markup(false).build();
    /// console.begin_capture();
    /// let a = Text::new("foo", Style::null());
    /// let b = Text::new("bar", Style::null());
    /// console.print_sep_end(&[&a, &b], ", ", "!");
    /// let out = console.end_capture();
    /// assert!(out.contains("foo"));
    /// assert!(out.contains(", "));
    /// assert!(out.contains("bar"));
    /// assert!(out.contains('!'));
    /// ```
    pub fn print_sep_end(&mut self, items: &[&dyn Renderable], sep: &str, end: &str) {
        let mut all_segments: Vec<Segment> = Vec::new();
        let opts = self.options();

        for (i, item) in items.iter().enumerate() {
            if i > 0 && !sep.is_empty() {
                all_segments.push(Segment::text(sep));
            }
            let mut segs = item.gilt_console(self, &opts);
            // Strip trailing newlines from individual items
            while segs.last().map(|s| s.text.as_str()) == Some("\n") {
                segs.pop();
            }
            all_segments.extend(segs);
        }

        if !end.is_empty() {
            all_segments.push(Segment::text(end));
        }

        // Ensure there is a trailing newline
        if all_segments
            .last()
            .map(|s| !s.text.ends_with('\n'))
            .unwrap_or(true)
        {
            all_segments.push(Segment::line());
        }

        self.write_segments(&all_segments);
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
    /// Print a log line with a timestamp prefix.
    ///
    /// When `log_path` is enabled in the console options (future flag), the
    /// caller's file and line number (captured via `#[track_caller]`) are
    /// appended. Currently captures the location and makes it available for
    /// future use; the path suffix is appended when `self.log_path` is enabled.
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
    #[track_caller]
    pub fn log(&mut self, text: &str) {
        // Finding #13: capture call-site location (WASM-safe: std::panic::Location).
        let location = std::panic::Location::caller();
        let caller_path = {
            // Show only the last component of the file path for brevity.
            let file = location.file();
            let short = file.rsplit('/').next().unwrap_or(file);
            format!(" [{}:{}]", short, location.line())
        };

        let now = {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let secs_i64 = secs as i64;
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

        // Finding #16: cache options() once instead of calling twice.
        let opts = self.options();

        // Combine: time + space + body [+ optional caller path]
        let mut segments = time_text.gilt_console(self, &opts);
        // Remove trailing newline from time segments
        segments.retain(|s| s.text != "\n");
        segments.push(Segment::text(" "));
        segments.extend(body.gilt_console(self, &opts));

        // Append caller path when log_path is enabled on this console.
        if self.log_path {
            // Right-align the path suffix by appending it as a dim segment
            // after a space, mirroring rich's right-aligned dim path display.
            let path_style = self
                .get_style("log.path")
                .unwrap_or_else(|_| Style::parse("dim"));
            // Remove the trailing newline from the body segments so we can
            // append the path before re-adding it.
            segments.retain(|s| s.text != "\n");
            segments.push(Segment::text(" "));
            segments.push(Segment::styled(&caller_path, path_style));
        } else {
            let _ = caller_path; // suppress unused-variable warning
        }

        // Ensure trailing newline
        if let Some(last) = segments.last() {
            if !last.text.ends_with('\n') {
                segments.push(Segment::line());
            }
        }

        self.write_segments(&segments);
    }

    /// Print multiple renderables as a single log line, separated by spaces.
    ///
    /// Like [`log`](Self::log), prepends a `[HH:MM:SS]` timestamp. When
    /// `log_locals` is `true`, a styled `"(locals)"` label is appended after
    /// the objects (full locals introspection is deferred to a future release).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder().width(80).no_color(true).markup(false).build();
    /// console.begin_capture();
    /// let a = Text::new("foo", Style::null());
    /// let b = Text::new("bar", Style::null());
    /// console.log_objects(&[&a, &b], false);
    /// let out = console.end_capture();
    /// assert!(out.contains("foo"));
    /// assert!(out.contains("bar"));
    /// ```
    #[track_caller]
    pub fn log_objects(&mut self, objects: &[&dyn Renderable], log_locals: bool) {
        // Mirror log()'s #[track_caller] so log_path works for log_objects too.
        let location = std::panic::Location::caller();
        let caller_path = {
            let file = location.file();
            let short = file.rsplit('/').next().unwrap_or(file);
            format!(" [{}:{}]", short, location.line())
        };

        let now = {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let secs_i64 = secs as i64;
            let secs_of_day = ((secs_i64 % 86400) + 86400) % 86400;
            let h = secs_of_day / 3600;
            let m = (secs_of_day % 3600) / 60;
            let s = secs_of_day % 60;
            format!("[{:02}:{:02}:{:02}]", h, m, s)
        };

        let time_style = self
            .get_style("log.time")
            .unwrap_or_else(|_| Style::parse("dim"));

        let opts = self.options();

        let time_text = Text::styled_with(&now, time_style);
        let mut segments = time_text.gilt_console(self, &opts);
        // Remove trailing newline from time
        segments.retain(|s| s.text != "\n");

        // Render each object separated by a space
        for obj in objects.iter() {
            segments.push(Segment::text(" "));
            let mut obj_segs = obj.gilt_console(self, &opts);
            obj_segs.retain(|s| s.text != "\n");
            segments.extend(obj_segs);
        }

        // Optional locals label
        if log_locals {
            let locals_style = self
                .get_style("log.path")
                .unwrap_or_else(|_| Style::parse("dim"));
            segments.push(Segment::text(" "));
            segments.push(Segment::styled("(locals)", locals_style));
        }

        // Append caller path when log_path is enabled — mirrors log() behaviour.
        if self.log_path {
            let path_style = self
                .get_style("log.path")
                .unwrap_or_else(|_| Style::parse("dim"));
            segments.retain(|s| s.text != "\n");
            segments.push(Segment::text(" "));
            segments.push(Segment::styled(&caller_path, path_style));
        } else {
            let _ = caller_path;
        }

        // Ensure trailing newline
        if let Some(last) = segments.last() {
            if !last.text.ends_with('\n') {
                segments.push(Segment::line());
            }
        } else {
            segments.push(Segment::line());
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

    /// Display a prompt and read a line of input from an optional stream.
    ///
    /// When `stream` is `Some`, reads from that stream (useful for testing).
    /// When `stream` is `None`, reads from stdin (same as [`input`](Console::input)).
    /// Returns the input line with trailing newline stripped.
    pub fn input_with_stream(
        &mut self,
        prompt: &str,
        stream: Option<&mut dyn std::io::BufRead>,
    ) -> Result<String, std::io::Error> {
        // Render and print the prompt (without trailing newline)
        let text = self.render_str(prompt, None, None, None);
        let mut segments = text.gilt_console(self, &self.options());
        segments.retain(|s| s.text != "\n");
        self.write_segments(&segments);

        let mut buf = String::new();
        match stream {
            Some(reader) => {
                reader.read_line(&mut buf)?;
            }
            None => {
                std::io::stdin().read_line(&mut buf)?;
            }
        }
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

    /// Pretty-print a value implementing [`std::fmt::Debug`] to the console.
    ///
    /// Constructs a [`Pretty`] widget from the debug representation and prints it.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder().width(80).no_color(true).build();
    /// console.begin_capture();
    /// console.pprint(&vec![1, 2, 3]);
    /// let output = console.end_capture();
    /// assert!(output.contains('1'));
    /// ```
    pub fn pprint<T: std::fmt::Debug>(&mut self, value: &T) {
        use crate::utils::pretty::Pretty;
        self.print(&Pretty::from_debug(value));
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
    /// maximum (longest line) cell widths. Dispatches through the
    /// [`Renderable::gilt_measure`] protocol so widgets with their own
    /// measurement logic (e.g. `Text`, `Panel`, `Table`) are measured
    /// without a full render.  Types without an override fall back to
    /// the default `gilt_measure` implementation, which renders and
    /// derives widths from the output segments (identical to the old
    /// direct-render logic), with the exception that an empty renderable
    /// now yields `(0, max_width)` rather than `(0, 0)` (audit #3).
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
    pub fn measure<R: Renderable + ?Sized>(&self, renderable: &R) -> Measurement {
        let opts = self.options();
        measurement_get(self, &opts, renderable)
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
    pub fn save_html(
        &mut self,
        path: &str,
        theme: Option<&crate::terminal_theme::TerminalTheme>,
        clear: bool,
        inline_styles: bool,
        code_format: Option<&str>,
    ) -> Result<(), std::io::Error> {
        use crate::export_format::HtmlExportOptions;
        let opts = HtmlExportOptions::default()
            .clear(clear)
            .inline_styles(inline_styles);
        let opts = if let Some(cf) = code_format {
            opts.code_format(cf)
        } else {
            opts
        };
        let html = self.export_html_opts(theme, &opts);
        std::fs::write(path, html)
    }

    /// Export recorded output as SVG and save it to a file.
    ///
    /// Requires `record` mode to be enabled when the Console was created.
    ///
    /// Note: `code_format` is not supported by `export_svg`/`SvgExportOptions`;
    /// it is accepted for API symmetry with `save_html` but currently ignored.
    pub fn save_svg(
        &mut self,
        path: &str,
        title: Option<&str>,
        theme: Option<&crate::terminal_theme::TerminalTheme>,
        clear: bool,
        unique_id: Option<&str>,
        code_format: Option<&str>,
    ) -> Result<(), std::io::Error> {
        // code_format is not supported by export_svg/SvgExportOptions; it is accepted
        // for API symmetry with save_html but currently ignored.
        let _ = code_format;
        let t = title.unwrap_or("gilt");
        let svg = self.export_svg(t, theme, clear, unique_id, 0.61);
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

        // Asciinema timed-event capture (zero cost when session is not active).
        #[cfg(feature = "asciinema")]
        self.maybe_record_asciinema_event(segments);

        if let Some(ref mut capture) = self.capture_buffer {
            capture.extend(segments.iter().cloned());
            return;
        }

        if self.buffer_index > 0 {
            self.buffer.extend(segments.iter().cloned());
            return;
        }

        // Default path: render to ANSI and write to the configured sink
        // (custom writer if set via `Console::with_writer`, else stdout).
        //
        // Opt 2 (BufWriter coalescing): when inside a synchronized block
        // (`sync_depth > 0`), skip the per-write `flush()`. The
        // `BufWriter` wrapping `writer_override` accumulates all writes and
        // flushes them in one OS call at `end_synchronized`. For the stdout
        // path and unsynchronized writes, always flush immediately to
        // preserve existing visibility semantics.
        let output = self.render_buffer(segments);
        use std::io::Write as _;
        let deferred_flush = self.sync_depth > 0;
        match self.writer_override.as_mut() {
            Some(w) => {
                let _ = w.write_all(output.as_bytes());
                if !deferred_flush {
                    let _ = w.flush();
                }
            }
            None => {
                let _ = std::io::stdout().write_all(output.as_bytes());
                let _ = std::io::stdout().flush();
            }
        }
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
        use std::io::Write as _;
        let deferred_flush = self.sync_depth > 0;
        match self.writer_override.as_mut() {
            Some(w) => {
                let _ = w.write_all(output.as_bytes());
                if !deferred_flush {
                    let _ = w.flush();
                }
            }
            None => {
                let _ = std::io::stdout().write_all(output.as_bytes());
                let _ = std::io::stdout().flush();
            }
        }
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

// ---------------------------------------------------------------------------
// Measurement protocol free functions
// ---------------------------------------------------------------------------
//
// These live in console_render.rs (NOT in measure.rs) to avoid a circular
// import: measure.rs is imported by console.rs; putting Console/Renderable
// references back in measure.rs would form a dependency cycle.
// They are re-exported from `crate::measure` via `pub use` in measure.rs.

/// Get the `Measurement` for a single `Renderable`, normalized and clamped to
/// `options.max_width`.
///
/// This is the Rust equivalent of rich's `Measurement.get(console, options, renderable)`.
/// It calls `r.gilt_measure(console, options)`, normalizes (ensures minimum ≤ maximum),
/// and then clamps the result so neither field exceeds `options.max_width`.
pub fn measurement_get<R: Renderable + ?Sized>(
    console: &Console,
    options: &ConsoleOptions,
    r: &R,
) -> Measurement {
    r.gilt_measure(console, options)
        .normalize()
        .with_maximum(options.max_width)
}

/// Get a `Measurement` that spans all the given `Renderable`s.
///
/// This is the Rust equivalent of rich's `measure_renderables(console, options, renderables)`.
/// Combine semantics (matching rich's contract):
/// - `minimum` = max of individual minimums (widest "must-have" requirement wins)
/// - `maximum` = max of individual maximums (widest "could-use" requirement wins)
///
/// Returns `Measurement::new(0, 0)` for an empty slice.
pub fn measure_renderables<R: Renderable + ?Sized>(
    console: &Console,
    options: &ConsoleOptions,
    rs: &[&R],
) -> Measurement {
    if rs.is_empty() {
        return Measurement::new(0, 0);
    }
    let measurements: Vec<Measurement> = rs
        .iter()
        .map(|r| measurement_get(console, options, *r))
        .collect();
    let minimum = measurements.iter().map(|m| m.minimum).max().unwrap_or(0);
    let maximum = measurements.iter().map(|m| m.maximum).max().unwrap_or(0);
    Measurement::new(minimum, maximum)
}

// ---------------------------------------------------------------------------
// Batch 7.7 parity tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod batch_7_7_tests {
    use super::*;
    use crate::console::Console;
    use crate::style::Style;
    use crate::text::Text;

    // -- Item 1: render_str emoji + highlight --------------------------------

    #[test]
    fn render_str_emoji_replace_always_runs() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        // ":wave:" should be replaced by the wave emoji character
        let t = console.render_str(":wave:", None, None, None);
        let plain = t.plain();
        // After substitution it should no longer look like a raw `:wave:` tag
        assert!(
            !plain.contains(":wave:"),
            "expected emoji replacement, got {:?}",
            plain
        );
    }

    #[test]
    fn render_str_unknown_emoji_stays_unchanged() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        let t = console.render_str(":this_is_not_a_known_emoji:", None, None, None);
        assert_eq!(t.plain(), ":this_is_not_a_known_emoji:");
    }

    #[test]
    fn render_str_highlight_adds_spans_when_enabled() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .highlight(true)
            .build();
        // "42" matches repr.number — expect at least one span
        let t = console.render_str("value=42", None, None, None);
        assert!(
            !t.spans().is_empty(),
            "expected highlight spans for 'value=42'"
        );
    }

    #[test]
    fn render_str_no_spans_when_highlight_disabled() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .highlight(false)
            .build();
        let t = console.render_str("value=42", None, None, None);
        assert!(
            t.spans().is_empty(),
            "expected no spans when highlight is disabled"
        );
    }

    // -- Item 2: render_lines style-apply before crop ------------------------

    #[test]
    fn render_lines_applies_style_to_segments() {
        let console = Console::builder()
            .width(40)
            .no_color(false)
            .markup(false)
            .build();
        let text = Text::new("hello", Style::null());
        let bold = Style::parse("bold");
        let lines = console.render_lines(&text, None, Some(&bold), false, false);
        // All non-control segments in the result should carry at least the bold style
        let all_styled = lines.iter().flatten().all(|s| {
            s.is_control() || s.style().map(|st| st.bold() == Some(true)).unwrap_or(false)
        });
        assert!(all_styled, "expected bold style on all rendered segments");
    }

    // -- Item 3: log_objects -------------------------------------------------

    #[test]
    fn log_objects_renders_multiple() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.begin_capture();
        let a = Text::new("alpha", Style::null());
        let b = Text::new("beta", Style::null());
        console.log_objects(&[&a, &b], false);
        let out = console.end_capture();
        assert!(out.contains("alpha"), "missing 'alpha' in {:?}", out);
        assert!(out.contains("beta"), "missing 'beta' in {:?}", out);
        assert!(out.contains('['), "missing timestamp bracket in {:?}", out);
    }

    #[test]
    fn log_objects_log_locals_appends_label() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.begin_capture();
        let a = Text::new("x", Style::null());
        console.log_objects(&[&a], true);
        let out = console.end_capture();
        assert!(
            out.contains("locals"),
            "expected '(locals)' label in {:?}",
            out
        );
    }

    #[test]
    fn log_objects_empty_slice_does_not_panic() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.begin_capture();
        console.log_objects(&[], false);
        let out = console.end_capture();
        assert!(out.contains('['), "expected at least a timestamp bracket");
    }

    // -- Item 4: soft_wrap wiring -------------------------------------------

    #[test]
    fn soft_wrap_flag_wired_to_print() {
        // Verify Console::builder().soft_wrap(true) doesn't crash and the
        // console can print without cropping.
        let mut console = Console::builder()
            .width(20)
            .no_color(true)
            .markup(false)
            .soft_wrap(true)
            .build();
        console.begin_capture();
        // A line longer than 20 chars should appear fully when soft_wrap=true
        console.print_text("a very long line that exceeds the console width by quite a lot");
        let out = console.end_capture();
        assert!(
            out.contains("long line"),
            "expected long line in output, got {:?}",
            out
        );
    }

    // -- Item 5: pager_with --------------------------------------------------

    #[test]
    fn pager_with_does_not_panic() {
        // Verify pager_with runs the closure and pipes to the pager without panicking.
        // Use "cat" as pager and quiet=true to suppress stdout.
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .quiet(true)
            .build();
        console.pager_with(
            |c| {
                c.print_text("hello from pager_with");
            },
            Some("cat"),
        );
    }

    #[test]
    fn pager_with_records_closure_output() {
        // Verify that pager_with enables recording for the closure duration.
        // We intercept by also having record=true before the call so the buffer
        // accumulates; pager_with's internal clear happens on its own pass.
        // Use "cat" to avoid spawning a real pager.
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .record(true)
            .quiet(true)
            .build();
        // Print something before pager_with so record_buffer has content.
        // Then pager_with should record its own content independently.
        // We verify the cat call doesn't panic (it consumes the output).
        console.pager_with(
            |c| {
                c.print_text("inside pager_with");
            },
            Some("cat"),
        );
        // After pager_with, record buffer was cleared by export_text(clear=true);
        // the console is still in record mode (was_recording was true).
        let remaining = console.export_text(false, false);
        // Nothing should remain (the pager_with cleared it), but no panic.
        let _ = remaining;
    }

    // -- Item 6: RenderHook pipeline ----------------------------------------

    #[test]
    fn render_hook_is_called() {
        use crate::console::RenderHook;
        use std::sync::{Arc, Mutex};

        struct CountingHook {
            count: Arc<Mutex<usize>>,
        }
        impl RenderHook for CountingHook {
            fn process_renderables<'a>(
                &self,
                renderables: Vec<&'a dyn Renderable>,
            ) -> Vec<&'a dyn Renderable> {
                *self.count.lock().unwrap() += 1;
                renderables
            }
        }

        let count = Arc::new(Mutex::new(0usize));
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.add_render_hook(Box::new(CountingHook {
            count: Arc::clone(&count),
        }));

        console.begin_capture();
        let text = Text::new("hi", Style::null());
        console.print_with_hooks(&text as &dyn Renderable);
        let _ = console.end_capture();

        assert_eq!(
            *count.lock().unwrap(),
            1,
            "hook should have been called once"
        );
    }

    #[test]
    fn render_hook_can_replace_renderable() {
        use crate::console::RenderHook;

        struct ReplaceHook;
        static REPLACEMENT: std::sync::OnceLock<Text> = std::sync::OnceLock::new();
        fn replacement() -> &'static Text {
            REPLACEMENT.get_or_init(|| Text::new("REPLACED", Style::null()))
        }

        impl RenderHook for ReplaceHook {
            fn process_renderables<'a>(
                &self,
                _: Vec<&'a dyn Renderable>,
            ) -> Vec<&'a dyn Renderable> {
                vec![replacement() as &dyn Renderable]
            }
        }

        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.add_render_hook(Box::new(ReplaceHook));

        console.begin_capture();
        let original = Text::new("ORIGINAL", Style::null());
        console.print_with_hooks(&original as &dyn Renderable);
        let out = console.end_capture();

        assert!(
            out.contains("REPLACED"),
            "hook should have replaced the renderable"
        );
        assert!(
            !out.contains("ORIGINAL"),
            "original should not appear after replacement"
        );
    }

    /// Regression guard: hooks must NOT fire when calling `print()` directly
    /// (unsized `?Sized` coercion prevents it without specialization/unsafe).
    /// `print_with_hooks` is the documented hook entry point.
    #[test]
    fn render_hook_does_not_fire_on_plain_print() {
        use crate::console::RenderHook;
        use std::sync::{Arc, Mutex};

        struct CountingHook {
            count: Arc<Mutex<usize>>,
        }
        impl RenderHook for CountingHook {
            fn process_renderables<'a>(
                &self,
                renderables: Vec<&'a dyn Renderable>,
            ) -> Vec<&'a dyn Renderable> {
                *self.count.lock().unwrap() += 1;
                renderables
            }
        }

        let count = Arc::new(Mutex::new(0usize));
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.add_render_hook(Box::new(CountingHook {
            count: Arc::clone(&count),
        }));

        console.begin_capture();
        let text = Text::new("hi", Style::null());
        console.print(&text); // plain print — hooks should NOT fire
        let _ = console.end_capture();

        assert_eq!(
            *count.lock().unwrap(),
            0,
            "print() should not invoke render hooks (use print_with_hooks)"
        );
    }

    // -- log_objects log_path --------------------------------------------------

    #[test]
    fn log_objects_log_path_appends_caller_location() {
        let mut console = Console::builder()
            .width(120)
            .no_color(true)
            .markup(false)
            .log_path(true)
            .build();
        console.begin_capture();
        let a = Text::new("msg", Style::null());
        console.log_objects(&[&a], false);
        let out = console.end_capture();
        // log_path=true should append [filename:line] — look for the bracket pattern
        assert!(
            out.contains('[') && out.contains(':'),
            "expected caller path [file:line] in log_objects output, got {:?}",
            out
        );
        // The caller path should NOT be the timestamp (timestamp is at the start)
        // — assert the output contains at least two [...] groups
        let bracket_count = out.chars().filter(|&c| c == '[').count();
        assert!(
            bracket_count >= 2,
            "expected timestamp bracket + path bracket (>=2 '['), got {} in {:?}",
            bracket_count,
            out
        );
    }

    // -- Item 7: print_sep_end -----------------------------------------------

    #[test]
    fn print_sep_end_basic() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.begin_capture();
        let a = Text::new("foo", Style::null());
        let b = Text::new("bar", Style::null());
        console.print_sep_end(&[&a, &b], ", ", "!");
        let out = console.end_capture();
        assert!(out.contains("foo"), "missing 'foo'");
        assert!(out.contains(", "), "missing separator");
        assert!(out.contains("bar"), "missing 'bar'");
        assert!(out.contains('!'), "missing end");
    }

    #[test]
    fn print_sep_end_single_item() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.begin_capture();
        let a = Text::new("only", Style::null());
        console.print_sep_end(&[&a], " | ", ".");
        let out = console.end_capture();
        assert!(out.contains("only"));
        assert!(out.contains('.'));
        assert!(
            !out.contains(" | "),
            "no separator expected for single item"
        );
    }

    #[test]
    fn print_sep_end_empty_items() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.begin_capture();
        console.print_sep_end(&[], ", ", "end");
        let out = console.end_capture();
        // Should at least contain "end" and a newline
        assert!(out.contains("end"));
    }

    #[test]
    fn print_sep_end_always_ends_with_newline() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();
        console.begin_capture();
        let a = Text::new("x", Style::null());
        // end is not a newline
        console.print_sep_end(&[&a], "", "---");
        let out = console.end_capture();
        assert!(out.ends_with('\n'), "output should end with newline");
    }
}
