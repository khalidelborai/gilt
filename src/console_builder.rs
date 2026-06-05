//! `ConsoleBuilder` extracted from console.rs in v1.2 Phase 3.
//! `Console::from_builder` (in console.rs) owns the actual Console
//! construction; this file holds the builder type and its setter methods.

use crate::color::ColorSystem;
use crate::console::Console;
use crate::theme::Theme;

/// Builder for constructing a `Console` with custom options.
pub struct ConsoleBuilder {
    pub(crate) color_system: Option<String>,
    pub(crate) color_system_override: Option<ColorSystem>,
    pub(crate) width: Option<usize>,
    pub(crate) height: Option<usize>,
    pub(crate) force_terminal: Option<bool>,
    pub(crate) record: bool,
    pub(crate) theme: Option<Theme>,
    pub(crate) markup: bool,
    pub(crate) highlight: bool,
    pub(crate) no_color: bool,
    pub(crate) no_color_explicit: bool,
    pub(crate) tab_size: usize,
    pub(crate) quiet: bool,
    pub(crate) soft_wrap: bool,
    pub(crate) safe_box: bool,
    pub(crate) log_path: bool,
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
            log_path: false,
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

    /// When `true`, `Console::log` appends the caller's file:line to each log line.
    ///
    /// Default: `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let mut c = Console::builder()
    ///     .width(80)
    ///     .no_color(true)
    ///     .markup(false)
    ///     .log_path(true)
    ///     .build();
    /// c.begin_capture();
    /// c.log("hello");
    /// let out = c.end_capture();
    /// assert!(out.contains("hello"));
    /// // The caller's file name appears when log_path is true.
    /// ```
    pub fn log_path(mut self, lp: bool) -> Self {
        self.log_path = lp;
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
    /// Build the `Console` instance with the configured options.
    pub fn build(self) -> Console {
        Console::from_builder(self)
    }
}
