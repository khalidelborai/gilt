//! Logging handler for the `log` crate.
//!
//! This module provides a [`RichHandler`] that implements [`log::Log`],
//! producing styled, formatted log output through gilt's [`Console`].
//!

use std::collections::HashMap;
use std::sync::Mutex;

use crate::cells::cell_len;
use crate::console::Console;
use crate::error::traceback::Traceback;
use crate::highlighter::{Highlighter, ReprHighlighter};
use crate::markup;
use crate::style::Style;
use crate::text::{JustifyMethod, Text};
use crate::widgets::table::Table;

// ---------------------------------------------------------------------------
// Default keywords (HTTP verbs, matching Python's RichHandler.KEYWORDS)
// ---------------------------------------------------------------------------

/// Default keywords highlighted in log messages.
const DEFAULT_KEYWORDS: &[&str] = &[
    "GET", "POST", "HEAD", "PUT", "DELETE", "OPTIONS", "TRACE", "PATCH",
];

// ---------------------------------------------------------------------------
// RichHandler
// ---------------------------------------------------------------------------

/// A [`log::Log`] implementation that produces styled, formatted log output
/// using gilt's [`Console`].
///
/// Each log record is rendered as a line with optional columns:
/// - **Time** (`HH:MM:SS`, dim style)
/// - **Level** (color-coded, 8 chars wide)
/// - **Message** (optionally parsed as markup, with keyword highlighting)
/// - **Path** (`module::path`, dim style, plus line number)
pub struct RichHandler {
    console: Mutex<Console>,
    show_time: bool,
    show_level: bool,
    show_path: bool,
    markup: bool,
    /// When `true`, suppress the timestamp on consecutive records that share
    /// the same wall-clock second, replacing it with spaces of equal width so
    /// columns remain aligned.
    omit_repeated_times: bool,
    /// When `true`, render the `file:line` location as a clickable OSC 8
    /// hyperlink using the `file://` URL scheme.
    enable_link_path: bool,
    /// When `true`, records whose message looks like an error chain are
    /// rendered via [`Traceback::from_error`] for full styled output.
    gilt_tracebacks: bool,
    keywords: Vec<String>,
    level_styles: HashMap<log::Level, Style>,
    /// Cache of the last-emitted cache key (days+HH:MM:SS) for `omit_repeated_times`.
    /// Using a date-inclusive key fixes the midnight repeat-suppression bug (item 7).
    last_time_str: Mutex<Option<String>>,
    /// Optional strftime-style time format string. Currently stored for future use;
    /// display always uses HH:MM:SS (no `time` crate dep). The cache key includes
    /// the date so midnight rollover never falsely suppresses a new timestamp.
    time_format: Option<String>,
    /// Highlighter applied to log messages. Default: [`ReprHighlighter`].
    highlighter: Box<dyn Highlighter + Send + Sync>,
    /// Minimum log level to emit. Records with lower severity are suppressed.
    min_level: log::LevelFilter,
}

impl RichHandler {
    /// Create a new `RichHandler` with sensible defaults.
    ///
    /// Uses a default [`Console`] and the default level styles.
    pub fn new() -> Self {
        RichHandler {
            console: Mutex::new(Console::new()),
            show_time: true,
            show_level: true,
            show_path: true,
            markup: false,
            omit_repeated_times: true,
            // Finding #15: rich defaults enable_link_path to true; was incorrectly false.
            enable_link_path: true,
            gilt_tracebacks: false,
            keywords: DEFAULT_KEYWORDS.iter().map(|s| s.to_string()).collect(),
            level_styles: Self::default_level_styles(),
            last_time_str: Mutex::new(None),
            time_format: None,
            highlighter: Box::new(ReprHighlighter),
            min_level: log::LevelFilter::Trace,
        }
    }

    /// Replace the console used for output.
    #[must_use]
    pub fn with_console(mut self, console: Console) -> Self {
        self.console = Mutex::new(console);
        self
    }

    /// Set whether to show the time column.
    #[must_use]
    pub fn with_show_time(mut self, show: bool) -> Self {
        self.show_time = show;
        self
    }

    /// Set whether to show the level column.
    #[must_use]
    pub fn with_show_level(mut self, show: bool) -> Self {
        self.show_level = show;
        self
    }

    /// Set whether to show the source path column.
    #[must_use]
    pub fn with_show_path(mut self, show: bool) -> Self {
        self.show_path = show;
        self
    }

    /// Set whether log messages are parsed as rich markup.
    #[must_use]
    pub fn with_markup(mut self, markup: bool) -> Self {
        self.markup = markup;
        self
    }

    /// Set the keywords to highlight in log messages.
    #[must_use]
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Override the level style map.
    #[must_use]
    pub fn with_level_styles(mut self, styles: HashMap<log::Level, Style>) -> Self {
        self.level_styles = styles;
        self
    }

    /// Suppress repeated timestamps on consecutive records sharing the same
    /// wall-clock second (default `true`).
    ///
    /// When suppressed, the time column is replaced with spaces of the same
    /// width so the path / level columns stay aligned.
    #[must_use]
    pub fn with_omit_repeated_times(mut self, omit: bool) -> Self {
        self.omit_repeated_times = omit;
        self
    }

    /// Render the `module::path:line` location as a clickable OSC 8 hyperlink
    /// using the `file://` URL scheme (default `false`).
    ///
    /// Useful in IDE-integrated terminals (VS Code, JetBrains, iTerm2) — the
    /// link opens the source file at the right line.
    #[must_use]
    pub fn with_enable_link_path(mut self, enable: bool) -> Self {
        self.enable_link_path = enable;
        self
    }

    /// Render error-bearing log records via [`Traceback`] for full styled
    /// output (default `false`).
    #[must_use]
    pub fn with_gilt_tracebacks(mut self, enable: bool) -> Self {
        self.gilt_tracebacks = enable;
        self
    }

    /// Set a strftime-style time format string.
    ///
    /// The format string is stored and used as part of the omit-repeated-times
    /// cache key (ensuring midnight rollover is handled correctly). Display
    /// currently always uses `HH:MM:SS` — no extra time crate dependency is
    /// introduced.
    #[must_use]
    pub fn with_time_format(mut self, fmt: String) -> Self {
        self.time_format = Some(fmt);
        self
    }

    /// Replace the highlighter applied to log messages.
    ///
    /// Default: [`ReprHighlighter`]. Pass [`NullHighlighter`] to disable.
    #[must_use]
    pub fn with_highlighter<H: Highlighter + Send + Sync + 'static>(mut self, h: H) -> Self {
        self.highlighter = Box::new(h);
        self
    }

    /// Set the minimum log level to emit.
    ///
    /// Records with lower severity than `level` are suppressed by [`enabled`](Self::enabled).
    /// Default: [`log::LevelFilter::Trace`] (pass everything).
    #[must_use]
    pub fn with_min_level(mut self, level: log::LevelFilter) -> Self {
        self.min_level = level;
        self
    }

    /// Return the default level style map.
    ///
    /// Finding #16: aligned to `default_styles.rs` theme values (single source
    /// of truth). Rich: error=bold red, warn=bold yellow, info=bold blue,
    /// debug=bold green, trace=dim. Previously info/debug were swapped.
    fn default_level_styles() -> HashMap<log::Level, Style> {
        let mut m = HashMap::new();
        m.insert(log::Level::Error, Style::parse("bold red"));
        m.insert(log::Level::Warn, Style::parse("bold yellow"));
        // logging.level.info  = blue  → bold blue
        m.insert(log::Level::Info, Style::parse("bold blue"));
        // logging.level.debug = green → bold green
        m.insert(log::Level::Debug, Style::parse("bold green"));
        m.insert(log::Level::Trace, Style::parse("dim"));
        m
    }

    /// Build the level column text, left-padded to 8 chars.
    ///
    /// Tries `console.get_style("logging.level.{name}")` first (item 2),
    /// falling back to the handler's level_styles map.
    #[cfg(test)]
    fn render_level(&self, level: log::Level, console: &Console) -> Text {
        let name = match level {
            log::Level::Error => "ERROR",
            log::Level::Warn => "WARN",
            log::Level::Info => "INFO",
            log::Level::Debug => "DEBUG",
            log::Level::Trace => "TRACE",
        };
        let lvl_lower = name.to_lowercase();
        let style = console
            .get_style(&format!("logging.level.{}", lvl_lower))
            .unwrap_or_else(|_| {
                self.level_styles
                    .get(&level)
                    .cloned()
                    .unwrap_or_else(Style::null)
            });
        let padded = format!("{:<8}", name);
        Text::styled_with(&padded, style)
    }

    /// Build the message column, optionally parsing markup and highlighting keywords.
    ///
    /// Finding #14: apply highlighter to all log messages to match Python's
    /// `RichHandler` behaviour (numbers, strings, booleans, etc. are highlighted
    /// in repr-style). Uses the configurable `self.highlighter` (item 4).
    #[cfg(test)]
    fn render_message(&self, record: &log::Record, console: &Console) -> Text {
        let msg = format!("{}", record.args());
        let mut text = if self.markup {
            let base = Style::null();
            markup::render(&msg, base).unwrap_or_else(|_| Text::new(&msg, Style::null()))
        } else {
            Text::new(&msg, Style::null())
        };

        // Apply configurable highlighter (item 4).
        self.highlighter.highlight(&mut text);

        // Keyword highlighting (item 1 — style from theme).
        if !self.keywords.is_empty() {
            let kw_style = console
                .get_style("logging.keyword")
                .unwrap_or_else(|_| Style::parse("bold on dark_green"));
            let words: Vec<&str> = self.keywords.iter().map(|s| s.as_str()).collect();
            text.highlight_words(&words, kw_style, false);
        }

        text
    }

    /// Build the plain path column (`module::path:line`) without link wrapping.
    /// Kept for tests that want the unwrapped form.
    #[cfg(test)]
    fn render_path(record: &log::Record) -> Text {
        let dim_style = Style::parse("dim");
        let module = record.module_path().unwrap_or("");
        let line = record.line().unwrap_or(0);
        let path_str = if !module.is_empty() {
            format!("{}:{}", module, line)
        } else {
            format!(":{}", line)
        };
        Text::styled_with(&path_str, dim_style)
    }

    /// Build the path column (`module::path:line`) optionally as an OSC 8
    /// hyperlink to a `file://` URL when `enable_link_path` is set.
    ///
    /// The link uses the record's `file()` (absolute or workspace-relative
    /// source path) for the URL when available; the visible text remains the
    /// `module::path:line` form for readability.
    fn render_path_with_link(&self, record: &log::Record) -> Text {
        let dim_style = Style::parse("dim");
        let module = record.module_path().unwrap_or("");
        let line = record.line().unwrap_or(0);
        let path_str = if !module.is_empty() {
            format!("{}:{}", module, line)
        } else {
            format!(":{}", line)
        };

        if self.enable_link_path {
            if let Some(file) = record.file() {
                // Build a file:// URL — use absolute path if available, else
                // pass through the relative path as-is (terminals/editors that
                // understand the protocol will resolve it).
                let abs = std::path::Path::new(file)
                    .canonicalize()
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| file.to_string());
                let url = format!("file://{}", abs);
                let link_style = Style::parse_strict(&format!("dim link {}", url))
                    .unwrap_or_else(|_| dim_style.clone());
                return Text::styled_with(&path_str, link_style);
            }
        }
        Text::styled_with(&path_str, dim_style)
    }

    /// Compose all columns into a grid row and print it.
    fn emit(&self, record: &log::Record) {
        // Traceback path: when enabled and the message is multi-line (the
        // common Display form for error chains via `{:#}`), render it through
        // a Panel-wrapped Traceback so the call/error chain is styled.
        let msg_str = format!("{}", record.args());
        if self.gilt_tracebacks && record.level() <= log::Level::Error && msg_str.contains('\n') {
            // No real backtrace available for the log record; pass the
            // multi-line message in as the panic message so the Panel-wrapped
            // Traceback renders the full chain.
            let tb = Traceback::from_panic(&msg_str, "");
            if let Ok(mut console) = self.console.lock() {
                console.print(&tb);
            }
            return;
        }

        // Lock console briefly to extract styles, then drop before building cells.
        let (level_style, kw_style) = if let Ok(console) = self.console.lock() {
            let lvl_lower = match record.level() {
                log::Level::Error => "error",
                log::Level::Warn => "warn",
                log::Level::Info => "info",
                log::Level::Debug => "debug",
                log::Level::Trace => "trace",
            };
            let level_style = console
                .get_style(&format!("logging.level.{}", lvl_lower))
                .unwrap_or_else(|_| {
                    self.level_styles
                        .get(&record.level())
                        .cloned()
                        .unwrap_or_else(Style::null)
                });
            let kw_style = console
                .get_style("logging.keyword")
                .unwrap_or_else(|_| Style::parse("bold on dark_green"));
            (level_style, kw_style)
        } else {
            (
                self.level_styles
                    .get(&record.level())
                    .cloned()
                    .unwrap_or_else(Style::null),
                Style::parse("bold on dark_green"),
            )
        };

        let mut cells: Vec<Text> = Vec::new();
        let mut headers: Vec<&str> = Vec::new();

        if self.show_time {
            cells.push(self.render_time_with_omit());
            headers.push("");
        }

        if self.show_level {
            // Build level cell directly using pre-fetched style (avoids re-locking).
            let name = match record.level() {
                log::Level::Error => "ERROR",
                log::Level::Warn => "WARN",
                log::Level::Info => "INFO",
                log::Level::Debug => "DEBUG",
                log::Level::Trace => "TRACE",
            };
            let padded = format!("{:<8}", name);
            cells.push(Text::styled_with(&padded, level_style));
            headers.push("");
        }

        // Build message cell using pre-fetched kw_style.
        let msg = format!("{}", record.args());
        let mut text = if self.markup {
            let base = Style::null();
            markup::render(&msg, base).unwrap_or_else(|_| Text::new(&msg, Style::null()))
        } else {
            Text::new(&msg, Style::null())
        };
        self.highlighter.highlight(&mut text);
        if !self.keywords.is_empty() {
            let words: Vec<&str> = self.keywords.iter().map(|s| s.as_str()).collect();
            text.highlight_words(&words, kw_style, false);
        }
        cells.push(text);
        headers.push("");

        if self.show_path {
            cells.push(self.render_path_with_link(record));
            headers.push("");
        }

        let mut grid = Table::grid(&headers);
        // Match rich LogRender: padding=(0,1) between columns, no left pad,
        // expand to full width so path column lands at a fixed right edge.
        grid.padding = (0, 1, 0, 0);
        grid.set_expand(true);

        // Right-justify the path column so it is flush with the terminal right edge.
        if self.show_path && !grid.columns.is_empty() {
            let last = grid.columns.len() - 1;
            grid.columns[last].justify = JustifyMethod::Right;
        }

        grid.add_row_text(&cells);

        if let Ok(mut console) = self.console.lock() {
            console.print(&grid);
        }
    }

    /// Compute the time string for this record, returning an equal-width
    /// blank string when `omit_repeated_times` is set and the second matches
    /// the previously-emitted time.
    ///
    /// Item 7: uses a date-inclusive cache key so midnight rollover never
    /// falsely suppresses a new timestamp.
    fn render_time_with_omit(&self) -> Text {
        let (display, cache_key) = Self::current_time_with_date();
        if self.omit_repeated_times {
            let mut last = self.last_time_str.lock().unwrap_or_else(|p| p.into_inner());
            if last.as_ref() == Some(&cache_key) {
                let blanks = " ".repeat(cell_len(&display));
                return Text::new(&blanks, Style::null());
            }
            *last = Some(cache_key);
        }
        let dim_style = Style::parse("dim");
        Text::styled_with(&display, dim_style)
    }

    /// Return `(display_str, cache_key)` where `display_str` is `HH:MM:SS`
    /// and `cache_key` includes the days-since-epoch so midnight rollover is
    /// handled correctly (item 7).
    fn current_time_with_date() -> (String, String) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let dur = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let total_secs = dur.as_secs();
        let hours = (total_secs / 3600) % 24;
        let minutes = (total_secs / 60) % 60;
        let seconds = total_secs % 60;
        let days = total_secs / 86400;
        let display = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        let cache_key = format!("{} {}", days, display);
        (display, cache_key)
    }

}

impl Default for RichHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl log::Log for RichHandler {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.min_level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.emit(record);
        }
    }

    fn flush(&self) {
        // Console output is not buffered in a way that needs explicit flushing.
    }
}

// ---------------------------------------------------------------------------
// install() convenience
// ---------------------------------------------------------------------------

/// Create a default [`RichHandler`] and install it as the global logger.
///
/// Sets the max log level to [`log::LevelFilter::Trace`] so all messages
/// are forwarded to the handler.
pub fn install() -> Result<(), log::SetLoggerError> {
    let handler = RichHandler::new();
    log::set_boxed_logger(Box::new(handler))?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;
    use crate::highlighter::NullHighlighter;

    fn test_console() -> Console {
        Console::builder().width(80).no_color(true).build()
    }

    // -- Default construction ------------------------------------------------

    #[test]
    fn test_default_construction() {
        let handler = RichHandler::new();
        assert!(handler.show_time);
        assert!(handler.show_level);
        assert!(handler.show_path);
        assert!(!handler.markup);
        assert!(!handler.gilt_tracebacks);
        assert!(!handler.keywords.is_empty());
        // Finding #15: enable_link_path defaults to true (matches rich).
        assert!(handler.enable_link_path);
    }

    #[test]
    fn test_default_trait() {
        let handler = RichHandler::default();
        assert!(handler.show_time);
    }

    // -- Builder methods -----------------------------------------------------

    #[test]
    fn test_builder_show_time() {
        let handler = RichHandler::new().with_show_time(false);
        assert!(!handler.show_time);
    }

    #[test]
    fn test_builder_show_level() {
        let handler = RichHandler::new().with_show_level(false);
        assert!(!handler.show_level);
    }

    #[test]
    fn test_builder_show_path() {
        let handler = RichHandler::new().with_show_path(false);
        assert!(!handler.show_path);
    }

    #[test]
    fn test_builder_markup() {
        let handler = RichHandler::new().with_markup(true);
        assert!(handler.markup);
    }

    #[test]
    fn test_builder_keywords() {
        let handler = RichHandler::new().with_keywords(vec!["FOO".to_string(), "BAR".to_string()]);
        assert_eq!(handler.keywords, vec!["FOO", "BAR"]);
    }

    #[test]
    fn test_builder_console() {
        let console = Console::builder().width(120).build();
        let _handler = RichHandler::new().with_console(console);
        // No panic means success; we cannot inspect the inner console easily.
    }

    #[test]
    fn test_builder_time_format() {
        let handler = RichHandler::new().with_time_format("%H:%M:%S".to_string());
        assert_eq!(handler.time_format.as_deref(), Some("%H:%M:%S"));
    }

    #[test]
    fn test_builder_with_highlighter_null() {
        let handler = RichHandler::new()
            .with_highlighter(NullHighlighter)
            .with_keywords(vec![]);
        let record = log::Record::builder()
            .args(format_args!("value=42"))
            .level(log::Level::Info)
            .build();
        let console = test_console();
        let text = handler.render_message(&record, &console);
        assert_eq!(text.plain(), "value=42");
        // NullHighlighter adds no spans; keywords empty so no keyword spans either.
        assert!(
            text.spans().is_empty(),
            "NullHighlighter should add no spans"
        );
    }

    #[test]
    fn test_builder_min_level() {
        let handler = RichHandler::new().with_min_level(log::LevelFilter::Warn);
        assert_eq!(handler.min_level, log::LevelFilter::Warn);
    }

    // -- Item 5: min_level filtering -----------------------------------------

    #[test]
    fn test_min_level_filters_correctly() {
        let handler = RichHandler::new().with_min_level(log::LevelFilter::Warn);

        let make_meta = |level: log::Level| {
            log::MetadataBuilder::new()
                .level(level)
                .target("test")
                .build()
        };

        assert!(
            !log::Log::enabled(&handler, &make_meta(log::Level::Trace)),
            "Trace should be filtered"
        );
        assert!(
            !log::Log::enabled(&handler, &make_meta(log::Level::Debug)),
            "Debug should be filtered"
        );
        assert!(
            !log::Log::enabled(&handler, &make_meta(log::Level::Info)),
            "Info should be filtered"
        );
        assert!(
            log::Log::enabled(&handler, &make_meta(log::Level::Warn)),
            "Warn should pass"
        );
        assert!(
            log::Log::enabled(&handler, &make_meta(log::Level::Error)),
            "Error should pass"
        );
    }

    // -- Level style mapping -------------------------------------------------

    #[test]
    fn test_level_styles_all_present() {
        let styles = RichHandler::default_level_styles();
        assert!(styles.contains_key(&log::Level::Error));
        assert!(styles.contains_key(&log::Level::Warn));
        assert!(styles.contains_key(&log::Level::Info));
        assert!(styles.contains_key(&log::Level::Debug));
        assert!(styles.contains_key(&log::Level::Trace));
    }

    #[test]
    fn test_error_style_is_bold_red() {
        let styles = RichHandler::default_level_styles();
        let error_style = styles.get(&log::Level::Error).unwrap();
        assert_eq!(error_style.bold(), Some(true));
        assert!(error_style.color().is_some());
        assert_eq!(error_style.color().unwrap().name(), "red");
    }

    #[test]
    fn test_warn_style_is_bold_yellow() {
        let styles = RichHandler::default_level_styles();
        let warn_style = styles.get(&log::Level::Warn).unwrap();
        assert_eq!(warn_style.bold(), Some(true));
        assert_eq!(warn_style.color().unwrap().name(), "yellow");
    }

    #[test]
    fn test_info_style_is_bold_blue() {
        // Finding #16: info is bold blue (logging.level.info = blue); was incorrectly bold green.
        let styles = RichHandler::default_level_styles();
        let info_style = styles.get(&log::Level::Info).unwrap();
        assert_eq!(info_style.bold(), Some(true));
        assert_eq!(info_style.color().unwrap().name(), "blue");
    }

    #[test]
    fn test_debug_style_is_bold_green() {
        // Finding #16: debug is bold green (logging.level.debug = green); was incorrectly bold blue.
        let styles = RichHandler::default_level_styles();
        let debug_style = styles.get(&log::Level::Debug).unwrap();
        assert_eq!(debug_style.bold(), Some(true));
        assert_eq!(debug_style.color().unwrap().name(), "green");
    }

    #[test]
    fn test_trace_style_is_dim() {
        let styles = RichHandler::default_level_styles();
        let trace_style = styles.get(&log::Level::Trace).unwrap();
        assert_eq!(trace_style.dim(), Some(true));
    }

    // -- Log formatting: time ------------------------------------------------

    #[test]
    fn test_render_time_format() {
        let handler = RichHandler::new();
        let time_text = handler.render_time_with_omit();
        let plain = time_text.plain().to_string();
        // HH:MM:SS pattern: 8 characters with colons at positions 2 and 5
        assert_eq!(plain.len(), 8);
        assert_eq!(plain.as_bytes()[2], b':');
        assert_eq!(plain.as_bytes()[5], b':');
    }

    #[test]
    fn test_render_time_has_dim_style() {
        let handler = RichHandler::new();
        let time_text = handler.render_time_with_omit();
        assert!(!time_text.spans().is_empty());
    }

    // -- Wave 5B: omit_repeated_times / enable_link_path / gilt_tracebacks ----

    #[test]
    fn omit_repeated_times_blanks_duplicate_timestamps() {
        let handler = RichHandler::new().with_omit_repeated_times(true);
        // First call captures and styles the timestamp.
        let first = handler.render_time_with_omit();
        // Second call within the same second returns same-width blanks.
        let second = handler.render_time_with_omit();
        assert_eq!(second.plain().len(), first.plain().len());
        assert!(
            second.plain().chars().all(|c| c == ' '),
            "expected all spaces, got {:?}",
            second.plain()
        );
    }

    #[test]
    fn omit_repeated_times_disabled_keeps_timestamp() {
        let handler = RichHandler::new().with_omit_repeated_times(false);
        let first = handler.render_time_with_omit();
        let second = handler.render_time_with_omit();
        assert_eq!(first.plain(), second.plain());
        assert!(!first.plain().chars().all(|c| c == ' '));
    }

    #[test]
    fn enable_link_path_wraps_module_in_osc8_link() {
        // Construct a synthetic record with a known file/module/line.
        let handler = RichHandler::new().with_enable_link_path(true);
        let record = log::RecordBuilder::new()
            .args(format_args!("hello"))
            .level(log::Level::Info)
            .target("test_target")
            .module_path(Some("test_target"))
            .file(Some("/tmp/some_test_source.rs"))
            .line(Some(42))
            .build();
        let text = handler.render_path_with_link(&record);
        // Either a span carries the link, or the styled output renders OSC 8.
        let spans = text.spans();
        let has_link = spans.iter().any(|s| s.style.link().is_some());
        assert!(
            has_link,
            "expected a span with a file:// link, got {:?}",
            spans
        );
    }

    #[test]
    fn enable_link_path_disabled_has_no_link() {
        let handler = RichHandler::new().with_enable_link_path(false);
        let record = log::RecordBuilder::new()
            .args(format_args!("hello"))
            .level(log::Level::Info)
            .module_path(Some("m"))
            .line(Some(1))
            .file(Some("/tmp/x.rs"))
            .build();
        let text = handler.render_path_with_link(&record);
        let has_link = text.spans().iter().any(|s| s.style.link().is_some());
        assert!(!has_link);
    }

    #[test]
    fn gilt_tracebacks_builder_sets_field() {
        let handler = RichHandler::new().with_gilt_tracebacks(true);
        assert!(handler.gilt_tracebacks);
    }

    // -- Log formatting: level -----------------------------------------------

    #[test]
    fn test_render_level_error() {
        let handler = RichHandler::new();
        let console = test_console();
        let text = handler.render_level(log::Level::Error, &console);
        assert_eq!(text.plain(), "ERROR   ");
    }

    #[test]
    fn test_render_level_warn() {
        let handler = RichHandler::new();
        let console = test_console();
        let text = handler.render_level(log::Level::Warn, &console);
        assert_eq!(text.plain(), "WARN    ");
    }

    #[test]
    fn test_render_level_info() {
        let handler = RichHandler::new();
        let console = test_console();
        let text = handler.render_level(log::Level::Info, &console);
        assert_eq!(text.plain(), "INFO    ");
    }

    #[test]
    fn test_render_level_debug() {
        let handler = RichHandler::new();
        let console = test_console();
        let text = handler.render_level(log::Level::Debug, &console);
        assert_eq!(text.plain(), "DEBUG   ");
    }

    #[test]
    fn test_render_level_trace() {
        let handler = RichHandler::new();
        let console = test_console();
        let text = handler.render_level(log::Level::Trace, &console);
        assert_eq!(text.plain(), "TRACE   ");
    }

    #[test]
    fn test_render_level_has_style() {
        let handler = RichHandler::new();
        let console = test_console();
        for level in &[
            log::Level::Error,
            log::Level::Warn,
            log::Level::Info,
            log::Level::Debug,
            log::Level::Trace,
        ] {
            let text = handler.render_level(*level, &console);
            assert!(
                !text.spans().is_empty(),
                "level {:?} should have a styled span",
                level
            );
        }
    }

    // -- Item 2: level style from theme override -----------------------------

    #[test]
    fn test_render_level_uses_theme_override() {
        let mut styles = std::collections::HashMap::new();
        styles.insert(
            "logging.level.error".to_string(),
            Style::parse("italic magenta"),
        );
        let theme = crate::color::theme::Theme::new(Some(styles), true);
        let console = Console::builder()
            .theme(theme)
            .no_color(false)
            .width(80)
            .build();
        let handler = RichHandler::new();
        let text = handler.render_level(log::Level::Error, &console);
        assert_eq!(text.plain(), "ERROR   ");
        // Style should come from the theme override (italic magenta), not the default (bold red).
        let span = text.spans().first().expect("should have a span");
        let resolved = span.style.clone();
        assert_eq!(resolved.italic(), Some(true), "expected italic from theme");
    }

    // -- Log formatting: path ------------------------------------------------

    #[test]
    fn test_render_path_with_module() {
        let record = log::Record::builder()
            .args(format_args!("test"))
            .level(log::Level::Info)
            .module_path(Some("my_crate::module"))
            .line(Some(42))
            .build();
        let text = RichHandler::render_path(&record);
        assert_eq!(text.plain(), "my_crate::module:42");
    }

    #[test]
    fn test_render_path_without_module() {
        let record = log::Record::builder()
            .args(format_args!("test"))
            .level(log::Level::Info)
            .line(Some(10))
            .build();
        let text = RichHandler::render_path(&record);
        assert_eq!(text.plain(), ":10");
    }

    #[test]
    fn test_render_path_has_dim_style() {
        let record = log::Record::builder()
            .args(format_args!("test"))
            .level(log::Level::Info)
            .module_path(Some("foo"))
            .line(Some(1))
            .build();
        let text = RichHandler::render_path(&record);
        assert!(!text.spans().is_empty());
    }

    // -- Show/hide time, level, path -----------------------------------------

    #[test]
    fn test_emit_no_time() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(true)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("hello world"))
            .level(log::Level::Info)
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        // Should NOT contain a time pattern
        assert!(
            !output.contains(':'),
            "output should not have time, got: {}",
            output
        );
        // Should contain the level and message
        assert!(output.contains("INFO"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_emit_no_level() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("hello world"))
            .level(log::Level::Warn)
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        assert!(!output.contains("WARN"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_emit_no_path() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("hello world"))
            .level(log::Level::Info)
            .module_path(Some("test_mod"))
            .line(Some(99))
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        assert!(!output.contains("test_mod"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_emit_with_path() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_path(true);

        let record = log::Record::builder()
            .args(format_args!("hello"))
            .level(log::Level::Debug)
            .module_path(Some("mymod"))
            .line(Some(7))
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        assert!(output.contains("mymod:7"));
    }

    // -- Markup parsing in messages ------------------------------------------

    #[test]
    fn test_render_message_plain() {
        let handler = RichHandler::new().with_markup(false).with_keywords(vec![]);
        let console = test_console();
        let record = log::Record::builder()
            .args(format_args!("simple message"))
            .level(log::Level::Info)
            .build();
        let text = handler.render_message(&record, &console);
        assert_eq!(text.plain(), "simple message");
    }

    #[test]
    fn test_render_message_with_markup() {
        let handler = RichHandler::new().with_markup(true).with_keywords(vec![]);
        let console = test_console();
        let record = log::Record::builder()
            .args(format_args!("[bold]hello[/bold] world"))
            .level(log::Level::Info)
            .build();
        let text = handler.render_message(&record, &console);
        // Plain text should have markup stripped
        assert_eq!(text.plain(), "hello world");
        // Should have a span for the bold markup
        assert!(!text.spans().is_empty());
    }

    // -- Different log levels ------------------------------------------------

    #[test]
    fn test_emit_all_levels() {
        let levels = [
            log::Level::Error,
            log::Level::Warn,
            log::Level::Info,
            log::Level::Debug,
            log::Level::Trace,
        ];
        let names = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

        for (level, name) in levels.iter().zip(names.iter()) {
            let console = Console::builder()
                .width(80)
                .no_color(true)
                .record(true)
                .markup(false)
                .build();
            let handler = RichHandler::new()
                .with_console(console)
                .with_show_time(false)
                .with_show_level(true)
                .with_show_path(false);

            let record = log::Record::builder()
                .args(format_args!("msg"))
                .level(*level)
                .build();
            handler.emit(&record);

            let mut console = handler.console.lock().unwrap();
            let output = console.export_text(true, false);
            assert!(
                output.contains(name),
                "expected '{}' in output for {:?}, got: {}",
                name,
                level,
                output
            );
        }
    }

    // -- Keyword highlighting ------------------------------------------------

    #[test]
    fn test_keyword_highlighting() {
        let handler = RichHandler::new()
            .with_markup(false)
            .with_keywords(vec!["GET".to_string(), "POST".to_string()]);

        let console = test_console();
        let record = log::Record::builder()
            .args(format_args!("GET /index.html 200"))
            .level(log::Level::Info)
            .build();
        let text = handler.render_message(&record, &console);
        assert_eq!(text.plain(), "GET /index.html 200");
        // Should have at least one span for the keyword "GET"
        assert!(!text.spans().is_empty(), "expected keyword span for GET");
    }

    #[test]
    fn test_no_keyword_highlighting_when_empty() {
        let handler = RichHandler::new().with_markup(false).with_keywords(vec![]);

        let console = test_console();
        let record = log::Record::builder()
            .args(format_args!("GET /index.html 200"))
            .level(log::Level::Info)
            .build();
        let text = handler.render_message(&record, &console);
        // Finding #14: ReprHighlighter runs on all messages, so spans may be
        // present even with no keywords. Plain text content must still be correct.
        assert_eq!(text.plain(), "GET /index.html 200");
    }

    // -- log::Log trait implementation ---------------------------------------

    #[test]
    fn test_log_trait_enabled_always_true() {
        let handler = RichHandler::new();
        let metadata = log::MetadataBuilder::new()
            .level(log::Level::Trace)
            .target("test")
            .build();
        assert!(log::Log::enabled(&handler, &metadata));
    }

    #[test]
    fn test_log_trait_log_produces_output() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(true)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("log trait test"))
            .level(log::Level::Info)
            .build();
        log::Log::log(&handler, &record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        assert!(output.contains("log trait test"));
    }

    #[test]
    fn test_log_trait_flush_does_not_panic() {
        let handler = RichHandler::new();
        log::Log::flush(&handler);
    }

    // -- install() function --------------------------------------------------

    // Note: log::set_logger can only be called once per process, so we test
    // the function signature rather than calling it multiple times.

    #[test]
    fn test_install_returns_result() {
        // We cannot actually call install() in tests because it's a global
        // singleton and other tests might conflict. Instead, verify the
        // function exists and returns the right type.
        let _: fn() -> Result<(), log::SetLoggerError> = install;
    }

    // -- Full integration: captured output -----------------------------------

    #[test]
    fn test_full_line_with_all_columns() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(true)
            .with_show_level(true)
            .with_show_path(true);

        let record = log::Record::builder()
            .args(format_args!("Server starting"))
            .level(log::Level::Info)
            .module_path(Some("my_app::server"))
            .line(Some(42))
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);

        // Time column (HH:MM:SS pattern)
        assert!(output.contains(':'), "expected time in output");
        // Level column
        assert!(output.contains("INFO"));
        // Message
        assert!(output.contains("Server starting"));
        // Path
        assert!(output.contains("my_app::server:42"));
    }

    #[test]
    fn test_full_line_minimal_columns() {
        let console = Console::builder()
            .width(80)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(false)
            .with_show_path(false);

        let record = log::Record::builder()
            .args(format_args!("bare message"))
            .level(log::Level::Error)
            .build();
        handler.emit(&record);

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        // Should contain only the message
        assert!(output.contains("bare message"));
        assert!(!output.contains("ERROR"));
    }

    #[test]
    fn test_default_keywords_present() {
        let handler = RichHandler::new();
        assert!(handler.keywords.contains(&"GET".to_string()));
        assert!(handler.keywords.contains(&"POST".to_string()));
        assert!(handler.keywords.contains(&"PUT".to_string()));
        assert!(handler.keywords.contains(&"DELETE".to_string()));
    }

    // -- Grid alignment: columns must be stable across different message lengths --

    /// Discriminating test: the PATH column MUST start at the same byte offset on
    /// both lines even when message lengths differ. Under the old flat-concat emit,
    /// path position = time(8) + sp(1) + level(8) + sp(1) + msg_len + sp(1), so
    /// "msg_len" is variable and the path drifts. With Table::grid the message
    /// column is padded to the widest cell, so path is always at the same offset.
    ///
    /// To verify RED: temporarily replace the `Table::grid` emit body with flat
    /// Text concat (see inline comment) and observe the assertion fail with two
    /// different offsets; restore to confirm GREEN.
    #[test]
    fn grid_alignment_path_column_stable_despite_variable_message_length() {
        let console = Console::builder()
            .width(120)
            .no_color(true)
            .record(true)
            .markup(false)
            .build();
        // show_time=false so only level + message + path — isolates the message-
        // length effect on path position without the fixed-width time prefix.
        let handler = RichHandler::new()
            .with_console(console)
            .with_show_time(false)
            .with_show_level(true)
            .with_show_path(true)
            .with_omit_repeated_times(false)
            .with_enable_link_path(false) // plain "module:line" form for easy find()
            .with_keywords(vec![]);

        // Short message → path follows closely under flat concat.
        // Long message  → path would be pushed further right under flat concat.
        let messages = ["A", "a much longer message here"];
        let module = "mymod";
        let line = 7u32;

        for msg in &messages {
            let msg_str = msg.to_string();
            let args = format_args!("{}", msg_str);
            let record = log::Record::builder()
                .args(args)
                .level(log::Level::Info)
                .module_path(Some(module))
                .line(Some(line))
                .build();
            handler.emit(&record);
            drop(msg_str);
        }

        let mut console = handler.console.lock().unwrap();
        let output = console.export_text(true, false);
        drop(console);

        let lines: Vec<&str> = output.lines().filter(|l| !l.is_empty()).collect();
        assert!(
            lines.len() >= 2,
            "expected at least 2 non-empty lines, got:\n{}",
            output
        );

        // Find the path token "mymod:7" on each line.
        let path_token = "mymod:7";
        let off0 = lines[0]
            .find(path_token)
            .unwrap_or_else(|| panic!("'{}' not found in first line: {:?}", path_token, lines[0]));
        let off1 = lines[1]
            .find(path_token)
            .unwrap_or_else(|| panic!("'{}' not found in second line: {:?}", path_token, lines[1]));

        // Under flat concat these differ: off0 ≈ 8+1+1+1 = 11, off1 ≈ 8+1+26+1 = 36.
        // Under Table::grid the message column is padded to max-message width, so
        // both offsets equal the same value.
        assert_eq!(
            off0, off1,
            "PATH column drifted (bug #28): offset {} vs {} — flat-concat regressed.\n\
             Line 0 (short msg):  {:?}\n\
             Line 1 (long  msg):  {:?}",
            off0, off1, lines[0], lines[1]
        );
    }
}
