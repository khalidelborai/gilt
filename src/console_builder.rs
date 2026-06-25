//! `ConsoleBuilder` extracted from console.rs in v1.2 Phase 3.
//! `Console::from_builder` (in console.rs) owns the actual Console
//! construction; this file holds the builder type and its setter methods.

use crate::color::ColorSystem;
use crate::console::Console;
use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Internal helper: try to load a JSON Theme from a file path (non-wasm only)
// ---------------------------------------------------------------------------

/// Load a [`Theme`] from a JSON file at `path`.
///
/// Returns `Some(theme)` on success, `None` on any error (missing file,
/// bad JSON, …).  Errors are intentionally non-fatal so callers fall back
/// to the default theme.
#[cfg(all(feature = "json", not(target_arch = "wasm32")))]
pub(crate) fn load_theme_from_path(path: &std::path::Path) -> Option<Theme> {
    let file = std::fs::File::open(path).ok()?;
    Theme::from_json_reader(file).ok()
}

/// Builder for constructing a `Console` with custom options.
pub struct ConsoleBuilder {
    pub(crate) color_system: Option<String>,
    pub(crate) color_system_override: Option<ColorSystem>,
    pub(crate) width: Option<usize>,
    pub(crate) height: Option<usize>,
    pub(crate) force_terminal: Option<bool>,
    pub(crate) record: bool,
    pub(crate) theme: Option<Theme>,
    /// Path to a JSON theme file, set via [`theme_from_path`](Self::theme_from_path).
    /// Resolved at [`build`](Self::build) time; takes priority over the
    /// [`theme`](Self::theme) builder method.
    #[cfg(all(feature = "json", not(target_arch = "wasm32")))]
    pub(crate) theme_path: Option<std::path::PathBuf>,
    pub(crate) markup: bool,
    pub(crate) highlight: bool,
    pub(crate) no_color: bool,
    pub(crate) no_color_explicit: bool,
    pub(crate) tab_size: usize,
    pub(crate) quiet: bool,
    pub(crate) soft_wrap: bool,
    pub(crate) safe_box: bool,
    /// Explicit override for `legacy_windows`. `None` (the default) lets
    /// the built console auto-detect legacy Windows mode at build time;
    /// `Some(val)` forces the value.
    pub(crate) legacy_windows: Option<bool>,
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
            #[cfg(all(feature = "json", not(target_arch = "wasm32")))]
            theme_path: None,
            markup: true,
            highlight: true,
            no_color: false,
            no_color_explicit: false,
            tab_size: 8,
            quiet: false,
            soft_wrap: false,
            safe_box: true,
            legacy_windows: None,
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

    /// Load a [`Theme`] from a JSON file at `path` and push it onto the
    /// new console's theme stack at construction time.
    ///
    /// If the file does not exist or cannot be parsed as JSON, the error is
    /// silently ignored and the console falls back to the default theme.
    ///
    /// Requires the `json` feature (enabled by default).  No-op on wasm
    /// targets (where filesystem access is unavailable).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "json", not(target_arch = "wasm32")))] {
    /// use gilt::console::Console;
    ///
    /// let console = Console::builder()
    ///     .theme_from_path(std::path::Path::new("my-theme.json"))
    ///     .build();
    /// # }
    /// ```
    #[cfg(all(feature = "json", not(target_arch = "wasm32")))]
    pub fn theme_from_path(mut self, path: &std::path::Path) -> Self {
        self.theme_path = Some(path.to_path_buf());
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

    /// Explicitly set the `legacy_windows` flag.
    ///
    /// When `legacy_windows` is `true`, gilt uses the legacy Windows
    /// console renderer (Win32 API) instead of ANSI/VT escape codes.
    ///
    /// On non-Windows hosts this is a forced override; gilt does not
    /// auto-detect legacy Windows on those targets. On Windows, this
    /// overrides auto-detection — pass `false` to confirm a modern
    /// terminal (Windows Terminal, etc.) even when one of the
    /// `WT_SESSION` / `WT_PROFILE_ID` / `TERM_PROGRAM` env vars is
    /// missing.
    ///
    /// Default: `None` (auto-detect).
    ///
    /// # Examples
    ///
    /// ```
    /// use gilt::console::Console;
    ///
    /// let console = Console::builder()
    ///     .width(80)
    ///     .with_legacy_windows(false)
    ///     .build();
    /// assert!(!console.legacy_windows());
    /// ```
    pub fn with_legacy_windows(mut self, val: bool) -> Self {
        self.legacy_windows = Some(val);
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

    /// Probe the terminal background and, if dark or light is detected,
    /// apply a matching built-in theme.
    ///
    /// When the `terminal-query` feature is enabled **and** the build is
    /// native (not wasm32), this calls
    /// [`query_terminal_background`](gilt::terminal_bg::query_terminal_background)
    /// and switches to a dark or light theme accordingly:
    ///
    /// - `ConsoleBackground::Dark`  → the default gilt theme (already tuned
    ///   for dark terminals; no change applied).
    /// - `ConsoleBackground::Light` → overrides key text colours to darker,
    ///   higher-contrast values for light backgrounds.
    /// - `ConsoleBackground::Unknown` → no change.
    ///
    /// When `terminal-query` is **not** enabled, this is a complete no-op so
    /// that default builds are unaffected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "terminal-query")] {
    /// use gilt::console::Console;
    ///
    /// let console = Console::builder()
    ///     .width(80)
    ///     .auto_theme()
    ///     .build();
    /// # }
    /// ```
    pub fn auto_theme(self) -> Self {
        #[cfg(all(feature = "terminal-query", not(target_arch = "wasm32")))]
        {
            use crate::terminal_bg::{query_terminal_background, ConsoleBackground};
            match query_terminal_background() {
                ConsoleBackground::Light => {
                    // Apply a light-background theme: darker text colours for
                    // better contrast on white/cream terminal backgrounds.
                    let mut styles = std::collections::HashMap::new();
                    styles.insert(
                        "markdown.h1".to_string(),
                        crate::style::Style::parse("bold dark_blue"),
                    );
                    styles.insert(
                        "markdown.h2".to_string(),
                        crate::style::Style::parse("bold dark_blue"),
                    );
                    styles.insert(
                        "markdown.code".to_string(),
                        crate::style::Style::parse("dark_red on grey93"),
                    );
                    styles.insert(
                        "repr.str".to_string(),
                        crate::style::Style::parse("dark_green"),
                    );
                    styles.insert(
                        "repr.number".to_string(),
                        crate::style::Style::parse("dark_blue"),
                    );
                    styles.insert(
                        "rule.line".to_string(),
                        crate::style::Style::parse("dark_blue"),
                    );
                    let theme = crate::theme::Theme::new(Some(styles), true);
                    self.theme(theme)
                }
                // Dark or Unknown: leave the default dark theme in place.
                ConsoleBackground::Dark | ConsoleBackground::Unknown => self,
            }
        }
        #[cfg(not(all(feature = "terminal-query", not(target_arch = "wasm32"))))]
        {
            // No-op when terminal-query is off.
            self
        }
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
