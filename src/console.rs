//! Console engine — the central orchestrator of gilt rendering output.
//!
//! The Console manages terminal capabilities, drives the rendering pipeline,
//! and handles output buffering, capture, and export.

use crate::color::ColorSystem;
use crate::color_env::{detect_color_env, ColorEnvOverride};
use crate::control::Control;
use crate::error::ConsoleError;
use crate::export_format::{CONSOLE_HTML_FORMAT, CONSOLE_SVG_FORMAT};
use crate::pager::Pager;
use crate::segment::Segment;
use crate::style::Style;
use crate::style_interner::StyleInterner;
use crate::terminal_theme::{TerminalTheme, DEFAULT_TERMINAL_THEME, SVG_EXPORT_THEME};
use crate::text::{JustifyMethod, OverflowMethod, Text};
use crate::theme::{Theme, ThemeStack};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

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
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment>;
}

impl Renderable for Text {
    fn gilt_console(&self, _console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
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
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        let text = console.render_str(self, None, options.justify, options.overflow);
        text.gilt_console(console, options)
    }
}

impl Renderable for String {
    fn gilt_console(&self, console: &Console, options: &ConsoleOptions) -> Vec<Segment> {
        self.as_str().gilt_console(console, options)
    }
}

// ConsoleBuilder moved to console_builder.rs (v1.2 Phase 3 split).
#[path = "console_builder.rs"]
mod console_builder;
pub use console_builder::ConsoleBuilder;

// Capture-mode methods (begin_capture / end_capture / render_widget_to_text)
// moved to console_capture.rs in v1.3 Phase 4 — methods stay on Console via
// a separate impl block.
#[path = "console_capture.rs"]
mod console_capture;

// Render path (render/print/log/rule/line/inspect/print_json/print_error/
// print_exception/write_segments/buffer ops) moved to console_render.rs in
// v1.3 Phase 5 — same multi-impl-block pattern.
#[path = "console_render.rs"]
mod console_render;

// ---------------------------------------------------------------------------
// Console
// ---------------------------------------------------------------------------

/// The central orchestrator of gilt rendering output.
///
/// Console manages terminal capabilities, drives the rendering pipeline,
/// and handles output buffering, capture, and export.
pub struct Console {
    // Configuration
    color_system: Option<ColorSystem>,
    width_override: Option<usize>,
    height_override: Option<usize>,
    force_terminal: Option<bool>,
    #[allow(dead_code)] // Reserved for future tab expansion support
    tab_size: usize,
    record: bool,
    markup_enabled: bool,
    highlight_enabled: bool,
    #[allow(dead_code)] // Reserved for future soft-wrap rendering
    soft_wrap: bool,
    no_color: bool,
    quiet: bool,
    #[allow(dead_code)] // Reserved for future safe box-drawing fallback
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
    /// Stack of nested live-display IDs.
    ///
    /// The top-of-stack ID is the currently-active Live; any preceding entries
    /// represent outer Live displays that have been suspended while a nested
    /// Live was started. Mirrors rich v14.1.0's `Console._live_stack` —
    /// allows Progress + Live + Status etc. to nest without each one
    /// clobbering the others' state.
    live_stack: Vec<usize>,

    /// Per-console style interner (foundation for L2 — see
    /// `.review/V0_11_DESIGN.md`). **Dormant in v0.11.0-alpha.1**: no
    /// caller currently interns or resolves through it. Wired in PR1b
    /// when `Segment::style` is converted from a field to a method.
    /// `Arc<Mutex<...>>` so the handle returned by `style_interner()` can
    /// be shared across threads (e.g. by a future Live integration) and
    /// to leave room for cross-Console resegmenting in PR3.
    style_interner: Arc<Mutex<StyleInterner>>,

    /// Optional output sink override. When `Some`, render output goes here
    /// instead of `std::io::stdout()`. Set via [`Console::with_writer`].
    /// Capture and record modes still take precedence.
    ///
    /// `+ Send + Sync` (not just `+ Send`) preserves `Console: Sync` so
    /// downstream code can wrap a Console in `Arc<…>` for cross-task
    /// sharing — same trait bounds Console had pre-v1.2.0.
    pub(crate) writer_override: Option<Box<dyn std::io::Write + Send + Sync>>,
}

impl Console {
    /// Create a Console with sensible defaults.
    ///
    /// Prefer [`Console::default()`] for the common one-line case — it
    /// calls this method. Reach for [`Console::builder`] only when you
    /// need explicit overrides (custom width, recording for export,
    /// forcing a specific color system).
    ///
    /// Defaults:
    /// - **Color**: TrueColor, auto-disabled by `NO_COLOR`,
    ///   auto-forced by `FORCE_COLOR` / `CLICOLOR`.
    /// - **Width**: auto-detected from terminal; falls back to 80.
    /// - **Markup**: enabled (`[bold]hi[/]`-style tags parsed by
    ///   [`print_text`](Self::print_text)).
    /// - **Recording**: off — enable via the builder for
    ///   [`export_html`](Self::export_html) / [`export_svg`](Self::export_svg).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut c = Console::default();           // recommended
    /// c.print_text("[bold green]ready[/]");
    /// ```
    pub fn new() -> Self {
        ConsoleBuilder::default().build()
    }

    /// Create a Console using the builder pattern. Use when you need
    /// explicit overrides on top of [`Console::default()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let console = Console::builder()
    ///     .width(120)
    ///     .record(true)       // for export_html / export_svg
    ///     .build();
    /// assert_eq!(console.width(), 120);
    /// ```
    pub fn builder() -> ConsoleBuilder {
        ConsoleBuilder::default()
    }

    /// Construct a `Console` from a fully-configured `ConsoleBuilder`.
    /// Called by `ConsoleBuilder::build`; lives here so `Console`'s
    /// private fields stay private to console.rs.
    pub(crate) fn from_builder(builder: ConsoleBuilder) -> Self {
        // Color system priority (highest first):
        //   1. color_system_override (explicit ColorSystem value)
        //   2. color_system (string, e.g. "truecolor")
        //   3. no_color(true) explicitly set by caller
        //   4. Environment vars (NO_COLOR, FORCE_COLOR, CLICOLOR_FORCE, CLICOLOR)
        //   5. Default: TrueColor
        let has_explicit_cs = matches!(
            builder.color_system.as_deref(),
            Some("standard" | "256" | "truecolor" | "windows")
        );

        let color_system = if let Some(cs) = builder.color_system_override {
            Some(cs)
        } else if has_explicit_cs {
            match builder.color_system.as_deref() {
                Some("standard") => Some(ColorSystem::Standard),
                Some("256") => Some(ColorSystem::EightBit),
                Some("truecolor") => Some(ColorSystem::TrueColor),
                Some("windows") => Some(ColorSystem::Windows),
                _ => unreachable!(),
            }
        } else if builder.no_color_explicit && builder.no_color {
            None
        } else {
            match detect_color_env() {
                ColorEnvOverride::NoColor => None,
                ColorEnvOverride::ForceColor => Some(ColorSystem::EightBit),
                ColorEnvOverride::ForceColorTruecolor => Some(ColorSystem::TrueColor),
                ColorEnvOverride::None => {
                    if builder.no_color {
                        None
                    } else {
                        Some(ColorSystem::TrueColor)
                    }
                }
            }
        };

        let theme = builder.theme.unwrap_or_else(|| Theme::new(None, true));
        let theme_stack = ThemeStack::new(theme);

        Console {
            color_system,
            width_override: builder.width,
            height_override: builder.height,
            force_terminal: builder.force_terminal,
            tab_size: builder.tab_size,
            record: builder.record,
            markup_enabled: builder.markup,
            highlight_enabled: builder.highlight,
            soft_wrap: builder.soft_wrap,
            no_color: builder.no_color,
            quiet: builder.quiet,
            safe_box: builder.safe_box,
            legacy_windows: false,
            base_style: None,
            theme_stack,
            buffer: Vec::new(),
            buffer_index: 0,
            record_buffer: Vec::new(),
            is_alt_screen: false,
            capture_buffer: None,
            live_stack: Vec::new(),
            style_interner: Arc::new(Mutex::new(StyleInterner::new())),
            writer_override: None,
        }
    }

    /// Send all output to a custom writer instead of `std::io::stdout()`.
    /// Use for log-to-file, in-memory testing, or piping into a pre-existing
    /// sink (e.g. a network socket). Capture and record modes still take
    /// precedence over the override.
    ///
    /// ```
    /// # use gilt::console::Console;
    /// let buf: Vec<u8> = Vec::new();
    /// let mut console = Console::default().with_writer(buf);
    /// console.print_text("hello");
    /// // output is in console's writer, not on stdout
    /// ```
    pub fn with_writer<W: std::io::Write + Send + Sync + 'static>(mut self, writer: W) -> Self {
        self.writer_override = Some(Box::new(writer));
        self
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
    ///
    /// Resolution order:
    /// 1. Explicit `force_terminal` set on the builder
    /// 2. `TTY_COMPATIBLE=1`/`0` environment override
    /// 3. `TERM` is set in the environment
    pub fn is_terminal(&self) -> bool {
        if let Some(forced) = self.force_terminal {
            return forced;
        }
        match crate::color::color_env::detect_tty_compatible() {
            crate::color::color_env::TtyOverride::ForceTty => return true,
            crate::color::color_env::TtyOverride::ForceNotTty => return false,
            crate::color::color_env::TtyOverride::None => {}
        }
        std::env::var("TERM").is_ok()
    }

    /// Whether the console should treat the user as interactive (prompts,
    /// progress bars with refresh, live updates, etc.).
    ///
    /// Resolution order:
    /// 1. `TTY_INTERACTIVE=1`/`0` environment override
    /// 2. Falls back to [`is_terminal`](Self::is_terminal)
    ///
    /// This is intentionally independent of TTY status so a user can pipe
    /// output to a file but still be prompted on stdin.
    pub fn is_interactive(&self) -> bool {
        match crate::color::color_env::detect_tty_interactive() {
            crate::color::color_env::TtyOverride::ForceTty => true,
            crate::color::color_env::TtyOverride::ForceNotTty => false,
            crate::color::color_env::TtyOverride::None => self.is_terminal(),
        }
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
        Style::parse_strict(name).map_err(|e| {
            ConsoleError::RenderError(format!("Failed to get style '{}': {}", name, e))
        })
    }

    /// Per-console style interner. **Dormant in v0.11.0-alpha.1** — no
    /// internal callers route through it yet. Exposed so `Segment` (PR1b)
    /// and the eventual L2 activation (PR3) can intern/resolve through
    /// the same id space as the parent `Console`.
    ///
    /// The returned handle is `Arc<Mutex<…>>` because a `Console::copy`
    /// (e.g. `begin_capture`) shares the id space with its origin.
    pub fn style_interner(&self) -> &Arc<Mutex<StyleInterner>> {
        &self.style_interner
    }

    /// Push a new theme onto the theme stack.
    pub fn push_theme(&mut self, theme: Theme) {
        self.theme_stack.push_theme(theme, true);
    }

    /// Pop the top theme from the theme stack.
    pub fn pop_theme(&mut self) {
        let _ = self.theme_stack.pop_theme();
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

    /// Render a [`Renderable`] at an arbitrary `(x, y)` position in the active
    /// alternate screen.
    ///
    /// Moves the cursor to the absolute position (0-indexed), prints the
    /// renderable, then leaves the cursor at the position the renderable's
    /// last segment ended at. Designed for partial-update UIs (Layouts,
    /// dashboards) running inside `enter_screen`.
    ///
    /// Has no effect — and silently does nothing — when the console is not
    /// currently in alt-screen mode, since absolute positioning would
    /// scribble on the user's main scrollback otherwise.
    pub fn update_screen(&mut self, x: usize, y: usize, renderable: &dyn Renderable) {
        if !self.is_alt_screen {
            return;
        }
        let ctrl = crate::utils::control::Control::move_to(x as i32, y as i32);
        self.write_segments(&[ctrl.segment]);
        self.print(renderable);
    }

    /// Render a slice of [`Segment`] lines at successive rows starting from
    /// `(x, y)` in the active alternate screen.
    ///
    /// Each `Vec<Segment>` in `lines` is treated as one line — printed at
    /// `(x, y + i)` for the i-th entry. Useful when you've already produced
    /// per-line segments via `Console::render` and want to splat them into a
    /// known position without going through a full Renderable wrapper.
    ///
    /// Like [`update_screen`](Self::update_screen), no-ops when not in
    /// alt-screen mode.
    pub fn update_screen_lines(&mut self, x: usize, y: usize, lines: &[Vec<Segment>]) {
        if !self.is_alt_screen {
            return;
        }
        for (i, line) in lines.iter().enumerate() {
            let ctrl = crate::utils::control::Control::move_to(x as i32, (y + i) as i32);
            self.write_segments(&[ctrl.segment]);
            self.write_segments(line);
        }
    }

    // -- Live display ID ----------------------------------------------------

    /// Push a Live-display ID onto the stack, making it the active Live.
    ///
    /// Returns `true` if the ID was pushed (always true; the API returns a
    /// `bool` for parity with rich's `set_live` which returned `False` when
    /// nesting was disabled — gilt always allows nesting).
    pub fn push_live(&mut self, live_id: usize) -> bool {
        self.live_stack.push(live_id);
        true
    }

    /// Pop the top Live-display ID off the stack. Returns the popped ID, or
    /// `None` if the stack was empty.
    pub fn pop_live(&mut self) -> Option<usize> {
        self.live_stack.pop()
    }

    /// Return the currently-active Live-display ID (the top of the stack), or
    /// `None` when no Live is active.
    pub fn current_live(&self) -> Option<usize> {
        self.live_stack.last().copied()
    }

    /// Number of currently-nested Live displays. `0` means no Live is active.
    pub fn live_depth(&self) -> usize {
        self.live_stack.len()
    }

    // -- Backwards-compatible single-slot API -------------------------------

    /// Set the active Live-display ID. `Some(id)` pushes (or replaces top);
    /// `None` clears the entire stack.
    ///
    /// Provided for source compatibility with the pre-nesting API. New code
    /// should prefer [`push_live`](Self::push_live) / [`pop_live`](Self::pop_live).
    pub fn set_live(&mut self, live_id: Option<usize>) {
        match live_id {
            Some(id) => {
                if let Some(top) = self.live_stack.last_mut() {
                    *top = id;
                } else {
                    self.live_stack.push(id);
                }
            }
            None => self.live_stack.clear(),
        }
    }

    /// Clear all Live IDs. Equivalent to `set_live(None)`.
    pub fn clear_live(&mut self) {
        self.live_stack.clear();
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
    /// let text = Text::styled("Red text", "red");
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

            if let Some(style) = segment.style() {
                if style.is_null() {
                    code.push_str(&escaped);
                    continue;
                }

                let css = style.get_html_style(Some(theme));
                if css.is_empty() {
                    code.push_str(&escaped);
                } else if inline_styles {
                    write!(code, "<span style=\"{}\">{}</span>", css, escaped).unwrap();
                } else {
                    // Use class-based styles
                    let class_name =
                        find_or_insert_class(&mut style_cache, &mut stylesheet, style, &css);
                    write!(code, "<span class=\"{}\">{}</span>", class_name, escaped).unwrap();
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

        // Pre-format numeric values into a shared buffer to avoid per-replace allocations.
        let mut buf = String::with_capacity(16);
        macro_rules! fmt_buf {
            ($fmt:literal, $val:expr) => {{
                buf.clear();
                write!(buf, $fmt, $val).unwrap();
                &buf
            }};
        }

        // Apply replacements that use the shared buffer one at a time,
        // cloning the formatted value so `buf` can be reused.
        let mut svg = CONSOLE_SVG_FORMAT.replace("{unique_id}", unique_id);
        svg = svg.replace("{char_height}", fmt_buf!("{:.1}", char_height));
        svg = svg.replace("{line_height}", fmt_buf!("{:.1}", line_height));
        svg = svg.replace("{width}", fmt_buf!("{:.0}", svg_width));
        svg = svg.replace("{height}", fmt_buf!("{:.0}", svg_height));
        svg = svg.replace("{terminal_width}", fmt_buf!("{:.0}", terminal_width));
        svg = svg.replace("{terminal_height}", fmt_buf!("{:.0}", terminal_height));
        svg = svg.replace("{terminal_x}", fmt_buf!("{:.0}", terminal_x));
        svg = svg.replace("{terminal_y}", fmt_buf!("{:.0}", terminal_y));
        svg = svg.replace("{chrome}", &chrome);
        svg = svg.replace("{matrix}", &matrix);
        svg = svg.replace("{backgrounds}", &backgrounds);
        svg = svg.replace("{styles}", &styles);
        svg = svg.replace("{lines}", &lines_defs);
        svg
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

// Export helpers moved to console_export.rs (v1.2 Phase 2 split).
#[path = "console_export.rs"]
mod console_export;
use console_export::*;
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "console_tests.rs"]
mod tests;
