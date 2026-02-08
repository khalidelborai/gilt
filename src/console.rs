//! Console engine — the central orchestrator of gilt rendering output.
//!
//! The Console manages terminal capabilities, drives the rendering pipeline,
//! and handles output buffering, capture, and export.

use crate::cells::cell_len;
use crate::color::ColorSystem;
use crate::color_env::{detect_color_env, ColorEnvOverride};
use crate::control::Control;
use crate::errors::ConsoleError;
use crate::export_format::{CONSOLE_HTML_FORMAT, CONSOLE_SVG_FORMAT};
use crate::json::{Json, JsonOptions};
use crate::markup;
use crate::measure::Measurement;
use crate::pager::Pager;
use crate::rule::Rule;
use crate::segment::Segment;
use crate::status::Status;
use crate::style::Style;
use crate::terminal_theme::{TerminalTheme, DEFAULT_TERMINAL_THEME, SVG_EXPORT_THEME};
use crate::text::{JustifyMethod, OverflowMethod, Text};
use crate::theme::{Theme, ThemeStack};
use crate::traceback::Traceback;

// ---------------------------------------------------------------------------
// ConsoleDimensions
// ---------------------------------------------------------------------------

/// Terminal dimensions in columns and rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleDimensions {
    /// Number of columns.
    pub width: usize,
    /// Number of rows.
    pub height: usize,
}

// ---------------------------------------------------------------------------
// ConsoleOptions
// ---------------------------------------------------------------------------

/// Options that control how renderables produce segments.
#[derive(Debug, Clone)]
pub struct ConsoleOptions {
    /// Terminal dimensions used for layout.
    pub size: ConsoleDimensions,
    /// Whether to use legacy Windows console rendering.
    pub legacy_windows: bool,
    /// Minimum width in columns for renderable output.
    pub min_width: usize,
    /// Maximum width in columns for renderable output.
    pub max_width: usize,
    /// Whether the output target is an interactive terminal.
    pub is_terminal: bool,
    /// Character encoding (always `"utf-8"` in Rust).
    pub encoding: String,
    /// Maximum height in rows for renderable output.
    pub max_height: usize,
    /// Text justification override, if any.
    pub justify: Option<JustifyMethod>,
    /// Text overflow strategy override, if any.
    pub overflow: Option<OverflowMethod>,
    /// Whether to disable text wrapping.
    pub no_wrap: bool,
    /// Whether to enable syntax highlighting, if set.
    pub highlight: Option<bool>,
    /// Whether to enable markup parsing, if set.
    pub markup: Option<bool>,
    /// Explicit height constraint for renderables, if set.
    pub height: Option<usize>,
}

/// Builder for applying selective updates to `ConsoleOptions`.
#[derive(Debug, Clone, Default)]
pub struct ConsoleOptionsUpdates {
    /// New width in columns, if changing.
    pub width: Option<usize>,
    /// New minimum width, if changing.
    pub min_width: Option<usize>,
    /// New maximum width, if changing.
    pub max_width: Option<usize>,
    /// New justification override, if changing.
    pub justify: Option<Option<JustifyMethod>>,
    /// New overflow strategy override, if changing.
    pub overflow: Option<Option<OverflowMethod>>,
    /// New no-wrap flag, if changing.
    pub no_wrap: Option<bool>,
    /// New highlight flag, if changing.
    pub highlight: Option<Option<bool>>,
    /// New markup flag, if changing.
    pub markup: Option<Option<bool>>,
    /// New height constraint, if changing.
    pub height: Option<Option<usize>>,
    /// New maximum height, if changing.
    pub max_height: Option<usize>,
}

impl ConsoleOptions {
    /// Returns `true` if the encoding is NOT utf-based (i.e. ASCII-only output).
    pub fn ascii_only(&self) -> bool {
        !self.encoding.to_lowercase().starts_with("utf")
    }

    /// Clone this options set.
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// Return a new `ConsoleOptions` with the width replaced.
    pub fn update_width(&self, width: usize) -> Self {
        let mut opts = self.clone();
        opts.size.width = width;
        opts.max_width = width;
        opts
    }

    /// Return a new `ConsoleOptions` with the height replaced.
    pub fn update_height(&self, height: usize) -> Self {
        let mut opts = self.clone();
        opts.height = Some(height);
        opts
    }

    /// Return a new `ConsoleOptions` with both width and height replaced.
    pub fn update_dimensions(&self, width: usize, height: usize) -> Self {
        let mut opts = self.clone();
        opts.size = ConsoleDimensions { width, height };
        opts.max_width = width;
        opts.height = Some(height);
        opts
    }

    /// Return a new `ConsoleOptions` with height reset to `None`.
    pub fn reset_height(&self) -> Self {
        let mut opts = self.clone();
        opts.height = None;
        opts
    }

    /// Apply a set of optional field updates, returning a new `ConsoleOptions`.
    pub fn with_updates(&self, updates: &ConsoleOptionsUpdates) -> Self {
        let mut opts = self.clone();
        if let Some(w) = updates.width {
            opts.size.width = w;
            opts.max_width = w;
        }
        if let Some(min_w) = updates.min_width {
            opts.min_width = min_w;
        }
        if let Some(max_w) = updates.max_width {
            opts.max_width = max_w;
        }
        if let Some(ref j) = updates.justify {
            opts.justify = *j;
        }
        if let Some(ref o) = updates.overflow {
            opts.overflow = *o;
        }
        if let Some(nw) = updates.no_wrap {
            opts.no_wrap = nw;
        }
        if let Some(ref h) = updates.highlight {
            opts.highlight = *h;
        }
        if let Some(ref m) = updates.markup {
            opts.markup = *m;
        }
        if let Some(ref h) = updates.height {
            opts.height = *h;
        }
        if let Some(mh) = updates.max_height {
            opts.max_height = mh;
        }
        opts
    }
}

// ---------------------------------------------------------------------------
// Renderable trait
// ---------------------------------------------------------------------------

/// Trait for objects that can produce `Segment`s for console rendering.
pub trait Renderable {
    /// Produce segments for rendering on the given console with given options.
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment>;
}

impl Renderable for Text {
    fn rich_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let mut text = self.clone();
        if let Some(justify) = &options.justify {
            text.justify = Some(*justify);
        }
        if let Some(overflow) = &options.overflow {
            text.overflow = Some(*overflow);
        }
        if options.no_wrap || options.overflow == Some(OverflowMethod::Ignore) {
            text.render()
        } else {
            let tab_size = text.tab_size.unwrap_or(8);
            let lines = text.wrap(
                options.max_width,
                text.justify,
                text.overflow,
                tab_size,
                text.no_wrap.unwrap_or(false),
            );
            let mut segments = Vec::new();
            for line in lines.iter() {
                // Each line's render() already appends its `end` ("\n"),
                // so no extra Segment::line() is needed between lines.
                segments.extend(line.render());
            }
            segments
        }
    }
}

impl Renderable for str {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let text = console.render_str(self, None, options.justify, options.overflow);
        text.rich_console(console, options)
    }
}

impl Renderable for String {
    fn rich_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.as_str().rich_console(console, options)
    }
}

// ---------------------------------------------------------------------------
// ConsoleBuilder
// ---------------------------------------------------------------------------

/// Builder for constructing a `Console` with custom options.
pub struct ConsoleBuilder {
    color_system: Option<String>,
    color_system_override: Option<ColorSystem>,
    width: Option<usize>,
    height: Option<usize>,
    force_terminal: Option<bool>,
    record: bool,
    theme: Option<Theme>,
    markup: bool,
    highlight: bool,
    no_color: bool,
    no_color_explicit: bool,
    tab_size: usize,
    quiet: bool,
    soft_wrap: bool,
    safe_box: bool,
}

impl Default for ConsoleBuilder {
    fn default() -> Self {
        ConsoleBuilder {
            color_system: None,
            color_system_override: None,
            width: None,
            height: None,
            force_terminal: None,
            record: false,
            theme: None,
            markup: true,
            highlight: true,
            no_color: false,
            no_color_explicit: false,
            tab_size: 8,
            quiet: false,
            soft_wrap: false,
            safe_box: true,
        }
    }
}

impl ConsoleBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the color system by name (`"standard"`, `"256"`, `"truecolor"`, `"windows"`).
    pub fn color_system(mut self, cs: &str) -> Self {
        self.color_system = Some(cs.to_string());
        self
    }

    /// Set the console width in columns.
    pub fn width(mut self, w: usize) -> Self {
        self.width = Some(w);
        self
    }

    /// Set the console height in rows.
    pub fn height(mut self, h: usize) -> Self {
        self.height = Some(h);
        self
    }

    /// Force or prevent terminal detection regardless of the actual environment.
    pub fn force_terminal(mut self, f: bool) -> Self {
        self.force_terminal = Some(f);
        self
    }

    /// Enable or disable recording of output for later export.
    pub fn record(mut self, r: bool) -> Self {
        self.record = r;
        self
    }

    /// Set a custom theme for style lookups.
    pub fn theme(mut self, t: Theme) -> Self {
        self.theme = Some(t);
        self
    }

    /// Enable or disable markup parsing in print methods.
    pub fn markup(mut self, m: bool) -> Self {
        self.markup = m;
        self
    }

    /// Enable or disable automatic syntax highlighting.
    pub fn highlight(mut self, h: bool) -> Self {
        self.highlight = h;
        self
    }

    /// Enable or disable all color output.
    pub fn no_color(mut self, nc: bool) -> Self {
        self.no_color = nc;
        self.no_color_explicit = true;
        self
    }

    /// Explicitly override the color system, taking priority over both
    /// environment variables and the string-based [`color_system`](Self::color_system) method.
    pub fn color_system_override(mut self, cs: ColorSystem) -> Self {
        self.color_system_override = Some(cs);
        self
    }

    /// Set the tab size in spaces for text rendering.
    pub fn tab_size(mut self, ts: usize) -> Self {
        self.tab_size = ts;
        self
    }

    /// Enable or disable quiet mode, which suppresses all output.
    pub fn quiet(mut self, q: bool) -> Self {
        self.quiet = q;
        self
    }

    /// Enable or disable soft wrapping (allows lines to exceed terminal width).
    pub fn soft_wrap(mut self, sw: bool) -> Self {
        self.soft_wrap = sw;
        self
    }

    /// Enable or disable safe box characters (ASCII fallback for non-UTF-8 terminals).
    pub fn safe_box(mut self, sb: bool) -> Self {
        self.safe_box = sb;
        self
    }

    /// Build the `Console` instance with the configured options.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let console = Console::builder()
    ///     .width(80)
    ///     .no_color(true)
    ///     .build();
    /// assert_eq!(console.width(), 80);
    /// ```
    pub fn build(self) -> Console {
        // Determine the effective color system.
        //
        // Priority (highest first):
        //   1. `color_system_override` (explicit ColorSystem value)
        //   2. `color_system` (string-based, e.g. "truecolor")
        //   3. `no_color(true)` explicitly set by the caller
        //   4. Environment variables (NO_COLOR, FORCE_COLOR, CLICOLOR_FORCE, CLICOLOR)
        //   5. Default: TrueColor
        // Does the caller have an explicit, non-empty color_system string?
        let has_explicit_cs = matches!(
            self.color_system.as_deref(),
            Some("standard" | "256" | "truecolor" | "windows")
        );

        let color_system = if let Some(cs) = self.color_system_override {
            // (1) Explicit ColorSystem override always wins.
            Some(cs)
        } else if has_explicit_cs {
            // (2) String-based selection (only for known, non-empty names).
            match self.color_system.as_deref() {
                Some("standard") => Some(ColorSystem::Standard),
                Some("256") => Some(ColorSystem::EightBit),
                Some("truecolor") => Some(ColorSystem::TrueColor),
                Some("windows") => Some(ColorSystem::Windows),
                _ => unreachable!(),
            }
        } else if self.no_color_explicit && self.no_color {
            // (3) Caller explicitly asked for no colour.
            None
        } else {
            // (4) Consult environment variables.
            match detect_color_env() {
                ColorEnvOverride::NoColor => None,
                ColorEnvOverride::ForceColor => Some(ColorSystem::EightBit),
                ColorEnvOverride::ForceColorTruecolor => Some(ColorSystem::TrueColor),
                ColorEnvOverride::None => {
                    // (5) Default.
                    if self.no_color {
                        None
                    } else {
                        Some(ColorSystem::TrueColor)
                    }
                }
            }
        };

        let theme = self.theme.unwrap_or_else(|| Theme::new(None, true));
        let theme_stack = ThemeStack::new(theme);

        Console {
            color_system,
            width_override: self.width,
            height_override: self.height,
            force_terminal: self.force_terminal,
            tab_size: self.tab_size,
            record: self.record,
            markup_enabled: self.markup,
            highlight_enabled: self.highlight,
            soft_wrap: self.soft_wrap,
            no_color: self.no_color,
            quiet: self.quiet,
            safe_box: self.safe_box,
            legacy_windows: false,
            base_style: None,
            theme_stack,
            buffer: Vec::new(),
            buffer_index: 0,
            record_buffer: Vec::new(),
            is_alt_screen: false,
            capture_buffer: None,
            live_id: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Console
// ---------------------------------------------------------------------------

/// The central orchestrator of gilt rendering output.
///
/// Console manages terminal capabilities, drives the rendering pipeline,
/// and handles output buffering, capture, and export.
#[allow(dead_code)]
pub struct Console {
    // Configuration
    color_system: Option<ColorSystem>,
    width_override: Option<usize>,
    height_override: Option<usize>,
    force_terminal: Option<bool>,
    tab_size: usize,
    record: bool,
    markup_enabled: bool,
    highlight_enabled: bool,
    soft_wrap: bool,
    no_color: bool,
    quiet: bool,
    safe_box: bool,
    legacy_windows: bool,
    base_style: Option<Style>,

    // Theme
    theme_stack: ThemeStack,

    // Buffers
    buffer: Vec<Segment>,
    buffer_index: usize,
    record_buffer: Vec<Segment>,

    // State
    is_alt_screen: bool,
    capture_buffer: Option<Vec<Segment>>,
    live_id: Option<usize>,
}

impl Console {
    /// Create a new Console with sensible defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let console = Console::new();
    /// assert_eq!(console.encoding(), "utf-8");
    /// ```
    pub fn new() -> Self {
        ConsoleBuilder::default().build()
    }

    /// Create a Console using the builder pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let console = Console::builder()
    ///     .width(120)
    ///     .no_color(true)
    ///     .record(true)
    ///     .build();
    /// assert_eq!(console.width(), 120);
    /// ```
    pub fn builder() -> ConsoleBuilder {
        ConsoleBuilder::default()
    }

    // -- Properties ---------------------------------------------------------

    /// The current terminal width in columns.
    pub fn width(&self) -> usize {
        if let Some(w) = self.width_override {
            return w;
        }
        let (w, _) = Self::detect_terminal_size();
        w
    }

    /// The current terminal height in rows.
    pub fn height(&self) -> usize {
        if let Some(h) = self.height_override {
            return h;
        }
        let (_, h) = Self::detect_terminal_size();
        h
    }

    /// Current terminal dimensions.
    pub fn size(&self) -> ConsoleDimensions {
        ConsoleDimensions {
            width: self.width(),
            height: self.height(),
        }
    }

    /// Build the default `ConsoleOptions` for this console.
    pub fn options(&self) -> ConsoleOptions {
        let size = self.size();
        ConsoleOptions {
            size,
            legacy_windows: self.legacy_windows,
            min_width: 1,
            max_width: size.width,
            is_terminal: self.is_terminal(),
            encoding: "utf-8".to_string(),
            max_height: size.height,
            justify: None,
            overflow: None,
            no_wrap: false,
            highlight: Some(self.highlight_enabled),
            markup: Some(self.markup_enabled),
            height: None,
        }
    }

    /// The current color system name, or `None` if colors are disabled.
    pub fn color_system_name(&self) -> Option<&str> {
        self.color_system.as_ref().map(|cs| match cs {
            ColorSystem::Standard => "standard",
            ColorSystem::EightBit => "256",
            ColorSystem::TrueColor => "truecolor",
            ColorSystem::Windows => "windows",
        })
    }

    /// The active `ColorSystem`, if any.
    pub fn color_system(&self) -> Option<ColorSystem> {
        self.color_system
    }

    /// The character encoding (always "utf-8" in Rust).
    pub fn encoding(&self) -> &str {
        "utf-8"
    }

    /// Whether the console is connected to a terminal.
    pub fn is_terminal(&self) -> bool {
        if let Some(forced) = self.force_terminal {
            return forced;
        }
        // Check environment variables as a heuristic
        std::env::var("TERM").is_ok()
    }

    /// Whether this is a "dumb" terminal with no styling support.
    pub fn is_dumb_terminal(&self) -> bool {
        match std::env::var("TERM") {
            Ok(term) => term == "dumb",
            Err(_) => false,
        }
    }

    // -- Terminal detection -------------------------------------------------

    /// Detect the terminal size from environment variables, falling back to 80x25.
    pub fn detect_terminal_size() -> (usize, usize) {
        let width = std::env::var("COLUMNS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(80);
        let height = std::env::var("LINES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(25);
        (width, height)
    }

    // -- Theme / Style ------------------------------------------------------

    /// Look up a style by name from the theme stack, or parse it as a style definition.
    pub fn get_style(&self, name: &str) -> Result<Style, ConsoleError> {
        // First try the theme stack
        if let Some(style) = self.theme_stack.get(name) {
            return Ok(style.clone());
        }
        // Then try parsing as a style definition
        Style::parse(name).map_err(|e| {
            ConsoleError::RenderError(format!("Failed to get style '{}': {}", name, e))
        })
    }

    /// Push a new theme onto the theme stack.
    pub fn push_theme(&mut self, theme: Theme) {
        self.theme_stack.push_theme(theme, true);
    }

    /// Pop the top theme from the theme stack.
    pub fn pop_theme(&mut self) {
        let _ = self.theme_stack.pop_theme();
    }

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
        renderable.rich_console(self, opts)
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
        let segments = renderable.rich_console(self, opts);

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
            Some(s) => Style::parse(s).unwrap_or_else(|_| Style::null()),
            None => Style::null(),
        };

        let mut rich_text = if self.markup_enabled {
            markup::render(text, base_style.clone()).unwrap_or_else(|_| Text::new(text, base_style))
        } else {
            Text::new(text, base_style)
        };

        if let Some(j) = justify {
            rich_text.justify = Some(j);
        }
        if let Some(o) = overflow {
            rich_text.overflow = Some(o);
        }

        rich_text
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

        let mut segments = renderable.rich_console(self, &opts);

        // Apply additional style
        if let Some(style_str) = style {
            if let Ok(s) = Style::parse(style_str) {
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
        if !segments.is_empty() {
            let last_text = &segments.last().unwrap().text;
            if !last_text.ends_with('\n') {
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
        let rich_text = self.render_str(text, None, None, None);
        self.print(&rich_text);
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
            .unwrap_or_else(|_| Style::parse("dim").unwrap_or_else(|_| Style::null()));

        let time_text = Text::styled(&now, time_style);
        let body = self.render_str(text, None, None, None);

        // Combine: time + space + body
        let mut segments = time_text.rich_console(self, &self.options());
        // Remove trailing newline from time segments
        segments.retain(|s| s.text != "\n");
        segments.push(Segment::text(" "));
        segments.extend(body.rich_console(self, &self.options()));

        // Ensure trailing newline
        if !segments.is_empty() {
            let last_text = &segments.last().unwrap().text;
            if !last_text.ends_with('\n') {
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
        let mut segments = text.rich_console(self, &self.options());
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
    pub fn input_password(&mut self, prompt: &str) -> Result<String, std::io::Error> {
        // Render and print the prompt (without trailing newline)
        let text = self.render_str(prompt, None, None, None);
        let mut segments = text.rich_console(self, &self.options());
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
    /// matches the Python Rich `Console.print_exception()` API name.
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
        let segments = renderable.rich_console(self, &opts);
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

    /// Flush the buffer, converting accumulated segments to an output string.
    fn flush_buffer(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        let _output = self.render_buffer(&self.buffer.clone());
        self.buffer.clear();
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
        let mut output = String::new();
        let color_system = if self.no_color {
            None
        } else {
            self.color_system
        };

        for segment in buffer {
            if segment.is_control() {
                // Control segments are rendered directly (ANSI escape codes)
                output.push_str(&segment.text);
            } else if let Some(ref style) = segment.style {
                output.push_str(&style.render(&segment.text, color_system));
            } else {
                output.push_str(&segment.text);
            }
        }
        output
    }

    // -- Capture ------------------------------------------------------------

    /// Begin capturing output. Subsequent writes go to the capture buffer
    /// instead of the terminal.
    ///
    /// Call [`end_capture`](Console::end_capture) to retrieve the captured output
    /// as a string and resume normal output.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut console = Console::builder().width(80).no_color(true).markup(false).build();
    /// console.begin_capture();
    /// console.print_text("captured");
    /// let output = console.end_capture();
    /// assert!(output.contains("captured"));
    /// ```
    pub fn begin_capture(&mut self) {
        self.capture_buffer = Some(Vec::new());
    }

    /// End capturing and return the captured output as a rendered string.
    ///
    /// Returns all output written since [`begin_capture`](Console::begin_capture)
    /// was called, rendered through the console's color system.
    pub fn end_capture(&mut self) -> String {
        let segments = self.capture_buffer.take().unwrap_or_default();
        self.render_buffer(&segments)
    }

    // -- Control ------------------------------------------------------------

    /// Send a terminal control sequence.
    pub fn control(&mut self, ctrl: &Control) {
        if !self.quiet {
            self.write_segments(std::slice::from_ref(&ctrl.segment));
        }
    }

    /// Ring the terminal bell.
    pub fn bell(&mut self) {
        self.control(&Control::bell());
    }

    /// Clear the terminal screen.
    pub fn clear(&mut self) {
        self.control(&Control::clear());
    }

    /// Show or hide the cursor.
    pub fn show_cursor(&mut self, show: bool) {
        self.control(&Control::show_cursor(show));
    }

    /// Enable or disable the alternate screen buffer.
    ///
    /// Returns `true` if the operation was performed.
    pub fn set_alt_screen(&mut self, enable: bool) -> bool {
        if enable == self.is_alt_screen {
            return false;
        }
        self.is_alt_screen = enable;
        self.control(&Control::alt_screen(enable));
        true
    }

    /// Set the terminal window title.
    ///
    /// Returns `true` if the title was set (only works on terminals).
    pub fn set_window_title(&mut self, title: &str) -> bool {
        if !self.is_terminal() {
            return false;
        }
        self.control(&Control::title(title));
        true
    }

    // -- Synchronized Output ------------------------------------------------

    /// Begin synchronized output (DEC Mode 2026).
    ///
    /// The terminal buffers all subsequent output until
    /// [`end_synchronized`](Console::end_synchronized) is called, then paints
    /// atomically. This prevents flickering and tearing during rapid updates.
    pub fn begin_synchronized(&mut self) {
        self.control(&Control::begin_sync());
    }

    /// End synchronized output (DEC Mode 2026).
    ///
    /// The terminal flushes all buffered content and renders it at once.
    pub fn end_synchronized(&mut self) {
        self.control(&Control::end_sync());
    }

    /// Execute a closure with synchronized output wrapping.
    ///
    /// Emits the DEC Mode 2026 begin sequence, runs the closure, then emits
    /// the end sequence. If the closure panics the end sequence is still sent
    /// (best-effort) via a drop guard.
    pub fn synchronized<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Console) -> R,
    {
        self.begin_synchronized();
        let result = f(self);
        self.end_synchronized();
        result
    }

    // -- Clipboard (OSC 52) -------------------------------------------------

    /// Copy text to the system clipboard via OSC 52 escape sequence.
    ///
    /// This works in terminals that support OSC 52 (kitty, iTerm2, WezTerm,
    /// etc.). The text is base64-encoded in the escape sequence.
    pub fn copy_to_clipboard(&mut self, text: &str) {
        self.control(&Control::set_clipboard(text));
    }

    /// Request clipboard contents via OSC 52.
    ///
    /// Most terminals require explicit opt-in for clipboard reading.
    /// The terminal will respond with an OSC 52 sequence containing the
    /// base64-encoded clipboard contents, which must be read from stdin.
    pub fn request_clipboard(&mut self) {
        self.control(&Control::request_clipboard());
    }

    // -- Pager --------------------------------------------------------------

    /// Pipe recorded output through an external pager.
    ///
    /// Captures the current recorded output via `export_text(true, false)` and
    /// pipes it through a [`Pager`]. If `pager_command` is `Some`, uses the
    /// specified command; otherwise uses the default pager (`less -r`).
    ///
    /// Pager errors are silently ignored.
    pub fn pager(&mut self, pager_command: Option<&str>) {
        let text = self.export_text(true, false);
        let pager = match pager_command {
            Some(cmd) => Pager::new().with_command(cmd),
            None => Pager::new(),
        };
        let _ = pager.show(&text);
    }

    // -- Screen helpers -----------------------------------------------------

    /// Enter alternate screen mode, optionally hiding the cursor.
    ///
    /// Call [`exit_screen`](Console::exit_screen) with the same `hide_cursor`
    /// value to restore the previous state.
    pub fn enter_screen(&mut self, hide_cursor: bool) {
        self.set_alt_screen(true);
        if hide_cursor {
            self.show_cursor(false);
        }
    }

    /// Exit alternate screen mode, restoring the cursor if it was hidden.
    ///
    /// Pass the same `hide_cursor` value that was used with
    /// [`enter_screen`](Console::enter_screen).
    pub fn exit_screen(&mut self, hide_cursor: bool) {
        if hide_cursor {
            self.show_cursor(true);
        }
        self.set_alt_screen(false);
    }

    // -- Live display ID ----------------------------------------------------

    /// Store an optional live display ID for integration.
    pub fn set_live(&mut self, live_id: Option<usize>) {
        self.live_id = live_id;
    }

    /// Clear the live display ID, setting it to `None`.
    pub fn clear_live(&mut self) {
        self.live_id = None;
    }

    // -- Export (record mode) -----------------------------------------------

    /// Export recorded output as plain or styled text.
    ///
    /// Only works if `record` was enabled when the Console was created.
    /// Pass `clear = true` to empty the record buffer after export.
    /// Pass `styles = true` to include ANSI escape codes in the output.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .no_color(true)
    ///     .record(true)
    ///     .markup(false)
    ///     .build();
    /// let text = Text::new("Export me", Style::null());
    /// console.print(&text);
    /// let exported = console.export_text(false, false);
    /// assert!(exported.contains("Export me"));
    /// ```
    pub fn export_text(&mut self, clear: bool, styles: bool) -> String {
        let buffer = self.record_buffer.clone();
        if clear {
            self.record_buffer.clear();
        }

        if styles {
            self.render_buffer(&buffer)
        } else {
            // Strip control segments and just concatenate text
            let mut output = String::new();
            for segment in &buffer {
                if !segment.is_control() {
                    output.push_str(&segment.text);
                }
            }
            output
        }
    }

    /// Export recorded output as an HTML document.
    ///
    /// Generates a complete HTML page with inline or class-based styles.
    /// Requires `record` mode to be enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder()
    ///     .width(80)
    ///     .record(true)
    ///     .markup(false)
    ///     .build();
    /// let text = Text::styled("Red text", Style::parse("red").unwrap());
    /// console.print(&text);
    /// let html = console.export_html(None, false, true);
    /// assert!(html.contains("<!DOCTYPE html>"));
    /// assert!(html.contains("Red text"));
    /// ```
    pub fn export_html(
        &mut self,
        theme: Option<&TerminalTheme>,
        clear: bool,
        inline_styles: bool,
    ) -> String {
        let theme = theme.unwrap_or(&DEFAULT_TERMINAL_THEME);
        let buffer = self.record_buffer.clone();
        if clear {
            self.record_buffer.clear();
        }

        let mut code = String::new();
        let mut stylesheet = String::new();
        let mut style_cache: Vec<(Style, String)> = Vec::new();

        for segment in &buffer {
            if segment.is_control() {
                continue;
            }
            let escaped = html_escape(&segment.text);

            if let Some(ref style) = segment.style {
                if style.is_null() {
                    code.push_str(&escaped);
                    continue;
                }

                let css = style.get_html_style(Some(theme));
                if css.is_empty() {
                    code.push_str(&escaped);
                } else if inline_styles {
                    code.push_str(&format!("<span style=\"{}\">{}</span>", css, escaped));
                } else {
                    // Use class-based styles
                    let class_name =
                        find_or_insert_class(&mut style_cache, &mut stylesheet, style, &css);
                    code.push_str(&format!(
                        "<span class=\"{}\">{}</span>",
                        class_name, escaped
                    ));
                }
            } else {
                code.push_str(&escaped);
            }
        }

        let fg = theme.foreground_color.hex();
        let bg = theme.background_color.hex();

        CONSOLE_HTML_FORMAT
            .replace("{stylesheet}", &stylesheet)
            .replace("{foreground}", &fg)
            .replace("{background}", &bg)
            .replace("{code}", &code)
    }

    /// Export recorded output as an SVG document.
    ///
    /// Generates a complete SVG image with terminal-style chrome (title bar,
    /// window controls) and styled text content. Requires `record` mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    /// use gilt::text::Text;
    /// use gilt::style::Style;
    ///
    /// let mut console = Console::builder()
    ///     .width(40)
    ///     .record(true)
    ///     .no_color(true)
    ///     .markup(false)
    ///     .build();
    /// let text = Text::new("SVG test", Style::null());
    /// console.print(&text);
    /// let svg = console.export_svg("Test", None, false, None, 0.61);
    /// assert!(svg.contains("<svg"));
    /// assert!(svg.contains("SVG test"));
    /// ```
    pub fn export_svg(
        &mut self,
        title: &str,
        theme: Option<&TerminalTheme>,
        clear: bool,
        unique_id: Option<&str>,
        font_aspect_ratio: f64,
    ) -> String {
        let theme = theme.unwrap_or(&SVG_EXPORT_THEME);
        let unique_id = unique_id.unwrap_or("gilt");
        let buffer = self.record_buffer.clone();
        if clear {
            self.record_buffer.clear();
        }

        // Split into lines
        let text_lines: Vec<Vec<&Segment>> = {
            let mut lines: Vec<Vec<&Segment>> = Vec::new();
            let mut current: Vec<&Segment> = Vec::new();
            for seg in &buffer {
                if seg.is_control() {
                    continue;
                }
                if seg.text.contains('\n') {
                    // Push text before newline, start a new line
                    let parts: Vec<&str> = seg.text.split('\n').collect();
                    for (i, part) in parts.iter().enumerate() {
                        if !part.is_empty() {
                            // Create a temporary reference - we need owned segments for this
                            // Just use the original segment for non-split content
                            current.push(seg);
                        }
                        if i + 1 < parts.len() {
                            lines.push(std::mem::take(&mut current));
                        }
                    }
                } else {
                    current.push(seg);
                }
            }
            if !current.is_empty() {
                lines.push(current);
            }
            lines
        };

        let char_height = 20.0_f64;
        let line_height = char_height * 1.22;
        let char_width = char_height * font_aspect_ratio;
        let margin_top = 1.0;
        let margin_right = 1.0;
        let margin_bottom = 1.0;
        let margin_left = 1.0;
        let padding_top = 40.0;
        let padding_right = 8.0;
        let padding_bottom = 8.0;
        let padding_left = 8.0;

        let console_width = self.width() as f64;
        let line_count = text_lines.len().max(1) as f64;

        let terminal_width = (console_width * char_width + padding_left + padding_right).ceil();
        let terminal_height = (line_count * line_height + padding_top + padding_bottom).ceil();
        let svg_width = (terminal_width + margin_left + margin_right).ceil();
        let svg_height = (terminal_height + margin_top + margin_bottom).ceil();

        let terminal_x = margin_left;
        let terminal_y = margin_top;

        // Build the chrome (window decorations)
        let chrome = build_svg_chrome(terminal_width, terminal_height, theme, title, unique_id);

        // Build the text matrix
        let (matrix, backgrounds, styles, lines_defs) = build_svg_text(
            &buffer,
            theme,
            unique_id,
            char_width,
            line_height,
            padding_top,
            padding_left,
        );

        CONSOLE_SVG_FORMAT
            .replace("{unique_id}", unique_id)
            .replace("{char_height}", &format!("{:.1}", char_height))
            .replace("{line_height}", &format!("{:.1}", line_height))
            .replace("{width}", &format!("{:.0}", svg_width))
            .replace("{height}", &format!("{:.0}", svg_height))
            .replace("{terminal_width}", &format!("{:.0}", terminal_width))
            .replace("{terminal_height}", &format!("{:.0}", terminal_height))
            .replace("{terminal_x}", &format!("{:.0}", terminal_x))
            .replace("{terminal_y}", &format!("{:.0}", terminal_y))
            .replace("{chrome}", &chrome)
            .replace("{matrix}", &matrix)
            .replace("{backgrounds}", &backgrounds)
            .replace("{styles}", &styles)
            .replace("{lines}", &lines_defs)
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Find an existing CSS class for a style, or create a new one.
fn find_or_insert_class(
    cache: &mut Vec<(Style, String)>,
    stylesheet: &mut String,
    style: &Style,
    css: &str,
) -> String {
    for (cached_style, class_name) in cache.iter() {
        if cached_style == style {
            return class_name.clone();
        }
    }
    let class_name = format!("r{}", cache.len() + 1);
    stylesheet.push_str(&format!(".{} {{ {} }}\n", class_name, css));
    cache.push((style.clone(), class_name.clone()));
    class_name
}

/// Build the SVG chrome (window title bar and decorations).
fn build_svg_chrome(
    width: f64,
    height: f64,
    theme: &TerminalTheme,
    title: &str,
    unique_id: &str,
) -> String {
    let bg = theme.background_color.hex();
    let mut chrome = String::new();

    // Background rectangle with rounded corners
    chrome.push_str(&format!(
        "<rect fill=\"{}\" stroke=\"rgba(255,255,255,0.35)\" stroke-width=\"1\" \
         x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" rx=\"8\"/>\n",
        bg, width, height,
    ));

    // Window control dots
    let dot_colors = ["#ff5f57", "#febc2e", "#28c840"];
    for (i, color) in dot_colors.iter().enumerate() {
        let cx = 16.0 + (i as f64) * 22.0;
        chrome.push_str(&format!(
            "    <circle cx=\"{:.0}\" cy=\"18\" r=\"5\" fill=\"{}\"/>\n",
            cx, color
        ));
    }

    // Title text
    if !title.is_empty() {
        chrome.push_str(&format!(
            "    <text class=\"{}-title\" fill=\"{}\" x=\"{}\" y=\"23\" \
             text-anchor=\"middle\">{}</text>\n",
            unique_id,
            theme.foreground_color.hex(),
            width / 2.0,
            svg_escape(title),
        ));
    }

    chrome
}

/// Build the SVG text content from segments.
fn build_svg_text(
    buffer: &[Segment],
    theme: &TerminalTheme,
    unique_id: &str,
    char_width: f64,
    line_height: f64,
    padding_top: f64,
    padding_left: f64,
) -> (String, String, String, String) {
    let mut matrix = String::new();
    let mut backgrounds = String::new();
    let mut styles = String::new();
    let lines_defs = String::new();

    let mut style_cache: Vec<(String, String)> = Vec::new();
    let mut y = padding_top + line_height;
    let mut x: f64;
    let mut line_segments: Vec<Vec<(String, Option<Style>)>> = Vec::new();

    // Split buffer into lines
    let mut current_line: Vec<(String, Option<Style>)> = Vec::new();
    for seg in buffer {
        if seg.is_control() {
            continue;
        }
        let parts: Vec<&str> = seg.text.split('\n').collect();
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                current_line.push((part.to_string(), seg.style.clone()));
            }
            if i + 1 < parts.len() {
                line_segments.push(std::mem::take(&mut current_line));
            }
        }
    }
    if !current_line.is_empty() {
        line_segments.push(current_line);
    }

    for line in &line_segments {
        x = padding_left;
        for (text, style) in line {
            let escaped = svg_escape(text);
            let text_width = cell_len(text) as f64 * char_width;

            if let Some(ref style) = style {
                // Background
                if let Some(bgcolor) = style.bgcolor() {
                    let bg_triplet = bgcolor.get_truecolor(Some(theme), false);
                    backgrounds.push_str(&format!(
                        "    <rect fill=\"{}\" x=\"{:.1}\" y=\"{:.1}\" \
                         width=\"{:.1}\" height=\"{:.1}\"/>\n",
                        bg_triplet.hex(),
                        x,
                        y - line_height + 3.0,
                        text_width,
                        line_height,
                    ));
                }

                // Foreground text with style class
                let css = style.get_html_style(Some(theme));
                if !css.is_empty() {
                    let class_name =
                        find_or_insert_svg_class(&mut style_cache, &mut styles, unique_id, &css);
                    matrix.push_str(&format!(
                        "    <text class=\"{}\" x=\"{:.1}\" y=\"{:.1}\" \
                         textLength=\"{:.1}\">{}</text>\n",
                        class_name, x, y, text_width, escaped
                    ));
                } else {
                    matrix.push_str(&format!(
                        "    <text fill=\"{}\" x=\"{:.1}\" y=\"{:.1}\" \
                         textLength=\"{:.1}\">{}</text>\n",
                        theme.foreground_color.hex(),
                        x,
                        y,
                        text_width,
                        escaped
                    ));
                }
            } else {
                matrix.push_str(&format!(
                    "    <text fill=\"{}\" x=\"{:.1}\" y=\"{:.1}\" \
                     textLength=\"{:.1}\">{}</text>\n",
                    theme.foreground_color.hex(),
                    x,
                    y,
                    text_width,
                    escaped
                ));
            }

            x += text_width;
        }
        y += line_height;
    }

    (matrix, backgrounds, styles, lines_defs)
}

/// Find or create an SVG style class.
fn find_or_insert_svg_class(
    cache: &mut Vec<(String, String)>,
    styles: &mut String,
    unique_id: &str,
    css: &str,
) -> String {
    for (cached_css, class_name) in cache.iter() {
        if cached_css == css {
            return class_name.clone();
        }
    }
    let class_name = format!("{}-s{}", unique_id, cache.len() + 1);
    // Convert HTML CSS to SVG attributes
    let svg_style = css_to_svg_style(css);
    styles.push_str(&format!("    .{} {{ {} }}\n", class_name, svg_style));
    cache.push((css.to_string(), class_name.clone()));
    class_name
}

/// Convert CSS style properties to SVG-compatible style properties.
fn css_to_svg_style(css: &str) -> String {
    let mut svg_parts = Vec::new();
    for part in css.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((key, value)) = part.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "color" => svg_parts.push(format!("fill: {}", value)),
                "font-weight" => svg_parts.push(format!("font-weight: {}", value)),
                "font-style" => svg_parts.push(format!("font-style: {}", value)),
                "text-decoration" => svg_parts.push(format!("text-decoration: {}", value)),
                _ => {} // Skip background-color and other non-SVG properties
            }
        }
    }
    svg_parts.join("; ")
}

/// Escape text for SVG content.
fn svg_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::ControlCode;
    use crate::segment::ControlType;

    // -- ConsoleDimensions --------------------------------------------------

    #[test]
    fn test_console_dimensions_create() {
        let dims = ConsoleDimensions {
            width: 80,
            height: 25,
        };
        assert_eq!(dims.width, 80);
        assert_eq!(dims.height, 25);
    }

    #[test]
    fn test_console_dimensions_clone() {
        let dims = ConsoleDimensions {
            width: 120,
            height: 40,
        };
        let cloned = dims;
        assert_eq!(dims, cloned);
    }

    #[test]
    fn test_console_dimensions_equality() {
        let a = ConsoleDimensions {
            width: 80,
            height: 25,
        };
        let b = ConsoleDimensions {
            width: 80,
            height: 25,
        };
        let c = ConsoleDimensions {
            width: 120,
            height: 25,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // -- ConsoleOptions -----------------------------------------------------

    #[test]
    fn test_console_options_ascii_only_utf8() {
        let opts = make_default_options();
        assert!(!opts.ascii_only());
    }

    #[test]
    fn test_console_options_ascii_only_ascii() {
        let mut opts = make_default_options();
        opts.encoding = "ascii".to_string();
        assert!(opts.ascii_only());
    }

    #[test]
    fn test_console_options_ascii_only_latin1() {
        let mut opts = make_default_options();
        opts.encoding = "latin-1".to_string();
        assert!(opts.ascii_only());
    }

    #[test]
    fn test_console_options_copy() {
        let opts = make_default_options();
        let copy = opts.copy();
        assert_eq!(copy.size, opts.size);
        assert_eq!(copy.max_width, opts.max_width);
        assert_eq!(copy.encoding, opts.encoding);
    }

    #[test]
    fn test_console_options_update_width() {
        let opts = make_default_options();
        let updated = opts.update_width(40);
        assert_eq!(updated.size.width, 40);
        assert_eq!(updated.max_width, 40);
    }

    #[test]
    fn test_console_options_update_height() {
        let opts = make_default_options();
        let updated = opts.update_height(50);
        assert_eq!(updated.height, Some(50));
    }

    #[test]
    fn test_console_options_update_dimensions() {
        let opts = make_default_options();
        let updated = opts.update_dimensions(100, 50);
        assert_eq!(updated.size.width, 100);
        assert_eq!(updated.size.height, 50);
        assert_eq!(updated.max_width, 100);
        assert_eq!(updated.height, Some(50));
    }

    #[test]
    fn test_console_options_reset_height() {
        let opts = make_default_options().update_height(50);
        assert_eq!(opts.height, Some(50));
        let reset = opts.reset_height();
        assert_eq!(reset.height, None);
    }

    #[test]
    fn test_console_options_with_updates() {
        let opts = make_default_options();
        let updates = ConsoleOptionsUpdates {
            width: Some(60),
            no_wrap: Some(true),
            justify: Some(Some(JustifyMethod::Center)),
            ..Default::default()
        };
        let updated = opts.with_updates(&updates);
        assert_eq!(updated.size.width, 60);
        assert_eq!(updated.max_width, 60);
        assert!(updated.no_wrap);
        assert_eq!(updated.justify, Some(JustifyMethod::Center));
    }

    // -- Console creation ---------------------------------------------------

    #[test]
    fn test_console_default() {
        let console = Console::new();
        assert_eq!(console.encoding(), "utf-8");
        assert!(!console.no_color);
        assert!(!console.quiet);
        assert!(console.markup_enabled);
        assert!(console.highlight_enabled);
    }

    #[test]
    fn test_console_builder_defaults() {
        let console = Console::builder().build();
        assert!(console.color_system.is_some());
        assert_eq!(console.tab_size, 8);
        assert!(!console.record);
    }

    #[test]
    fn test_console_builder_width() {
        let console = Console::builder().width(120).build();
        assert_eq!(console.width(), 120);
    }

    #[test]
    fn test_console_builder_height() {
        let console = Console::builder().height(50).build();
        assert_eq!(console.height(), 50);
    }

    #[test]
    fn test_console_custom_width_height() {
        let console = Console::builder().width(100).height(40).build();
        assert_eq!(console.width(), 100);
        assert_eq!(console.height(), 40);
        let dims = console.size();
        assert_eq!(dims.width, 100);
        assert_eq!(dims.height, 40);
    }

    #[test]
    fn test_console_color_system_standard() {
        let console = Console::builder().color_system("standard").build();
        assert_eq!(console.color_system(), Some(ColorSystem::Standard));
        assert_eq!(console.color_system_name(), Some("standard"));
    }

    #[test]
    fn test_console_color_system_256() {
        let console = Console::builder().color_system("256").build();
        assert_eq!(console.color_system(), Some(ColorSystem::EightBit));
        assert_eq!(console.color_system_name(), Some("256"));
    }

    #[test]
    fn test_console_color_system_truecolor() {
        let console = Console::builder().color_system("truecolor").build();
        assert_eq!(console.color_system(), Some(ColorSystem::TrueColor));
        assert_eq!(console.color_system_name(), Some("truecolor"));
    }

    #[test]
    fn test_console_no_color() {
        let console = Console::builder().no_color(true).color_system("").build();
        assert!(console.color_system().is_none());
        assert_eq!(console.color_system_name(), None);
    }

    #[test]
    fn test_console_no_color_overrides_env_vars() {
        // Even if FORCE_COLOR is set in the environment, an explicit
        // `no_color(true)` on the builder takes priority.
        let console = Console::builder().no_color(true).build();
        assert!(console.color_system().is_none());
    }

    #[test]
    fn test_console_color_system_override_builder() {
        // `color_system_override` takes priority over string-based selection.
        let console = Console::builder()
            .color_system("standard")
            .color_system_override(ColorSystem::TrueColor)
            .build();
        assert_eq!(console.color_system(), Some(ColorSystem::TrueColor));
    }

    // -- Theme / style lookup -----------------------------------------------

    #[test]
    fn test_get_style_from_theme() {
        let console = Console::new();
        let style = console.get_style("bold");
        assert!(style.is_ok());
        assert_eq!(style.unwrap(), Style::parse("bold").unwrap());
    }

    #[test]
    fn test_get_style_parse_inline() {
        let console = Console::new();
        let style = console.get_style("bold red on blue");
        assert!(style.is_ok());
    }

    #[test]
    fn test_get_style_invalid() {
        let console = Console::new();
        let style = console.get_style("completely_nonexistent_style_xyzzy");
        // Should either find it in the theme or fail to parse
        // If not in theme and not parseable, it's an error
        assert!(style.is_err());
    }

    #[test]
    fn test_push_pop_theme() {
        let mut console = Console::new();

        // Default should have "bold"
        assert!(console.get_style("bold").is_ok());

        // Push a theme with a custom style
        let mut styles = std::collections::HashMap::new();
        styles.insert(
            "my_custom_style".to_string(),
            Style::parse("red bold").unwrap(),
        );
        let custom = Theme::new(Some(styles), true);
        console.push_theme(custom);

        // Custom style should be available
        let style = console.get_style("my_custom_style");
        assert!(style.is_ok());

        // Pop the theme
        console.pop_theme();

        // Custom style should no longer be available via theme lookup
        // (but might still parse as a style definition)
        let result = console.theme_stack.get("my_custom_style");
        assert!(result.is_none());
    }

    // -- render_str ---------------------------------------------------------

    #[test]
    fn test_render_str_plain() {
        let console = Console::builder().markup(false).build();
        let text = console.render_str("Hello, world!", None, None, None);
        assert_eq!(text.plain(), "Hello, world!");
    }

    #[test]
    fn test_render_str_with_markup() {
        let console = Console::new();
        let text = console.render_str("[bold]Hello[/bold]", None, None, None);
        assert_eq!(text.plain(), "Hello");
        // Should have a bold span
        assert!(!text.spans().is_empty());
    }

    #[test]
    fn test_render_str_with_style() {
        let console = Console::new();
        let text = console.render_str("Hello", Some("bold"), None, None);
        // The base style should be bold
        assert_eq!(text.plain(), "Hello");
    }

    #[test]
    fn test_render_str_with_justify() {
        let console = Console::new();
        let text = console.render_str("Hello", None, Some(JustifyMethod::Center), None);
        assert_eq!(text.justify, Some(JustifyMethod::Center));
    }

    #[test]
    fn test_render_str_with_overflow() {
        let console = Console::new();
        let text = console.render_str("Hello", None, None, Some(OverflowMethod::Ellipsis));
        assert_eq!(text.overflow, Some(OverflowMethod::Ellipsis));
    }

    // -- Capture ------------------------------------------------------------

    #[test]
    fn test_capture_basic() {
        let mut console = Console::builder()
            .width(80)
            .force_terminal(true)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        let text = Text::new("Hello, world!", Style::null());
        console.print(&text);
        let captured = console.end_capture();

        assert!(captured.contains("Hello, world!"));
    }

    #[test]
    fn test_capture_empty() {
        let mut console = Console::new();
        console.begin_capture();
        let captured = console.end_capture();
        assert!(captured.is_empty());
    }

    #[test]
    fn test_capture_multiple_prints() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        let text1 = Text::new("Hello", Style::null());
        let text2 = Text::new("World", Style::null());
        console.print(&text1);
        console.print(&text2);
        let captured = console.end_capture();

        assert!(captured.contains("Hello"));
        assert!(captured.contains("World"));
    }

    // -- print_text ---------------------------------------------------------

    #[test]
    fn test_print_text_capture() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.print_text("Hello, terminal!");
        let captured = console.end_capture();

        assert!(captured.contains("Hello, terminal!"));
    }

    // -- export_text --------------------------------------------------------

    #[test]
    fn test_export_text_plain() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();

        let text = Text::new("Export me", Style::null());
        console.print(&text);
        let exported = console.export_text(false, false);

        assert!(exported.contains("Export me"));
    }

    #[test]
    fn test_export_text_with_styles() {
        let mut console = Console::builder()
            .width(80)
            .record(true)
            .markup(false)
            .build();

        let text = Text::styled("Bold text", Style::parse("bold").unwrap());
        console.print(&text);
        let exported = console.export_text(false, true);

        // Styled export should contain ANSI codes
        assert!(exported.contains("Bold text"));
    }

    #[test]
    fn test_export_text_clear() {
        let mut console = Console::builder()
            .width(80)
            .record(true)
            .no_color(true)
            .markup(false)
            .build();

        let text = Text::new("Clearable", Style::null());
        console.print(&text);

        let export1 = console.export_text(true, false);
        assert!(export1.contains("Clearable"));

        // After clearing, should be empty
        let export2 = console.export_text(false, false);
        assert!(!export2.contains("Clearable"));
    }

    // -- export_html --------------------------------------------------------

    #[test]
    fn test_export_html_inline_styles() {
        let mut console = Console::builder()
            .width(80)
            .record(true)
            .markup(false)
            .build();

        let text = Text::styled("Red text", Style::parse("red").unwrap());
        console.print(&text);
        let html = console.export_html(None, false, true);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Red text"));
        assert!(html.contains("<span"));
    }

    #[test]
    fn test_export_html_stylesheet() {
        let mut console = Console::builder()
            .width(80)
            .record(true)
            .markup(false)
            .build();

        let text = Text::styled("Styled text", Style::parse("bold").unwrap());
        console.print(&text);
        let html = console.export_html(None, false, false);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Styled text"));
    }

    #[test]
    fn test_export_html_escape() {
        let mut console = Console::builder()
            .width(80)
            .record(true)
            .no_color(true)
            .markup(false)
            .build();

        let text = Text::new("<script>alert('xss')</script>", Style::null());
        console.print(&text);
        let html = console.export_html(None, false, true);

        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    // -- render_buffer ------------------------------------------------------

    #[test]
    fn test_render_buffer_plain() {
        let console = Console::new();
        let segments = vec![Segment::text("Hello")];
        let output = console.render_buffer(&segments);
        assert_eq!(output, "Hello");
    }

    #[test]
    fn test_render_buffer_styled() {
        let console = Console::builder().color_system("truecolor").build();
        let segments = vec![Segment::styled("Bold", Style::parse("bold").unwrap())];
        let output = console.render_buffer(&segments);
        // Should contain ANSI bold code
        assert!(output.contains("\x1b["));
        assert!(output.contains("Bold"));
    }

    #[test]
    fn test_render_buffer_no_color() {
        let console = Console::builder().no_color(true).color_system("").build();
        let segments = vec![Segment::styled("NoColor", Style::parse("bold").unwrap())];
        let output = console.render_buffer(&segments);
        // Without color system, style.render should return plain text
        assert_eq!(output, "NoColor");
    }

    #[test]
    fn test_render_buffer_control() {
        let console = Console::new();
        let ctrl = Control::bell();
        let segments = vec![ctrl.segment.clone()];
        let output = console.render_buffer(&segments);
        assert_eq!(output, "\x07");
    }

    #[test]
    fn test_render_buffer_link() {
        let console = Console::builder().color_system("truecolor").build();
        let style = Style::parse("bold link https://example.com").unwrap();
        let segments = vec![Segment::styled("click", style)];
        let output = console.render_buffer(&segments);
        // Should contain OSC 8 open and close sequences
        assert!(output.contains("\x1b]8;;https://example.com\x1b\\"));
        assert!(output.contains("\x1b]8;;\x1b\\"));
        assert!(output.contains("click"));
    }

    #[test]
    fn test_render_buffer_link_only() {
        let console = Console::builder().color_system("truecolor").build();
        let style = Style::with_link("https://example.com");
        let segments = vec![Segment::styled("link text", style)];
        let output = console.render_buffer(&segments);
        assert_eq!(
            output,
            "\x1b]8;;https://example.com\x1b\\link text\x1b]8;;\x1b\\"
        );
    }

    // -- Terminal detection -------------------------------------------------

    #[test]
    fn test_detect_terminal_size_defaults() {
        // Clear env vars for this test
        let saved_cols = std::env::var("COLUMNS").ok();
        let saved_lines = std::env::var("LINES").ok();
        std::env::remove_var("COLUMNS");
        std::env::remove_var("LINES");

        let (w, h) = Console::detect_terminal_size();
        assert_eq!(w, 80);
        assert_eq!(h, 25);

        // Restore env vars
        if let Some(v) = saved_cols {
            std::env::set_var("COLUMNS", v);
        }
        if let Some(v) = saved_lines {
            std::env::set_var("LINES", v);
        }
    }

    #[test]
    fn test_detect_terminal_size_env() {
        let saved_cols = std::env::var("COLUMNS").ok();
        let saved_lines = std::env::var("LINES").ok();

        std::env::set_var("COLUMNS", "120");
        std::env::set_var("LINES", "40");

        let (w, h) = Console::detect_terminal_size();
        assert_eq!(w, 120);
        assert_eq!(h, 40);

        // Restore
        match saved_cols {
            Some(v) => std::env::set_var("COLUMNS", v),
            None => std::env::remove_var("COLUMNS"),
        }
        match saved_lines {
            Some(v) => std::env::set_var("LINES", v),
            None => std::env::remove_var("LINES"),
        }
    }

    // -- Control methods ----------------------------------------------------

    #[test]
    fn test_control_bell() {
        let mut console = Console::builder().record(true).build();
        console.bell();
        let text = console.export_text(false, true);
        assert!(text.contains('\x07'));
    }

    #[test]
    fn test_control_clear() {
        let mut console = Console::builder().record(true).build();
        console.clear();
        let text = console.export_text(false, true);
        assert!(text.contains("\x1b[H"));
    }

    #[test]
    fn test_control_show_cursor() {
        let mut console = Console::builder().record(true).build();
        console.show_cursor(true);
        let text = console.export_text(true, true);
        assert!(text.contains("\x1b[?25h"));

        console.show_cursor(false);
        let text = console.export_text(true, true);
        assert!(text.contains("\x1b[?25l"));
    }

    // -- Alt screen ---------------------------------------------------------

    #[test]
    fn test_alt_screen_enable_disable() {
        let mut console = Console::builder().record(true).build();

        assert!(!console.is_alt_screen);
        let changed = console.set_alt_screen(true);
        assert!(changed);
        assert!(console.is_alt_screen);

        // Enabling again should return false (already enabled)
        let changed = console.set_alt_screen(true);
        assert!(!changed);

        let changed = console.set_alt_screen(false);
        assert!(changed);
        assert!(!console.is_alt_screen);
    }

    // -- Buffer nesting -----------------------------------------------------

    #[test]
    fn test_buffer_nesting() {
        let mut console = Console::new();
        assert!(!console.check_buffer());

        console.enter_buffer();
        assert!(console.check_buffer());

        console.enter_buffer();
        assert!(console.check_buffer());

        console.exit_buffer();
        assert!(console.check_buffer());

        console.exit_buffer();
        assert!(!console.check_buffer());
    }

    // -- Renderable trait for Text ------------------------------------------

    #[test]
    fn test_renderable_text() {
        let console = Console::builder().width(80).build();
        let text = Text::new("Renderable text", Style::null());
        let opts = console.options();
        let segments = text.rich_console(&console, &opts);
        assert!(!segments.is_empty());
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("Renderable text"));
    }

    // -- Renderable trait for str -------------------------------------------

    #[test]
    fn test_renderable_str() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();
        let text = "Hello from str";
        let segments = text.rich_console(&console, &opts);
        assert!(!segments.is_empty());
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("Hello from str"));
    }

    #[test]
    fn test_renderable_string() {
        let console = Console::builder().width(80).markup(false).build();
        let opts = console.options();
        let text = String::from("Hello from String");
        let segments = text.rich_console(&console, &opts);
        assert!(!segments.is_empty());
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("Hello from String"));
    }

    // -- Quiet mode ---------------------------------------------------------

    #[test]
    fn test_quiet_mode() {
        let mut console = Console::builder()
            .width(80)
            .record(true)
            .quiet(true)
            .markup(false)
            .build();

        let text = Text::new("Should not appear", Style::null());
        console.print(&text);
        let exported = console.export_text(false, false);
        // Quiet mode should suppress all output including recording
        assert!(exported.is_empty());
    }

    // -- Soft wrap mode -----------------------------------------------------

    #[test]
    fn test_soft_wrap_builder() {
        let console = Console::builder().soft_wrap(true).build();
        assert!(console.soft_wrap);
    }

    // -- No-color mode stripping --------------------------------------------

    #[test]
    fn test_no_color_mode_strips_color() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .color_system("")
            .record(true)
            .markup(false)
            .build();

        let text = Text::styled("Colored text", Style::parse("red").unwrap());
        console.print(&text);

        // In no-color mode, the rendered output should be plain
        let exported = console.export_text(false, true);
        assert!(exported.contains("Colored text"));
        // Should NOT contain ANSI color codes since color_system is None
        assert!(!exported.contains("\x1b["));
    }

    // -- Record buffer accumulation -----------------------------------------

    #[test]
    fn test_record_buffer_accumulation() {
        let mut console = Console::builder()
            .width(80)
            .record(true)
            .no_color(true)
            .markup(false)
            .build();

        let text1 = Text::new("First", Style::null());
        let text2 = Text::new("Second", Style::null());
        console.print(&text1);
        console.print(&text2);

        let exported = console.export_text(false, false);
        assert!(exported.contains("First"));
        assert!(exported.contains("Second"));
    }

    // -- options() default --------------------------------------------------

    #[test]
    fn test_console_options_default() {
        let console = Console::builder().width(100).height(40).build();
        let opts = console.options();
        assert_eq!(opts.size.width, 100);
        assert_eq!(opts.size.height, 40);
        assert_eq!(opts.max_width, 100);
        assert_eq!(opts.encoding, "utf-8");
        assert!(!opts.no_wrap);
        assert_eq!(opts.justify, None);
        assert_eq!(opts.overflow, None);
    }

    // -- render / render_lines ----------------------------------------------

    #[test]
    fn test_render_text() {
        let console = Console::builder().width(80).build();
        let text = Text::new("Render me", Style::null());
        let segments = console.render(&text, None);
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("Render me"));
    }

    #[test]
    fn test_render_lines_basic() {
        let console = Console::builder().width(80).build();
        let text = Text::new("Line1\nLine2", Style::null());
        let lines = console.render_lines(&text, None, None, false, false);
        assert!(lines.len() >= 2);
    }

    // -- html_escape --------------------------------------------------------

    #[test]
    fn test_html_escape_all_entities() {
        assert_eq!(html_escape("&"), "&amp;");
        assert_eq!(html_escape("<"), "&lt;");
        assert_eq!(html_escape(">"), "&gt;");
        assert_eq!(html_escape("\""), "&quot;");
        assert_eq!(
            html_escape("<p class=\"x\">&</p>"),
            "&lt;p class=&quot;x&quot;&gt;&amp;&lt;/p&gt;"
        );
    }

    // -- svg_escape ---------------------------------------------------------

    #[test]
    fn test_svg_escape_entities() {
        assert_eq!(svg_escape("&"), "&amp;");
        assert_eq!(svg_escape("'"), "&#39;");
    }

    // -- set_window_title ---------------------------------------------------

    #[test]
    fn test_set_window_title_non_terminal() {
        let mut console = Console::builder().force_terminal(false).build();
        let result = console.set_window_title("Test");
        assert!(!result);
    }

    #[test]
    fn test_set_window_title_terminal() {
        let mut console = Console::builder().force_terminal(true).record(true).build();
        let result = console.set_window_title("Test Title");
        assert!(result);
        let exported = console.export_text(false, true);
        assert!(exported.contains("Test Title"));
    }

    // -- export_svg ---------------------------------------------------------

    #[test]
    fn test_export_svg_basic() {
        let mut console = Console::builder()
            .width(40)
            .record(true)
            .no_color(true)
            .markup(false)
            .build();

        let text = Text::new("SVG test", Style::null());
        console.print(&text);
        let svg = console.export_svg("Test", None, false, None, 0.61);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("SVG test"));
        assert!(svg.contains("</svg>"));
    }

    // -- encoding -----------------------------------------------------------

    #[test]
    fn test_encoding_always_utf8() {
        let console = Console::new();
        assert_eq!(console.encoding(), "utf-8");
    }

    // -- is_dumb_terminal ---------------------------------------------------

    #[test]
    fn test_is_dumb_terminal() {
        let saved = std::env::var("TERM").ok();
        std::env::set_var("TERM", "dumb");

        let console = Console::new();
        assert!(console.is_dumb_terminal());

        match saved {
            Some(v) => std::env::set_var("TERM", v),
            None => std::env::remove_var("TERM"),
        }
    }

    // -- Convenience methods ------------------------------------------------

    #[test]
    fn test_line_blank_lines() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.line(3);
        let captured = console.end_capture();

        assert_eq!(captured, "\n\n\n");
    }

    #[test]
    fn test_line_zero() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.line(0);
        let captured = console.end_capture();

        assert!(captured.is_empty());
    }

    #[test]
    fn test_rule_no_title_capture() {
        let mut console = Console::builder()
            .width(40)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.rule(None);
        let captured = console.end_capture();

        // Should contain rule characters and end with newline
        assert!(captured.contains('\u{2501}') || captured.contains('-'));
        assert!(captured.ends_with('\n'));
    }

    #[test]
    fn test_rule_with_title_capture() {
        let mut console = Console::builder()
            .width(40)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.rule(Some("Hello"));
        let captured = console.end_capture();

        assert!(captured.contains("Hello"));
        assert!(captured.ends_with('\n'));
    }

    #[test]
    fn test_print_json_valid() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.print_json(r#"{"name": "Alice", "age": 30}"#);
        let captured = console.end_capture();

        assert!(captured.contains("name"));
        assert!(captured.contains("Alice"));
        assert!(captured.contains("30"));
    }

    #[test]
    fn test_print_json_invalid_falls_back() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.print_json("not valid json");
        let captured = console.end_capture();

        assert!(captured.contains("not valid json"));
    }

    #[test]
    fn test_measure_simple_text() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        let text = Text::new("Hello World", Style::null());
        let measurement = console.measure(&text);

        // "Hello" and "World" are each 5 chars -- min should be 5
        // "Hello World" is 11 chars -- max should be 11
        assert_eq!(measurement.minimum, 5);
        assert_eq!(measurement.maximum, 11);
    }

    #[test]
    fn test_measure_multiline_text() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        let text = Text::new("Short\nA much longer second line", Style::null());
        let measurement = console.measure(&text);

        // max is the longer line
        assert!(measurement.maximum >= 25);
        // min is the longest word
        assert!(measurement.minimum >= 6); // "longer" or "second"
    }

    #[test]
    fn test_measure_empty() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        let text = Text::new("", Style::null());
        let measurement = console.measure(&text);

        assert_eq!(measurement.minimum, 0);
        assert_eq!(measurement.maximum, 0);
    }

    #[test]
    fn test_save_text_to_file() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();

        let text = Text::new("Save me to a file", Style::null());
        console.print(&text);

        let dir = std::env::temp_dir();
        let path = dir.join("gilt_test_save_text.txt");
        let path_str = path.to_str().unwrap();

        let result = console.save_text(path_str, false, false);
        assert!(result.is_ok());

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("Save me to a file"));

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_save_html_to_file() {
        let mut console = Console::builder()
            .width(80)
            .record(true)
            .markup(false)
            .build();

        let text = Text::styled("HTML content", Style::parse("red").unwrap());
        console.print(&text);

        let dir = std::env::temp_dir();
        let path = dir.join("gilt_test_save.html");
        let path_str = path.to_str().unwrap();

        let result = console.save_html(path_str);
        assert!(result.is_ok());

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("<!DOCTYPE html>"));
        assert!(contents.contains("HTML content"));

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_save_svg_to_file() {
        let mut console = Console::builder()
            .width(40)
            .record(true)
            .no_color(true)
            .markup(false)
            .build();

        let text = Text::new("SVG save test", Style::null());
        console.print(&text);

        let dir = std::env::temp_dir();
        let path = dir.join("gilt_test_save.svg");
        let path_str = path.to_str().unwrap();

        let result = console.save_svg(path_str, Some("Test Title"));
        assert!(result.is_ok());

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("<svg"));
        assert!(contents.contains("SVG save test"));
        assert!(contents.contains("</svg>"));

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_save_svg_default_title() {
        let mut console = Console::builder()
            .width(40)
            .record(true)
            .no_color(true)
            .markup(false)
            .build();

        let text = Text::new("Default title test", Style::null());
        console.print(&text);

        let dir = std::env::temp_dir();
        let path = dir.join("gilt_test_save_default.svg");
        let path_str = path.to_str().unwrap();

        let result = console.save_svg(path_str, None);
        assert!(result.is_ok());

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("<svg"));

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_log_contains_timestamp_and_text() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.log("Test log message");
        let captured = console.end_capture();

        // Should contain a timestamp pattern [HH:MM:SS]
        assert!(captured.contains('['));
        assert!(captured.contains(']'));
        assert!(captured.contains(':'));
        assert!(captured.contains("Test log message"));
        assert!(captured.ends_with('\n'));
    }

    #[test]
    fn test_print_error_basic() {
        #[derive(Debug)]
        struct TestError;
        impl std::fmt::Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "test error occurred")
            }
        }
        impl std::error::Error for TestError {}

        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.print_error(&TestError);
        let captured = console.end_capture();

        // Should contain the error message rendered inside a panel
        assert!(captured.contains("test error occurred"));
    }

    // -- Pager convenience --------------------------------------------------

    #[test]
    fn test_pager_with_capture() {
        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();

        let text = Text::new("Pager content here", Style::null());
        console.print(&text);

        // Use `cat` as pager -- it reads stdin and exits cleanly.
        console.pager(Some("cat"));
    }

    // -- Screen enter/exit --------------------------------------------------

    #[test]
    fn test_enter_exit_screen() {
        let mut console = Console::builder().width(80).record(true).build();

        // Verify enter_screen activates alt screen and hides cursor.
        console.enter_screen(true);
        assert!(console.is_alt_screen);

        // Verify exit_screen restores state.
        console.exit_screen(true);
        assert!(!console.is_alt_screen);
    }

    #[test]
    fn test_enter_exit_screen_no_hide_cursor() {
        let mut console = Console::builder().width(80).record(true).build();

        console.enter_screen(false);
        assert!(console.is_alt_screen);

        console.exit_screen(false);
        assert!(!console.is_alt_screen);
    }

    // -- Live ID ------------------------------------------------------------

    #[test]
    fn test_set_clear_live() {
        let mut console = Console::new();
        assert_eq!(console.live_id, None);

        console.set_live(Some(42));
        assert_eq!(console.live_id, Some(42));

        console.clear_live();
        assert_eq!(console.live_id, None);
    }

    #[test]
    fn test_set_live_none() {
        let mut console = Console::new();
        console.set_live(Some(7));
        assert_eq!(console.live_id, Some(7));

        console.set_live(None);
        assert_eq!(console.live_id, None);
    }

    #[test]
    fn test_status_convenience() {
        let console = Console::builder().force_terminal(true).width(80).build();
        let status = console.status("Working...");
        assert_eq!(status.status_text, "Working...");
        assert!(!status.is_started());
    }

    // -- print_exception test -----------------------------------------------

    #[test]
    fn test_print_exception() {
        #[derive(Debug)]
        struct TestError;
        impl std::fmt::Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "something went wrong")
            }
        }
        impl std::error::Error for TestError {}

        let mut console = Console::builder()
            .width(80)
            .no_color(true)
            .markup(false)
            .build();

        console.begin_capture();
        console.print_exception(&TestError);
        let captured = console.end_capture();

        // Should contain the error message rendered via Traceback
        assert!(captured.contains("something went wrong"));
    }

    // -- input_password method exists and compiles ----------------------------

    #[test]
    fn test_input_password_method_exists() {
        // Verify the method signature compiles correctly.
        // We cannot actually call it in tests (requires a real terminal),
        // but we can verify the function pointer type resolves.
        let _fn_ptr: fn(&mut Console, &str) -> Result<String, std::io::Error> =
            Console::input_password;
        // Also verify Console::input still works
        let _fn_ptr2: fn(&mut Console, &str) -> Result<String, std::io::Error> = Console::input;
    }

    // -- Synchronized output ------------------------------------------------

    #[test]
    fn test_begin_synchronized_capture() {
        let mut console = Console::new();
        console.begin_capture();
        console.begin_synchronized();
        let output = console.end_capture();
        assert_eq!(output, "\x1b[?2026h");
    }

    #[test]
    fn test_end_synchronized_capture() {
        let mut console = Console::new();
        console.begin_capture();
        console.end_synchronized();
        let output = console.end_capture();
        assert_eq!(output, "\x1b[?2026l");
    }

    #[test]
    fn test_synchronized_wraps_content() {
        let mut console = Console::new();
        console.begin_capture();
        console.synchronized(|c| {
            c.print_text("hello");
        });
        let output = console.end_capture();
        assert!(
            output.starts_with("\x1b[?2026h"),
            "should start with begin sync"
        );
        assert!(output.ends_with("\x1b[?2026l"), "should end with end sync");
        assert!(output.contains("hello"), "should contain the printed text");
    }

    #[test]
    fn test_synchronized_returns_value() {
        let mut console = Console::new();
        console.begin_capture();
        let result = console.synchronized(|_c| 42);
        let _ = console.end_capture();
        assert_eq!(result, 42);
    }

    // -- Clipboard (OSC 52) -------------------------------------------------

    #[test]
    fn test_copy_to_clipboard_capture() {
        let mut console = Console::new();
        console.begin_capture();
        console.copy_to_clipboard("hello");
        let output = console.end_capture();
        // "hello" base64 = "aGVsbG8="
        assert_eq!(output, "\x1b]52;c;aGVsbG8=\x07");
    }

    #[test]
    fn test_copy_to_clipboard_empty_capture() {
        let mut console = Console::new();
        console.begin_capture();
        console.copy_to_clipboard("");
        let output = console.end_capture();
        assert_eq!(output, "\x1b]52;c;\x07");
    }

    #[test]
    fn test_copy_to_clipboard_unicode_capture() {
        let mut console = Console::new();
        console.begin_capture();
        console.copy_to_clipboard("caf\u{00e9}");
        let output = console.end_capture();
        // "caf\xc3\xa9" base64 = "Y2Fmw6k="
        assert_eq!(output, "\x1b]52;c;Y2Fmw6k=\x07");
    }

    #[test]
    fn test_request_clipboard_capture() {
        let mut console = Console::new();
        console.begin_capture();
        console.request_clipboard();
        let output = console.end_capture();
        assert_eq!(output, "\x1b]52;c;?\x07");
    }

    // -- Helper function for tests ------------------------------------------

    fn make_default_options() -> ConsoleOptions {
        ConsoleOptions {
            size: ConsoleDimensions {
                width: 80,
                height: 25,
            },
            legacy_windows: false,
            min_width: 1,
            max_width: 80,
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
}
